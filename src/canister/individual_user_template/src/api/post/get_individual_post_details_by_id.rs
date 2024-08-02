use ic_cdk_macros::query;
use shared_utils::{
    canister_specific::individual_user_template::types::{
        post::{PostDetailsForFrontend, PostDetailsForFrontendV1}, profile::UserProfileDetailsForFrontend,
    },
    common::utils::system_time,
};

use crate::{
    api::canister_management::update_last_access_time::update_last_canister_functionality_access_time,
    CANISTER_DATA,
};

#[deprecated(note = "use get_individual_post_details_by_id_v1 instead")]
#[query]
pub fn get_individual_post_details_by_id(post_id: u64) -> PostDetailsForFrontend {
    let api_caller = ic_cdk::caller();

    update_last_canister_functionality_access_time();

    CANISTER_DATA.with(|canister_data_ref_cell| {
        let post = canister_data_ref_cell
            .borrow()
            .all_created_posts
            .get(&post_id)
            .unwrap()
            .clone();
        let profile = &canister_data_ref_cell.borrow().profile;
        let followers = &canister_data_ref_cell.borrow().principals_that_follow_me;
        let following = &canister_data_ref_cell.borrow().principals_i_follow;
        let token_balance = &canister_data_ref_cell.borrow().my_token_balance;

        post.get_post_details_for_frontend_for_this_post(
            UserProfileDetailsForFrontend {
                display_name: profile.display_name.clone(),
                followers_count: followers.len() as u64,
                following_count: following.len() as u64,
                principal_id: profile.principal_id.unwrap(),
                profile_picture_url: profile.profile_picture_url.clone(),
                profile_stats: profile.profile_stats,
                unique_user_name: profile.unique_user_name.clone(),
                lifetime_earnings: token_balance.lifetime_earnings,
                referrer_details: profile.referrer_details.clone(),
            },
            api_caller,
            &system_time::get_current_system_time_from_ic(),
            &canister_data_ref_cell.borrow().room_details_map,
            &canister_data_ref_cell.borrow().post_principal_map,
            &canister_data_ref_cell.borrow().slot_details_map,
        )
    })
}

#[query]
pub fn get_individual_post_details_by_id_v1(post_id: u64) -> PostDetailsForFrontendV1 {
    let api_caller = ic_cdk::caller();

    update_last_canister_functionality_access_time();

    CANISTER_DATA.with(|canister_data_ref_cell| {
        let post = canister_data_ref_cell
            .borrow()
            .all_created_posts
            .get(&post_id)
            .unwrap()
            .clone();
        let profile = &canister_data_ref_cell.borrow().profile;
        let followers = &canister_data_ref_cell.borrow().principals_that_follow_me;
        let following = &canister_data_ref_cell.borrow().principals_i_follow;
        let token_balance = &canister_data_ref_cell.borrow().my_token_balance_v1;

        post.get_post_details_for_frontend_for_this_post_v1(
            UserProfileDetailsForFrontend {
                display_name: profile.display_name.clone(),
                followers_count: followers.len() as u64,
                following_count: following.len() as u64,
                principal_id: profile.principal_id.unwrap(),
                profile_picture_url: profile.profile_picture_url.clone(),
                profile_stats: profile.profile_stats,
                unique_user_name: profile.unique_user_name.clone(),
                lifetime_earnings: token_balance.lifetime_earnings,
                referrer_details: profile.referrer_details.clone(),
            },
            api_caller,
            &system_time::get_current_system_time_from_ic(),
            &canister_data_ref_cell.borrow().room_details_map_v1,
            &canister_data_ref_cell.borrow().post_principal_map,
            &canister_data_ref_cell.borrow().slot_details_map_v1,
        )
    })
}
