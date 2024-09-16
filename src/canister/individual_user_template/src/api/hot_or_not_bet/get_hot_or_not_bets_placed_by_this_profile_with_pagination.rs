use ic_cdk_macros::query;
use shared_utils::canister_specific::individual_user_template::types::hot_or_not::{PlacedBetDetail, PlacedBetDetailV1};

use crate::{
    api::canister_management::update_last_access_time::update_last_canister_functionality_access_time,
    CANISTER_DATA,
};

const PAGINATION_PAGE_SIZE: usize = 10;

#[deprecated(note = "use get_hot_or_not_bets_placed_by_this_profile_with_pagination_v1 instead")]
#[query]
fn get_hot_or_not_bets_placed_by_this_profile_with_pagination(
    last_index_sent: usize,
) -> Vec<PlacedBetDetail> {
    update_last_canister_functionality_access_time();
    CANISTER_DATA.with(|canister_data_ref_cell| {
        canister_data_ref_cell
            .borrow()
            .all_hot_or_not_bets_placed
            .iter()
            .skip(last_index_sent)
            .take(PAGINATION_PAGE_SIZE)
            .map(|(_, placed_bet_detail)| placed_bet_detail.clone())
            .collect()
    })
}

#[query]
fn get_hot_or_not_bets_placed_by_this_profile_with_pagination_v1(
    last_index_sent: usize,
) -> Vec<PlacedBetDetailV1> {
    update_last_canister_functionality_access_time();
    CANISTER_DATA.with(|canister_data_ref_cell| {
        canister_data_ref_cell
            .borrow()
            .all_hot_or_not_bets_placed_v1
            .iter()
            .skip(last_index_sent)
            .take(PAGINATION_PAGE_SIZE)
            .map(|(_, placed_bet_detail)| placed_bet_detail.clone())
            .collect()
    })
}

