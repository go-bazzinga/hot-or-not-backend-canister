use std::{collections::BTreeMap, time::SystemTime};

use candid::{CandidType, Deserialize, Principal};
use serde::Serialize;

use super::{error::BetOnCurrentlyViewingPostError, post::Post};

#[derive(CandidType, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum BettingStatus {
    BettingOpen {
        started_at: SystemTime,
        number_of_participants: u8,
        ongoing_slot: u8,
        ongoing_room: u64,
        has_this_user_participated_in_this_post: Option<bool>,
    },
    BettingClosed,
}

pub const MAXIMUM_NUMBER_OF_SLOTS: u8 = 48;
pub const DURATION_OF_EACH_SLOT_IN_SECONDS: u64 = 60 * 60;
pub const TOTAL_DURATION_OF_ALL_SLOTS_IN_SECONDS: u64 =
    MAXIMUM_NUMBER_OF_SLOTS as u64 * DURATION_OF_EACH_SLOT_IN_SECONDS;

#[derive(CandidType)]
pub enum UserStatusForSpecificHotOrNotPost {
    NotParticipatedYet,
    AwaitingResult(BetDetail),
    ResultAnnounced(BetResult),
}

#[derive(CandidType)]
pub enum BetResult {
    Won(u64),
    Lost,
    Draw,
}

#[derive(CandidType)]
pub struct BetDetail {
    amount: u64,
    bet_direction: BetDirection,
    bet_made_at: SystemTime,
}

#[derive(CandidType, Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
pub enum BetDirection {
    Hot,
    Not,
}

#[derive(Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct HotOrNotBetId {
    pub canister_id: Principal,
    pub post_id: u64,
}

#[derive(CandidType, Clone, Deserialize, Debug, Serialize, Default)]
pub struct HotOrNotDetails {
    pub score: u64,
    pub aggregate_stats: AggregateStats,
    pub slot_history: BTreeMap<SlotId, SlotDetails>,
}

#[derive(CandidType, Clone, Deserialize, Debug, Serialize, Default)]
pub struct AggregateStats {
    pub total_number_of_hot_bets: u64,
    pub total_number_of_not_bets: u64,
    pub total_amount_bet: u64,
}

pub type SlotId = u8;

#[derive(CandidType, Clone, Deserialize, Default, Debug, Serialize)]
pub struct SlotDetails {
    pub room_details: BTreeMap<RoomId, RoomDetails>,
}

pub type RoomId = u64;

#[derive(CandidType, Clone, Deserialize, Default, Debug, Serialize)]
pub struct RoomDetails {
    pub bets_made: BTreeMap<BetMaker, BetDetails>,
}

pub type BetMaker = Principal;

#[derive(CandidType, Clone, Deserialize, Debug, Serialize)]
pub struct BetDetails {
    pub amount: u64,
    pub bet_direction: BetDirection,
}

impl Post {
    pub fn get_hot_or_not_betting_status_for_this_post(
        &self,
        current_time_when_request_being_made: &SystemTime,
        current_request_maker: &Principal,
    ) -> BettingStatus {
        let betting_status = match current_time_when_request_being_made
            .duration_since(self.created_at)
            .unwrap()
            .as_secs()
        {
            // * contest is still ongoing
            0..=TOTAL_DURATION_OF_ALL_SLOTS_IN_SECONDS => {
                let started_at = self.created_at;
                let numerator = current_time_when_request_being_made
                    .duration_since(started_at)
                    .unwrap()
                    .as_secs();

                let denominator = DURATION_OF_EACH_SLOT_IN_SECONDS;
                let currently_ongoing_slot = ((numerator / denominator) + 1) as u8;

                let temp_hot_or_not_default = &HotOrNotDetails::default();
                let temp_slot_details_default = &SlotDetails::default();
                let room_details = &self
                    .hot_or_not_details
                    .as_ref()
                    .unwrap_or(temp_hot_or_not_default)
                    .slot_history
                    .get(&currently_ongoing_slot)
                    .unwrap_or(temp_slot_details_default)
                    .room_details;

                let temp_room_details_default = &RoomDetails::default();
                let currently_active_room = room_details
                    .last_key_value()
                    .unwrap_or((&1, temp_room_details_default));
                let number_of_participants = currently_active_room.1.bets_made.len() as u8;
                BettingStatus::BettingOpen {
                    started_at,
                    number_of_participants,
                    ongoing_slot: currently_ongoing_slot,
                    ongoing_room: *currently_active_room.0 as u64,
                    has_this_user_participated_in_this_post: if *current_request_maker
                        == Principal::anonymous()
                    {
                        None
                    } else {
                        Some(
                            self.has_this_principal_already_bet_on_this_post(current_request_maker),
                        )
                    },
                }
            }
            // * contest is over
            _ => BettingStatus::BettingClosed,
        };

        betting_status
    }

    pub fn has_this_principal_already_bet_on_this_post(
        &self,
        principal_making_bet: &Principal,
    ) -> bool {
        self.hot_or_not_details
            .as_ref()
            .unwrap()
            .slot_history
            .iter()
            .map(|(_, slot_details)| slot_details.room_details.iter())
            .flatten()
            .map(|(_, room_details)| room_details.bets_made.iter())
            .flatten()
            .any(|(principal, _)| principal == principal_making_bet)
    }

    pub fn place_hot_or_not_bet(
        &mut self,
        api_caller: &Principal,
        bet_amount: u64,
        bet_direction: &BetDirection,
        current_time_when_request_being_made: &SystemTime,
    ) -> Result<BettingStatus, BetOnCurrentlyViewingPostError> {
        let betting_status = self.get_hot_or_not_betting_status_for_this_post(
            current_time_when_request_being_made,
            api_caller,
        );

        match betting_status {
            BettingStatus::BettingOpen {
                ongoing_slot,
                ongoing_room,
                has_this_user_participated_in_this_post,
                ..
            } => {
                if has_this_user_participated_in_this_post.is_none()
                    || has_this_user_participated_in_this_post.unwrap()
                {
                    return Err(BetOnCurrentlyViewingPostError::UserAlreadyParticipatedInThisPost);
                }

                let mut hot_or_not_details = self
                    .hot_or_not_details
                    .take()
                    .unwrap_or(HotOrNotDetails::default());
                let slot_history = hot_or_not_details
                    .slot_history
                    .entry(ongoing_slot)
                    .or_default();
                let room_details = slot_history.room_details.entry(ongoing_room).or_default();
                let bets_made_currently = &mut room_details.bets_made;

                // * Update slot history details
                if bets_made_currently.len() < 100 {
                    bets_made_currently.insert(
                        api_caller.clone(),
                        BetDetails {
                            amount: bet_amount,
                            bet_direction: bet_direction.clone(),
                        },
                    );
                } else {
                    let new_room_number = ongoing_room + 1;
                    let mut bets_made = BTreeMap::default();
                    bets_made.insert(
                        api_caller.clone(),
                        BetDetails {
                            amount: bet_amount,
                            bet_direction: bet_direction.clone(),
                        },
                    );
                    slot_history
                        .room_details
                        .insert(new_room_number, RoomDetails { bets_made });
                }

                // * Update aggregate stats
                hot_or_not_details.aggregate_stats.total_amount_bet += bet_amount;
                match bet_direction {
                    BetDirection::Hot => {
                        hot_or_not_details.aggregate_stats.total_number_of_hot_bets += 1;
                    }
                    BetDirection::Not => {
                        hot_or_not_details.aggregate_stats.total_number_of_not_bets += 1;
                    }
                }

                self.hot_or_not_details = Some(hot_or_not_details);

                let slot_history = &self.hot_or_not_details.as_ref().unwrap().slot_history;
                let started_at = self.created_at;
                let number_of_participants = slot_history
                    .last_key_value()
                    .unwrap()
                    .1
                    .room_details
                    .last_key_value()
                    .unwrap()
                    .1
                    .bets_made
                    .len() as u8;
                let ongoing_slot = *slot_history.last_key_value().unwrap().0;
                let ongoing_room = *slot_history
                    .last_key_value()
                    .unwrap()
                    .1
                    .room_details
                    .last_key_value()
                    .unwrap()
                    .0;
                Ok(BettingStatus::BettingOpen {
                    started_at,
                    number_of_participants,
                    ongoing_slot,
                    ongoing_room,
                    has_this_user_participated_in_this_post: Some(true),
                })
            }
            BettingStatus::BettingClosed => Err(BetOnCurrentlyViewingPostError::BettingClosed),
        }
    }
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use test_utils::setup::test_constants::get_mock_user_alice_principal_id;

    use crate::canister_specific::individual_user_template::types::post::PostDetailsFromFrontend;

    use super::*;

    #[test]
    fn test_get_hot_or_not_betting_status_for_this_post() {
        let mut post = Post::new(
            0,
            PostDetailsFromFrontend {
                description: "Doggos and puppers".into(),
                hashtags: vec!["doggo".into(), "pupper".into()],
                video_uid: "abcd#1234".into(),
                creator_consent_for_inclusion_in_hot_or_not: true,
            },
            &SystemTime::now(),
        );

        let result = post.get_hot_or_not_betting_status_for_this_post(
            &SystemTime::now()
                .checked_add(Duration::from_secs(
                    TOTAL_DURATION_OF_ALL_SLOTS_IN_SECONDS + 1,
                ))
                .unwrap(),
            &Principal::anonymous(),
        );

        assert_eq!(result, BettingStatus::BettingClosed);

        let current_time = SystemTime::now();

        let result = post
            .get_hot_or_not_betting_status_for_this_post(&current_time, &Principal::anonymous());

        assert_eq!(
            result,
            BettingStatus::BettingOpen {
                started_at: post.created_at,
                number_of_participants: 0,
                ongoing_slot: 1,
                ongoing_room: 1,
                has_this_user_participated_in_this_post: None,
            }
        );

        let result = post.get_hot_or_not_betting_status_for_this_post(
            &current_time
                .checked_add(Duration::from_secs(
                    DURATION_OF_EACH_SLOT_IN_SECONDS * 2 + 1,
                ))
                .unwrap(),
            &Principal::anonymous(),
        );

        assert_eq!(
            result,
            BettingStatus::BettingOpen {
                started_at: post.created_at,
                number_of_participants: 0,
                ongoing_slot: 3,
                ongoing_room: 1,
                has_this_user_participated_in_this_post: None,
            }
        );

        let result = post.place_hot_or_not_bet(
            &get_mock_user_alice_principal_id(),
            100,
            &BetDirection::Hot,
            &current_time
                .checked_add(Duration::from_secs(
                    DURATION_OF_EACH_SLOT_IN_SECONDS * 2 + 1,
                ))
                .unwrap(),
        );

        assert!(result.is_ok());

        let result = post.get_hot_or_not_betting_status_for_this_post(
            &current_time
                .checked_add(Duration::from_secs(
                    DURATION_OF_EACH_SLOT_IN_SECONDS * 2 + 1,
                ))
                .unwrap(),
            &get_mock_user_alice_principal_id(),
        );

        assert_eq!(
            result,
            BettingStatus::BettingOpen {
                started_at: post.created_at,
                number_of_participants: 1,
                ongoing_slot: 3,
                ongoing_room: 1,
                has_this_user_participated_in_this_post: Some(true),
            }
        );

        (100..200).for_each(|num| {
            let result = post.place_hot_or_not_bet(
                &Principal::from_slice(&[num]),
                100,
                &BetDirection::Hot,
                &current_time
                    .checked_add(Duration::from_secs(
                        DURATION_OF_EACH_SLOT_IN_SECONDS * 2 + 1,
                    ))
                    .unwrap(),
            );

            assert!(result.is_ok());
        });

        let result = post.place_hot_or_not_bet(
            &Principal::from_slice(&[200]),
            100,
            &BetDirection::Hot,
            &current_time
                .checked_add(Duration::from_secs(
                    DURATION_OF_EACH_SLOT_IN_SECONDS * 2 + 1,
                ))
                .unwrap(),
        );

        assert!(result.is_ok());

        let result = post.get_hot_or_not_betting_status_for_this_post(
            &current_time
                .checked_add(Duration::from_secs(
                    DURATION_OF_EACH_SLOT_IN_SECONDS * 2 + 1,
                ))
                .unwrap(),
            &Principal::from_slice(&[100]),
        );

        assert_eq!(
            result,
            BettingStatus::BettingOpen {
                started_at: post.created_at,
                number_of_participants: 2,
                ongoing_slot: 3,
                ongoing_room: 2,
                has_this_user_participated_in_this_post: Some(true),
            }
        );

        let result = post.place_hot_or_not_bet(
            &get_mock_user_alice_principal_id(),
            100,
            &BetDirection::Hot,
            &current_time
                .checked_add(Duration::from_secs(
                    DURATION_OF_EACH_SLOT_IN_SECONDS * 2 + 1,
                ))
                .unwrap(),
        );

        assert!(result.is_err());

        let result = post.get_hot_or_not_betting_status_for_this_post(
            &current_time
                .checked_add(Duration::from_secs(
                    DURATION_OF_EACH_SLOT_IN_SECONDS * 2 + 1,
                ))
                .unwrap(),
            &get_mock_user_alice_principal_id(),
        );

        assert_eq!(
            result,
            BettingStatus::BettingOpen {
                started_at: post.created_at,
                number_of_participants: 2,
                ongoing_slot: 3,
                ongoing_room: 2,
                has_this_user_participated_in_this_post: Some(true),
            }
        );

        let result = post.place_hot_or_not_bet(
            &get_mock_user_alice_principal_id(),
            100,
            &BetDirection::Hot,
            &current_time
                .checked_add(Duration::from_secs(
                    DURATION_OF_EACH_SLOT_IN_SECONDS * 4 + 1,
                ))
                .unwrap(),
        );

        assert!(result.is_err());

        let result = post.get_hot_or_not_betting_status_for_this_post(
            &current_time
                .checked_add(Duration::from_secs(
                    DURATION_OF_EACH_SLOT_IN_SECONDS * 4 + 1,
                ))
                .unwrap(),
            &get_mock_user_alice_principal_id(),
        );

        assert_eq!(
            result,
            BettingStatus::BettingOpen {
                started_at: post.created_at,
                number_of_participants: 0,
                ongoing_slot: 5,
                ongoing_room: 1,
                has_this_user_participated_in_this_post: Some(true),
            }
        );
    }

    #[test]
    fn test_has_this_principal_already_bet_on_this_post() {
        let mut post = Post::new(
            0,
            PostDetailsFromFrontend {
                description: "Doggos and puppers".into(),
                hashtags: vec!["doggo".into(), "pupper".into()],
                video_uid: "abcd#1234".into(),
                creator_consent_for_inclusion_in_hot_or_not: true,
            },
            &SystemTime::now(),
        );

        let result =
            post.has_this_principal_already_bet_on_this_post(&get_mock_user_alice_principal_id());

        assert_eq!(result, false);

        post.place_hot_or_not_bet(
            &get_mock_user_alice_principal_id(),
            100,
            &BetDirection::Hot,
            &SystemTime::now(),
        )
        .ok();

        let result =
            post.has_this_principal_already_bet_on_this_post(&get_mock_user_alice_principal_id());

        assert_eq!(result, true);
    }

    #[test]
    fn test_place_hot_or_not_bet() {
        let mut post = Post::new(
            0,
            PostDetailsFromFrontend {
                description: "Doggos and puppers".into(),
                hashtags: vec!["doggo".into(), "pupper".into()],
                video_uid: "abcd#1234".into(),
                creator_consent_for_inclusion_in_hot_or_not: true,
            },
            &SystemTime::now(),
        );

        assert!(post.hot_or_not_details.is_some());

        let result = post.place_hot_or_not_bet(
            &get_mock_user_alice_principal_id(),
            100,
            &BetDirection::Hot,
            &SystemTime::now()
                .checked_add(Duration::from_secs(
                    TOTAL_DURATION_OF_ALL_SLOTS_IN_SECONDS + 1,
                ))
                .unwrap(),
        );

        assert_eq!(result, Err(BetOnCurrentlyViewingPostError::BettingClosed));

        let result = post.place_hot_or_not_bet(
            &get_mock_user_alice_principal_id(),
            100,
            &BetDirection::Hot,
            &SystemTime::now(),
        );

        assert_eq!(
            result,
            Ok(BettingStatus::BettingOpen {
                started_at: post.created_at,
                number_of_participants: 1,
                ongoing_slot: 1,
                ongoing_room: 1,
                has_this_user_participated_in_this_post: Some(true)
            })
        );
        let hot_or_not_details = post.hot_or_not_details.clone().unwrap();
        assert_eq!(hot_or_not_details.slot_history.len(), 1);
        let room_details = &hot_or_not_details
            .slot_history
            .get(&1)
            .unwrap()
            .room_details;
        assert_eq!(room_details.len(), 1);
        let bets_made = &room_details.get(&1).unwrap().bets_made;
        assert_eq!(bets_made.len(), 1);
        assert_eq!(
            bets_made
                .get(&get_mock_user_alice_principal_id())
                .unwrap()
                .amount,
            100
        );
        assert_eq!(
            bets_made
                .get(&get_mock_user_alice_principal_id())
                .unwrap()
                .bet_direction,
            BetDirection::Hot
        );
        assert_eq!(hot_or_not_details.aggregate_stats.total_amount_bet, 100);
        assert_eq!(
            hot_or_not_details.aggregate_stats.total_number_of_hot_bets,
            1
        );
        assert_eq!(
            hot_or_not_details.aggregate_stats.total_number_of_not_bets,
            0
        );

        let result = post.place_hot_or_not_bet(
            &get_mock_user_alice_principal_id(),
            100,
            &BetDirection::Hot,
            &SystemTime::now(),
        );
        assert!(result.is_err());
    }
}
