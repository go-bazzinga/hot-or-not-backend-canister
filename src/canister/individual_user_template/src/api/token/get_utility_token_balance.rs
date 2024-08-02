use ic_cdk_macros::query;

use crate::{
    api::canister_management::update_last_access_time::update_last_canister_functionality_access_time,
    CANISTER_DATA,
};

#[deprecated(note = "Please use get_utility_token_balance_v1")]
#[query]
fn get_utility_token_balance() -> u64 {
    update_last_canister_functionality_access_time();
    CANISTER_DATA.with(|canister_data_ref_cell| {
        canister_data_ref_cell
            .borrow()
            .my_token_balance
            .utility_token_balance
    })
}

#[query]
fn get_utility_token_balance_v1() -> u64 {
    update_last_canister_functionality_access_time();
    CANISTER_DATA.with(|canister_data_ref_cell| {
        canister_data_ref_cell
            .borrow()
            .my_token_balance_v1
            .utility_token_balance
    })
}
