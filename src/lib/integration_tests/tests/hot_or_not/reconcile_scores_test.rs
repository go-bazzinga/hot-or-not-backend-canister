use std::{collections::HashMap, time::Duration};

use candid::encode_one;
use pocket_ic::{PocketIc, WasmResult};
use shared_utils::{
    canister_specific::{
        individual_user_template::types::{
            arg::{IndividualUserTemplateInitArgs, PlaceBetArg},
            error::BetOnCurrentlyViewingPostError,
            hot_or_not::{BetDirection, BettingStatus, PlacedBetDetail},
            post::PostDetailsFromFrontend,
            profile::UserProfileDetailsForFrontend,
        },
        post_cache::types::arg::PostCacheInitArgs,
    },
    common::types::known_principal::KnownPrincipalType,
};
use test_utils::setup::test_constants::{
    get_mock_user_alice_principal_id, get_mock_user_bob_principal_id,
    get_mock_user_charlie_principal_id, get_mock_user_dan_principal_id,
};

const OLD_INDIVIDUAL_TEMPLATE_WASM_PATH: &str =
    "../../../target/wasm32-unknown-unknown/release/old_individual_user_template.wasm.gz";
const INDIVIDUAL_TEMPLATE_WASM_PATH: &str =
    "../../../target/wasm32-unknown-unknown/release/individual_user_template.wasm.gz";
const POST_CACHE_WASM_PATH: &str =
    "../../../target/wasm32-unknown-unknown/release/post_cache.wasm.gz";

fn old_individual_template_canister_wasm() -> Vec<u8> {
    std::fs::read(OLD_INDIVIDUAL_TEMPLATE_WASM_PATH).unwrap()
}

fn individual_template_canister_wasm() -> Vec<u8> {
    std::fs::read(INDIVIDUAL_TEMPLATE_WASM_PATH).unwrap()
}

fn post_cache_canister_wasm() -> Vec<u8> {
    std::fs::read(POST_CACHE_WASM_PATH).unwrap()
}

#[test]
#[ignore]
fn reconcile_scores_test() {
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
    let individual_template_wasm_bytes = old_individual_template_canister_wasm();

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

    // Init individual template canister - bob

    let bob_individual_template_canister_id = pic.create_canister();
    pic.add_cycles(bob_individual_template_canister_id, 2_000_000_000_000);

    let individual_template_args = IndividualUserTemplateInitArgs {
        known_principal_ids: Some(known_prinicipal_values.clone()),
        profile_owner: Some(bob_principal_id),
        upgrade_version_number: None,
        url_to_send_canister_metrics_to: None,
        version: "1".to_string(),
        proof_of_participation: None,
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
        proof_of_participation: None,
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
        proof_of_participation: None,
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

    // Forward timer
    pic.advance_time(Duration::from_secs(60 * 60));
    pic.tick();

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
    assert_eq!(bob_rewards.lifetime_earnings, 1000);

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
    assert_eq!(bob_token_balance, 800);

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
    assert_eq!(dan_rewards.lifetime_earnings, 1000);

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
    assert_eq!(dan_token_balance, 800);

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

    // Forward timer
    pic.advance_time(Duration::from_secs(60 * 60));
    pic.tick();

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
    assert_eq!(alice_rewards.lifetime_earnings, 1060);

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
    assert_eq!(alice_token_balance, 960);

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
    assert_eq!(bob_rewards.lifetime_earnings, 1030);

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
    assert_eq!(bob_token_balance, 830);

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
    assert_eq!(dan_rewards.lifetime_earnings, 1000);

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
    assert_eq!(dan_token_balance, 700);

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

    // Upgrade individual canisters

    // Alice individual canister

    let individual_template_wasm_bytes = individual_template_canister_wasm();

    let individual_template_args = IndividualUserTemplateInitArgs {
        known_principal_ids: Some(known_prinicipal_values.clone()),
        profile_owner: Some(alice_principal_id),
        upgrade_version_number: None,
        url_to_send_canister_metrics_to: None,
        version: "1".to_string(),
        proof_of_participation: None
    };
    let individual_template_args_bytes = encode_one(individual_template_args).unwrap();

    let res = pic.upgrade_canister(
        alice_individual_template_canister_id,
        individual_template_wasm_bytes.clone(),
        individual_template_args_bytes,
        None,
    );
    if let Err(e) = res {
        panic!("Error: {:?}", e);
    }

    // Bob individual canister

    let individual_template_args = IndividualUserTemplateInitArgs {
        known_principal_ids: Some(known_prinicipal_values.clone()),
        profile_owner: Some(bob_principal_id),
        upgrade_version_number: None,
        url_to_send_canister_metrics_to: None,
        version: "1".to_string(),
        proof_of_participation: None
    };
    let individual_template_args_bytes = encode_one(individual_template_args).unwrap();

    let res = pic.upgrade_canister(
        bob_individual_template_canister_id,
        individual_template_wasm_bytes.clone(),
        individual_template_args_bytes,
        None,
    );
    if let Err(e) = res {
        panic!("Error: {:?}", e);
    }

    // Dan individual canister

    let individual_template_args = IndividualUserTemplateInitArgs {
        known_principal_ids: Some(known_prinicipal_values.clone()),
        profile_owner: Some(dan_principal_id),
        upgrade_version_number: None,
        url_to_send_canister_metrics_to: None,
        version: "1".to_string(),
        proof_of_participation: None,
    };
    let individual_template_args_bytes = encode_one(individual_template_args).unwrap();

    let res = pic.upgrade_canister(
        dan_individual_template_canister_id,
        individual_template_wasm_bytes.clone(),
        individual_template_args_bytes,
        None,
    );
    if let Err(e) = res {
        panic!("Error: {:?}", e);
    }

    // Charlie individual canister

    let individual_template_args = IndividualUserTemplateInitArgs {
        known_principal_ids: Some(known_prinicipal_values.clone()),
        profile_owner: Some(admin_principal_id),
        upgrade_version_number: None,
        url_to_send_canister_metrics_to: None,
        version: "1".to_string(),
        proof_of_participation: None
    };
    let individual_template_args_bytes = encode_one(individual_template_args).unwrap();

    let res = pic.upgrade_canister(
        charlie_individual_template_canister_id,
        individual_template_wasm_bytes.clone(),
        individual_template_args_bytes,
        None,
    );
    if let Err(e) = res {
        panic!("Error: {:?}", e);
    }

    // Print canisuter ids
    println!(
        "Alice canister id : {} , principal id : {}",
        alice_individual_template_canister_id, alice_principal_id
    );
    println!(
        "Bob canister id : {}, principal id : {}
        ",
        bob_individual_template_canister_id, bob_principal_id
    );
    println!(
        "Dan canister id : {}, principal id : {}
        ",
        dan_individual_template_canister_id, dan_principal_id
    );
    println!(
        "Charlie canister id : {}, principal id : {}",
        charlie_individual_template_canister_id, admin_principal_id
    );

    // Advance timer

    pic.advance_time(Duration::from_secs(100));
    pic.tick();

    // Get hot or not details
    // Alice uses get_hot_or_not_bet_details_for_this_post

    let hot_or_not_bet_details = pic
        .query_call(
            alice_individual_template_canister_id,
            alice_principal_id,
            "get_hot_or_not_bet_details_for_this_post",
            encode_one(0 as u64).unwrap(),
        )
        .map(|reply_payload| {
            let bet_details: BettingStatus = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_hot_or_not_bet_details_for_this_post failed\n"),
            };
            bet_details
        })
        .unwrap();
    println!("Hot or not bet details: {:?}", hot_or_not_bet_details);

    let hot_or_not_bet_details = pic
        .query_call(
            alice_individual_template_canister_id,
            alice_principal_id,
            "get_hot_or_not_bet_details_for_this_post",
            encode_one(1 as u64).unwrap(),
        )
        .map(|reply_payload| {
            let bet_details: BettingStatus = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_hot_or_not_bet_details_for_this_post failed\n"),
            };
            bet_details
        })
        .unwrap();
    println!("Hot or not bet details 2: {:?}", hot_or_not_bet_details);

    // Verify reconcilation
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
