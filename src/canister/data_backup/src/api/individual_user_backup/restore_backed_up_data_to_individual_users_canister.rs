use candid::Principal;
use ic_cdk::api::call;
use shared_utils::{
    canister_specific::data_backup::types::all_user_data::AllUserData,
    common::types::{known_principal::KnownPrincipalType, storable_principal::StorablePrincipal},
};

use crate::CANISTER_DATA;

#[ic_cdk::update]
#[candid::candid_method(update)]
async fn restore_backed_up_data_to_individual_users_canister(
    user_principal_id: Principal,
) -> String {
    // * Get the caller principal ID.
    let caller_principal_id = ic_cdk::caller();

    if !(CANISTER_DATA.with(|canister_data_ref_cell| {
        *canister_data_ref_cell
            .borrow()
            .heap_data
            .known_principal_ids
            .get(&KnownPrincipalType::UserIdGlobalSuperAdmin)
            .unwrap()
            == caller_principal_id
    })) {
        return "Unauthorized".to_string();
    }

    let users_data = CANISTER_DATA.with(|canister_data_ref_cell| {
        canister_data_ref_cell
            .borrow()
            .user_principal_id_to_all_user_data_map
            .get(&StorablePrincipal(user_principal_id))
    });

    if users_data.is_none() {
        return "No user data found".to_string();
    }

    let users_data = users_data.unwrap();

    send_posts(&users_data).await;
    send_utility_token_balance_v1(&users_data).await;
    send_utility_token_history_v1(&users_data).await;
    send_principals_i_follow(&users_data).await;
    send_principals_that_follow_me(&users_data).await;
    send_profile_data(&users_data).await;

    "Success".to_string()
}

const CHUNK_SIZE: usize = 10;

async fn send_profile_data(users_data: &AllUserData) {
    let canister_id_to_send_to = users_data.user_canister_id;

    let _: () = call::call(
        canister_id_to_send_to,
        "receive_my_profile_from_data_backup_canister",
        (users_data.canister_data.profile.clone(),),
    )
    .await
    .expect("Failed to call the receive_my_profile_from_data_backup_canister method on the individual user's canister");
}

async fn send_principals_that_follow_me(users_data: &AllUserData) {
    let canister_id_to_send_to = users_data.user_canister_id;

    let principals_that_follow_me_vec = users_data
        .canister_data
        .principals_that_follow_me
        .iter()
        .copied()
        .collect::<Vec<_>>();

    let principals_that_follow_me_vec_chunks = principals_that_follow_me_vec
        .chunks(CHUNK_SIZE)
        .collect::<Vec<_>>();

    for chunk in principals_that_follow_me_vec_chunks {
        let _: () = call::call(
            canister_id_to_send_to,
            "receive_principals_that_follow_me_from_data_backup_canister",
            (chunk.to_vec(),),
        )
        .await
        .expect("Failed to call the receive_principals_that_follow_me_from_data_backup_canister method on the individual user's canister");
    }
}

async fn send_principals_i_follow(users_data: &AllUserData) {
    let canister_id_to_send_to = users_data.user_canister_id;

    let principals_i_follow_vec = users_data
        .canister_data
        .principals_i_follow
        .iter()
        .copied()
        .collect::<Vec<_>>();

    let principals_i_follow_vec_chunks = principals_i_follow_vec
        .chunks(CHUNK_SIZE)
        .collect::<Vec<_>>();

    for chunk in principals_i_follow_vec_chunks {
        let _: () = call::call(
            canister_id_to_send_to,
            "receive_principals_i_follow_from_data_backup_canister",
            (chunk.to_vec(),),
        )
        .await
        .expect("Failed to call the receive_principals_i_follow_from_data_backup_canister method on the individual user's canister");
    }
}

#[deprecated(note = "use send_utility_token_balance_v1 instead")]
async fn send_utility_token_history(users_data: &AllUserData) {
    let canister_id_to_send_to = users_data.user_canister_id;

    let all_utility_token_transactions_vec = users_data
        .canister_data
        .token_data
        .utility_token_transaction_history
        .iter()
        .map(|(id, token_event)| (*id, token_event.clone()))
        .collect::<Vec<_>>();

    let all_utility_token_transactions_chunks = all_utility_token_transactions_vec
        .chunks(CHUNK_SIZE)
        .collect::<Vec<_>>();

    for chunk in all_utility_token_transactions_chunks {
        let _: () = call::call(
            canister_id_to_send_to,
            "receive_my_utility_token_transaction_history_from_data_backup_canister",
            (chunk.to_vec(),),
        )
        .await
        .expect("Failed to call the receive_my_utility_token_transaction_history_from_data_backup_canister method on the individual user's canister");
    }
}


async fn send_utility_token_history_v1(users_data: &AllUserData) {
    let canister_id_to_send_to = users_data.user_canister_id;

    let all_utility_token_transactions_vec = users_data
        .canister_data
        .token_data
        .utility_token_transaction_history
        .iter()
        .map(|(id, token_event)| (*id, token_event.clone()))
        .collect::<Vec<_>>();

    let all_utility_token_transactions_chunks = all_utility_token_transactions_vec
        .chunks(CHUNK_SIZE)
        .collect::<Vec<_>>();

    for chunk in all_utility_token_transactions_chunks {
        let _: () = call::call(
            canister_id_to_send_to,
            "receive_my_utility_token_transaction_history_from_data_backup_canister_v1",
            (chunk.to_vec(),),
        )
        .await
        .expect("Failed to call the receive_my_utility_token_transaction_history_from_data_backup_canister_v1 method on the individual user's canister");
    }
}


#[deprecated(note = "use send_utility_token_balance_v1 instead")]
async fn send_utility_token_balance(users_data: &AllUserData) {
    let canister_id_to_send_to = users_data.user_canister_id;

    let _: () = call::call(
        canister_id_to_send_to,
        "receive_my_utility_token_balance_from_data_backup_canister",
        (users_data.canister_data.token_data.utility_token_balance,),
    )
    .await
    .expect("Failed to call the receive_my_utility_token_balance_from_data_backup_canister method on the individual user's canister");
}

async fn send_utility_token_balance_v1(users_data: &AllUserData) {
    let canister_id_to_send_to = users_data.user_canister_id;

    let _: () = call::call(
        canister_id_to_send_to,
        "receive_my_utility_token_balance_from_data_backup_canister_v1",
        (users_data.canister_data.token_data.utility_token_balance,),
    )
    .await
    .expect("Failed to call the receive_my_utility_token_balance_from_data_backup_canister_v1 method on the individual user's canister");
}

async fn send_posts(users_data: &AllUserData) {
    let canister_id_to_send_to = users_data.user_canister_id;

    let all_created_posts_vec = users_data
        .canister_data
        .all_created_posts
        .values()
        .cloned()
        .collect::<Vec<_>>();

    let all_created_posts_chunks = all_created_posts_vec.chunks(CHUNK_SIZE).collect::<Vec<_>>();

    for chunk in all_created_posts_chunks {
        let _: () = call::call(
            canister_id_to_send_to,
            "receive_my_created_posts_from_data_backup_canister",
            (chunk.to_vec(),),
        )
        .await
        .expect("Failed to call the receive_my_created_posts_from_data_backup_canister method on the individual user's canister");
    }
}
