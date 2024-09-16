use candid::Principal;
use ic_cdk_macros::update;
use shared_utils::{
    canister_specific::individual_user_template::types::{
        arg::PlaceBetArg,
        error::BetOnCurrentlyViewingPostError,
        hot_or_not::{
            BetOutcomeForBetMaker, BettingStatus, BettingStatusV1, PlacedBetDetail,
            PlacedBetDetailV1,
        },
    },
    common::{
        types::utility_token::token_event::{StakeEvent, TokenEvent, TokenEventV1},
        utils::system_time,
    },
};

use crate::{
    api::canister_management::update_last_access_time::update_last_canister_functionality_access_time,
    data_model::CanisterData, CANISTER_DATA,
};

#[deprecated(note = "use bet_on_currently_viewing_post_v1 instead")]
#[update]
async fn bet_on_currently_viewing_post(
    place_bet_arg: PlaceBetArg,
) -> Result<BettingStatus, BetOnCurrentlyViewingPostError> {
    let bet_maker_principal_id = ic_cdk::caller();
    let current_time = system_time::get_current_system_time_from_ic();

    CANISTER_DATA.with(|canister_data_ref_cell| {
        validate_incoming_bet(
            &canister_data_ref_cell.borrow(),
            &bet_maker_principal_id,
            &place_bet_arg,
        )
    })?;

    update_last_canister_functionality_access_time();
    update_token_balance_before_bet_happens(place_bet_arg.bet_amount);

    let response = ic_cdk::call::<_, (Result<BettingStatus, BetOnCurrentlyViewingPostError>,)>(
        place_bet_arg.post_canister_id,
        "receive_bet_from_bet_makers_canister",
        (
            place_bet_arg.clone(),
            CANISTER_DATA.with(|canister_data_ref_cell| {
                canister_data_ref_cell
                    .borrow()
                    .profile
                    .principal_id
                    .unwrap()
            }),
        ),
    )
    .await
    .map_err(|_| {
        update_token_balance_after_bet_placement_fails(place_bet_arg.bet_amount);
        BetOnCurrentlyViewingPostError::PostCreatorCanisterCallFailed
    })?
    .0?;

    match response {
        BettingStatus::BettingClosed => {
            update_token_balance_after_bet_placement_fails(place_bet_arg.bet_amount);
            return Err(BetOnCurrentlyViewingPostError::BettingClosed);
        }
        BettingStatus::BettingOpen {
            ongoing_slot,
            ongoing_room,
            ..
        } => {
            CANISTER_DATA.with(|canister_data_ref_cell| {
                let canister_data = &mut canister_data_ref_cell.borrow_mut();

                let my_token_balance = &mut canister_data.my_token_balance;
                my_token_balance.handle_token_event(TokenEvent::Stake {
                    amount: place_bet_arg.bet_amount,
                    details: StakeEvent::BetOnHotOrNotPost {
                        post_canister_id: place_bet_arg.post_canister_id,
                        post_id: place_bet_arg.post_id,
                        bet_amount: place_bet_arg.bet_amount,
                        bet_direction: place_bet_arg.bet_direction.clone(),
                    },
                    timestamp: current_time,
                });

                let all_hot_or_not_bets_placed = &mut canister_data.all_hot_or_not_bets_placed;
                all_hot_or_not_bets_placed.insert(
                    (place_bet_arg.post_canister_id, place_bet_arg.post_id),
                    PlacedBetDetail {
                        canister_id: place_bet_arg.post_canister_id,
                        post_id: place_bet_arg.post_id,
                        slot_id: ongoing_slot,
                        room_id: ongoing_room,
                        bet_direction: place_bet_arg.bet_direction,
                        bet_placed_at: current_time,
                        amount_bet: place_bet_arg.bet_amount,
                        outcome_received: BetOutcomeForBetMaker::default(),
                    },
                );
            });
        }
    }

    Ok(response)
}

#[update]
async fn bet_on_currently_viewing_post_v1(
    place_bet_arg: PlaceBetArg,
) -> Result<BettingStatusV1, BetOnCurrentlyViewingPostError> {
    let bet_maker_principal_id = ic_cdk::caller();
    let current_time = system_time::get_current_system_time_from_ic();

    CANISTER_DATA.with(|canister_data_ref_cell| {
        validate_incoming_bet_v1(
            &canister_data_ref_cell.borrow(),
            &bet_maker_principal_id,
            &place_bet_arg,
        )
    })?;

    update_last_canister_functionality_access_time();
    update_token_balance_before_bet_happens(place_bet_arg.bet_amount);

    let response = ic_cdk::call::<_, (Result<BettingStatusV1, BetOnCurrentlyViewingPostError>,)>(
        place_bet_arg.post_canister_id,
        "receive_bet_from_bet_makers_canister_v1",
        (
            place_bet_arg.clone(),
            CANISTER_DATA.with(|canister_data_ref_cell| {
                canister_data_ref_cell
                    .borrow()
                    .profile
                    .principal_id
                    .unwrap()
            }),
        ),
    )
    .await
    .map_err(|_| {
        update_token_balance_after_bet_placement_fails(place_bet_arg.bet_amount);
        BetOnCurrentlyViewingPostError::PostCreatorCanisterCallFailed
    })?
    .0?;

    match response {
        // this case should never match in yral game implementation
        BettingStatusV1::BettingClosed => {
            update_token_balance_after_bet_placement_fails(place_bet_arg.bet_amount);
            return Err(BetOnCurrentlyViewingPostError::BettingClosed);
        }
        BettingStatusV1::BettingOpen {
            ongoing_slot,
            ongoing_room,
            ..
        } => {
            CANISTER_DATA.with(|canister_data_ref_cell| {
                let canister_data = &mut canister_data_ref_cell.borrow_mut();

                let my_token_balance = &mut canister_data.my_token_balance_v1;
                my_token_balance.handle_token_event(TokenEventV1::Stake {
                    amount: place_bet_arg.bet_amount,
                    details: StakeEvent::BetOnHotOrNotPost {
                        post_canister_id: place_bet_arg.post_canister_id,
                        post_id: place_bet_arg.post_id,
                        bet_amount: place_bet_arg.bet_amount,
                        bet_direction: place_bet_arg.bet_direction.clone(),
                    },
                    timestamp: current_time,
                });

                let all_hot_or_not_bets_placed = &mut canister_data.all_hot_or_not_bets_placed_v1;
                all_hot_or_not_bets_placed.insert(
                    (place_bet_arg.post_canister_id, place_bet_arg.post_id),
                    PlacedBetDetailV1 {
                        canister_id: place_bet_arg.post_canister_id,
                        post_id: place_bet_arg.post_id,
                        slot_id: ongoing_slot,
                        room_id: ongoing_room,
                        bet_direction: place_bet_arg.bet_direction,
                        bet_placed_at: current_time,
                        amount_bet: place_bet_arg.bet_amount,
                        outcome_received: BetOutcomeForBetMaker::default(),
                    },
                );
            });
        }
    }

    Ok(response)
}

// this #[update] is for local testing only see: src/lib/integration_tests/tests/upgrade/excessive_tokens_test.rs
// #[update]
pub fn update_token_balance_before_bet_happens(bet_amount: u64) {
    CANISTER_DATA.with_borrow_mut(|canister_data| {
        canister_data
            .my_token_balance
            .adjust_balance_pre_bet(bet_amount);
    });
}

fn update_token_balance_after_bet_placement_fails(bet_amount: u64) {
    CANISTER_DATA.with_borrow_mut(|canister_data| {
        canister_data
            .my_token_balance
            .adjust_balance_for_failed_bet_placement(bet_amount);
    });
}

#[deprecated(note = "use validate_incoming_bet_v1 instead")]
fn validate_incoming_bet(
    canister_data: &CanisterData,
    bet_maker_principal_id: &Principal,
    place_bet_arg: &PlaceBetArg,
) -> Result<(), BetOnCurrentlyViewingPostError> {
    if *bet_maker_principal_id == Principal::anonymous() {
        return Err(BetOnCurrentlyViewingPostError::UserNotLoggedIn);
    }

    let profile_owner = canister_data
        .profile
        .principal_id
        .ok_or(BetOnCurrentlyViewingPostError::UserPrincipalNotSet)?;

    if *bet_maker_principal_id != profile_owner {
        return Err(BetOnCurrentlyViewingPostError::Unauthorized);
    }

    let utlility_token_balance = canister_data.my_token_balance.get_utility_token_balance();

    if utlility_token_balance < place_bet_arg.bet_amount {
        return Err(BetOnCurrentlyViewingPostError::InsufficientBalance);
    }

    if canister_data
        .all_hot_or_not_bets_placed
        .contains_key(&(place_bet_arg.post_canister_id, place_bet_arg.post_id))
    {
        return Err(BetOnCurrentlyViewingPostError::UserAlreadyParticipatedInThisPost);
    }

    Ok(())
}

fn validate_incoming_bet_v1(
    canister_data: &CanisterData,
    bet_maker_principal_id: &Principal,
    place_bet_arg: &PlaceBetArg,
) -> Result<(), BetOnCurrentlyViewingPostError> {
    if *bet_maker_principal_id == Principal::anonymous() {
        return Err(BetOnCurrentlyViewingPostError::UserNotLoggedIn);
    }

    let profile_owner = canister_data
        .profile
        .principal_id
        .ok_or(BetOnCurrentlyViewingPostError::UserPrincipalNotSet)?;

    if *bet_maker_principal_id != profile_owner {
        return Err(BetOnCurrentlyViewingPostError::Unauthorized);
    }

    let utlility_token_balance = canister_data
        .my_token_balance_v1
        .get_utility_token_balance();

    if utlility_token_balance < place_bet_arg.bet_amount {
        return Err(BetOnCurrentlyViewingPostError::InsufficientBalance);
    }

    if canister_data
        .all_hot_or_not_bets_placed_v1
        .contains_key(&(place_bet_arg.post_canister_id, place_bet_arg.post_id))
    {
        return Err(BetOnCurrentlyViewingPostError::UserAlreadyParticipatedInThisPost);
    }

    Ok(())
}

#[cfg(test)]
mod test_validate_incoming_bet_v1_mod {
    use std::time::SystemTime;

    use shared_utils::{
        canister_specific::individual_user_template::types::hot_or_not::BetDirection,
        common::types::utility_token::token_event::NewSlotType,
    };
    use test_utils::setup::test_constants::{
        get_mock_user_alice_canister_id, get_mock_user_alice_principal_id,
        get_mock_user_bob_principal_id,
    };

    use super::*;

    #[test]
    fn test_validate_incoming_bet_v1() {
        let mut canister_data = CanisterData::default();

        let result = validate_incoming_bet_v1(
            &canister_data,
            &Principal::anonymous(),
            &PlaceBetArg {
                post_canister_id: get_mock_user_alice_canister_id(),
                post_id: 0,
                bet_amount: 100,
                bet_direction: BetDirection::Hot,
            },
        );

        assert_eq!(result, Err(BetOnCurrentlyViewingPostError::UserNotLoggedIn));

        canister_data.profile.principal_id = Some(get_mock_user_alice_principal_id());

        let result = validate_incoming_bet_v1(
            &canister_data,
            &get_mock_user_bob_principal_id(),
            &PlaceBetArg {
                post_canister_id: get_mock_user_alice_canister_id(),
                post_id: 0,
                bet_amount: 100,
                bet_direction: BetDirection::Hot,
            },
        );

        assert_eq!(result, Err(BetOnCurrentlyViewingPostError::Unauthorized));

        let result = validate_incoming_bet_v1(
            &canister_data,
            &get_mock_user_alice_principal_id(),
            &PlaceBetArg {
                post_canister_id: get_mock_user_alice_canister_id(),
                post_id: 0,
                bet_amount: 100,
                bet_direction: BetDirection::Hot,
            },
        );

        assert_eq!(
            result,
            Err(BetOnCurrentlyViewingPostError::InsufficientBalance)
        );

        canister_data.my_token_balance_v1.utility_token_balance = 1000;

        let result = validate_incoming_bet_v1(
            &canister_data,
            &get_mock_user_alice_principal_id(),
            &PlaceBetArg {
                post_canister_id: get_mock_user_alice_canister_id(),
                post_id: 0,
                bet_amount: 100,
                bet_direction: BetDirection::Hot,
            },
        );

        assert_eq!(result, Ok(()));

        canister_data.all_hot_or_not_bets_placed_v1.insert(
            (get_mock_user_alice_canister_id(), 0),
            PlacedBetDetailV1 {
                canister_id: get_mock_user_alice_canister_id(),
                post_id: 0,
                slot_id: NewSlotType(1),
                room_id: 1,
                amount_bet: 100,
                bet_direction: BetDirection::Hot,
                bet_placed_at: SystemTime::now(),
                outcome_received: BetOutcomeForBetMaker::default(),
            },
        );

        let result = validate_incoming_bet_v1(
            &canister_data,
            &get_mock_user_alice_principal_id(),
            &PlaceBetArg {
                post_canister_id: get_mock_user_alice_canister_id(),
                post_id: 0,
                bet_amount: 100,
                bet_direction: BetDirection::Hot,
            },
        );

        assert_eq!(
            result,
            Err(BetOnCurrentlyViewingPostError::UserAlreadyParticipatedInThisPost)
        );
    }
}

#[cfg(test)]
mod test_validate_incoming_bet {
    use std::time::SystemTime;

    use shared_utils::canister_specific::individual_user_template::types::hot_or_not::BetDirection;
    use test_utils::setup::test_constants::{
        get_mock_user_alice_canister_id, get_mock_user_alice_principal_id,
        get_mock_user_bob_principal_id,
    };

    use super::*;

    #[test]
    fn test_validate_incoming_bet() {
        let mut canister_data = CanisterData::default();

        let result = validate_incoming_bet(
            &canister_data,
            &Principal::anonymous(),
            &PlaceBetArg {
                post_canister_id: get_mock_user_alice_canister_id(),
                post_id: 0,
                bet_amount: 100,
                bet_direction: BetDirection::Hot,
            },
        );

        assert_eq!(result, Err(BetOnCurrentlyViewingPostError::UserNotLoggedIn));

        canister_data.profile.principal_id = Some(get_mock_user_alice_principal_id());

        let result = validate_incoming_bet(
            &canister_data,
            &get_mock_user_bob_principal_id(),
            &PlaceBetArg {
                post_canister_id: get_mock_user_alice_canister_id(),
                post_id: 0,
                bet_amount: 100,
                bet_direction: BetDirection::Hot,
            },
        );

        assert_eq!(result, Err(BetOnCurrentlyViewingPostError::Unauthorized));

        let result = validate_incoming_bet(
            &canister_data,
            &get_mock_user_alice_principal_id(),
            &PlaceBetArg {
                post_canister_id: get_mock_user_alice_canister_id(),
                post_id: 0,
                bet_amount: 100,
                bet_direction: BetDirection::Hot,
            },
        );

        assert_eq!(
            result,
            Err(BetOnCurrentlyViewingPostError::InsufficientBalance)
        );

        canister_data.my_token_balance.utility_token_balance = 1000;

        let result = validate_incoming_bet(
            &canister_data,
            &get_mock_user_alice_principal_id(),
            &PlaceBetArg {
                post_canister_id: get_mock_user_alice_canister_id(),
                post_id: 0,
                bet_amount: 100,
                bet_direction: BetDirection::Hot,
            },
        );

        assert_eq!(result, Ok(()));

        canister_data.all_hot_or_not_bets_placed.insert(
            (get_mock_user_alice_canister_id(), 0),
            PlacedBetDetail {
                canister_id: get_mock_user_alice_canister_id(),
                post_id: 0,
                slot_id: 1,
                room_id: 1,
                amount_bet: 100,
                bet_direction: BetDirection::Hot,
                bet_placed_at: SystemTime::now(),
                outcome_received: BetOutcomeForBetMaker::default(),
            },
        );

        let result = validate_incoming_bet(
            &canister_data,
            &get_mock_user_alice_principal_id(),
            &PlaceBetArg {
                post_canister_id: get_mock_user_alice_canister_id(),
                post_id: 0,
                bet_amount: 100,
                bet_direction: BetDirection::Hot,
            },
        );

        assert_eq!(
            result,
            Err(BetOnCurrentlyViewingPostError::UserAlreadyParticipatedInThisPost)
        );
    }
}
