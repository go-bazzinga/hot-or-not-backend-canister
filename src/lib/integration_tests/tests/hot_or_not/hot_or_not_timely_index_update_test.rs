use std::{collections::HashMap, time::Duration};

use candid::encode_one;
use pocket_ic::{PocketIc, WasmResult};
use shared_utils::{
    canister_specific::{
        individual_user_template::types::{
            arg::IndividualUserTemplateInitArgs,
            post::{Post, PostDetailsFromFrontend},
        },
        post_cache::types::arg::PostCacheInitArgs,
    },
    common::types::{
        known_principal::KnownPrincipalType,
        top_posts::post_score_index_item::{PostScoreIndexItemV1, PostStatus},
    },
    types::canister_specific::post_cache::error_types::TopPostsFetchError,
};
use test_utils::setup::test_constants::{
    get_mock_user_alice_principal_id, get_mock_user_bob_principal_id,
    get_mock_user_charlie_principal_id, get_mock_user_dan_principal_id,
};

const POST_CACHE_WASM_PATH: &str =
    "../../../target/wasm32-unknown-unknown/release/post_cache.wasm.gz";
const INDIVIDUAL_TEMPLATE_WASM_PATH: &str =
    "../../../target/wasm32-unknown-unknown/release/individual_user_template.wasm.gz";

#[test]
#[ignore]
fn hot_or_not_timely_update_test() {
    let pic = PocketIc::new();

    let alice_principal_id = get_mock_user_alice_principal_id();
    let bob_principal_id = get_mock_user_bob_principal_id();
    let admin_principal_id = get_mock_user_charlie_principal_id();

    // Init post cache canister

    let post_cache_canister_id = pic.create_canister();
    pic.add_cycles(post_cache_canister_id, 2_000_000_000_000);

    let post_cache_wasm_bytes = post_cache_canister_wasm();
    let post_cache_args = PostCacheInitArgs {
        known_principal_ids: None,
        upgrade_version_number: None,
        version: "1".to_string(),
    };
    let post_cache_args_bytes = encode_one(post_cache_args).unwrap();

    pic.install_canister(
        post_cache_canister_id,
        post_cache_wasm_bytes,
        post_cache_args_bytes,
        None,
    );

    // Individual template canisters

    let individual_template_wasm_bytes = individual_template_canister_wasm();
    let mut known_prinicipal_values = HashMap::new();
    known_prinicipal_values.insert(
        KnownPrincipalType::CanisterIdPostCache,
        post_cache_canister_id,
    );
    known_prinicipal_values.insert(
        KnownPrincipalType::UserIdGlobalSuperAdmin,
        admin_principal_id,
    );

    // Init individual template canister - alice

    let alice_individual_template_canister_id = pic.create_canister();
    pic.add_cycles(alice_individual_template_canister_id, 2_000_000_000_000);

    let individual_template_args = IndividualUserTemplateInitArgs {
        known_principal_ids: Some(known_prinicipal_values.clone()),
        profile_owner: Some(alice_principal_id),
        upgrade_version_number: None,
        url_to_send_canister_metrics_to: None,
        version: "1".to_string(),
        proof_of_participation: None,
    };
    let individual_template_args_bytes = encode_one(individual_template_args).unwrap();

    pic.install_canister(
        alice_individual_template_canister_id,
        individual_template_wasm_bytes.clone(),
        individual_template_args_bytes,
        None,
    );

    // Create posts
    // Alice creates a post

    let alice_post_1 = PostDetailsFromFrontend {
        is_nsfw: false,
        description: "This is a fun video to watch".to_string(),
        hashtags: vec!["fun".to_string(), "video".to_string()],
        video_uid: "abcd#1234".to_string(),
        creator_consent_for_inclusion_in_hot_or_not: true,
    };
    let res = pic
        .update_call(
            alice_individual_template_canister_id,
            alice_principal_id,
            "add_post_v2",
            encode_one(alice_post_1).unwrap(),
        )
        .map(|reply_payload| {
            let newly_created_post_id_result: Result<u64, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 add_post failed\n"),
            };
            newly_created_post_id_result.unwrap()
        })
        .unwrap();

    let alice_post_2 = PostDetailsFromFrontend {
        is_nsfw: false,
        description: "This is a fun video to watch 2".to_string(),
        hashtags: vec!["fun".to_string(), "video".to_string()],
        video_uid: "abcd#12345".to_string(),
        creator_consent_for_inclusion_in_hot_or_not: true,
    };
    let res = pic
        .update_call(
            alice_individual_template_canister_id,
            alice_principal_id,
            "add_post_v2",
            encode_one(alice_post_2).unwrap(),
        )
        .map(|reply_payload| {
            let newly_created_post_id_result: Result<u64, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 add_post failed\n"),
            };
            newly_created_post_id_result.unwrap()
        })
        .unwrap();

    // Forward timer
    pic.advance_time(Duration::from_secs(48 * 60 * 60 + 1));

    let alice_post_3 = PostDetailsFromFrontend {
        is_nsfw: false,
        description: "This is a fun video to watch - alice".to_string(),
        hashtags: vec!["fun".to_string(), "video".to_string()],
        video_uid: "abcd#123456".to_string(),
        creator_consent_for_inclusion_in_hot_or_not: true,
    };
    let res = pic
        .update_call(
            alice_individual_template_canister_id,
            alice_principal_id,
            "add_post_v2",
            encode_one(alice_post_3).unwrap(),
        )
        .map(|reply_payload| {
            let newly_created_post_id_result: Result<u64, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 add_post failed\n"),
            };
            newly_created_post_id_result.unwrap()
        })
        .unwrap();

    // thread::sleep(Duration::from_secs(5));

    // Call post cache canister to get the home feed posts
    let res = pic
        .query_call(
            post_cache_canister_id,
            bob_principal_id,
            "get_top_posts_aggregated_from_canisters_on_this_network_for_home_feed_cursor",
            candid::encode_args((0 as u64, 10 as u64)).unwrap(),
        )
        .map(|reply_payload| {
            let posts: Result<Vec<PostScoreIndexItemV1>, TopPostsFetchError> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_posts failed\n"),
            };
            posts
        })
        .unwrap();

    let posts = res.unwrap();
    assert_eq!(posts.len(), 3);
    assert_eq!(posts[0].post_id, 0);
    assert_eq!(posts[1].post_id, 1);
    assert_eq!(posts[2].post_id, 2);

    // Call post cache canister to get the hot or not feed posts
    let res = pic
        .query_call(
            post_cache_canister_id,
            bob_principal_id,
            "get_top_posts_aggregated_from_canisters_on_this_network_for_hot_or_not_feed_cursor",
            candid::encode_args((0 as u64, 10 as u64)).unwrap(),
        )
        .map(|reply_payload| {
            let posts: Result<Vec<PostScoreIndexItemV1>, TopPostsFetchError> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_posts failed\n"),
            };
            posts
        })
        .unwrap();

    let posts = res.unwrap();
    assert_eq!(posts.len(), 3);
    assert_eq!(posts[0].post_id, 2);
    assert_eq!(posts[1].post_id, 0);
    assert_eq!(posts[2].post_id, 1);

    // Update to redytoview
    // Alice updates the post to ready to view

    let res = pic
        .update_call(
            alice_individual_template_canister_id,
            admin_principal_id,
            "update_post_as_ready_to_view",
            candid::encode_args((0 as u64,)).unwrap(),
        )
        .map(|reply_payload| {
            let _ = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 update_post_status failed\n"),
            };
        })
        .unwrap();

    // Get post details

    let res = pic
        .query_call(
            alice_individual_template_canister_id,
            admin_principal_id,
            "get_entire_individual_post_detail_by_id",
            candid::encode_args((0 as u64,)).unwrap(),
        )
        .map(|reply_payload| {
            let post: Result<Post, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_post_details failed\n"),
            };
            post
        })
        .unwrap();

    // Call post cache canister to get the hot or not feed posts
    let res = pic
        .query_call(
            post_cache_canister_id,
            bob_principal_id,
            "get_top_posts_aggregated_from_canisters_on_this_network_for_hot_or_not_feed_cursor",
            candid::encode_args((0 as u64, 10 as u64)).unwrap(),
        )
        .map(|reply_payload| {
            let posts: Result<Vec<PostScoreIndexItemV1>, TopPostsFetchError> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_posts failed\n"),
            };
            posts
        })
        .unwrap();

    let posts = res.unwrap();
    assert_eq!(posts.len(), 3);
    assert_eq!(posts[0].post_id, 2);
    assert_eq!(posts[1].post_id, 1);
    assert_eq!(posts[2].post_id, 0);
    assert_eq!(posts[2].status, PostStatus::ReadyToView);
}

fn individual_template_canister_wasm() -> Vec<u8> {
    std::fs::read(INDIVIDUAL_TEMPLATE_WASM_PATH).unwrap()
}

fn post_cache_canister_wasm() -> Vec<u8> {
    std::fs::read(POST_CACHE_WASM_PATH).unwrap()
}
