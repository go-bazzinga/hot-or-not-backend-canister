use ic_cdk_macros::update;
use shared_utils::common::types::{
    known_principal::KnownPrincipalType, utility_token::token_event::{TokenEvent, TokenEventV1},
};

use crate::CANISTER_DATA;

#[deprecated(
    note = "use receive_my_utility_token_transaction_history_from_data_backup_canister_v1"
)]
#[update]
fn receive_my_utility_token_transaction_history_from_data_backup_canister(
    all_token_events_chunk_vec: Vec<(u64, TokenEvent)>,
) {
    let caller = ic_cdk::caller();
    let data_backup_canister_id = CANISTER_DATA.with(|canister_data_ref_cell| {
        canister_data_ref_cell
            .borrow()
            .known_principal_ids
            .get(&KnownPrincipalType::CanisterIdDataBackup)
            .cloned()
            .unwrap()
    });

    if caller != data_backup_canister_id {
        return;
    }

    CANISTER_DATA.with(|canister_data_ref_cell| {
        let mut canister_data = canister_data_ref_cell.borrow_mut();

        for (id, token_event) in all_token_events_chunk_vec {
            canister_data
                .my_token_balance
                .utility_token_transaction_history
                .insert(id, token_event);
        }
    });
}

#[update]
fn receive_my_utility_token_transaction_history_from_data_backup_canister_v1(
    all_token_events_chunk_vec: Vec<(u64, TokenEventV1)>,
) {
    let caller = ic_cdk::caller();
    let data_backup_canister_id = CANISTER_DATA.with(|canister_data_ref_cell| {
        canister_data_ref_cell
            .borrow()
            .known_principal_ids
            .get(&KnownPrincipalType::CanisterIdDataBackup)
            .cloned()
            .unwrap()
    });

    if caller != data_backup_canister_id {
        return;
    }

    CANISTER_DATA.with(|canister_data_ref_cell| {
        let mut canister_data = canister_data_ref_cell.borrow_mut();

        for (id, token_event) in all_token_events_chunk_vec {
            canister_data
                .my_token_balance_v1
                .utility_token_transaction_history
                .insert(id, token_event);
        }
    });
}
