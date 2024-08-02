use std::{borrow::BorrowMut, collections::BTreeMap, error::Error};

use candid::Principal;
use ic_cdk::{
    api::management_canister::main::{canister_info, CanisterInfoRequest},
    call, caller, id,
};
use shared_utils::{
    canister_specific::individual_user_template::types::{
        migration::{MigrationErrors, MigrationInfo},
        post::{Post, PostDetailsFromFrontend},
        session::SessionType,
    },
    common::{
        types::{known_principal::KnownPrincipalType, utility_token::token_event::{TokenEvent, TokenEventV1}},
        utils::system_time::{get_current_system_time, get_current_system_time_from_ic},
    },
};

use crate::{
    api::post::add_post_v2::{self, add_post_to_memory},
    CANISTER_DATA,
};

#[derive(PartialEq, Clone, Copy)]
pub enum SubnetType {
    HotorNot,
    Yral,
}

pub trait Migration {
    async fn transfer_tokens_and_posts(
        &self,
        caller: Principal,
        to_individual_user: IndividualUser,
    ) -> Result<(), MigrationErrors>;

    async fn recieve_tokens_and_posts(
        &self,
        from_individual_user: IndividualUser,
        token_amount: u64,
        posts: Vec<Post>,
    ) -> Result<(), MigrationErrors>;
}

#[derive(Copy, Clone)]
pub struct IndividualUser {
    pub canister_id: Principal,
    pub profile_principal: Principal,
    pub subnet_type: SubnetType,
    pub migration_status: Option<MigrationInfo>,
}

impl IndividualUser {
    pub async fn from_canister_data() -> Result<Self, MigrationErrors> {
        let (profile_principal, migration_info) = CANISTER_DATA.with_borrow(|canister_data| {
            let Some(profile_principal) = canister_data.profile.principal_id else {
                return Err(MigrationErrors::UserNotRegistered);
            };
            if canister_data.session_type != Some(SessionType::RegisteredSession) {
                return Err(MigrationErrors::UserNotRegistered);
            }
            Ok((profile_principal, canister_data.migration_info))
        })?;

        IndividualUser::new(id(), profile_principal, Some(migration_info)).await
    }

    pub async fn new(
        canister_id: Principal,
        profile_principal: Principal,
        migration_info: Option<MigrationInfo>,
    ) -> Result<IndividualUser, MigrationErrors> {
        let (canister_info,) = canister_info(CanisterInfoRequest {
            canister_id,
            num_requested_changes: None,
        })
        .await
        .map_err(|_e| MigrationErrors::CanisterInfoFailed)?;

        let hot_or_not_subnet_orchestrator_canister_id =
            CANISTER_DATA.with_borrow(|canister_data| {
                canister_data
                    .known_principal_ids
                    .get(&KnownPrincipalType::CanisterIdHotOrNotSubnetOrchestrator)
                    .copied()
                    .ok_or(MigrationErrors::HotOrNotSubnetCanisterIdNotFound)
            })?;

        let subnet_type = if canister_info
            .controllers
            .contains(&hot_or_not_subnet_orchestrator_canister_id)
        {
            SubnetType::HotorNot
        } else {
            SubnetType::Yral
        };

        Ok(IndividualUser {
            canister_id,
            profile_principal,
            subnet_type,
            migration_status: migration_info,
        })
    }

    async fn request_cycles_for_migration(&self) -> Result<(), MigrationErrors> {
        let cycles_amount = 500_000_000_u128; //0.5 Billion

        let subnet_orchestrator = CANISTER_DATA.with_borrow(|canister_data| {
            canister_data
                .known_principal_ids
                .get(&KnownPrincipalType::CanisterIdUserIndex)
                .cloned()
                .ok_or(MigrationErrors::UserIndexCanisterIdNotFound)
        })?;

        let (res,): (Result<(), String>,) =
            call(subnet_orchestrator, "request_cycles", (cycles_amount,))
                .await
                .map_err(|e| MigrationErrors::RequestCycleFromUserIndexFailed(e.1))?;

        res.map_err(MigrationErrors::RequestCycleFromUserIndexFailed)
    }

    fn transfer_checks(
        &self,
        caller: Principal,
        to_individual_user: IndividualUser,
    ) -> Result<(), MigrationErrors> {
        if self.profile_principal != caller {
            return Err(MigrationErrors::Unauthorized);
        }

        if self.subnet_type != SubnetType::HotorNot {
            return Err(MigrationErrors::InvalidFromCanister);
        }
        if to_individual_user.subnet_type == SubnetType::HotorNot {
            return Err(MigrationErrors::InvalidToCanister);
        }

        let Some(migration_info) = self.migration_status else {
            return Err(MigrationErrors::MigrationInfoNotFound);
        };

        if migration_info != MigrationInfo::NotMigrated {
            return Err(MigrationErrors::AlreadyMigrated);
        }

        Ok(())
    }

    async fn transfer_tokens(
        &self,
        caller: Principal,
        to_individual_user: IndividualUser,
    ) -> Result<(), MigrationErrors> {
        let token =
            CANISTER_DATA.with_borrow(|canister_data| canister_data.my_token_balance_v1.clone());

        let (transfer_res,): (Result<(), MigrationErrors>,) = call(
            to_individual_user.canister_id,
            "receive_data_from_hotornot",
            (
                self.profile_principal,
                token.utility_token_balance,
                Vec::<Post>::new(),
            ),
        )
        .await
        .map_err(|e| MigrationErrors::TransferToCanisterCallFailed(e.1))?;

        match transfer_res {
            Ok(()) => CANISTER_DATA.with_borrow_mut(|canister_data| {
                canister_data
                    .my_token_balance_v1
                    .handle_token_event(TokenEventV1::Transfer {
                        amount: token.utility_token_balance,
                        to_account: to_individual_user.profile_principal,
                        timestamp: get_current_system_time(),
                    });

                canister_data.migration_info = MigrationInfo::MigratedToYral {
                    account_principal: to_individual_user.profile_principal,
                };

                Ok(())
            }),
            Err(e) => Err(e),
        }
    }
    fn transfer_posts(&self, to_individual_user: IndividualUser) -> Result<(), MigrationErrors> {
        ic_cdk::spawn(transfer_posts_task(
            self.profile_principal,
            to_individual_user,
        ));

        Ok(())
    }

    fn receive_posts(&self, posts: Vec<Post>) {
        CANISTER_DATA.with_borrow_mut(|canister_data| {
            let transfer_posts: Vec<PostDetailsFromFrontend> = posts
                .into_iter()
                .map(PostDetailsFromFrontend::from)
                .collect();

            let current_system_time = get_current_system_time();
            transfer_posts.iter().for_each(|post| {
                add_post_to_memory(canister_data, post, &current_system_time);
            });
        })
    }

    fn receive_tokens(
        &self,
        from_individual_user: IndividualUser,
        token_amount: u64,
    ) -> Result<(), MigrationErrors> {
        CANISTER_DATA.with_borrow_mut(|canister_data| {
            if token_amount > 0 && canister_data.migration_info != MigrationInfo::NotMigrated {
                return Err(MigrationErrors::AlreadyUsedForMigration);
            }

            canister_data
                .my_token_balance
                .handle_token_event(TokenEvent::Receive {
                    amount: token_amount,
                    from_account: from_individual_user.profile_principal,
                    timestamp: get_current_system_time_from_ic(),
                });

            canister_data.migration_info = MigrationInfo::MigratedFromHotOrNot {
                account_principal: from_individual_user.profile_principal,
            };
            Ok(())
        })
    }
}

async fn transfer_posts_task(profile_principal: Principal, to_individual_user: IndividualUser) {
    let posts: Vec<Post> = CANISTER_DATA
        .with_borrow(|canister_data| canister_data.all_created_posts.clone())
        .into_iter()
        .map(|v| v.1)
        .collect();

    let post_chunks = posts.chunks(10);

    for posts in post_chunks {
        let transfer_res: Result<(Result<(), MigrationErrors>,), MigrationErrors> = call(
            to_individual_user.canister_id,
            "receive_data_from_hotornot",
            (profile_principal, 0_u64, posts.to_vec()),
        )
        .await
        .map_err(|e| MigrationErrors::TransferToCanisterCallFailed(e.1));
    }
}

impl Migration for IndividualUser {
    async fn transfer_tokens_and_posts(
        &self,
        caller: Principal,
        to_individual_user: IndividualUser,
    ) -> Result<(), MigrationErrors> {
        self.transfer_checks(caller, to_individual_user)?;

        let res = self.transfer_tokens(caller, to_individual_user).await;

        match res {
            Ok(()) => {
                self.request_cycles_for_migration().await?;
                let _ = self.transfer_posts(to_individual_user);
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    async fn recieve_tokens_and_posts(
        &self,
        from_individual_user: IndividualUser,
        token_amount: u64,
        posts: Vec<Post>,
    ) -> Result<(), MigrationErrors> {
        if from_individual_user.subnet_type != SubnetType::HotorNot {
            return Err(MigrationErrors::Unauthorized);
        }

        if token_amount > 0 {
            self.receive_tokens(from_individual_user, token_amount)?;
            self.request_cycles_for_migration().await?;
        }

        self.receive_posts(posts);

        Ok(())
    }
}
