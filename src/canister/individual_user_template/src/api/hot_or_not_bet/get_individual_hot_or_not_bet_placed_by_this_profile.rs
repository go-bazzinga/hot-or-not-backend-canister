use ic_cdk::api::management_canister::provisional::CanisterId;
use ic_cdk_macros::query;

use shared_utils::{
    canister_specific::individual_user_template::types::hot_or_not::{PlacedBetDetail, PlacedBetDetailV1},
    common::types::app_primitive_type::PostId,
};

use crate::{
    api::canister_management::update_last_access_time::update_last_canister_functionality_access_time,
    CANISTER_DATA,
};


#[deprecated(note = "use get_individual_hot_or_not_bet_placed_by_this_profile_v1 instead")]
#[query]
fn get_individual_hot_or_not_bet_placed_by_this_profile(
    canister_id: CanisterId,
    post_id: PostId,
) -> Option<PlacedBetDetail> {
    update_last_canister_functionality_access_time();
    CANISTER_DATA.with(|canister_data_ref_cell| {
        canister_data_ref_cell
            .borrow()
            .all_hot_or_not_bets_placed
            .get(&(canister_id, post_id))
            .cloned()
    })
}

#[query]
fn get_individual_hot_or_not_bet_placed_by_this_profile_v1(
    canister_id: CanisterId,
    post_id: PostId,
) -> Option<PlacedBetDetailV1> {
    update_last_canister_functionality_access_time();
    CANISTER_DATA.with(|canister_data_ref_cell| {
        canister_data_ref_cell
            .borrow()
            .all_hot_or_not_bets_placed_v1
            .get(&(canister_id, post_id))
            .cloned()
    })
}

