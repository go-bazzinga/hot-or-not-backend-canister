use candid::{Nat, Principal};
use test_utils::setup::{env::pocket_ic_env::execute_query, test_constants::{get_mock_user_alice_principal_id, get_mock_user_bob_principal_id, get_mock_user_charlie_principal_id, get_mock_user_dan_principal_id, get_mock_user_tom_principal_id}};

use crate::{tokens_to_e8s, CDaoHarness};


#[test]
fn cdao_airdrop_test_with_referral_height_3() {
    let harness = CDaoHarness::init();

    let alice_principal = get_mock_user_alice_principal_id();
    let bob_principal = get_mock_user_bob_principal_id();
    let charlie_principal = get_mock_user_charlie_principal_id();
    let dan_principal = get_mock_user_dan_principal_id();
    let tom_principal = get_mock_user_tom_principal_id();

    let alice = harness.provision_individual_canister(alice_principal, None);
    let atoken = harness.create_new_token(alice.clone());
    let mut alice_can_atoken_bal = harness.icrc1_balance(atoken.ledger, alice.user_canister_id);
    let tx_fee: Nat = execute_query(
        &harness.pic,
        Principal::anonymous(),
        atoken.ledger,
        "icrc1_fee",
        &()
    );

    let bob = harness.provision_individual_canister(bob_principal, Some(alice.clone()));
    let _charlie = harness.provision_individual_canister(charlie_principal, Some(alice.clone()));
    // wait for referral rewards to spread
    for _ in 0..20 {
        harness.pic.tick();
    }
    alice_can_atoken_bal -= (tokens_to_e8s(1) + tx_fee.clone()) * 2u32;

    let bob_atoken_bal = harness.icrc1_balance(atoken.ledger, bob_principal);
    let charlie_atoken_bal = harness.icrc1_balance(atoken.ledger, charlie_principal);
    assert_eq!(bob_atoken_bal, tokens_to_e8s(1));
    assert_eq!(charlie_atoken_bal, tokens_to_e8s(1));

    let alice_can_atoken_bal_after = harness.icrc1_balance(atoken.ledger, alice.user_canister_id);
    assert_eq!(alice_can_atoken_bal_after, alice_can_atoken_bal);

    let btoken = harness.create_new_token(bob.clone());
    for _ in 0..20 {
        harness.pic.tick();
    }

    let alice_btoken_bal = harness.icrc1_balance(btoken.ledger, alice_principal);
    assert_eq!(alice_btoken_bal, tokens_to_e8s(1));

    let mut bob_can_btoken_bal = harness.icrc1_balance(btoken.ledger, bob.user_canister_id);

    let dan = harness.provision_individual_canister(dan_principal, Some(bob.clone()));
    let tom = harness.provision_individual_canister(tom_principal, Some(bob.clone()));
    for _ in 0..30 {
        harness.pic.tick();
    }
    bob_can_btoken_bal -= (tokens_to_e8s(1) + tx_fee.clone()) * 2u32;
    alice_can_atoken_bal -= (tokens_to_e8s(1) + tx_fee.clone()) * 2u32;

    let dan_atoken_bal = harness.icrc1_balance(atoken.ledger, dan_principal);
    let tom_atoken_bal = harness.icrc1_balance(atoken.ledger, tom_principal);
    let dan_btoken_bal = harness.icrc1_balance(btoken.ledger, dan_principal);
    let tom_btoken_bal = harness.icrc1_balance(btoken.ledger, tom_principal);

    assert_eq!(dan_atoken_bal, tokens_to_e8s(1));
    assert_eq!(tom_atoken_bal, tokens_to_e8s(1));
    assert_eq!(dan_btoken_bal, tokens_to_e8s(1));
    assert_eq!(tom_btoken_bal, tokens_to_e8s(1));

    let alice_can_atoken_bal_after = harness.icrc1_balance(atoken.ledger, alice.user_canister_id);
    let bob_can_btoken_bal_after = harness.icrc1_balance(btoken.ledger, bob.user_canister_id);
    assert_eq!(alice_can_atoken_bal_after, alice_can_atoken_bal);
    assert_eq!(bob_can_btoken_bal_after, bob_can_btoken_bal);

    let a2token = harness.create_new_token(alice);
    for _ in 0..30 {
        harness.pic.tick();
    }

    let bob_a2token_bal = harness.icrc1_balance(a2token.ledger, bob_principal);
    let charlie_a2token_bal = harness.icrc1_balance(a2token.ledger, charlie_principal);
    let dan_a2token_bal = harness.icrc1_balance(a2token.ledger, dan_principal);
    let tom_a2token_bal = harness.icrc1_balance(a2token.ledger, tom_principal);

    assert_eq!(bob_a2token_bal, tokens_to_e8s(1));
    assert_eq!(charlie_a2token_bal, tokens_to_e8s(1));
    assert_eq!(dan_a2token_bal, tokens_to_e8s(1));
    assert_eq!(tom_a2token_bal, tokens_to_e8s(1));

    let dtoken = harness.create_new_token(dan.clone());
    let ttoken = harness.create_new_token(tom.clone());
    for _ in 0..30 {
        harness.pic.tick();
    }

    let bob_dtoken_bal = harness.icrc1_balance(dtoken.ledger, bob_principal);
    let charlie_dtoken_bal = harness.icrc1_balance(dtoken.ledger, charlie_principal);
    let tom_dtoken_bal = harness.icrc1_balance(dtoken.ledger, tom_principal);
    let alice_dtoken_bal = harness.icrc1_balance(dtoken.ledger, alice_principal);

    let bob_ttoken_bal = harness.icrc1_balance(ttoken.ledger, bob_principal); 
    let charlie_ttoken_bal = harness.icrc1_balance(ttoken.ledger, charlie_principal);
    let dan_ttoken_bal = harness.icrc1_balance(ttoken.ledger, dan_principal);
    let alice_ttoken_bal = harness.icrc1_balance(ttoken.ledger, alice_principal);

    assert_eq!(bob_dtoken_bal, tokens_to_e8s(1));
    assert_eq!(alice_dtoken_bal, tokens_to_e8s(1));
    assert_eq!(bob_ttoken_bal, tokens_to_e8s(1));
    assert_eq!(alice_ttoken_bal, tokens_to_e8s(1));

    assert_eq!(charlie_dtoken_bal, 0u32);
    assert_eq!(tom_dtoken_bal, 0u32);
    assert_eq!(charlie_ttoken_bal, 0u32);
    assert_eq!(dan_ttoken_bal, 0u32);
}