use std::time::SystemTime;

use candid::Principal;
use ic_cdk::api::call;
use shared_utils::common::{
    types::{
        known_principal::KnownPrincipalType, top_posts::post_score_index_item::PostScoreIndexItemV1,
    },
    utils::system_time,
};

use crate::{data_model::CanisterData, CANISTER_DATA};

pub fn send_update_post_cache(post_id: &u64) {
    let current_time = system_time::get_current_system_time();
    let canisters_own_principal_id = ic_cdk::id();

    let (home_feed_index_score_item, hot_or_not_index_score_item): (
        Option<PostScoreIndexItemV1>,
        Option<PostScoreIndexItemV1>,
    ) = CANISTER_DATA.with(|canister_data_ref_cell| {
        update_local_cache_get_items(
            &mut canister_data_ref_cell.borrow_mut(),
            *post_id,
            current_time,
            canisters_own_principal_id,
        )
    });

    let post_cache_canister_principal_id = CANISTER_DATA.with(|canister_data_ref_cell| {
        canister_data_ref_cell
            .borrow()
            .known_principal_ids
            .get(&KnownPrincipalType::CanisterIdPostCache)
            .cloned()
            .unwrap()
    });

    if home_feed_index_score_item.is_some() {
        let home_feed_index_score_clone = home_feed_index_score_item.clone();
        let _ = call::notify(
            post_cache_canister_principal_id,
            "update_post_home_feed",
            (home_feed_index_score_clone.unwrap(),),
        );
        let _ = call::notify(
            post_cache_canister_principal_id,
            "update_post_yral_feed",
            (home_feed_index_score_item.unwrap(),),
        );

    }

    if hot_or_not_index_score_item.is_some() {
        let _ = call::notify(
            post_cache_canister_principal_id,
            "update_post_hot_or_not_feed",
            (hot_or_not_index_score_item.unwrap(),),
        );
    }
}

pub fn update_local_cache_get_items(
    canister_data: &mut CanisterData,
    post_id: u64,
    current_time: SystemTime,
    canisters_own_principal_id: Principal,
) -> (Option<PostScoreIndexItemV1>, Option<PostScoreIndexItemV1>) {
    let all_posts = &mut canister_data.all_created_posts;
    if !all_posts.contains_key(&post_id) {
        return (None, None);
    }
    let mut home_feed_index_score_item: Option<PostScoreIndexItemV1> = None;
    let mut hot_or_not_index_score_item: Option<PostScoreIndexItemV1> = None;

    let mut post_to_synchronise = all_posts.get(&post_id).unwrap().clone();

    post_to_synchronise.recalculate_home_feed_score(&current_time);

    let current_home_feed_score = post_to_synchronise.home_feed_score.current_score;

    home_feed_index_score_item = Some(PostScoreIndexItemV1 {
        post_id: post_to_synchronise.id,
        score: current_home_feed_score,
        publisher_canister_id: canisters_own_principal_id,
        is_nsfw: post_to_synchronise.is_nsfw,
        status: post_to_synchronise.status,
        created_at: Some(post_to_synchronise.created_at),
    });
    post_to_synchronise.home_feed_score.last_synchronized_score = current_home_feed_score;
    post_to_synchronise.home_feed_score.last_synchronized_at = current_time;

    if post_to_synchronise.hot_or_not_details.is_some() {
        post_to_synchronise.recalculate_hot_or_not_feed_score(&current_time);
        let current_hot_or_not_feed_score = post_to_synchronise
            .hot_or_not_details
            .as_ref()
            .unwrap()
            .hot_or_not_feed_score
            .current_score;

        hot_or_not_index_score_item = Some(PostScoreIndexItemV1 {
            post_id: post_to_synchronise.id,
            score: current_hot_or_not_feed_score,
            publisher_canister_id: canisters_own_principal_id,
            is_nsfw: post_to_synchronise.is_nsfw,
            status: post_to_synchronise.status,
            created_at: Some(post_to_synchronise.created_at),
        });
        post_to_synchronise
            .hot_or_not_details
            .as_mut()
            .unwrap()
            .hot_or_not_feed_score
            .last_synchronized_score = current_hot_or_not_feed_score;
        post_to_synchronise
            .hot_or_not_details
            .as_mut()
            .unwrap()
            .hot_or_not_feed_score
            .last_synchronized_at = current_time;
    }

    all_posts.insert(post_id, post_to_synchronise);

    (home_feed_index_score_item, hot_or_not_index_score_item)
}
