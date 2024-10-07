use std::time::Duration;

use candid::{encode_one, Principal};
use pocket_ic::WasmResult;
use shared_utils::{
    canister_specific::individual_user_template::types::{
        arg::{IndividualUserTemplateInitArgs, PlaceBetArg},
        error::BetOnCurrentlyViewingPostError,
        hot_or_not::{BetDetails, BetDirection, BettingStatus},
        post::PostDetailsFromFrontend,
    },
    common::types::known_principal::KnownPrincipalType,
};
use test_utils::setup::{
    env::pocket_ic_env::get_new_pocket_ic_env,
    test_constants::{
        get_mock_user_alice_principal_id, get_mock_user_bob_principal_id,
        get_mock_user_charlie_principal_id, get_mock_user_dan_principal_id,
        get_mock_user_lucy_principal_id, get_mock_user_tom_principal_id,
    },
};

#[test]
fn when_bet_maker_places_bet_on_a_post_it_is_assigned_a_slot_id_and_the_outcome_resolved_after_that_slot_has_finished(
) {
    let (pocket_ic, known_principal_map) = get_new_pocket_ic_env();

    let alice_principal = get_mock_user_alice_principal_id();
    let bob_princpal: Principal = get_mock_user_bob_principal_id();
    let charlie_principal: Principal = get_mock_user_charlie_principal_id();
    let tom_principal = get_mock_user_tom_principal_id();
    let dan_principal = get_mock_user_dan_principal_id();
    let lucy_principal = get_mock_user_lucy_principal_id();
    let mut bob_winnigs = 0_u64;
    let mut charlie_winnings = 0_u64;
    let mut dan_winnings = 0_u64;
    let mut lucy_winnings = 0_u64;
    let mut tom_winnigns = 0_u64;

    let platform_orchestrator_canister_id = known_principal_map
        .get(&KnownPrincipalType::CanisterIdPlatformOrchestrator)
        .copied()
        .unwrap();

    let global_admin_principal = known_principal_map
        .get(&KnownPrincipalType::UserIdGlobalSuperAdmin)
        .copied()
        .unwrap();

    let application_subnets = pocket_ic.topology().get_app_subnets();

    let subnet_orchestrator_canister_id_0 = pocket_ic
        .update_call(
            platform_orchestrator_canister_id,
            global_admin_principal,
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

    let subnet_orchestrator_canister_id_1 = pocket_ic
        .update_call(
            platform_orchestrator_canister_id,
            global_admin_principal,
            "provision_subnet_orchestrator_canister",
            candid::encode_one(application_subnets[1]).unwrap(),
        )
        .map(|res| {
            let canister_id_result: Result<Principal, String> = match res {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("Canister call failed"),
            };
            canister_id_result.unwrap()
        })
        .unwrap();

    for i in 0..50 {
        pocket_ic.tick();
    }

    //Post Creator Canister
    let alice_canister_id = pocket_ic
        .update_call(
            subnet_orchestrator_canister_id_0,
            alice_principal,
            "get_requester_principals_canister_id_create_if_not_exists",
            candid::encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let canister_id_res: Result<Principal, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!(
                    "\n🛑 get_requester_principals_canister_id_create_if_not_exists failed\n"
                ),
            };
            canister_id_res
        })
        .unwrap()
        .unwrap();

    let bob_canister_id = pocket_ic
        .update_call(
            subnet_orchestrator_canister_id_1,
            bob_princpal,
            "get_requester_principals_canister_id_create_if_not_exists",
            candid::encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let canister_id_res: Result<Principal, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!(
                    "\n🛑 get_requester_principals_canister_id_create_if_not_exists failed\n"
                ),
            };
            canister_id_res
        })
        .unwrap()
        .unwrap();

    let charlie_canister_id = pocket_ic
        .update_call(
            subnet_orchestrator_canister_id_1,
            charlie_principal,
            "get_requester_principals_canister_id_create_if_not_exists",
            candid::encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let canister_id_res: Result<Principal, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!(
                    "\n🛑 get_requester_principals_canister_id_create_if_not_exists failed\n"
                ),
            };
            canister_id_res
        })
        .unwrap()
        .unwrap();

    let dan_canister_id = pocket_ic
        .update_call(
            subnet_orchestrator_canister_id_1,
            dan_principal,
            "get_requester_principals_canister_id_create_if_not_exists",
            candid::encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let canister_id_res: Result<Principal, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!(
                    "\n🛑 get_requester_principals_canister_id_create_if_not_exists failed\n"
                ),
            };
            canister_id_res
        })
        .unwrap()
        .unwrap();

    let lucy_canister_id = pocket_ic
        .update_call(
            subnet_orchestrator_canister_id_1,
            lucy_principal,
            "get_requester_principals_canister_id_create_if_not_exists",
            candid::encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let canister_id_res: Result<Principal, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!(
                    "\n🛑 get_requester_principals_canister_id_create_if_not_exists failed\n"
                ),
            };
            canister_id_res
        })
        .unwrap()
        .unwrap();

    let tom_canister_id = pocket_ic
        .update_call(
            subnet_orchestrator_canister_id_1,
            tom_principal,
            "get_requester_principals_canister_id_create_if_not_exists",
            candid::encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let canister_id_res: Result<Principal, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!(
                    "\n🛑 get_requester_principals_canister_id_create_if_not_exists failed\n"
                ),
            };
            canister_id_res
        })
        .unwrap()
        .unwrap();

    // Create Posts on Alice Canister
    let alice_post_0 = PostDetailsFromFrontend {
        is_nsfw: false,
        description: "This is a fun video to watch".to_string(),
        hashtags: vec!["fun".to_string(), "video".to_string()],
        video_uid: "abcd#1234".to_string(),
        creator_consent_for_inclusion_in_hot_or_not: false,
    };

    let alice_post_id_0 = pocket_ic
        .update_call(
            alice_canister_id,
            alice_principal,
            "add_post_v2",
            encode_one(alice_post_0).unwrap(),
        )
        .map(|reply_payload| {
            let newly_created_post_id_result: Result<u64, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 add_post failed\n"),
            };
            newly_created_post_id_result.unwrap()
        })
        .unwrap();

    //Bob initial token balance
    let bob_initial_token_balance = pocket_ic
        .query_call(
            bob_canister_id,
            bob_princpal,
            "get_utility_token_balance",
            candid::encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let token_balance: u64 = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                WasmResult::Reject(e) => panic!("bob initial token balance fetch failed {e}"),
            };
            token_balance
        })
        .unwrap();
    assert_eq!(bob_initial_token_balance, 1_000);

    /****** 4th Slot Post 0 *************/
    pocket_ic.advance_time(Duration::from_secs(3 * 60 * 60)); // send the bet in 4th slot
    pocket_ic.tick();

    let bob_bet_arg = PlaceBetArg {
        post_canister_id: alice_canister_id,
        post_id: alice_post_id_0,
        bet_amount: 100,
        bet_direction: BetDirection::Hot,
    };

    let bet_status = pocket_ic
        .update_call(
            bob_canister_id,
            bob_princpal,
            "bet_on_currently_viewing_post",
            encode_one(bob_bet_arg).unwrap(),
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
    bob_winnigs += 80;

    if let BettingStatus::BettingOpen { ongoing_slot, .. } = bet_status {
        assert_eq!(ongoing_slot, 4)
    } else {
        assert!(
            false,
            "Betting Status should be open and ongoing_slot should be 4"
        )
    }
    /**************************************/

    /****** 10th Slot Post 0 *************/
    pocket_ic.advance_time(Duration::from_secs(6 * 60 * 60));
    pocket_ic.tick();

    let charlie_bet_arg = PlaceBetArg {
        post_canister_id: alice_canister_id,
        post_id: alice_post_id_0,
        bet_amount: 50,
        bet_direction: BetDirection::Not,
    };
    let bet_status = pocket_ic
        .update_call(
            charlie_canister_id,
            charlie_principal,
            "bet_on_currently_viewing_post",
            encode_one(charlie_bet_arg).unwrap(),
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

    if let BettingStatus::BettingOpen { ongoing_slot, .. } = bet_status {
        assert_eq!(ongoing_slot, 10)
    } else {
        assert!(
            false,
            "Betting Status should be open and ongoing_slot should be 10"
        )
    }
    charlie_winnings += 40;

    /********************************/

    /****** 11th Slot Post 0 *************/
    pocket_ic.advance_time(Duration::from_secs(1 * 60 * 60));
    pocket_ic.tick();

    let dan_bet_arg = PlaceBetArg {
        post_canister_id: alice_canister_id,
        post_id: alice_post_id_0,
        bet_amount: 100,
        bet_direction: BetDirection::Hot,
    };
    let bet_status = pocket_ic
        .update_call(
            dan_canister_id,
            dan_principal,
            "bet_on_currently_viewing_post",
            encode_one(dan_bet_arg).unwrap(),
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

    if let BettingStatus::BettingOpen { ongoing_slot, .. } = bet_status {
        assert_eq!(ongoing_slot, 11)
    } else {
        assert!(
            false,
            "Betting Status should be open and ongoing_slot should be 10"
        )
    }

    pocket_ic.advance_time(Duration::from_secs(20 * 60));
    pocket_ic.tick();

    let lucy_bet_arg = PlaceBetArg {
        post_canister_id: alice_canister_id,
        post_id: alice_post_id_0,
        bet_amount: 50,
        bet_direction: BetDirection::Hot,
    };
    let bet_status = pocket_ic
        .update_call(
            lucy_canister_id,
            lucy_principal,
            "bet_on_currently_viewing_post",
            encode_one(lucy_bet_arg).unwrap(),
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

    if let BettingStatus::BettingOpen { ongoing_slot, .. } = bet_status {
        assert_eq!(ongoing_slot, 11)
    } else {
        assert!(
            false,
            "Betting Status should be open and ongoing_slot should be 10"
        )
    }

    pocket_ic.advance_time(Duration::from_secs(20 * 60));
    pocket_ic.tick();

    let tom_bet_arg = PlaceBetArg {
        post_canister_id: alice_canister_id,
        post_id: alice_post_id_0,
        bet_amount: 50,
        bet_direction: BetDirection::Hot,
    };
    let bet_status = pocket_ic
        .update_call(
            tom_canister_id,
            tom_principal,
            "bet_on_currently_viewing_post",
            encode_one(tom_bet_arg).unwrap(),
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

    if let BettingStatus::BettingOpen { ongoing_slot, .. } = bet_status {
        assert_eq!(ongoing_slot, 11)
    } else {
        assert!(
            false,
            "Betting Status should be open and ongoing_slot should be 10"
        )
    }

    for _ in 0..10 {
        pocket_ic.tick();
    }

    dan_winnings += 80;
    lucy_winnings += 40;
    tom_winnigns += 40;

    /********************************/

    // /********* Upgrade Caniters ****************/
    pocket_ic.advance_time(Duration::from_secs(20 * 60));
    pocket_ic.tick();

    let individual_user_template_wasm = include_bytes!(
        "../../../../../target/wasm32-unknown-unknown/release/individual_user_template.wasm.gz"
    );

    let individual_user_template_upgrade_args = IndividualUserTemplateInitArgs {
        version: "v1.1.0".into(),
        upgrade_version_number: None,
        known_principal_ids: None,
        profile_owner: None,
        url_to_send_canister_metrics_to: None,
        proof_of_participation: None,
    };

    pocket_ic
        .upgrade_canister(
            alice_canister_id,
            individual_user_template_wasm.to_vec(),
            candid::encode_one(individual_user_template_upgrade_args).unwrap(),
            Some(subnet_orchestrator_canister_id_0),
        )
        .unwrap();

    pocket_ic.tick();

    // /********************************/
    // Assert on final results

    pocket_ic.advance_time(Duration::from_secs(50 * 60 * 60));
    for _ in 0..20 {
        pocket_ic.tick();
    }

    let bob_final_token_balance: u64 = pocket_ic
        .query_call(
            bob_canister_id,
            bob_princpal,
            "get_utility_token_balance",
            candid::encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let token_balance: u64 = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                WasmResult::Reject(e) => panic!("bob initial token balance fetch failed {e}"),
            };
            token_balance
        })
        .unwrap();

    assert_eq!(
        bob_final_token_balance,
        bob_initial_token_balance + bob_winnigs
    );

    let charlie_final_token_balance: u64 = pocket_ic
        .query_call(
            charlie_canister_id,
            charlie_principal,
            "get_utility_token_balance",
            candid::encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let token_balance: u64 = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                WasmResult::Reject(e) => panic!("bob initial token balance fetch failed {e}"),
            };
            token_balance
        })
        .unwrap();

    assert_eq!(charlie_final_token_balance, 1_000 + charlie_winnings);

    let dan_final_token_balance: u64 = pocket_ic
        .query_call(
            dan_canister_id,
            dan_principal,
            "get_utility_token_balance",
            candid::encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let token_balance: u64 = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                WasmResult::Reject(e) => panic!("bob initial token balance fetch failed {e}"),
            };
            token_balance
        })
        .unwrap();

    assert_eq!(dan_final_token_balance, 1_000 + dan_winnings);

    let lucy_final_token_balance: u64 = pocket_ic
        .query_call(
            lucy_canister_id,
            lucy_principal,
            "get_utility_token_balance",
            candid::encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let token_balance: u64 = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                WasmResult::Reject(e) => panic!("bob initial token balance fetch failed {e}"),
            };
            token_balance
        })
        .unwrap();

    assert_eq!(lucy_final_token_balance, 1_000 + lucy_winnings);

    let tom_final_token_balance: u64 = pocket_ic
        .query_call(
            tom_canister_id,
            tom_principal,
            "get_utility_token_balance",
            candid::encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let token_balance: u64 = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                WasmResult::Reject(e) => panic!("bob initial token balance fetch failed {e}"),
            };
            token_balance
        })
        .unwrap();

    assert_eq!(tom_final_token_balance, 1_000 + tom_winnigns);
}
