use std::{collections::HashMap, time::Duration};

use candid::{encode_one, Principal};
use ic_cdk::api::management_canister::main::CanisterId;
use pocket_ic::{PocketIc, WasmResult};
use shared_utils::{
    canister_specific::{
        individual_user_template::types::{
            arg::{IndividualUserTemplateInitArgs, PlaceBetArg},
            error::BetOnCurrentlyViewingPostError,
            hot_or_not::{BetDirection, BettingStatus, BettingStatusV1, PlacedBetDetail},
            post::PostDetailsFromFrontend,
            profile::UserProfileDetailsForFrontend,
        },
        post_cache::types::arg::PostCacheInitArgs,
    },
    common::types::known_principal::KnownPrincipalType,
};
use test_utils::setup::test_constants::{
    get_global_super_admin_principal_id, get_mock_user_alice_principal_id,
    get_mock_user_bob_principal_id, get_mock_user_charlie_principal_id,
    get_mock_user_dan_principal_id,
};

const INDIVIDUAL_TEMPLATE_WASM_PATH: &str =
    "../../../target/wasm32-unknown-unknown/release/individual_user_template.wasm.gz";
const POST_CACHE_WASM_PATH: &str =
    "../../../target/wasm32-unknown-unknown/release/post_cache.wasm.gz";

fn individual_template_canister_wasm() -> Vec<u8> {
    std::fs::read(INDIVIDUAL_TEMPLATE_WASM_PATH).unwrap()
}

fn post_cache_canister_wasm() -> Vec<u8> {
    std::fs::read(POST_CACHE_WASM_PATH).unwrap()
}

#[test]
fn hotornot_game_simulation_test() {
    let pic = PocketIc::new();

    let alice_principal_id = get_mock_user_alice_principal_id();
    let bob_principal_id = get_mock_user_bob_principal_id();
    let dan_principal_id = get_mock_user_dan_principal_id();
    let admin_principal_id = get_mock_user_charlie_principal_id();

    let post_cache_canister_id = pic.create_canister();
    pic.add_cycles(post_cache_canister_id, 2_000_000_000_000);

    let mut known_prinicipal_values = HashMap::new();
    known_prinicipal_values.insert(
        KnownPrincipalType::CanisterIdPostCache,
        post_cache_canister_id,
    );
    known_prinicipal_values.insert(
        KnownPrincipalType::UserIdGlobalSuperAdmin,
        admin_principal_id,
    );
    known_prinicipal_values.insert(KnownPrincipalType::CanisterIdUserIndex, admin_principal_id);

    let post_cache_wasm_bytes = post_cache_canister_wasm();
    let post_cache_args = PostCacheInitArgs {
        known_principal_ids: Some(known_prinicipal_values.clone()),
        upgrade_version_number: Some(1),
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

    // Init individual template canister - alice

    let alice_individual_template_canister_id = pic.create_canister();
    pic.add_cycles(alice_individual_template_canister_id, 2_000_000_000_000);

    let individual_template_args = IndividualUserTemplateInitArgs {
        known_principal_ids: Some(known_prinicipal_values.clone()),
        profile_owner: Some(alice_principal_id),
        upgrade_version_number: None,
        url_to_send_canister_metrics_to: None,
        version: "1".to_string(),
    };
    let individual_template_args_bytes = encode_one(individual_template_args).unwrap();

    pic.install_canister(
        alice_individual_template_canister_id,
        individual_template_wasm_bytes.clone(),
        individual_template_args_bytes,
        None,
    );

    // Init individual template canister - bob

    let bob_individual_template_canister_id = pic.create_canister();
    pic.add_cycles(bob_individual_template_canister_id, 2_000_000_000_000);

    let individual_template_args = IndividualUserTemplateInitArgs {
        known_principal_ids: Some(known_prinicipal_values.clone()),
        profile_owner: Some(bob_principal_id),
        upgrade_version_number: None,
        url_to_send_canister_metrics_to: None,
        version: "1".to_string(),
    };
    let individual_template_args_bytes = encode_one(individual_template_args).unwrap();

    pic.install_canister(
        bob_individual_template_canister_id,
        individual_template_wasm_bytes.clone(),
        individual_template_args_bytes,
        None,
    );

    // Init individual template canister - dan

    let dan_individual_template_canister_id = pic.create_canister();
    pic.add_cycles(dan_individual_template_canister_id, 2_000_000_000_000);

    let individual_template_args = IndividualUserTemplateInitArgs {
        known_principal_ids: Some(known_prinicipal_values.clone()),
        profile_owner: Some(dan_principal_id),
        upgrade_version_number: None,
        url_to_send_canister_metrics_to: None,
        version: "1".to_string(),
    };
    let individual_template_args_bytes = encode_one(individual_template_args).unwrap();

    pic.install_canister(
        dan_individual_template_canister_id,
        individual_template_wasm_bytes.clone(),
        individual_template_args_bytes,
        None,
    );

    // Init charlie individual template canister

    let charlie_individual_template_canister_id = pic.create_canister();
    pic.add_cycles(charlie_individual_template_canister_id, 2_000_000_000_000);

    let individual_template_args = IndividualUserTemplateInitArgs {
        known_principal_ids: Some(known_prinicipal_values.clone()),
        profile_owner: Some(admin_principal_id),
        upgrade_version_number: None,
        url_to_send_canister_metrics_to: None,
        version: "1".to_string(),
    };
    let individual_template_args_bytes = encode_one(individual_template_args).unwrap();

    pic.install_canister(
        charlie_individual_template_canister_id,
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
    let res1 = pic
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
    let res2 = pic
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

    // Top up Bob's account
    let reward = pic.update_call(
        bob_individual_template_canister_id,
        admin_principal_id,
        "get_rewarded_for_signing_up",
        encode_one(()).unwrap(),
    );

    // Top up Dan's account
    let reward = pic.update_call(
        dan_individual_template_canister_id,
        admin_principal_id,
        "get_rewarded_for_signing_up",
        encode_one(()).unwrap(),
    );

    // Top up Charlie's account
    let reward = pic.update_call(
        charlie_individual_template_canister_id,
        admin_principal_id,
        "get_rewarded_for_signing_up",
        encode_one(()).unwrap(),
    );

    // Bob places bet on Alice post 1
    let bob_place_bet_arg = PlaceBetArg {
        post_canister_id: alice_individual_template_canister_id,
        post_id: res1,
        bet_amount: 100,
        bet_direction: BetDirection::Hot,
    };
    let bet_status = pic
        .update_call(
            bob_individual_template_canister_id,
            bob_principal_id,
            "bet_on_currently_viewing_post",
            encode_one(bob_place_bet_arg).unwrap(),
        )
        .map(|reply_payload| {
            let bet_status: Result<BettingStatus, BetOnCurrentlyViewingPostError> =
                match reply_payload {
                    WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                    _ => panic!("\n🛑 place_bet failed\n"),
                };
            bet_status.unwrap()
        })
        .unwrap();
    ic_cdk::println!("Bet status: {:?}", bet_status);

    // Bob places bet on Alice post 2
    let bob_place_bet_arg = PlaceBetArg {
        post_canister_id: alice_individual_template_canister_id,
        post_id: res2,
        bet_amount: 100,
        bet_direction: BetDirection::Not,
    };
    let bet_status = pic
        .update_call(
            bob_individual_template_canister_id,
            bob_principal_id,
            "bet_on_currently_viewing_post",
            encode_one(bob_place_bet_arg).unwrap(),
        )
        .map(|reply_payload| {
            let bet_status: Result<BettingStatus, BetOnCurrentlyViewingPostError> =
                match reply_payload {
                    WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                    _ => panic!("\n🛑 place_bet failed\n"),
                };
            bet_status.unwrap()
        })
        .unwrap();
    ic_cdk::println!("Bet status: {:?}", bet_status);

    // Dan places bet on Alice post 1
    let dan_place_bet_arg = PlaceBetArg {
        post_canister_id: alice_individual_template_canister_id,
        post_id: res1,
        bet_amount: 100,
        bet_direction: BetDirection::Hot,
    };
    let bet_status = pic
        .update_call(
            dan_individual_template_canister_id,
            dan_principal_id,
            "bet_on_currently_viewing_post",
            encode_one(dan_place_bet_arg).unwrap(),
        )
        .map(|reply_payload| {
            let bet_status: Result<BettingStatus, BetOnCurrentlyViewingPostError> =
                match reply_payload {
                    WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                    _ => panic!("\n🛑 place_bet failed\n"),
                };
            bet_status.unwrap()
        })
        .unwrap();
    ic_cdk::println!("Bet status: {:?}", bet_status);

    // Dan places bet on Alice post 2
    let dan_place_bet_arg = PlaceBetArg {
        post_canister_id: alice_individual_template_canister_id,
        post_id: res2,
        bet_amount: 100,
        bet_direction: BetDirection::Not,
    };
    let bet_status = pic
        .update_call(
            dan_individual_template_canister_id,
            dan_principal_id,
            "bet_on_currently_viewing_post",
            encode_one(dan_place_bet_arg).unwrap(),
        )
        .map(|reply_payload| {
            let bet_status: Result<BettingStatus, BetOnCurrentlyViewingPostError> =
                match reply_payload {
                    WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                    _ => panic!("\n🛑 place_bet failed\n"),
                };
            bet_status.unwrap()
        })
        .unwrap();
    ic_cdk::println!("Bet status: {:?}", bet_status);

    // Charlie places bet on Alice post 1
    let charlie_place_bet_arg = PlaceBetArg {
        post_canister_id: alice_individual_template_canister_id,
        post_id: res1,
        bet_amount: 100,
        bet_direction: BetDirection::Not,
    };
    let bet_status = pic
        .update_call(
            charlie_individual_template_canister_id,
            admin_principal_id,
            "bet_on_currently_viewing_post",
            encode_one(charlie_place_bet_arg).unwrap(),
        )
        .map(|reply_payload| {
            let bet_status: Result<BettingStatus, BetOnCurrentlyViewingPostError> =
                match reply_payload {
                    WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                    _ => panic!("\n🛑 place_bet failed\n"),
                };
            bet_status.unwrap()
        })
        .unwrap();
    ic_cdk::println!("Bet status: {:?}", bet_status);

    // Charlie places bet on Alice post 2
    let charlie_place_bet_arg = PlaceBetArg {
        post_canister_id: alice_individual_template_canister_id,
        post_id: res2,
        bet_amount: 100,
        bet_direction: BetDirection::Hot,
    };
    let bet_status = pic
        .update_call(
            charlie_individual_template_canister_id,
            admin_principal_id,
            "bet_on_currently_viewing_post",
            encode_one(charlie_place_bet_arg).unwrap(),
        )
        .map(|reply_payload| {
            let bet_status: Result<BettingStatus, BetOnCurrentlyViewingPostError> =
                match reply_payload {
                    WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                    _ => panic!("\n🛑 place_bet failed\n"),
                };
            bet_status.unwrap()
        })
        .unwrap();
    ic_cdk::println!("Bet status: {:?}", bet_status);

    // Show alice rewards

    let alice_rewards = pic
        .query_call(
            alice_individual_template_canister_id,
            alice_principal_id,
            "get_profile_details",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let profile: UserProfileDetailsForFrontend = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_rewards failed\n"),
            };
            profile
        })
        .unwrap();
    println!("Alice rewards: {:?}", alice_rewards);

    let alice_token_balance = pic
        .query_call(
            alice_individual_template_canister_id,
            alice_principal_id,
            "get_utility_token_balance",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let token_balance: u64 = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_token_balance failed\n"),
            };
            token_balance
        })
        .unwrap();
    println!("Alice token balance: {:?}", alice_token_balance);

    // Show bob rewards

    let bob_rewards = pic
        .query_call(
            bob_individual_template_canister_id,
            bob_principal_id,
            "get_profile_details",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let profile: UserProfileDetailsForFrontend = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_rewards failed\n"),
            };
            profile
        })
        .unwrap();
    println!("Bob rewards: {:?}", bob_rewards);

    let bob_token_balance = pic
        .query_call(
            bob_individual_template_canister_id,
            bob_principal_id,
            "get_utility_token_balance",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let token_balance: u64 = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_token_balance failed\n"),
            };
            token_balance
        })
        .unwrap();
    println!("Bob token balance: {:?}", bob_token_balance);

    // Show dan rewards

    let dan_rewards = pic
        .query_call(
            dan_individual_template_canister_id,
            dan_principal_id,
            "get_profile_details",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let profile: UserProfileDetailsForFrontend = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_rewards failed\n"),
            };
            profile
        })
        .unwrap();
    println!("Dan rewards: {:?}", dan_rewards);

    let dan_token_balance = pic
        .query_call(
            dan_individual_template_canister_id,
            dan_principal_id,
            "get_utility_token_balance",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let token_balance: u64 = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_token_balance failed\n"),
            };
            token_balance
        })
        .unwrap();
    println!("Dan token balance: {:?}", dan_token_balance);

    // Show charlie rewards

    let charlie_rewards = pic
        .query_call(
            charlie_individual_template_canister_id,
            admin_principal_id,
            "get_profile_details",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let profile: UserProfileDetailsForFrontend = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_rewards failed\n"),
            };
            profile
        })
        .unwrap();
    println!("Charlie rewards: {:?}", charlie_rewards);

    let charlie_token_balance = pic
        .query_call(
            charlie_individual_template_canister_id,
            admin_principal_id,
            "get_utility_token_balance",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let token_balance: u64 = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_token_balance failed\n"),
            };
            token_balance
        })
        .unwrap();
    println!("Charlie token balance: {:?}", charlie_token_balance);

    // // Forward timer
    // pic.advance_time(Duration::from_secs(60 * 60));
    // pic.tick();

    // Forward timer
    pic.advance_time(Duration::from_secs(60 * 60 + 30));
    for _i in 1..=5 {
        pic.tick();
    }

    // Forward timer for another post
    pic.advance_time(Duration::from_secs(10));
    for _i in 1..=5 {
        pic.tick();
    }

    // Show alice rewards

    let alice_rewards = pic
        .query_call(
            alice_individual_template_canister_id,
            alice_principal_id,
            "get_profile_details",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let profile: UserProfileDetailsForFrontend = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_rewards failed\n"),
            };
            profile
        })
        .unwrap();
    println!("Alice rewards: {:?}", alice_rewards);
    assert_eq!(alice_rewards.lifetime_earnings, 60);

    let alice_token_balance = pic
        .query_call(
            alice_individual_template_canister_id,
            alice_principal_id,
            "get_utility_token_balance",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let token_balance: u64 = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_token_balance failed\n"),
            };
            token_balance
        })
        .unwrap();
    println!("Alice token balance: {:?}", alice_token_balance);
    assert_eq!(alice_token_balance, 60);

    // Show bob rewards

    let bob_rewards = pic
        .query_call(
            bob_individual_template_canister_id,
            bob_principal_id,
            "get_profile_details",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let profile: UserProfileDetailsForFrontend = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_rewards failed\n"),
            };
            profile
        })
        .unwrap();
    println!("Bob rewards: {:?}", bob_rewards);
    assert_eq!(bob_rewards.lifetime_earnings, 1160);

    let bob_token_balance = pic
        .query_call(
            bob_individual_template_canister_id,
            bob_principal_id,
            "get_utility_token_balance",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let token_balance: u64 = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_token_balance failed\n"),
            };
            token_balance
        })
        .unwrap();
    println!("Bob token balance: {:?}", bob_token_balance);
    assert_eq!(bob_token_balance, 1160);

    // Show dan rewards

    let dan_rewards = pic
        .query_call(
            dan_individual_template_canister_id,
            dan_principal_id,
            "get_profile_details",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let profile: UserProfileDetailsForFrontend = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_rewards failed\n"),
            };
            profile
        })
        .unwrap();
    println!("Dan rewards: {:?}", dan_rewards);
    assert_eq!(dan_rewards.lifetime_earnings, 1160);

    let dan_token_balance = pic
        .query_call(
            dan_individual_template_canister_id,
            dan_principal_id,
            "get_utility_token_balance",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let token_balance: u64 = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_token_balance failed\n"),
            };
            token_balance
        })
        .unwrap();
    println!("Dan token balance: {:?}", dan_token_balance);
    assert_eq!(dan_token_balance, 1160);

    // Show charlie rewards

    let charlie_rewards = pic
        .query_call(
            charlie_individual_template_canister_id,
            admin_principal_id,
            "get_profile_details",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let profile: UserProfileDetailsForFrontend = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_rewards failed\n"),
            };
            profile
        })
        .unwrap();
    println!("Charlie rewards: {:?}", charlie_rewards);
    assert_eq!(charlie_rewards.lifetime_earnings, 1000);

    let charlie_token_balance = pic
        .query_call(
            charlie_individual_template_canister_id,
            admin_principal_id,
            "get_utility_token_balance",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let token_balance: u64 = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_token_balance failed\n"),
            };
            token_balance
        })
        .unwrap();
    println!("Charlie token balance: {:?}", charlie_token_balance);
    assert_eq!(charlie_token_balance, 800);

    // Bob creates a post

    let bob_post_1 = PostDetailsFromFrontend {
        is_nsfw: false,
        description: "This is a fun video to watch - bob".to_string(),
        hashtags: vec!["fun".to_string(), "video".to_string()],
        video_uid: "abcd#1234bob".to_string(),
        creator_consent_for_inclusion_in_hot_or_not: true,
    };
    let res1 = pic
        .update_call(
            bob_individual_template_canister_id,
            bob_principal_id,
            "add_post_v2",
            encode_one(bob_post_1).unwrap(),
        )
        .map(|reply_payload| {
            let newly_created_post_id_result: Result<u64, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 add_post failed\n"),
            };
            newly_created_post_id_result.unwrap()
        })
        .unwrap();

    // Top up Alice's account
    let reward = pic.update_call(
        alice_individual_template_canister_id,
        admin_principal_id,
        "get_rewarded_for_signing_up",
        encode_one(()).unwrap(),
    );

    // Alice places bet on Bob post 1
    let alice_place_bet_arg = PlaceBetArg {
        post_canister_id: bob_individual_template_canister_id,
        post_id: res1,
        bet_amount: 100,
        bet_direction: BetDirection::Not,
    };
    let bet_status = pic
        .update_call(
            alice_individual_template_canister_id,
            alice_principal_id,
            "bet_on_currently_viewing_post",
            encode_one(alice_place_bet_arg).unwrap(),
        )
        .map(|reply_payload| {
            let bet_status: Result<BettingStatus, BetOnCurrentlyViewingPostError> =
                match reply_payload {
                    WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                    _ => panic!("\n🛑 place_bet failed\n"),
                };
            bet_status.unwrap()
        })
        .unwrap();
    ic_cdk::println!("Bet status: {:?}", bet_status);

    // Dan places bet on Bob post 1
    let dan_place_bet_arg = PlaceBetArg {
        post_canister_id: bob_individual_template_canister_id,
        post_id: res1,
        bet_amount: 100,
        bet_direction: BetDirection::Not,
    };
    let bet_status = pic
        .update_call(
            dan_individual_template_canister_id,
            dan_principal_id,
            "bet_on_currently_viewing_post",
            encode_one(dan_place_bet_arg).unwrap(),
        )
        .map(|reply_payload| {
            let bet_status: Result<BettingStatus, BetOnCurrentlyViewingPostError> =
                match reply_payload {
                    WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                    _ => panic!("\n🛑 place_bet failed\n"),
                };
            bet_status.unwrap()
        })
        .unwrap();
    ic_cdk::println!("Bet status: {:?}", bet_status);

    // Charlie places bet on Bob post 1
    let charlie_place_bet_arg = PlaceBetArg {
        post_canister_id: bob_individual_template_canister_id,
        post_id: res1,
        bet_amount: 100,
        bet_direction: BetDirection::Hot,
    };
    let bet_status = pic
        .update_call(
            charlie_individual_template_canister_id,
            admin_principal_id,
            "bet_on_currently_viewing_post",
            encode_one(charlie_place_bet_arg).unwrap(),
        )
        .map(|reply_payload| {
            let bet_status: Result<BettingStatus, BetOnCurrentlyViewingPostError> =
                match reply_payload {
                    WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                    _ => panic!("\n🛑 place_bet failed\n"),
                };
            bet_status.unwrap()
        })
        .unwrap();
    ic_cdk::println!("Bet status: {:?}", bet_status);

    // // Forward timer
    // pic.advance_time(Duration::from_secs(60 * 60));
    // pic.tick();

    // Forward timer
    pic.advance_time(Duration::from_secs(60 * 60 + 10));
    for _i in 1..=10 {
        pic.tick();
    }
    // Show alice rewards

    let alice_rewards = pic
        .query_call(
            alice_individual_template_canister_id,
            alice_principal_id,
            "get_profile_details",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let profile: UserProfileDetailsForFrontend = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_rewards failed\n"),
            };
            profile
        })
        .unwrap();
    println!("Alice rewards: {:?}", alice_rewards);
    assert_eq!(alice_rewards.lifetime_earnings, 1140);

    let alice_token_balance = pic
        .query_call(
            alice_individual_template_canister_id,
            alice_principal_id,
            "get_utility_token_balance",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let token_balance: u64 = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_token_balance failed\n"),
            };
            token_balance
        })
        .unwrap();
    println!("Alice token balance: {:?}", alice_token_balance);
    assert_eq!(alice_token_balance, 1140);

    // Show bob rewards

    let bob_rewards = pic
        .query_call(
            bob_individual_template_canister_id,
            bob_principal_id,
            "get_profile_details",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let profile: UserProfileDetailsForFrontend = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_rewards failed\n"),
            };
            profile
        })
        .unwrap();
    println!("Bob rewards: {:?}", bob_rewards);
    assert_eq!(bob_rewards.lifetime_earnings, 1190);

    let bob_token_balance = pic
        .query_call(
            bob_individual_template_canister_id,
            bob_principal_id,
            "get_utility_token_balance",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let token_balance: u64 = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_token_balance failed\n"),
            };
            token_balance
        })
        .unwrap();
    println!("Bob token balance: {:?}", bob_token_balance);
    assert_eq!(bob_token_balance, 1190);

    // Show dan rewards

    let dan_rewards = pic
        .query_call(
            dan_individual_template_canister_id,
            dan_principal_id,
            "get_profile_details",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let profile: UserProfileDetailsForFrontend = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_rewards failed\n"),
            };
            profile
        })
        .unwrap();
    println!("Dan rewards: {:?}", dan_rewards);
    assert_eq!(dan_rewards.lifetime_earnings, 1240);

    let dan_token_balance = pic
        .query_call(
            dan_individual_template_canister_id,
            dan_principal_id,
            "get_utility_token_balance",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let token_balance: u64 = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_token_balance failed\n"),
            };
            token_balance
        })
        .unwrap();
    println!("Dan token balance: {:?}", dan_token_balance);
    assert_eq!(dan_token_balance, 1240);

    // Show charlie rewards

    let charlie_rewards = pic
        .query_call(
            charlie_individual_template_canister_id,
            admin_principal_id,
            "get_profile_details",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let profile: UserProfileDetailsForFrontend = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_rewards failed\n"),
            };
            profile
        })
        .unwrap();
    println!("Charlie rewards: {:?}", charlie_rewards);
    assert_eq!(charlie_rewards.lifetime_earnings, 1000);

    let charlie_token_balance = pic
        .query_call(
            charlie_individual_template_canister_id,
            admin_principal_id,
            "get_utility_token_balance",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let token_balance: u64 = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_token_balance failed\n"),
            };
            token_balance
        })
        .unwrap();
    println!("Charlie token balance: {:?}", charlie_token_balance);
    assert_eq!(charlie_token_balance, 700);
}




#[test]
fn hotornot_game_simulation_test_yral() {
    let pic = PocketIc::new();

    let alice_principal_id = get_mock_user_alice_principal_id();
    let bob_principal_id = get_mock_user_bob_principal_id();
    let dan_principal_id = get_mock_user_dan_principal_id();
    let admin_principal_id = get_mock_user_charlie_principal_id();

    let post_cache_canister_id = pic.create_canister();
    pic.add_cycles(post_cache_canister_id, 2_000_000_000_000);

    let mut known_prinicipal_values = HashMap::new();
    known_prinicipal_values.insert(
        KnownPrincipalType::CanisterIdPostCache,
        post_cache_canister_id,
    );
    known_prinicipal_values.insert(
        KnownPrincipalType::UserIdGlobalSuperAdmin,
        admin_principal_id,
    );
    known_prinicipal_values.insert(KnownPrincipalType::CanisterIdUserIndex, admin_principal_id);

    let post_cache_wasm_bytes = post_cache_canister_wasm();
    let post_cache_args = PostCacheInitArgs {
        known_principal_ids: Some(known_prinicipal_values.clone()),
        upgrade_version_number: Some(1),
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

    // Init individual template canister - alice

    let alice_individual_template_canister_id = pic.create_canister();
    pic.add_cycles(alice_individual_template_canister_id, 2_000_000_000_000);

    let individual_template_args = IndividualUserTemplateInitArgs {
        known_principal_ids: Some(known_prinicipal_values.clone()),
        profile_owner: Some(alice_principal_id),
        upgrade_version_number: None,
        url_to_send_canister_metrics_to: None,
        version: "1".to_string(),
    };
    let individual_template_args_bytes = encode_one(individual_template_args).unwrap();

    pic.install_canister(
        alice_individual_template_canister_id,
        individual_template_wasm_bytes.clone(),
        individual_template_args_bytes,
        None,
    );

    // Init individual template canister - bob

    let bob_individual_template_canister_id = pic.create_canister();
    pic.add_cycles(bob_individual_template_canister_id, 2_000_000_000_000);

    let individual_template_args = IndividualUserTemplateInitArgs {
        known_principal_ids: Some(known_prinicipal_values.clone()),
        profile_owner: Some(bob_principal_id),
        upgrade_version_number: None,
        url_to_send_canister_metrics_to: None,
        version: "1".to_string(),
    };
    let individual_template_args_bytes = encode_one(individual_template_args).unwrap();

    pic.install_canister(
        bob_individual_template_canister_id,
        individual_template_wasm_bytes.clone(),
        individual_template_args_bytes,
        None,
    );

    // Init individual template canister - dan

    let dan_individual_template_canister_id = pic.create_canister();
    pic.add_cycles(dan_individual_template_canister_id, 2_000_000_000_000);

    let individual_template_args = IndividualUserTemplateInitArgs {
        known_principal_ids: Some(known_prinicipal_values.clone()),
        profile_owner: Some(dan_principal_id),
        upgrade_version_number: None,
        url_to_send_canister_metrics_to: None,
        version: "1".to_string(),
    };
    let individual_template_args_bytes = encode_one(individual_template_args).unwrap();

    pic.install_canister(
        dan_individual_template_canister_id,
        individual_template_wasm_bytes.clone(),
        individual_template_args_bytes,
        None,
    );

    // Init charlie individual template canister

    let charlie_individual_template_canister_id = pic.create_canister();
    pic.add_cycles(charlie_individual_template_canister_id, 2_000_000_000_000);

    let individual_template_args = IndividualUserTemplateInitArgs {
        known_principal_ids: Some(known_prinicipal_values.clone()),
        profile_owner: Some(admin_principal_id),
        upgrade_version_number: None,
        url_to_send_canister_metrics_to: None,
        version: "1".to_string(),
    };
    let individual_template_args_bytes = encode_one(individual_template_args).unwrap();

    pic.install_canister(
        charlie_individual_template_canister_id,
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
    let res1 = pic
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
    let res2 = pic
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

    // Top up Bob's account
    let reward = pic.update_call(
        bob_individual_template_canister_id,
        admin_principal_id,
        "get_rewarded_for_signing_up_v1",
        encode_one(()).unwrap(),
    );

    ic_cdk::println!("Bob rewards: {:?}", reward);

    // Top up Dan's account
    let reward = pic.update_call(
        dan_individual_template_canister_id,
        admin_principal_id,
        "get_rewarded_for_signing_up_v1",
        encode_one(()).unwrap(),
    );

    // Top up Charlie's account
    let reward = pic.update_call(
        charlie_individual_template_canister_id,
        admin_principal_id,
        "get_rewarded_for_signing_up_v1",
        encode_one(()).unwrap(),
    );

    // Bob places bet on Alice post 1
    let bob_place_bet_arg = PlaceBetArg {
        post_canister_id: alice_individual_template_canister_id,
        post_id: res1,
        bet_amount: 100,
        bet_direction: BetDirection::Hot,
    };


    let bob_rewards = pic
        .query_call(
            bob_individual_template_canister_id,
            bob_principal_id,
            "get_profile_details_v3",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let profile: UserProfileDetailsForFrontend = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_rewards failed\n"),
            };
            profile
        })
        .unwrap();
    println!("Bob rewards before user_bets_on_post: {:?}", bob_rewards);

    let bet_status = user_bets_on_post(
        &pic,
        bob_principal_id,
        bob_individual_template_canister_id,
        bob_place_bet_arg,
    );

    ic_cdk::println!("Bet status: {:?}", bet_status);

    // Bob places bet on Alice post 2
    let bob_place_bet_arg = PlaceBetArg {
        post_canister_id: alice_individual_template_canister_id,
        post_id: res2,
        bet_amount: 100,
        bet_direction: BetDirection::Not,
    };

    let bet_status = user_bets_on_post(
        &pic,
        bob_principal_id,
        bob_individual_template_canister_id,
        bob_place_bet_arg,
    );

    ic_cdk::println!("Bet status: {:?}", bet_status);

    // Dan places bet on Alice post 1
    let dan_place_bet_arg = PlaceBetArg {
        post_canister_id: alice_individual_template_canister_id,
        post_id: res1,
        bet_amount: 100,
        bet_direction: BetDirection::Hot,
    };

    let bet_status = user_bets_on_post(
        &pic,
        dan_principal_id,
        dan_individual_template_canister_id,
        dan_place_bet_arg,
    );

    ic_cdk::println!("Bet status: {:?}", bet_status);

    // Dan places bet on Alice post 2
    let dan_place_bet_arg = PlaceBetArg {
        post_canister_id: alice_individual_template_canister_id,
        post_id: res2,
        bet_amount: 100,
        bet_direction: BetDirection::Not,
    };
    let bet_status = user_bets_on_post(
        &pic,
        dan_principal_id,
        dan_individual_template_canister_id,
        dan_place_bet_arg,
    );
   
    ic_cdk::println!("Bet status: {:?}", bet_status);

    // Charlie places bet on Alice post 1
    let charlie_place_bet_arg = PlaceBetArg {
        post_canister_id: alice_individual_template_canister_id,
        post_id: res1,
        bet_amount: 100,
        bet_direction: BetDirection::Not,
    };
    let bet_status = user_bets_on_post(
        &pic,
        admin_principal_id,
        charlie_individual_template_canister_id,
        charlie_place_bet_arg,
    );
 
    ic_cdk::println!("Bet status: {:?}", bet_status);

    // Charlie places bet on Alice post 2
    let charlie_place_bet_arg = PlaceBetArg {
        post_canister_id: alice_individual_template_canister_id,
        post_id: res2,
        bet_amount: 100,
        bet_direction: BetDirection::Hot,
    };
    let bet_status = user_bets_on_post(
        &pic,
        admin_principal_id,
        charlie_individual_template_canister_id,
        charlie_place_bet_arg,
    );

    ic_cdk::println!("Bet status: {:?}", bet_status);

    // Show alice rewards

    let alice_rewards = pic
        .query_call(
            alice_individual_template_canister_id,
            alice_principal_id,
            "get_profile_details_v3",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let profile: UserProfileDetailsForFrontend = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_rewards failed\n"),
            };
            profile
        })
        .unwrap();
    println!("Alice rewards: {:?}", alice_rewards);

    let alice_token_balance = pic
        .query_call(
            alice_individual_template_canister_id,
            alice_principal_id,
            "get_utility_token_balance",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let token_balance: u64 = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_token_balance failed\n"),
            };
            token_balance
        })
        .unwrap();
    println!("Alice token balance: {:?}", alice_token_balance);

    // Show bob rewards

    let bob_rewards = pic
        .query_call(
            bob_individual_template_canister_id,
            bob_principal_id,
            "get_profile_details_v3",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let profile: UserProfileDetailsForFrontend = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_rewards failed\n"),
            };
            profile
        })
        .unwrap();
    println!("Bob rewards: {:?}", bob_rewards);

    let bob_token_balance = pic
        .query_call(
            bob_individual_template_canister_id,
            bob_principal_id,
            "get_utility_token_balance_v1",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let token_balance: u64 = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_token_balance failed\n"),
            };
            token_balance
        })
        .unwrap();
    println!("Bob token balance: {:?}", bob_token_balance);

    // Show dan rewards

    let dan_rewards = pic
        .query_call(
            dan_individual_template_canister_id,
            dan_principal_id,
            "get_profile_details_v3",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let profile: UserProfileDetailsForFrontend = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_rewards failed\n"),
            };
            profile
        })
        .unwrap();
    println!("Dan rewards: {:?}", dan_rewards);

    let dan_token_balance = pic
        .query_call(
            dan_individual_template_canister_id,
            dan_principal_id,
            "get_utility_token_balance_v1",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let token_balance: u64 = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_token_balance failed\n"),
            };
            token_balance
        })
        .unwrap();
    println!("Dan token balance: {:?}", dan_token_balance);

    // Show charlie rewards

    let charlie_rewards = pic
        .query_call(
            charlie_individual_template_canister_id,
            admin_principal_id,
            "get_profile_details_v3",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let profile: UserProfileDetailsForFrontend = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_rewards failed\n"),
            };
            profile
        })
        .unwrap();
    println!("Charlie rewards: {:?}", charlie_rewards);

    let charlie_token_balance = pic
        .query_call(
            charlie_individual_template_canister_id,
            admin_principal_id,
            "get_utility_token_balance_v1",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let token_balance: u64 = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_token_balance failed\n"),
            };
            token_balance
        })
        .unwrap();
    println!("Charlie token balance: {:?}", charlie_token_balance);

    // Forward timer
    pic.advance_time(Duration::from_secs(60 * 60 + 30));
    for _i in 1..=5 {
        pic.tick();
    }

    // Forward timer for another post
    pic.advance_time(Duration::from_secs(10));
    for _i in 1..=5 {
        pic.tick();
    }

    // Show alice rewards

    let alice_rewards = pic
        .query_call(
            alice_individual_template_canister_id,
            alice_principal_id,
            "get_profile_details_v3",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let profile: UserProfileDetailsForFrontend = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_rewards failed\n"),
            };
            profile
        })
        .unwrap();
    println!("Alice rewards: {:?}", alice_rewards);
    assert_eq!(alice_rewards.lifetime_earnings, 60);

    let alice_token_balance = pic
        .query_call(
            alice_individual_template_canister_id,
            alice_principal_id,
            "get_utility_token_balance_v1",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let token_balance: u64 = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_token_balance failed\n"),
            };
            token_balance
        })
        .unwrap();
    println!("Alice token balance: {:?}", alice_token_balance);
    assert_eq!(alice_token_balance, 60);

    // Show bob rewards

    let bob_rewards = pic
        .query_call(
            bob_individual_template_canister_id,
            bob_principal_id,
            "get_profile_details_v3",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let profile: UserProfileDetailsForFrontend = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_rewards failed\n"),
            };
            profile
        })
        .unwrap();
    println!("Bob rewards: {:?}", bob_rewards);
    assert_eq!(bob_rewards.lifetime_earnings, 1160);

    let bob_token_balance = pic
        .query_call(
            bob_individual_template_canister_id,
            bob_principal_id,
            "get_utility_token_balance_v1",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let token_balance: u64 = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_token_balance failed\n"),
            };
            token_balance
        })
        .unwrap();
    println!("Bob token balance: {:?}", bob_token_balance);
    assert_eq!(bob_token_balance, 1160);

    // Show dan rewards

    let dan_rewards = pic
        .query_call(
            dan_individual_template_canister_id,
            dan_principal_id,
            "get_profile_details_v3",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let profile: UserProfileDetailsForFrontend = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_rewards failed\n"),
            };
            profile
        })
        .unwrap();
    println!("Dan rewards: {:?}", dan_rewards);
    assert_eq!(dan_rewards.lifetime_earnings, 1160);

    let dan_token_balance = pic
        .query_call(
            dan_individual_template_canister_id,
            dan_principal_id,
            "get_utility_token_balance_v1",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let token_balance: u64 = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_token_balance failed\n"),
            };
            token_balance
        })
        .unwrap();
    println!("Dan token balance: {:?}", dan_token_balance);
    assert_eq!(dan_token_balance, 1160);

    // Show charlie rewards

    let charlie_rewards = pic
        .query_call(
            charlie_individual_template_canister_id,
            admin_principal_id,
            "get_profile_details_v3",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let profile: UserProfileDetailsForFrontend = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_rewards failed\n"),
            };
            profile
        })
        .unwrap();
    println!("Charlie rewards: {:?}", charlie_rewards);
    assert_eq!(charlie_rewards.lifetime_earnings, 1000);

    let charlie_token_balance = pic
        .query_call(
            charlie_individual_template_canister_id,
            admin_principal_id,
            "get_utility_token_balance_v1",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let token_balance: u64 = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_token_balance failed\n"),
            };
            token_balance
        })
        .unwrap();
    println!("Charlie token balance: {:?}", charlie_token_balance);
    assert_eq!(charlie_token_balance, 800);

    // Bob creates a post

    let bob_post_1 = PostDetailsFromFrontend {
        is_nsfw: false,
        description: "This is a fun video to watch - bob".to_string(),
        hashtags: vec!["fun".to_string(), "video".to_string()],
        video_uid: "abcd#1234bob".to_string(),
        creator_consent_for_inclusion_in_hot_or_not: true,
    };
    let res1 = pic
        .update_call(
            bob_individual_template_canister_id,
            bob_principal_id,
            "add_post_v2",
            encode_one(bob_post_1).unwrap(),
        )
        .map(|reply_payload| {
            let newly_created_post_id_result: Result<u64, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 add_post failed\n"),
            };
            newly_created_post_id_result.unwrap()
        })
        .unwrap();

    // Top up Alice's account
    let reward = pic.update_call(
        alice_individual_template_canister_id,
        admin_principal_id,
        "get_rewarded_for_signing_up_v1",
        encode_one(()).unwrap(),
    );

    // Alice places bet on Bob post 1
    let alice_place_bet_arg = PlaceBetArg {
        post_canister_id: bob_individual_template_canister_id,
        post_id: res1,
        bet_amount: 100,
        bet_direction: BetDirection::Not,
    };
    let bet_status = user_bets_on_post(
        &pic,
        alice_principal_id,
        alice_individual_template_canister_id,
        alice_place_bet_arg,
    );
  
    ic_cdk::println!("Bet status: {:?}", bet_status);

    // Dan places bet on Bob post 1
    let dan_place_bet_arg = PlaceBetArg {
        post_canister_id: bob_individual_template_canister_id,
        post_id: res1,
        bet_amount: 100,
        bet_direction: BetDirection::Not,
    };

    let bet_status = user_bets_on_post(
        &pic,
        dan_principal_id,
        dan_individual_template_canister_id,
        dan_place_bet_arg,
    );

    ic_cdk::println!("Bet status: {:?}", bet_status);

    // Charlie places bet on Bob post 1
    let charlie_place_bet_arg = PlaceBetArg {
        post_canister_id: bob_individual_template_canister_id,
        post_id: res1,
        bet_amount: 100,
        bet_direction: BetDirection::Hot,
    };
    let bet_status = user_bets_on_post(
        &pic,
        admin_principal_id,
        charlie_individual_template_canister_id,
        charlie_place_bet_arg,
    );

    ic_cdk::println!("Bet status: {:?}", bet_status);
 
    // Forward timer
    pic.advance_time(Duration::from_secs(60 * 60 + 10));
    for _i in 1..=10 {
        pic.tick();
    }
    // Show alice rewards

    let alice_rewards = pic
        .query_call(
            alice_individual_template_canister_id,
            alice_principal_id,
            "get_profile_details_v3",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let profile: UserProfileDetailsForFrontend = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_rewards failed\n"),
            };
            profile
        })
        .unwrap();
    println!("Alice rewards: {:?}", alice_rewards);
    assert_eq!(alice_rewards.lifetime_earnings, 1140);

    let alice_token_balance = pic
        .query_call(
            alice_individual_template_canister_id,
            alice_principal_id,
            "get_utility_token_balance_v1",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let token_balance: u64 = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_token_balance failed\n"),
            };
            token_balance
        })
        .unwrap();
    println!("Alice token balance: {:?}", alice_token_balance);
    assert_eq!(alice_token_balance, 1140);

    // Show bob rewards

    let bob_rewards = pic
        .query_call(
            bob_individual_template_canister_id,
            bob_principal_id,
            "get_profile_details_v3",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let profile: UserProfileDetailsForFrontend = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_rewards failed\n"),
            };
            profile
        })
        .unwrap();
    println!("Bob rewards: {:?}", bob_rewards);
    assert_eq!(bob_rewards.lifetime_earnings, 1190);

    let bob_token_balance = pic
        .query_call(
            bob_individual_template_canister_id,
            bob_principal_id,
            "get_utility_token_balance_v1",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let token_balance: u64 = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_token_balance failed\n"),
            };
            token_balance
        })
        .unwrap();
    println!("Bob token balance: {:?}", bob_token_balance);
    assert_eq!(bob_token_balance, 1190);

    // Show dan rewards

    let dan_rewards = pic
        .query_call(
            dan_individual_template_canister_id,
            dan_principal_id,
            "get_profile_details_v3",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let profile: UserProfileDetailsForFrontend = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_rewards failed\n"),
            };
            profile
        })
        .unwrap();
    println!("Dan rewards: {:?}", dan_rewards);
    assert_eq!(dan_rewards.lifetime_earnings, 1240);

    let dan_token_balance = pic
        .query_call(
            dan_individual_template_canister_id,
            dan_principal_id,
            "get_utility_token_balance_v1",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let token_balance: u64 = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_token_balance failed\n"),
            };
            token_balance
        })
        .unwrap();
    println!("Dan token balance: {:?}", dan_token_balance);
    assert_eq!(dan_token_balance, 1240);

    // Show charlie rewards

    let charlie_rewards = pic
        .query_call(
            charlie_individual_template_canister_id,
            admin_principal_id,
            "get_profile_details_v3",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let profile: UserProfileDetailsForFrontend = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_rewards failed\n"),
            };
            profile
        })
        .unwrap();
    println!("Charlie rewards: {:?}", charlie_rewards);
    assert_eq!(charlie_rewards.lifetime_earnings, 1000);

    let charlie_token_balance = pic
        .query_call(
            charlie_individual_template_canister_id,
            admin_principal_id,
            "get_utility_token_balance_v1",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let token_balance: u64 = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_token_balance failed\n"),
            };
            token_balance
        })
        .unwrap();
    println!("Charlie token balance: {:?}", charlie_token_balance);
    assert_eq!(charlie_token_balance, 700);
}

fn user_bets_on_post(
    pic: &PocketIc,
    bet_maker_principal_id: Principal,
    bet_maker_individual_template_canister_id: CanisterId,
    bet_maker_place_bet_arg: PlaceBetArg,
) -> BettingStatusV1 {
    pic.update_call(
        bet_maker_individual_template_canister_id,
        bet_maker_principal_id,
        "bet_on_currently_viewing_post_v1",
        encode_one(bet_maker_place_bet_arg).unwrap(),
    )
    .map(|reply_payload| {
        let bet_status: Result<BettingStatusV1, BetOnCurrentlyViewingPostError> =
            match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 place_bet failed\n"),
            };
        bet_status.unwrap()
    })
    .unwrap()
}

#[test]
#[ignore]
fn hotornot_game_simulation_test_2() {
    let pic = PocketIc::new();
    let admin_principal_id = get_global_super_admin_principal_id();

    let post_cache_canister_id = pic.create_canister();
    pic.add_cycles(post_cache_canister_id, 2_000_000_000_000);

    let mut known_prinicipal_values = HashMap::new();
    known_prinicipal_values.insert(
        KnownPrincipalType::CanisterIdPostCache,
        post_cache_canister_id,
    );
    known_prinicipal_values.insert(
        KnownPrincipalType::UserIdGlobalSuperAdmin,
        admin_principal_id,
    );
    known_prinicipal_values.insert(KnownPrincipalType::CanisterIdUserIndex, admin_principal_id);

    let post_cache_wasm_bytes = post_cache_canister_wasm();
    let post_cache_args = PostCacheInitArgs {
        known_principal_ids: Some(known_prinicipal_values.clone()),
        upgrade_version_number: Some(1),
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

    // Init N canisters
    let mut individual_template_canister_ids = vec![];
    for i in 1..=111 {
        let individual_template_canister_id = pic.create_canister();
        pic.add_cycles(individual_template_canister_id, 2_000_000_000_000);

        let individual_template_args = IndividualUserTemplateInitArgs {
            known_principal_ids: Some(known_prinicipal_values.clone()),
            profile_owner: Some(Principal::self_authenticating((i as usize).to_ne_bytes())),
            upgrade_version_number: None,
            url_to_send_canister_metrics_to: None,
            version: "1".to_string(),
        };
        let individual_template_args_bytes = encode_one(individual_template_args).unwrap();

        pic.install_canister(
            individual_template_canister_id,
            individual_template_wasm_bytes.clone(),
            individual_template_args_bytes,
            None,
        );

        let reward = pic.update_call(
            individual_template_canister_id,
            admin_principal_id,
            "get_rewarded_for_signing_up",
            encode_one(()).unwrap(),
        );

        individual_template_canister_ids.push(individual_template_canister_id);

        if i % 10 == 0 {
            println!(
                "Installed {} canisters",
                individual_template_canister_ids.len()
            );
        }
    }

    let last_individual_template_canister_id = individual_template_canister_ids.pop().unwrap();
    let last_individual_template_principal_id =
        Principal::self_authenticating((111 as usize).to_ne_bytes());

    // Create a post

    let last_post_1 = PostDetailsFromFrontend {
        is_nsfw: false,
        description: "This is a fun video to watch".to_string(),
        hashtags: vec!["fun".to_string(), "video".to_string()],
        video_uid: "abcd#1234".to_string(),
        creator_consent_for_inclusion_in_hot_or_not: true,
    };
    let res1 = pic
        .update_call(
            last_individual_template_canister_id,
            last_individual_template_principal_id,
            "add_post_v2",
            encode_one(last_post_1).unwrap(),
        )
        .map(|reply_payload| {
            let newly_created_post_id_result: Result<u64, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 add_post failed\n"),
            };
            newly_created_post_id_result.unwrap()
        })
        .unwrap();

    // All 500 users bet on the post

    for i in 1..=110 {
        let bob_place_bet_arg = PlaceBetArg {
            post_canister_id: last_individual_template_canister_id,
            post_id: res1,
            bet_amount: 100,
            bet_direction: BetDirection::Hot,
        };
        let bet_status = pic
            .update_call(
                individual_template_canister_ids[i - 1],
                Principal::self_authenticating((i as usize).to_ne_bytes()),
                "bet_on_currently_viewing_post",
                encode_one(bob_place_bet_arg).unwrap(),
            )
            .map(|reply_payload| {
                let bet_status: Result<BettingStatus, BetOnCurrentlyViewingPostError> =
                    match reply_payload {
                        WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                        _ => panic!("\n🛑 place_bet failed\n"),
                    };
                bet_status.unwrap()
            })
            .unwrap();
        // ic_cdk::println!("Bet status: {:?}", bet_status);
        if i % 10 == 0 {
            println!("Betted for {} users", i);
        }
    }

    // Forward timer
    pic.advance_time(Duration::from_secs(60 * 60));
    pic.tick();

    // Check rewards

    let last_rewards = pic
        .query_call(
            last_individual_template_canister_id,
            last_individual_template_principal_id,
            "get_profile_details",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let profile: UserProfileDetailsForFrontend = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_rewards failed\n"),
            };
            profile
        })
        .unwrap();
    assert_eq!(last_rewards.lifetime_earnings, 2100);
    // println!("Last rewards: {:?}", last_rewards.lifetime_earnings);

    let last_token_balance = pic
        .query_call(
            last_individual_template_canister_id,
            last_individual_template_principal_id,
            "get_utility_token_balance",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let token_balance: u64 = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_token_balance failed\n"),
            };
            token_balance
        })
        .unwrap();
    assert_eq!(last_token_balance, 2100);
    // println!("Last token balance: {:?}", last_token_balance);

    for i in 1..=110 {
        let rewards = pic
            .query_call(
                individual_template_canister_ids[i - 1],
                Principal::self_authenticating((i as usize).to_ne_bytes()),
                "get_profile_details",
                encode_one(()).unwrap(),
            )
            .map(|reply_payload| {
                let profile: UserProfileDetailsForFrontend = match reply_payload {
                    WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                    _ => panic!("\n🛑 get_rewards failed\n"),
                };
                profile
            })
            .unwrap();
        assert_eq!(rewards.lifetime_earnings, 1080);
        // println!("Rewards for user {}: {:?}", i, rewards.lifetime_earnings);

        let token_balance = pic
            .query_call(
                individual_template_canister_ids[i - 1],
                Principal::self_authenticating((i as usize).to_ne_bytes()),
                "get_utility_token_balance",
                encode_one(()).unwrap(),
            )
            .map(|reply_payload| {
                let token_balance: u64 = match reply_payload {
                    WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                    _ => panic!("\n🛑 get_token_balance failed\n"),
                };
                token_balance
            })
            .unwrap();
        assert_eq!(token_balance, 1080);
        // println!("Token balance for user {}: {:?}", i, token_balance);
    }
}






#[test]
#[ignore]
fn hotornot_game_simulation_test_after_yral_game_3() {
    let pic = PocketIc::new();
    let admin_principal_id = get_global_super_admin_principal_id();

    let post_cache_canister_id = pic.create_canister();
    pic.add_cycles(post_cache_canister_id, 2_000_000_000_000);

    let mut known_prinicipal_values = HashMap::new();
    known_prinicipal_values.insert(
        KnownPrincipalType::CanisterIdPostCache,
        post_cache_canister_id,
    );
    known_prinicipal_values.insert(
        KnownPrincipalType::UserIdGlobalSuperAdmin,
        admin_principal_id,
    );
    known_prinicipal_values.insert(KnownPrincipalType::CanisterIdUserIndex, admin_principal_id);

    let post_cache_wasm_bytes = post_cache_canister_wasm();
    let post_cache_args = PostCacheInitArgs {
        known_principal_ids: Some(known_prinicipal_values.clone()),
        upgrade_version_number: Some(1),
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

    // Init N canisters
    let mut individual_template_canister_ids = vec![];
    for i in 1..=111 {
        let individual_template_canister_id = pic.create_canister();
        pic.add_cycles(individual_template_canister_id, 2_000_000_000_000);

        let individual_template_args = IndividualUserTemplateInitArgs {
            known_principal_ids: Some(known_prinicipal_values.clone()),
            profile_owner: Some(Principal::self_authenticating((i as usize).to_ne_bytes())),
            upgrade_version_number: None,
            url_to_send_canister_metrics_to: None,
            version: "1".to_string(),
        };
        let individual_template_args_bytes = encode_one(individual_template_args).unwrap();

        pic.install_canister(
            individual_template_canister_id,
            individual_template_wasm_bytes.clone(),
            individual_template_args_bytes,
            None,
        );

        let reward = pic.update_call(
            individual_template_canister_id,
            admin_principal_id,
            "get_rewarded_for_signing_up_v1",
            encode_one(()).unwrap(),
        );

        individual_template_canister_ids.push(individual_template_canister_id);

        if i % 10 == 0 {
            println!(
                "Installed {} canisters",
                individual_template_canister_ids.len()
            );
        }
    }

    let last_individual_template_canister_id = individual_template_canister_ids.pop().unwrap();
    let last_individual_template_principal_id =
        Principal::self_authenticating((111 as usize).to_ne_bytes());

    // Create a post

    let last_post_1 = PostDetailsFromFrontend {
        is_nsfw: false,
        description: "This is a fun video to watch".to_string(),
        hashtags: vec!["fun".to_string(), "video".to_string()],
        video_uid: "abcd#1234".to_string(),
        creator_consent_for_inclusion_in_hot_or_not: true,
    };
    let res1 = pic
        .update_call(
            last_individual_template_canister_id,
            last_individual_template_principal_id,
            "add_post_v2",
            encode_one(last_post_1).unwrap(),
        )
        .map(|reply_payload| {
            let newly_created_post_id_result: Result<u64, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 add_post failed\n"),
            };
            newly_created_post_id_result.unwrap()
        })
        .unwrap();

    // All 500 users bet on the post

    for i in 1..=110 {
        let bob_place_bet_arg = PlaceBetArg {
            post_canister_id: last_individual_template_canister_id,
            post_id: res1,
            bet_amount: 100,
            bet_direction: BetDirection::Hot,
        };
        let bet_status = pic
            .update_call(
                individual_template_canister_ids[i - 1],
                Principal::self_authenticating((i as usize).to_ne_bytes()),
                "bet_on_currently_viewing_post_v1",
                encode_one(bob_place_bet_arg).unwrap(),
            )
            .map(|reply_payload| {
                let bet_status: Result<BettingStatusV1, BetOnCurrentlyViewingPostError> =
                    match reply_payload {
                        WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                        _ => panic!("\n🛑 place_bet failed\n"),
                    };
                bet_status.unwrap()
            })
            .unwrap();
        // ic_cdk::println!("Bet status: {:?}", bet_status);
        if i % 10 == 0 {
            println!("Betted for {} users", i);
        }
    }

    // Forward timer
    pic.advance_time(Duration::from_secs(60 * 60+10));
    for _ in 0..5 {
        pic.tick();
    }
    // Check rewards

    let last_rewards = pic
        .query_call(
            last_individual_template_canister_id,
            last_individual_template_principal_id,
            "get_profile_details_v3",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let profile: UserProfileDetailsForFrontend = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_rewards failed\n"),
            };
            profile
        })
        .unwrap();
    assert_eq!(last_rewards.lifetime_earnings, 2100);
    // println!("Last rewards: {:?}", last_rewards.lifetime_earnings);

    let last_token_balance = pic
        .query_call(
            last_individual_template_canister_id,
            last_individual_template_principal_id,
            "get_utility_token_balance_v1",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let token_balance: u64 = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_token_balance failed\n"),
            };
            token_balance
        })
        .unwrap();
    assert_eq!(last_token_balance, 2100);
    // println!("Last token balance: {:?}", last_token_balance);

    for i in 1..=110 {
        let rewards = pic
            .query_call(
                individual_template_canister_ids[i - 1],
                Principal::self_authenticating((i as usize).to_ne_bytes()),
                "get_profile_details_v3",
                encode_one(()).unwrap(),
            )
            .map(|reply_payload| {
                let profile: UserProfileDetailsForFrontend = match reply_payload {
                    WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                    _ => panic!("\n🛑 get_rewards failed\n"),
                };
                profile
            })
            .unwrap();
        assert_eq!(rewards.lifetime_earnings, 1080);
        // println!("Rewards for user {}: {:?}", i, rewards.lifetime_earnings);

        let token_balance = pic
            .query_call(
                individual_template_canister_ids[i - 1],
                Principal::self_authenticating((i as usize).to_ne_bytes()),
                "get_utility_token_balance_v1",
                encode_one(()).unwrap(),
            )
            .map(|reply_payload| {
                let token_balance: u64 = match reply_payload {
                    WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                    _ => panic!("\n🛑 get_token_balance failed\n"),
                };
                token_balance
            })
            .unwrap();
        assert_eq!(token_balance, 1080);
        // println!("Token balance for user {}: {:?}", i, token_balance);
    }
}
