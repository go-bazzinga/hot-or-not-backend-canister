use std::time::Duration;

use ic_cdk::api::call::{self, ArgDecoderConfig};
use ic_cdk_macros::post_upgrade;
use shared_utils::{
    canister_specific::{
        individual_user_template::types::post::PostDetailsForFrontend,
        post_cache::types::arg::PostCacheInitArgs,
    },
    common::{
        types::{
            top_posts::{post_score_home_index, post_score_index_item::PostScoreIndexItemV1},
            version_details::VersionDetails,
        },
        utils::stable_memory_serializer_deserializer,
    },
};

use crate::{data_model::CanisterData, CANISTER_DATA};

use super::pre_upgrade::BUFFER_SIZE_BYTES;

#[post_upgrade]
fn post_upgrade() {
    restore_data_from_stable_memory();
    save_upgrade_args_to_memory();
    migrate_data();
}

fn restore_data_from_stable_memory() {
    match stable_memory_serializer_deserializer::deserialize_from_stable_memory::<CanisterData>(
        BUFFER_SIZE_BYTES,
    ) {
        Ok(canister_data) => {
            CANISTER_DATA.with(|canister_data_ref_cell| {
                *canister_data_ref_cell.borrow_mut() = canister_data;
            });
        }
        Err(e) => {
            ic_cdk::print(format!("Error: {:?}", e));
            panic!("Failed to restore canister data from stable memory");
        }
    }
}

fn save_upgrade_args_to_memory() {
    let upgrade_args =
        ic_cdk::api::call::arg_data::<(PostCacheInitArgs,)>(ArgDecoderConfig::default()).0;

    CANISTER_DATA.with(|canister_data_ref_cell| {
        let mut canister_data_ref_cell = canister_data_ref_cell.borrow_mut();

        if let Some(upgrade_version_number) = upgrade_args.upgrade_version_number {
            canister_data_ref_cell.version_details.version_number = upgrade_version_number;
        }

        if let Some(known_principal_map) = upgrade_args.known_principal_ids {
            canister_data_ref_cell.known_principal_ids = known_principal_map;
        }

        canister_data_ref_cell.version_details.version = upgrade_args.version;
    });
}

const DELAY_FOR_MIGRATING_DATA: Duration = Duration::from_secs(1);
fn migrate_data() {
    ic_cdk_timers::set_timer(DELAY_FOR_MIGRATING_DATA, || {
        ic_cdk::spawn(migrate_data_impl());
    });
}

async fn migrate_data_impl() {
    // Migrate Home Feed

    let old_post_score_home_index = CANISTER_DATA.with(|canister_data_ref_cell| {
        let canister_data_ref_cell = canister_data_ref_cell.borrow_mut();

        canister_data_ref_cell
            .posts_index_sorted_by_home_feed_score_v1
            .clone()
    });

    for post in old_post_score_home_index.iter() {
        let post_id = post.post_id;
        let publisher_canister_id = post.publisher_canister_id;

        let post_details: PostDetailsForFrontend = match call::call(
            publisher_canister_id,
            "get_individual_post_details_by_id",
            (post_id,),
        )
        .await
        {
            Ok((post_details,)) => post_details,
            Err((rejection_code, err)) => {
                ic_cdk::print(format!(
                    "Error: get_individual_post_details_by_id failed with rejection code: {:?}, error: {}",
                    rejection_code, err
                ));
                continue;
            }
        };

        let new_post = PostScoreIndexItemV1 {
            score: post_details.home_feed_ranking_score,
            post_id: post_details.id,
            publisher_canister_id,
            is_nsfw: post_details.is_nsfw,
            created_at: Some(post_details.created_at),
            status: post_details.status,
        };

        CANISTER_DATA.with(|canister_data_ref_cell| {
            let mut canister_data_ref_cell = canister_data_ref_cell.borrow_mut();
            canister_data_ref_cell
                .posts_index_sorted_by_home_feed_score_v1
                .replace(&new_post);

            // Migrate Yral Feed
            canister_data_ref_cell
                .posts_index_sorted_by_yral_feed_score
                .replace(&new_post);
        });
    }

    // Migrate Hot or Not Feed

    let old_post_score_hot_or_not_index = CANISTER_DATA.with(|canister_data_ref_cell| {
        let canister_data_ref_cell = canister_data_ref_cell.borrow_mut();

        canister_data_ref_cell
            .posts_index_sorted_by_hot_or_not_feed_score_v1
            .clone()
    });

    for post in old_post_score_hot_or_not_index.iter() {
        let post_id = post.post_id;
        let publisher_canister_id = post.publisher_canister_id;

        let post_details: PostDetailsForFrontend = match call::call(
            publisher_canister_id,
            "get_individual_post_details_by_id",
            (post_id,),
        )
        .await
        {
            Ok((post_details,)) => post_details,
            Err((rejection_code, err)) => {
                ic_cdk::print(format!(
                            "Error: get_individual_post_details_by_id failed with rejection code: {:?}, error: {}",
                            rejection_code, err
                        ));
                continue;
            }
        };

        if let Some(hot_or_not_feed_ranking_score) = post_details.hot_or_not_feed_ranking_score {
            let new_post = PostScoreIndexItemV1 {
                score: hot_or_not_feed_ranking_score,
                post_id: post_details.id,
                publisher_canister_id,
                is_nsfw: post_details.is_nsfw,
                created_at: Some(post_details.created_at),
                status: post_details.status,
            };

            CANISTER_DATA.with(|canister_data_ref_cell| {
                let mut canister_data_ref_cell = canister_data_ref_cell.borrow_mut();
                canister_data_ref_cell
                    .posts_index_sorted_by_hot_or_not_feed_score_v1
                    .replace(&new_post);
                // Migrate Yral Feed

                canister_data_ref_cell
                    .posts_index_sorted_by_yral_feed_score
                    .replace(&new_post);
            });
        }
    }

}
