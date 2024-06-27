use ic_cdk_macros::update;
use shared_utils::common::types::top_posts::post_score_index_item::{
    PostScoreIndexItemV1, PostStatus,
};

use crate::{data_model::CanisterData, CANISTER_DATA};

#[update]
fn update_post_yral_feed(post: PostScoreIndexItemV1) {
    let caller = ic_cdk::caller();
    if post.publisher_canister_id != caller {
        return;
    }

    CANISTER_DATA.with(|canister_data| {
        let mut canister_data = canister_data.borrow_mut();

        update_post_yral_feed_impl(post, &mut canister_data);
    });
}

fn update_post_yral_feed_impl(post: PostScoreIndexItemV1, canister_data: &mut CanisterData) {
    let item_prescence_index = &mut canister_data
        .posts_index_sorted_by_yral_feed_score_v1
        .item_presence_index;

    let global_id = (post.publisher_canister_id, post.post_id);
    if let Some(_) = item_prescence_index.get(&global_id) {
        if post.status == PostStatus::BannedDueToUserReporting {
            canister_data
                .posts_index_sorted_by_yral_feed_score_v1
                .remove(&post);
        } else {
            canister_data
                .posts_index_sorted_by_yral_feed_score_v1
                .replace(&post);
        }
    }
}

#[cfg(test)]
mod tests {
    use candid::Principal;
    use shared_utils::common::types::top_posts::post_score_index_item::PostStatus;
    use std::time::SystemTime;

    use super::*;

    #[test]
    fn test_update_post_yral_feed_impl() {
        let mut canister_data = CanisterData::default();
        let created_at_now = SystemTime::now();

        let post = PostScoreIndexItemV1 {
            post_id: 1,
            score: 1,
            publisher_canister_id: Principal::from_text("aaaaa-aa").unwrap(),
            is_nsfw: false,
            status: PostStatus::ReadyToView,
            created_at: Some(created_at_now),
        };

        canister_data
            .posts_index_sorted_by_yral_feed_score_v1
            .replace(&post);

        let new_post = PostScoreIndexItemV1 {
            post_id: 1,
            score: 10,
            publisher_canister_id: Principal::from_text("aaaaa-aa").unwrap(),
            is_nsfw: true,
            status: PostStatus::ReadyToView,
            created_at: Some(created_at_now),
        };

        update_post_yral_feed_impl(new_post, &mut canister_data);

        let iter_posts = canister_data
            .posts_index_sorted_by_yral_feed_score_v1
            .iter()
            .collect::<Vec<_>>();

        assert_eq!(iter_posts.len(), 1);
        assert_eq!(iter_posts[0].post_id, 1);
        assert_eq!(iter_posts[0].score, 10);
        assert_eq!(
            iter_posts[0].publisher_canister_id,
            Principal::from_text("aaaaa-aa").unwrap()
        );
        assert_eq!(iter_posts[0].is_nsfw, true);
        assert_eq!(iter_posts[0].status, PostStatus::ReadyToView);
        assert_eq!(iter_posts[0].created_at, Some(created_at_now));
    }

    #[test]
    fn test_update_post_yral_feed_impl_banned() {
        let mut canister_data = CanisterData::default();
        let created_at_now = SystemTime::now();

        let post = PostScoreIndexItemV1 {
            post_id: 1,
            score: 1,
            publisher_canister_id: Principal::from_text("aaaaa-aa").unwrap(),
            is_nsfw: false,
            status: PostStatus::ReadyToView,
            created_at: Some(created_at_now),
        };

        canister_data
            .posts_index_sorted_by_yral_feed_score_v1
            .replace(&post);

        let new_post = PostScoreIndexItemV1 {
            post_id: 1,
            score: 10,
            publisher_canister_id: Principal::from_text("aaaaa-aa").unwrap(),
            is_nsfw: true,
            status: PostStatus::BannedDueToUserReporting,
            created_at: Some(created_at_now),
        };

        update_post_yral_feed_impl(new_post, &mut canister_data);

        let iter_posts = canister_data
            .posts_index_sorted_by_yral_feed_score_v1
            .iter()
            .collect::<Vec<_>>();

        assert_eq!(iter_posts.len(), 0);
    }
}
