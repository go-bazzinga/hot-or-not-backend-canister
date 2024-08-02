use candid::Principal;
use ic_cdk::api::call;
use ic_cdk_macros::update;
use shared_utils::common::types::known_principal::KnownPrincipalType;

use crate::CANISTER_DATA;

#[update]
async fn backup_data_to_backup_canister(
    canister_owner_principal_id: Principal,
    canister_id: Principal,
) {
    let api_caller = ic_cdk::caller();

    let user_index_canister_principal_id = CANISTER_DATA.with(|canister_data_ref_cell| {
        canister_data_ref_cell
            .borrow()
            .known_principal_ids
            .get(&KnownPrincipalType::CanisterIdUserIndex)
            .cloned()
            .unwrap()
    });

    let global_super_admin_principal_id = CANISTER_DATA.with(|canister_data_ref_cell| {
        canister_data_ref_cell
            .borrow()
            .known_principal_ids
            .get(&KnownPrincipalType::UserIdGlobalSuperAdmin)
            .cloned()
            .unwrap()
    });

    let is_caller_user_index_canister = api_caller == user_index_canister_principal_id;
    ic_cdk::print(format!(
        "🥫 is_caller_user_index_canister: {}",
        is_caller_user_index_canister
    ));
    let is_caller_global_super_admin = api_caller == global_super_admin_principal_id;
    ic_cdk::print(format!(
        "🥫 is_caller_global_super_admin: {}",
        is_caller_global_super_admin
    ));
    let does_passed_canister_id_match_this_users_canister_id = canister_id == ic_cdk::id();
    ic_cdk::print(format!("🥫 canister_id: {}", canister_id));
    ic_cdk::print(format!("🥫 ic_cdk::id(): {}", ic_cdk::id()));
    ic_cdk::print(format!(
        "🥫 does_passed_canister_id_match_this_users_canister_id: {}",
        does_passed_canister_id_match_this_users_canister_id
    ));

    if !is_caller_user_index_canister && !is_caller_global_super_admin {
        return;
    }

    if !does_passed_canister_id_match_this_users_canister_id {
        return;
    }

    let data_backup_canister_id = CANISTER_DATA.with(|canister_data_ref_cell| {
        *canister_data_ref_cell
            .borrow()
            .known_principal_ids
            .get(&KnownPrincipalType::CanisterIdDataBackup)
            .unwrap()
    });

    ic_cdk::print(format!(
        "🥫 Backing up data for canister with id: {}",
        canister_id
    ));

    send_profile_data(
        &data_backup_canister_id,
        &canister_owner_principal_id,
        &canister_id,
    )
    .await;
    send_all_created_posts(&data_backup_canister_id, &canister_owner_principal_id).await;
    send_all_token_data_v1(&data_backup_canister_id, &canister_owner_principal_id).await;
    send_all_follower_following_data(&data_backup_canister_id, &canister_owner_principal_id).await;
}

const CHUNK_SIZE: usize = 10;

async fn send_profile_data(
    data_backup_canister_id: &Principal,
    canister_owner_principal_id: &Principal,
    canister_id: &Principal,
) {
    let profile_data = CANISTER_DATA
        .with(|canister_data_ref_cell| canister_data_ref_cell.borrow().profile.clone());

    if profile_data.display_name.is_none() && profile_data.profile_picture_url.is_none() {
        return;
    }

    let _: () = call::call(
            *data_backup_canister_id,
            "receive_profile_details_from_individual_user_canister",
            (profile_data, *canister_owner_principal_id, *canister_id),
        )
        .await
        .expect("Failed to call the receive_profile_details_from_individual_user_canister method on the data_backup canister");
}

async fn send_all_created_posts(
    data_backup_canister_id: &Principal,
    canister_owner_principal_id: &Principal,
) {
    let all_created_posts = CANISTER_DATA
        .with(|canister_data_ref_cell| canister_data_ref_cell.borrow().all_created_posts.clone());

    let all_created_posts_vec = all_created_posts.values().cloned().collect::<Vec<_>>();

    let all_created_posts_chunks = all_created_posts_vec.chunks(CHUNK_SIZE).collect::<Vec<_>>();

    for chunk in all_created_posts_chunks {
        let _: () = call::call(
            *data_backup_canister_id,
            "receive_all_user_posts_from_individual_user_canister",
            (chunk.to_vec(), *canister_owner_principal_id),
        )
        .await
        .expect("Failed to call the receive_all_user_posts_from_individual_user_canister method on the data_backup canister");
    }
}

#[deprecated(note = "use send_all_token_data_v1 instead")]
async fn send_all_token_data(
    data_backup_canister_id: &Principal,
    canister_owner_principal_id: &Principal,
) {
    let token_data = CANISTER_DATA
        .with(|canister_data_ref_cell| canister_data_ref_cell.borrow().my_token_balance.clone());

    let _: () = call::call(
            *data_backup_canister_id,
            "receive_current_token_balance_from_individual_user_canister",
            (token_data.utility_token_balance, *canister_owner_principal_id),
        )
        .await
        .expect("Failed to call the receive_current_token_balance_from_individual_user_canister method on the data_backup canister");

    let all_token_transactions = token_data
        .utility_token_transaction_history
        .iter()
        .map(|(token_transaction_id, token_event)| (*token_transaction_id, (*token_event).clone()))
        .collect::<Vec<_>>();

    let all_token_transactions_chunks = all_token_transactions
        .chunks(CHUNK_SIZE)
        .collect::<Vec<_>>();

    for chunk in all_token_transactions_chunks {
        let _: () = call::call(
            *data_backup_canister_id,
            "receive_all_token_transactions_from_individual_user_canister",
            (chunk.to_vec(), *canister_owner_principal_id),
        )
        .await
        .expect("Failed to call the receive_all_token_transactions_from_individual_user_canister method on the data_backup canister");
    }
}


async fn send_all_token_data_v1(
    data_backup_canister_id: &Principal,
    canister_owner_principal_id: &Principal,
) {
    let token_data = CANISTER_DATA
        .with(|canister_data_ref_cell| canister_data_ref_cell.borrow().my_token_balance_v1.clone());

    let _: () = call::call(
            *data_backup_canister_id,
            "receive_current_token_balance_from_individual_user_canister",
            (token_data.utility_token_balance, *canister_owner_principal_id),
        )
        .await
        .expect("Failed to call the receive_current_token_balance_from_individual_user_canister method on the data_backup canister");

    let all_token_transactions = token_data
        .utility_token_transaction_history
        .iter()
        .map(|(token_transaction_id, token_event)| (*token_transaction_id, (*token_event).clone()))
        .collect::<Vec<_>>();

    let all_token_transactions_chunks = all_token_transactions
        .chunks(CHUNK_SIZE)
        .collect::<Vec<_>>();

    for chunk in all_token_transactions_chunks {
        let _: () = call::call(
            *data_backup_canister_id,
            "receive_all_token_transactions_from_individual_user_canister",
            (chunk.to_vec(), *canister_owner_principal_id),
        )
        .await
        .expect("Failed to call the receive_all_token_transactions_from_individual_user_canister method on the data_backup canister");
    }
}

async fn send_all_follower_following_data(
    _data_backup_canister_id: &Principal,
    _canister_owner_principal_id: &Principal,
) {
    // let principals_i_follow = CANISTER_DATA.with(|canister_data_ref_cell| {
    //     canister_data_ref_cell
    //         .borrow()
    //         .follow_data
    //         .following
    //         .clone()
    // });

    // let principals_i_follow_vec = principals_i_follow
    //     .iter()
    //     .map(|principal_id| *principal_id)
    //     .collect::<Vec<_>>();

    // let principals_i_follow_chunks = principals_i_follow_vec
    //     .chunks(CHUNK_SIZE)
    //     .collect::<Vec<_>>();

    // for chunk in principals_i_follow_chunks {
    //     let _response: () = call::call(
    //         data_backup_canister_id.clone(),
    //         "receive_principals_i_follow_from_individual_user_canister",
    //         (chunk.to_vec(), *canister_owner_principal_id),
    //     )
    //     .await
    //     .expect("Failed to call the receive_principals_i_follow_from_individual_user_canister method on the data_backup canister");
    // }

    // let principals_that_follow_me = CANISTER_DATA.with(|canister_data_ref_cell| {
    //     canister_data_ref_cell
    //         .borrow()
    //         .principals_that_follow_me
    //         .clone()
    // });

    // let principals_that_follow_me_vec = principals_that_follow_me
    //     .iter()
    //     .map(|principal_id| *principal_id)
    //     .collect::<Vec<_>>();

    // let principals_that_follow_me_chunks = principals_that_follow_me_vec
    //     .chunks(CHUNK_SIZE)
    //     .collect::<Vec<_>>();

    // for chunk in principals_that_follow_me_chunks {
    //     let _response: () = call::call(
    //         data_backup_canister_id.clone(),
    //         "receive_principals_that_follow_me_from_individual_user_canister",
    //         (chunk.to_vec(), *canister_owner_principal_id),
    //     )
    //     .await
    //     .expect("Failed to call the receive_principals_that_follow_me_from_individual_user_canister method on the data_backup canister");
    // }
}
