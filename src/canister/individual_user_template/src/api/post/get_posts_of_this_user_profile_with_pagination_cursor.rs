use std::time::SystemTime;

use candid::Principal;
use ic_cdk_macros::query;
use shared_utils::{
    canister_specific::individual_user_template::types::{
        error::GetPostsOfUserProfileError, post::{PostDetailsForFrontend, PostDetailsForFrontendV1},
        profile::UserProfileDetailsForFrontend,
    },
    common::{types::top_posts::post_score_index_item::PostStatus, utils::system_time},
    pagination::{self, PaginationError},
};

use crate::{
    api::canister_management::update_last_access_time::update_last_canister_functionality_access_time,
    data_model::CanisterData, CANISTER_DATA,
};

#[query]
fn get_posts_of_this_user_profile_with_pagination_cursor(
    from_inclusive_index: u64,
    limit: u64,
) -> Result<Vec<PostDetailsForFrontend>, GetPostsOfUserProfileError> {
    update_last_canister_functionality_access_time();

    let api_caller = ic_cdk::caller();
    let current_time = system_time::get_current_system_time_from_ic();
    CANISTER_DATA.with_borrow(|canister_data| {
        get_posts_of_this_user_profile_with_pagination_cursor_impl(
            from_inclusive_index,
            limit,
            canister_data,
            api_caller,
            current_time,
        )
    })
}

#[deprecated(note = "use get_posts_of_this_user_profile_with_pagination_cursor_impl_v1 instead")]
fn get_posts_of_this_user_profile_with_pagination_cursor_impl(
    from_inclusive_index: u64,
    limit: u64,
    canister_data: &CanisterData,
    api_caller: Principal,
    current_time: SystemTime,
) -> Result<Vec<PostDetailsForFrontend>, GetPostsOfUserProfileError> {
    let (from_inclusive_index, limit) = pagination::get_pagination_bounds_cursor(
        from_inclusive_index,
        limit,
        canister_data.all_created_posts.len() as u64,
    )
    .map_err(|e| match e {
        PaginationError::InvalidBoundsPassed => GetPostsOfUserProfileError::InvalidBoundsPassed,
        PaginationError::ReachedEndOfItemsList => GetPostsOfUserProfileError::ReachedEndOfItemsList,
        PaginationError::ExceededMaxNumberOfItemsAllowedInOneRequest => {
            GetPostsOfUserProfileError::ExceededMaxNumberOfItemsAllowedInOneRequest
        }
    })?;

    let res_posts = canister_data
        .all_created_posts
        .iter()
        .filter(|(_, post)| post.status != PostStatus::BannedDueToUserReporting)
        .rev()
        .skip(from_inclusive_index as usize)
        .take(limit as usize)
        .map(|(id, post)| {
            let profile = &canister_data.profile;
            let followers = &canister_data.principals_that_follow_me;
            let following = &canister_data.principals_i_follow;
            let token_balance = &canister_data.my_token_balance;

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
                &current_time,
                &canister_data.room_details_map,
                &canister_data.post_principal_map,
                &canister_data.slot_details_map,
            )
        })
        .collect();

    Ok(res_posts)
}


fn get_posts_of_this_user_profile_with_pagination_cursor_impl_v1(
    from_inclusive_index: u64,
    limit: u64,
    canister_data: &CanisterData,
    api_caller: Principal,
    current_time: SystemTime,
) -> Result<Vec<PostDetailsForFrontendV1>, GetPostsOfUserProfileError> {
    let (from_inclusive_index, limit) = pagination::get_pagination_bounds_cursor(
        from_inclusive_index,
        limit,
        canister_data.all_created_posts.len() as u64,
    )
    .map_err(|e| match e {
        PaginationError::InvalidBoundsPassed => GetPostsOfUserProfileError::InvalidBoundsPassed,
        PaginationError::ReachedEndOfItemsList => GetPostsOfUserProfileError::ReachedEndOfItemsList,
        PaginationError::ExceededMaxNumberOfItemsAllowedInOneRequest => {
            GetPostsOfUserProfileError::ExceededMaxNumberOfItemsAllowedInOneRequest
        }
    })?;

    let res_posts = canister_data
        .all_created_posts
        .iter()
        .filter(|(_, post)| post.status != PostStatus::BannedDueToUserReporting)
        .rev()
        .skip(from_inclusive_index as usize)
        .take(limit as usize)
        .map(|(id, post)| {
            let profile = &canister_data.profile;
            let followers = &canister_data.principals_that_follow_me;
            let following = &canister_data.principals_i_follow;
            let token_balance = &canister_data.my_token_balance_v1;

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
                &current_time,
                &canister_data.room_details_map_v1,
                &canister_data.post_principal_map,
                &canister_data.slot_details_map_v1,
            )
        })
        .collect();

    Ok(res_posts)
}

#[cfg(test)]
mod test {
    use std::{
        collections::{BTreeMap, HashSet},
        time::SystemTime,
    };

    use shared_utils::{
        canister_specific::individual_user_template::types::post::{
            FeedScore, Post, PostViewStatistics,
        },
        common::types::top_posts::post_score_index_item::PostStatus,
    };

    use super::*;

    #[test]
    fn test_get_posts_of_this_user_profile_with_pagination_cursor_impl() {
        let mut canister_data = CanisterData::default();
        canister_data.profile.principal_id = Some(Principal::anonymous());

        let posts = vec![
            Post {
                id: 1,
                description: "test post".into(),
                hashtags: Vec::new(),
                video_uid: String::from(""),
                status: PostStatus::ReadyToView,
                created_at: SystemTime::now(),
                likes: HashSet::new(),
                share_count: 0,
                view_stats: PostViewStatistics::default(),
                home_feed_score: FeedScore::default(),
                creator_consent_for_inclusion_in_hot_or_not: true,
                hot_or_not_details: None,
                is_nsfw: false,
            },
            Post {
                id: 2,
                description: "test post".into(),
                hashtags: Vec::new(),
                video_uid: String::from(""),
                status: PostStatus::ReadyToView,
                created_at: SystemTime::now(),
                likes: HashSet::new(),
                share_count: 0,
                view_stats: PostViewStatistics::default(),
                home_feed_score: FeedScore::default(),
                creator_consent_for_inclusion_in_hot_or_not: true,
                hot_or_not_details: None,
                is_nsfw: false,
            },
            Post {
                id: 3,
                description: "test post".into(),
                hashtags: Vec::new(),
                video_uid: String::from(""),
                status: PostStatus::ReadyToView,
                created_at: SystemTime::now(),
                likes: HashSet::new(),
                share_count: 0,
                view_stats: PostViewStatistics::default(),
                home_feed_score: FeedScore::default(),
                creator_consent_for_inclusion_in_hot_or_not: true,
                hot_or_not_details: None,
                is_nsfw: false,
            },
            Post {
                id: 4,
                description: "test post".into(),
                hashtags: Vec::new(),
                video_uid: String::from(""),
                status: PostStatus::ReadyToView,
                created_at: SystemTime::now(),
                likes: HashSet::new(),
                share_count: 0,
                view_stats: PostViewStatistics::default(),
                home_feed_score: FeedScore::default(),
                creator_consent_for_inclusion_in_hot_or_not: true,
                hot_or_not_details: None,
                is_nsfw: false,
            },
            Post {
                id: 5,
                description: "test post".into(),
                hashtags: Vec::new(),
                video_uid: String::from(""),
                status: PostStatus::BannedDueToUserReporting,
                created_at: SystemTime::now(),
                likes: HashSet::new(),
                share_count: 0,
                view_stats: PostViewStatistics::default(),
                home_feed_score: FeedScore::default(),
                creator_consent_for_inclusion_in_hot_or_not: true,
                hot_or_not_details: None,
                is_nsfw: false,
            },
            Post {
                id: 6,
                description: "test post".into(),
                hashtags: Vec::new(),
                video_uid: String::from(""),
                status: PostStatus::ReadyToView,
                created_at: SystemTime::now(),
                likes: HashSet::new(),
                share_count: 0,
                view_stats: PostViewStatistics::default(),
                home_feed_score: FeedScore::default(),
                creator_consent_for_inclusion_in_hot_or_not: true,
                hot_or_not_details: None,
                is_nsfw: false,
            },
        ];

        let all_created_posts: BTreeMap<u64, Post> =
            posts.into_iter().map(|post| (post.id, post)).collect();
        let api_caller = Principal::anonymous();
        let current_time = SystemTime::now();

        canister_data.all_created_posts = all_created_posts;

        // Test with NSFW filter
        let result = super::get_posts_of_this_user_profile_with_pagination_cursor_impl(
            0,
            3,
            &canister_data,
            api_caller,
            current_time,
        );

        let posts = result.unwrap();
        assert_eq!(posts.len(), 3);
        assert_eq!(posts[0].id, 6);
        assert_eq!(posts[1].id, 4);
        assert_eq!(posts[2].id, 3);

        // Test with NSFW filter
        let result = super::get_posts_of_this_user_profile_with_pagination_cursor_impl(
            4,
            1,
            &canister_data,
            api_caller,
            current_time,
        );

        assert!(result.is_ok());
        let posts = result.unwrap();
        assert_eq!(posts.len(), 1);
        assert_eq!(posts[0].id, 1);
    }
}


#[cfg(test)]
mod test_v1 {
    use std::{
        collections::{BTreeMap, HashSet},
        time::SystemTime,
    };

    use shared_utils::{
        canister_specific::individual_user_template::types::post::{
            FeedScore, Post, PostViewStatistics,
        },
        common::types::top_posts::post_score_index_item::PostStatus,
    };

    use super::*;

    #[test]
    fn test_get_posts_of_this_user_profile_with_pagination_cursor_impl_v1() {
        let mut canister_data = CanisterData::default();
        canister_data.profile.principal_id = Some(Principal::anonymous());

        let posts = vec![
            Post {
                id: 1,
                description: "test post".into(),
                hashtags: Vec::new(),
                video_uid: String::from(""),
                status: PostStatus::ReadyToView,
                created_at: SystemTime::now(),
                likes: HashSet::new(),
                share_count: 0,
                view_stats: PostViewStatistics::default(),
                home_feed_score: FeedScore::default(),
                creator_consent_for_inclusion_in_hot_or_not: true,
                hot_or_not_details: None,
                is_nsfw: false,
            },
            Post {
                id: 2,
                description: "test post".into(),
                hashtags: Vec::new(),
                video_uid: String::from(""),
                status: PostStatus::ReadyToView,
                created_at: SystemTime::now(),
                likes: HashSet::new(),
                share_count: 0,
                view_stats: PostViewStatistics::default(),
                home_feed_score: FeedScore::default(),
                creator_consent_for_inclusion_in_hot_or_not: true,
                hot_or_not_details: None,
                is_nsfw: false,
            },
            Post {
                id: 3,
                description: "test post".into(),
                hashtags: Vec::new(),
                video_uid: String::from(""),
                status: PostStatus::ReadyToView,
                created_at: SystemTime::now(),
                likes: HashSet::new(),
                share_count: 0,
                view_stats: PostViewStatistics::default(),
                home_feed_score: FeedScore::default(),
                creator_consent_for_inclusion_in_hot_or_not: true,
                hot_or_not_details: None,
                is_nsfw: false,
            },
            Post {
                id: 4,
                description: "test post".into(),
                hashtags: Vec::new(),
                video_uid: String::from(""),
                status: PostStatus::ReadyToView,
                created_at: SystemTime::now(),
                likes: HashSet::new(),
                share_count: 0,
                view_stats: PostViewStatistics::default(),
                home_feed_score: FeedScore::default(),
                creator_consent_for_inclusion_in_hot_or_not: true,
                hot_or_not_details: None,
                is_nsfw: false,
            },
            Post {
                id: 5,
                description: "test post".into(),
                hashtags: Vec::new(),
                video_uid: String::from(""),
                status: PostStatus::BannedDueToUserReporting,
                created_at: SystemTime::now(),
                likes: HashSet::new(),
                share_count: 0,
                view_stats: PostViewStatistics::default(),
                home_feed_score: FeedScore::default(),
                creator_consent_for_inclusion_in_hot_or_not: true,
                hot_or_not_details: None,
                is_nsfw: false,
            },
            Post {
                id: 6,
                description: "test post".into(),
                hashtags: Vec::new(),
                video_uid: String::from(""),
                status: PostStatus::ReadyToView,
                created_at: SystemTime::now(),
                likes: HashSet::new(),
                share_count: 0,
                view_stats: PostViewStatistics::default(),
                home_feed_score: FeedScore::default(),
                creator_consent_for_inclusion_in_hot_or_not: true,
                hot_or_not_details: None,
                is_nsfw: false,
            },
        ];

        let all_created_posts: BTreeMap<u64, Post> =
            posts.into_iter().map(|post| (post.id, post)).collect();
        let api_caller = Principal::anonymous();
        let current_time = SystemTime::now();

        canister_data.all_created_posts = all_created_posts;

        // Test with NSFW filter
        let result = super::get_posts_of_this_user_profile_with_pagination_cursor_impl_v1(
            0,
            3,
            &canister_data,
            api_caller,
            current_time,
        );

        let posts = result.unwrap();
        assert_eq!(posts.len(), 3);
        assert_eq!(posts[0].id, 6);
        assert_eq!(posts[1].id, 4);
        assert_eq!(posts[2].id, 3);

        // Test with NSFW filter
        let result = super::get_posts_of_this_user_profile_with_pagination_cursor_impl_v1(
            4,
            1,
            &canister_data,
            api_caller,
            current_time,
        );

        assert!(result.is_ok());
        let posts = result.unwrap();
        assert_eq!(posts.len(), 1);
        assert_eq!(posts[0].id, 1);
    }
}
