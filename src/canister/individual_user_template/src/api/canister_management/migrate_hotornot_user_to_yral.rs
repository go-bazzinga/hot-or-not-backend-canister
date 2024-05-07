use crate::CANISTER_DATA;
use candid::Principal;
use ic_cdk::{
    api::management_canister::main::{canister_info, CanisterInfoRequest},
    caller,
};
use ic_cdk_macros::update;
use shared_utils::{
    canister_specific::individual_user_template::types::{migration::MigrationInfo, post::Post},
    common::{
        types::{known_principal::KnownPrincipalType, utility_token::token_event::TokenEvent},
        utils::system_time::get_current_system_time_from_ic,
    },
    constant::{ConstantsWrapper, Controller},
};
use std::collections::BTreeMap;

#[update]
pub async fn transfer_tokens_and_posts(to_account: Principal) -> Result<String, String> {
    let profile_owner =
        CANISTER_DATA.with_borrow(|canister_data| canister_data.profile.principal_id.unwrap());

    if profile_owner != caller() {
        return Err("Unauthorized caller".to_owned());
    }

    // Users on hotornot subnet are allowed to migrate, others are unauthorized
    if let Err(error) = check_canister_subnet_type(ic_cdk::id(), SubnetType::NotHotOrNot).await {
        return Err(format!(
            "Unauthorized controller, not in hotornot subnet: {}",
            error
        ));
    }

    CANISTER_DATA.with_borrow_mut(|canister_data| {
        let result = canister_data
            .session_type
            .ok_or("Canister not yet assigned".to_owned());
        canister_data.last_access_time = Some(get_current_system_time_from_ic());
        result
    })?;

    let current_time = get_current_system_time_from_ic();
    let amount = CANISTER_DATA
        .with_borrow(|canister_data| canister_data.my_token_balance.utility_token_balance);
    let all_created_posts =
        CANISTER_DATA.with_borrow(|canister_data| canister_data.all_created_posts.clone());

    let user_index_canister_id = CANISTER_DATA.with_borrow(|canister_data| {
        let canister_id = canister_data
            .known_principal_ids
            .get(&KnownPrincipalType::CanisterIdUserIndex);
        canister_id.copied()
    });

    if user_index_canister_id.is_none() {
        return Err("canister_id not found in user_index".to_owned());
    }

    let to_account_canister_id = match ic_cdk::call::<(Principal,), (Option<Principal>,)>(
        user_index_canister_id.unwrap(),
        "get_user_canister_id_from_user_principal_id",
        (to_account,),
    )
    .await
    {
        Ok(canister_id) => match canister_id.0 {
            Some(id) => id,
            None => return Err("user_index canister id not found".to_owned()),
        },
        Err(error) => {
            return Err(format!(
                "Failed to call get_user_canister_id_from_user_principal_id: {:?}: {}",
                error.0, error.1
            ));
        }
    };

    match ic_cdk::call::<(u64, Principal, BTreeMap<u64, Post>), (Result<bool, String>,)>(
        to_account_canister_id,
        "receive_data_from_hotornot",
        (amount, profile_owner, all_created_posts),
    )
    .await
    {
        Ok(_) => CANISTER_DATA.with_borrow_mut(|canister_data| {
            canister_data
                .my_token_balance
                .handle_token_event(TokenEvent::Transfer {
                    amount,
                    to_account,
                    timestamp: current_time,
                });
            canister_data.migration_info = MigrationInfo::MigratedFromHotOrNot {
                to_yral_principal_id: to_account,
            };
        }),
        Err(error) => {
            return Err(format!(
                "Failed to call receive: {:?}: {}",
                error.0, error.1
            ));
        }
    }

    Ok("Success".into())
}

#[derive(PartialEq, Eq)]
enum SubnetType {
    HotOrNot,
    NotHotOrNot,
}

async fn check_canister_subnet_type(
    canister_id: Principal,
    subnet_type: SubnetType,
) -> Result<bool, String> {
    let mut list = Vec::<String>::new();
    match canister_info(CanisterInfoRequest {
        canister_id,
        num_requested_changes: None,
    })
    .await
    {
        Ok(canister_response) => {
            let mut matched = SubnetType::NotHotOrNot;
            let controller_id = Controller {}.get_hot_or_not_controller_id();
            list.push(controller_id.to_owned());
            for controller in canister_response.0.controllers {
                list.push(controller.to_text());
                if controller.to_text().eq(&controller_id) {
                    matched = SubnetType::HotOrNot;
                    break;
                }
            }
            Ok(matched.eq(&subnet_type))
        }
        Err(error) => Err(format!(
            "canister_info rejection: {:?} : {}",
            error.0, error.1
        )),
    }
}

#[update]
pub async fn receive_data_from_hotornot(
    amount: u64,
    from_account: Principal,
    all_created_posts: BTreeMap<u64, Post>,
) -> Result<String, String> {
    let profile_owner =
        CANISTER_DATA.with_borrow(|canister_data| canister_data.profile.principal_id.unwrap());

    if profile_owner != caller() {
        return Err("Unauthorized caller".to_owned());
    }

    // Users not on hotornot subnet are allowed to receive, others are unauthorized
    if check_canister_subnet_type(ic_cdk::id(), SubnetType::HotOrNot)
        .await
        .is_err()
    {
        return Err("Unauthorized controller, found in hotornot subnet".to_owned());
    }
    if CANISTER_DATA.with_borrow(|canister_data| {
        matches!(
            canister_data.migration_info,
            MigrationInfo::MigratedToYral {
                from_hotornot_principal_id: _
            }
        )
    }) {
        return Err("Already Migrated".to_owned());
    };

    let current_time = get_current_system_time_from_ic();

    CANISTER_DATA.with_borrow_mut(|canister_data| {
        canister_data
            .session_type
            .ok_or(String::from("Canister not yet assigned"))
            .map(|_| "".to_owned())?;

        let last_post_id = match canister_data.all_created_posts.last_key_value() {
            Some((id, _)) => *id,
            None => 0,
        };
        canister_data
            .my_token_balance
            .handle_token_event(TokenEvent::Receive {
                amount,
                from_account,
                timestamp: current_time,
            });

        for (id, post) in all_created_posts {
            canister_data
                .all_created_posts
                .insert(last_post_id + id, post);
        }

        canister_data.migration_info = MigrationInfo::MigratedToYral {
            from_hotornot_principal_id: from_account,
        };
        Ok("Success".to_owned())
    })
}