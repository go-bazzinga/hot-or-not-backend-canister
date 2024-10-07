use std::{collections::HashMap, time::Duration};

use candid::{encode_one, Principal};
use pocket_ic::{PocketIc, WasmResult};
use shared_utils::{
    canister_specific::{
        individual_user_template::types::{
            arg::{IndividualUserTemplateInitArgs, PlaceBetArg},
            error::BetOnCurrentlyViewingPostError,
            hot_or_not::{
                BetDetails, BetDirection, BetMakerInformedStatus, BetPayout, BettingStatus,
                PlacedBetDetail,
            },
            post::{PostDetailsForFrontend, PostDetailsFromFrontend},
            profile::UserProfileDetailsForFrontend,
        },
        post_cache::types::arg::PostCacheInitArgs,
    },
    common::types::known_principal::{self, KnownPrincipalType},
};
use test_utils::setup::{
    env::pocket_ic_env::get_new_pocket_ic_env,
    test_constants::{
        get_mock_user_alice_principal_id, get_mock_user_bob_principal_id,
        get_mock_user_charlie_principal_id, get_mock_user_dan_principal_id,
    },
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
fn hotornot_game_simulation_test_2() {
    let (pic, known_principals) = get_new_pocket_ic_env();

    let platform_canister_id = known_principals
        .get(&KnownPrincipalType::CanisterIdPlatformOrchestrator)
        .cloned()
        .unwrap();

    let global_admin = known_principals
        .get(&KnownPrincipalType::UserIdGlobalSuperAdmin)
        .cloned()
        .unwrap();

    let application_subnets = pic.topology().get_app_subnets();

    let subnet_orchestrator_canister_id = pic
        .update_call(
            platform_canister_id,
            global_admin,
            "provision_subnet_orchestrator_canister",
            candid::encode_one(application_subnets[0]).unwrap(),
        )
        .map(|res| {
            let canister_id_result: Result<Principal, String> = match res {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("Canister call failed"),
            };
            canister_id_result.unwrap()
        })
        .unwrap();

    for _ in 0..50 {
        pic.tick()
    }

    // Init N canisters
    let mut individual_template_canister_ids = vec![];
    for i in 1..=111 {
        let individual_template_canister_id = pic
            .update_call(
                subnet_orchestrator_canister_id,
                Principal::self_authenticating((i as usize).to_ne_bytes()),
                "get_requester_principals_canister_id_create_if_not_exists",
                candid::encode_one(()).unwrap(),
            )
            .map(|reply_payload| {
                let response: Result<Principal, String> = match reply_payload {
                    WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                    _ => panic!("\n🛑 get requester principals canister id failed\n"),
                };
                response
            })
            .unwrap()
            .unwrap();

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

    let post_used_for_betting = pic
        .query_call(
            last_individual_template_canister_id,
            Principal::anonymous(),
            "get_individual_post_details_by_id",
            candid::encode_args((res1,)).unwrap(),
        )
        .map(|reply_payload| {
            let post_details: PostDetailsForFrontend = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_individual_post_details_by_id failed\n"),
            };
            post_details
        })
        .unwrap();

    pic.add_cycles(last_individual_template_canister_id, 2_000_000_000_000); //recharge by 2T cycles

    // All 500 users bet on the post

    for i in 1..=110 {
        let place_bet_arg = PlaceBetArg {
            post_canister_id: last_individual_template_canister_id,
            post_id: post_used_for_betting.id,
            bet_amount: 100,
            bet_direction: BetDirection::Hot,
        };

        let individual_user_principal = Principal::self_authenticating((i as usize).to_ne_bytes());
        let individual_user_canister = individual_template_canister_ids[i - 1];

        let bet_status = pic
            .update_call(
                individual_user_canister,
                individual_user_principal,
                "bet_on_currently_viewing_post",
                encode_one(place_bet_arg).unwrap(),
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

        assert_eq!(
            bet_status,
            BettingStatus::BettingOpen {
                started_at: post_used_for_betting.created_at,
                number_of_participants: if i <= 100 { i as u8 } else { (i % 100) as u8 },
                ongoing_slot: 1,
                ongoing_room: if i > 100 { 2 } else { 1 },
                has_this_user_participated_in_this_post: Some(true)
            },
            "bet status failed for {i}"
        );

        let token_balance = pic
            .query_call(
                individual_user_canister,
                individual_user_principal,
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
        assert_eq!(token_balance, 900);

        // ic_cdk::println!("Bet status: {:?}", bet_status);
        if i % 10 == 0 {
            println!("Betted for {} users", i);
        }
        pic.tick();
        pic.tick();
    }

    // Forward timer
    pic.advance_time(Duration::from_secs(65 * 60));
    for _ in 0..20 {
        pic.tick();
    }

    let post_creator_cycle_balance = pic.cycle_balance(last_individual_template_canister_id);

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
