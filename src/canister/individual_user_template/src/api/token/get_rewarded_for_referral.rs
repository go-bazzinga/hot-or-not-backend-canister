use crate::{
    api::{canister_management::update_last_access_time::update_last_canister_functionality_access_time, referral::coyn_token_reward_for_referral},
    CANISTER_DATA,
};
use candid::Principal;
use ic_cdk_macros::update;
use shared_utils::common::types::known_principal::KnownPrincipalType;

#[deprecated = "use new methods in crate::api::referral"]
#[update]
fn get_rewarded_for_referral(referrer: Principal, referree: Principal) {
    // * access control
    let request_maker = ic_cdk::caller();
    let user_index_canister_principal_id = CANISTER_DATA.with(|canister_data_ref_cell| {
        canister_data_ref_cell
            .borrow()
            .known_principal_ids
            .get(&KnownPrincipalType::CanisterIdUserIndex)
            .cloned()
            .unwrap()
    });

    if user_index_canister_principal_id != request_maker {
        return;
    }

    update_last_canister_functionality_access_time();

    coyn_token_reward_for_referral(referrer, referree);
}
