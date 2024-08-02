use ic_cdk_macros::update;
use shared_utils::common::types::known_principal::KnownPrincipalType;

use crate::CANISTER_DATA;

#[deprecated(note = "use receive_my_utility_token_balance_from_data_backup_canister_v1")]
#[update]
fn receive_my_utility_token_balance_from_data_backup_canister(token_balance: u64) {
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
        canister_data_ref_cell
            .borrow_mut()
            .my_token_balance
            .utility_token_balance = token_balance;
    });
}


#[update]
fn receive_my_utility_token_balance_from_data_backup_canister_v1(token_balance: u64) {
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
        canister_data_ref_cell
            .borrow_mut()
            .my_token_balance_v1
            .utility_token_balance = token_balance;
    });
}