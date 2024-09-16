use std::time::{Duration, SystemTime};

use ic_cdk_macros::update;
use shared_utils::{
    canister_specific::individual_user_template::types::post::{Post, PostDetailsFromFrontend},
    common::utils::system_time,
};

use crate::{
    api::{
        canister_management::update_last_access_time::update_last_canister_functionality_access_time,
        hot_or_not_bet::tabulate_hot_or_not_outcome_for_post_slot::tabulate_hot_or_not_outcome_for_post_slot,
    },
    data_model::CanisterData,
    CANISTER_DATA,
};

use super::update_scores_and_share_with_post_cache_if_difference_beyond_threshold::update_scores_and_share_with_post_cache_if_difference_beyond_threshold;

/// #### Access Control
/// Only the user whose profile details are stored in this canister can create a post.
#[update]
fn add_post_v2(post_details: PostDetailsFromFrontend) -> Result<u64, String> {
    // * access control
    let current_caller = ic_cdk::caller();
    let my_principal_id = CANISTER_DATA
        .with(|canister_data_ref_cell| canister_data_ref_cell.borrow().profile.principal_id);
    if my_principal_id != Some(current_caller) {
        return Err(
            "Only the user whose profile details are stored in this canister can create a post."
                .to_string(),
        );
    };

    update_last_canister_functionality_access_time();

    let response = CANISTER_DATA.with(|canister_data_ref_cell| {
        add_post_to_memory(
            &mut canister_data_ref_cell.borrow_mut(),
            &post_details,
            &system_time::get_current_system_time_from_ic(),
        )
    });

    let post_id = response;

    update_scores_and_share_with_post_cache_if_difference_beyond_threshold(&post_id);

    // if post_details.creator_consent_for_inclusion_in_hot_or_not {
        // * schedule hot_or_not outcome tabulation for the 48 hours after the post is created
        // (1..=48).for_each(|slot_number: u8| {
        //     ic_cdk_timers::set_timer(
        //         Duration::from_secs(slot_number as u64 * 60 * 60),
        //         move || {
        //             CANISTER_DATA.with(|canister_data_ref_cell| {
        //                 tabulate_hot_or_not_outcome_for_post_slot(
        //                     &mut canister_data_ref_cell.borrow_mut(),
        //                     post_id,
        //                     slot_number,
        //                 );
        //             });
        //         },
        //     );
        // });
    // }

    Ok(post_id)
}

pub fn add_post_to_memory(
    canister_data: &mut CanisterData,
    post_details: &PostDetailsFromFrontend,
    current_system_time: &SystemTime,
) -> u64 {
    let new_post = Post::new(
        canister_data.all_created_posts.len() as u64,
        post_details,
        current_system_time,
    );
    let new_post_id = new_post.id;
    canister_data
        .all_created_posts
        .insert(new_post.id, new_post);

    new_post_id
}
