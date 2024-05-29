use std::{borrow::BorrowMut, collections::BTreeMap, error::Error};

use candid::{CandidType, Principal};
use ic_cdk::{
    api::{
        call, canister_balance128,
        management_canister::main::{canister_info, CanisterInfoRequest},
    },
    call, caller, id,
};
use serde::{Deserialize, Serialize};
use shared_utils::{
    canister_specific::individual_user_template::types::{migration::MigrationInfo, post::Post},
    common::{
        types::utility_token::token_event::TokenEvent,
        utils::system_time::{get_current_system_time, get_current_system_time_from_ic},
    },
    constant::HOT_OR_NOT_SUBNET_ORCHESTRATOR_CANISTER_ID,
};

use crate::{data_model::CanisterData, CANISTER_DATA};

#[derive(PartialEq)]
pub enum SubnetType {
    HotorNot,
    Yral,
}

pub trait Migration {
    async fn transfer_tokens_and_posts(
        &self,
        to_individual_user: IndividualUser,
    ) -> Result<(), MigrationErrors>;

    fn recieve_tokens_and_posts(
        &self,
        from_individual_user: IndividualUser,
        token_amount: u64,
        posts: BTreeMap<u64, Post>,
    ) -> Result<(), MigrationErrors>;
}

#[derive(Serialize, Deserialize, Debug, CandidType)]
pub enum MigrationErrors {
    InvalidToCanister,
    InvalidFromCanister,
    MigrationInfoNotFound,
    AlreadyMigrated,
    TransferToCanisterCallFailed,
    CanisterInfoFailed,
    UserNotRegistered,
    Unauthorized,
}

pub struct MigrateCanister {
    profile_principal: Principal,
    canister_id: Principal,
}

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

        let subnet_type = if canister_info
            .controllers
            .contains(&Principal::from_text(HOT_OR_NOT_SUBNET_ORCHESTRATOR_CANISTER_ID).unwrap())
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
}

impl Migration for IndividualUser {
    async fn transfer_tokens_and_posts(
        &self,
        to_individual_user: IndividualUser,
    ) -> Result<(), MigrationErrors> {
        if self.profile_principal != caller() {
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

        let (posts, token) = CANISTER_DATA.with_borrow(|canister_data| {
            (
                canister_data.all_created_posts.clone(),
                canister_data.my_token_balance.clone(),
            )
        });

        let (transfer_res,): (Result<(), MigrationErrors>,) = call(
            to_individual_user.canister_id,
            "receive_data_from_hotornot",
            (self.profile_principal, token.utility_token_balance, posts),
        )
        .await
        .map_err(|_e| MigrationErrors::TransferToCanisterCallFailed)?;

        match transfer_res {
            Ok(()) => CANISTER_DATA.with_borrow_mut(|canister_data| {
                canister_data
                    .my_token_balance
                    .handle_token_event(TokenEvent::Transfer {
                        amount: token.utility_token_balance,
                        to_account: to_individual_user.profile_principal,
                        timestamp: get_current_system_time_from_ic(),
                    });

                canister_data.migration_info = MigrationInfo::MigratedToYral {
                    account_principal: to_individual_user.profile_principal,
                };

                Ok(())
            }),
            Err(e) => Err(e),
        }
    }

    fn recieve_tokens_and_posts(
        &self,
        from_individual_user: IndividualUser,
        token_amout: u64,
        posts: BTreeMap<u64, Post>,
    ) -> Result<(), MigrationErrors> {
        CANISTER_DATA.with_borrow_mut(|canister_data| {
            canister_data.all_created_posts.extend(posts.into_iter());
            canister_data
                .my_token_balance
                .handle_token_event(TokenEvent::Receive {
                    amount: token_amout,
                    from_account: from_individual_user.profile_principal,
                    timestamp: get_current_system_time_from_ic(),
                });

            canister_data.migration_info = MigrationInfo::MigratedFromHotOrNot {
                account_principal: from_individual_user.profile_principal,
            }
        });

        Ok(())
    }
}