use ic_cdk_macros::query;
use std::time::SystemTime;

use candid::Principal;
use shared_utils::{
    canister_specific::individual_user_template::types::hot_or_not::{BettingStatus, BettingStatusV1},
    common::utils::system_time::{self},
};

use crate::{
    api::canister_management::update_last_access_time::update_last_canister_functionality_access_time,
    data_model::CanisterData, CANISTER_DATA,
};

// #[ic_cdk::query]
// #[candid::candid_method(query)]
// fn get_hot_or_not_bet_details_for_this_post_old(post_id: u64) -> BettingStatus {
//     let request_maker = ic_cdk::caller();

//     CANISTER_DATA.with(|canister_data_ref_cell| {
//         get_hot_or_not_bet_details_for_this_post_impl_old(
//             &canister_data_ref_cell.borrow(),
//             &system_time::get_current_system_time_from_ic(),
//             &request_maker,
//             post_id,
//         )
//     })
// }

// fn get_hot_or_not_bet_details_for_this_post_impl_old(
//     canister_data: &CanisterData,
//     current_time: &SystemTime,
//     request_maker: &Principal,
//     post_id: u64,
// ) -> BettingStatus {
//     canister_data
//         .all_created_posts
//         .get(&post_id)
//         .unwrap()
//         .get_hot_or_not_betting_status_for_this_post(current_time, request_maker)
// }

#[deprecated(note = "use get_hot_or_not_bet_details_for_this_post_v2 instead")]
#[query]
fn get_hot_or_not_bet_details_for_this_post(post_id: u64) -> BettingStatus {
    let request_maker = ic_cdk::caller();
    update_last_canister_functionality_access_time();

    CANISTER_DATA.with(|canister_data_ref_cell| {
        get_hot_or_not_bet_details_for_this_post_impl(
            &canister_data_ref_cell.borrow(),
            &system_time::get_current_system_time_from_ic(),
            &request_maker,
            post_id,
        )
    })
}

#[deprecated(note = "use get_hot_or_not_bet_details_for_this_post_impl_v2 instead")]
fn get_hot_or_not_bet_details_for_this_post_impl(
    canister_data: &CanisterData,
    current_time: &SystemTime,
    request_maker: &Principal,
    post_id: u64,
) -> BettingStatus {
    let post = canister_data.all_created_posts.get(&post_id).unwrap();

     post.get_hot_or_not_betting_status_for_this_post_v1(
        current_time,
        request_maker,
        &canister_data.room_details_map,
        &canister_data.post_principal_map,
        &canister_data.slot_details_map,
    )
}

#[query]
fn get_hot_or_not_bet_details_for_this_post_v2(post_id: u64) -> BettingStatusV1 {
    let request_maker = ic_cdk::caller();
    update_last_canister_functionality_access_time();

    CANISTER_DATA.with(|canister_data_ref_cell| {
        get_hot_or_not_bet_details_for_this_post_impl_v2(
            &canister_data_ref_cell.borrow(),
            &system_time::get_current_system_time_from_ic(),
            &request_maker,
            post_id,
        )
    })
}

fn get_hot_or_not_bet_details_for_this_post_impl_v2(
    canister_data: &CanisterData,
    current_time: &SystemTime,
    request_maker: &Principal,
    post_id: u64,
) -> BettingStatusV1 {
    let post = canister_data.all_created_posts.get(&post_id).unwrap();

    post.get_hot_or_not_betting_status_for_this_post_v2(
        current_time,
        request_maker,
        &canister_data.room_details_map_v1,
        &canister_data.post_principal_map,
        &canister_data.slot_details_map_v1,
    )
}


#[cfg(test)]
mod test_v2 {
    use std::{
        collections::HashSet,
        time::{Duration, SystemTime},
    };

    use shared_utils::{
        canister_specific::individual_user_template::types::{
            hot_or_not::HotOrNotDetails,
            post::{FeedScore, Post, PostViewStatistics},
        },
        common::types::top_posts::post_score_index_item::PostStatus,
    };

    use super::*;

    #[test]
    fn test_get_hot_or_not_bet_details_for_this_post_impl() {
        let mut canister_data = CanisterData::default();
        let post_id = 0;

        canister_data.all_created_posts.insert(
            0,
            Post {
                id: 0,
                is_nsfw: false,
                description: "Singing and dancing".to_string(),
                hashtags: vec!["sing".to_string(), "dance".to_string()],
                video_uid: "video#0001".to_string(),
                status: PostStatus::ReadyToView,
                created_at: SystemTime::now(),
                likes: HashSet::new(),
                share_count: 0,
                view_stats: PostViewStatistics::default(),
                home_feed_score: FeedScore::default(),
                creator_consent_for_inclusion_in_hot_or_not: true,
                hot_or_not_details: Some(HotOrNotDetails::default()),
            },
        );

        let result = get_hot_or_not_bet_details_for_this_post_impl_v2(
            &canister_data,
            &SystemTime::now(),
            &Principal::anonymous(),
            post_id,
        );
        match result {
            BettingStatusV1::BettingOpen { .. } => {}
            _ => panic!("Expected BettingStatus::BettingOpen"),
        }

        let result = get_hot_or_not_bet_details_for_this_post_impl_v2(
            &canister_data,
            &SystemTime::now()
                .checked_add(Duration::from_secs(60 * 60 * 48 + 10))
                .unwrap(),
            &Principal::anonymous(),
            post_id,
        );
        match result {
            BettingStatusV1::BettingOpen { started_at, number_of_participants, ongoing_slot, ongoing_room, has_this_user_participated_in_this_post } => {}
            _ => panic!("Expected BettingStatus::BettingClosed"),
        }
    }
}


#[cfg(test)]
mod test {
    use std::{
        collections::HashSet,
        time::{Duration, SystemTime},
    };

    use shared_utils::{
        canister_specific::individual_user_template::types::{
            hot_or_not::HotOrNotDetails,
            post::{FeedScore, Post, PostViewStatistics},
        },
        common::types::top_posts::post_score_index_item::PostStatus,
    };

    use super::*;

    #[test]
    fn test_get_hot_or_not_bet_details_for_this_post_impl() {
        let mut canister_data = CanisterData::default();
        let post_id = 0;

        canister_data.all_created_posts.insert(
            0,
            Post {
                id: 0,
                is_nsfw: false,
                description: "Singing and dancing".to_string(),
                hashtags: vec!["sing".to_string(), "dance".to_string()],
                video_uid: "video#0001".to_string(),
                status: PostStatus::ReadyToView,
                created_at: SystemTime::now(),
                likes: HashSet::new(),
                share_count: 0,
                view_stats: PostViewStatistics::default(),
                home_feed_score: FeedScore::default(),
                creator_consent_for_inclusion_in_hot_or_not: true,
                hot_or_not_details: Some(HotOrNotDetails::default()),
            },
        );

        let result = get_hot_or_not_bet_details_for_this_post_impl(
            &canister_data,
            &SystemTime::now(),
            &Principal::anonymous(),
            post_id,
        );
        match result {
            BettingStatus::BettingOpen { .. } => {}
            _ => panic!("Expected BettingStatus::BettingOpen"),
        }

        let result = get_hot_or_not_bet_details_for_this_post_impl(
            &canister_data,
            &SystemTime::now()
                .checked_add(Duration::from_secs(60 * 60 * 48 + 10))
                .unwrap(),
            &Principal::anonymous(),
            post_id,
        );
        match result {
            BettingStatus::BettingClosed => {}
            _ => panic!("Expected BettingStatus::BettingClosed"),
        }
    }
}
