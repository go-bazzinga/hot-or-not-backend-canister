use std::{borrow::Cow, cmp::Ordering, collections::BTreeMap, time::SystemTime};

use candid::{CandidType, Decode, Deserialize, Encode, Principal};
use ic_cdk::api::management_canister::provisional::CanisterId;
use ic_stable_structures::{
    memory_manager::VirtualMemory, storable::Bound, DefaultMemoryImpl, Storable,
};
use serde::Serialize;

use crate::common::types::{
    app_primitive_type::PostId,
    utility_token::token_event::{
        HotOrNotOutcomePayoutEvent, HotOrNotOutcomePayoutEventV1, NewSlotType, TokenEvent, TokenEventV1, HOT_OR_NOT_BET_CREATOR_COMMISSION_PERCENTAGE, HOT_OR_NOT_BET_WINNINGS_MULTIPLIER
    },
};

use super::{
    error::BetOnCurrentlyViewingPostError,
    post::{FeedScore, Post},
    token::{TokenBalance, TokenBalanceV1},
};

#[deprecated(note = "use `BettingStatusV1` instead")]
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

#[derive(CandidType, PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
pub enum BettingStatusV1 {
    BettingOpen {
        started_at: SystemTime,
        number_of_participants: u8,
        ongoing_slot: NewSlotType,
        ongoing_room: u64,
        has_this_user_participated_in_this_post: Option<bool>,
    },
    BettingClosed,
}

impl From<BettingStatus> for BettingStatusV1 {
    fn from(status: BettingStatus) -> Self {
        match status {
            BettingStatus::BettingOpen {
                started_at,
                number_of_participants,
                ongoing_slot,
                ongoing_room,
                has_this_user_participated_in_this_post,
            } => BettingStatusV1::BettingOpen {
                started_at,
                number_of_participants,
                ongoing_slot: ongoing_slot.into(), // Assuming you have a way to convert u8 to NewSlotType
                ongoing_room,
                has_this_user_participated_in_this_post,
            },
            BettingStatus::BettingClosed => BettingStatusV1::BettingClosed,
        }
    }
}

impl From<BettingStatusV1> for BettingStatus {
    fn from(status: BettingStatusV1) -> Self {
        match status {
            BettingStatusV1::BettingOpen {
                started_at,
                number_of_participants,
                ongoing_slot,
                ongoing_room,
                has_this_user_participated_in_this_post,
            } => BettingStatus::BettingOpen {
                started_at,
                number_of_participants,
                ongoing_slot: ongoing_slot.into(), // Assuming you have a way to convert NewSlotType to u8
                ongoing_room,
                has_this_user_participated_in_this_post,
            },
            BettingStatusV1::BettingClosed => BettingStatus::BettingClosed,
        }
    }
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
    pub hot_or_not_feed_score: FeedScore,
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
    pub bet_outcome: RoomBetPossibleOutcomes,
    pub room_bets_total_pot: u64,
    pub total_hot_bets: u64,
    pub total_not_bets: u64,
}

#[derive(CandidType, Clone, Deserialize, Debug, Serialize)]
pub struct SlotDetailsV1 {
    pub active_room_id: RoomId,
}

impl Default for SlotDetailsV1 {
    fn default() -> Self {
        SlotDetailsV1 { active_room_id: 1 }
    }
}

const MAX_SLOT_DETAILS_VALUE_SIZE: u32 = 100 as u32;

impl Storable for SlotDetailsV1 {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: MAX_SLOT_DETAILS_VALUE_SIZE,
        is_fixed_size: false,
    };
}

#[derive(CandidType, Clone, Deserialize, Default, Debug, Serialize)]
pub struct RoomDetailsV1 {
    pub bet_outcome: RoomBetPossibleOutcomes,
    pub room_bets_total_pot: u64,
    pub total_hot_bets: u64,
    pub total_not_bets: u64,
}
const MAX_ROOM_DETAILS_VALUE_SIZE: u32 = 100 as u32;

impl Storable for RoomDetailsV1 {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: MAX_ROOM_DETAILS_VALUE_SIZE,
        is_fixed_size: false,
    };
}

pub type BetMaker = Principal;

#[derive(CandidType, Clone, Deserialize, Debug, Serialize)]
pub struct BetDetails {
    pub amount: u64,
    pub bet_direction: BetDirection,
    pub payout: BetPayout,
    pub bet_maker_canister_id: CanisterId,
}
const MAX_BET_DETAILS_VALUE_SIZE: u32 = 200 as u32;

impl Storable for BetDetails {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: MAX_BET_DETAILS_VALUE_SIZE,
        is_fixed_size: false,
    };
}

#[derive(
    CandidType, Clone, Deserialize, Debug, Serialize, Ord, PartialOrd, Eq, PartialEq, Hash,
)]
pub struct StablePrincipal(pub Principal);

impl Default for StablePrincipal {
    fn default() -> Self {
        Self(Principal::anonymous())
    }
}

impl Storable for StablePrincipal {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 60,
        is_fixed_size: false,
    };
}

pub type BetMakerPrincipal = StablePrincipal;

#[deprecated(note = "use GlobalBetIdV1 instead")]
#[derive(
    CandidType,
    Clone,
    Deserialize,
    Debug,
    Serialize,
    Ord,
    PartialOrd,
    Eq,
    PartialEq,
    Default,
    Copy,
    Hash,
)]
pub struct GlobalRoomId(pub PostId, pub SlotId, pub RoomId);

impl Storable for GlobalRoomId {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 50,
        is_fixed_size: false,
    };
}

#[derive(
    CandidType,
    Clone,
    Deserialize,
    Debug,
    Serialize,
    Ord,
    PartialOrd,
    Eq,
    PartialEq,
    Default,
    Copy,
    Hash,
)]
pub struct GlobalRoomIdV1(pub PostId, pub NewSlotType, pub RoomId);

impl Storable for GlobalRoomIdV1 {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        // let mut bytes = vec![];
        // ciborium::ser::into_writer(self, &mut bytes).unwrap();
        
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 50,
        is_fixed_size: false,
    };
}

impl From<GlobalRoomId> for GlobalRoomIdV1 {
    fn from(v1: GlobalRoomId) -> Self {
        GlobalRoomIdV1(v1.0, v1.1.into(), v1.2)
    }
}

#[deprecated(note = "use GlobalBetIdV1 instead")]
#[derive(
    CandidType, Clone, Deserialize, Debug, Serialize, Ord, PartialOrd, Eq, PartialEq, Default,
)]
pub struct GlobalBetId(pub GlobalRoomId, pub BetMakerPrincipal);

impl Storable for GlobalBetId {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 100,
        is_fixed_size: false,
    };
}

#[derive(
    CandidType, Clone, Deserialize, Debug, Serialize, Ord, PartialOrd, Eq, PartialEq, Default,
)]
pub struct GlobalBetIdV1(pub GlobalRoomIdV1, pub BetMakerPrincipal);

impl Storable for GlobalBetIdV1 {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 100,
        is_fixed_size: false,
    };
}

impl From<GlobalBetId> for GlobalBetIdV1 {
    fn from(global_bet_id: GlobalBetId) -> Self {
        GlobalBetIdV1(
            global_bet_id.0.into(), // Assuming you have a way to convert GlobalRoomId to GlobalRoomIdV1
            global_bet_id.1,
        )
    }
}

#[derive(Clone, Deserialize, Debug, CandidType, Serialize, Default)]
pub enum BetPayout {
    #[default]
    NotCalculatedYet,
    Calculated(u64),
}

#[derive(CandidType, Clone, Default, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum RoomBetPossibleOutcomes {
    #[default]
    BetOngoing,
    HotWon,
    NotWon,
    Draw,
}

#[deprecated(note = "use `PlacedBetDetailV1` instead")]
#[derive(Deserialize, Serialize, Clone, CandidType, Debug, PartialEq, Eq)]
pub struct PlacedBetDetail {
    pub canister_id: CanisterId,
    pub post_id: PostId,
    pub slot_id: SlotId,
    pub room_id: RoomId,
    pub amount_bet: u64,
    pub bet_direction: BetDirection,
    pub bet_placed_at: SystemTime,
    pub outcome_received: BetOutcomeForBetMaker,
}

#[derive(Deserialize, Serialize, Clone, CandidType, Debug, PartialEq, Eq)]
pub struct PlacedBetDetailV1 {
    pub canister_id: CanisterId,
    pub post_id: PostId,
    pub slot_id: NewSlotType,
    pub room_id: RoomId,
    pub amount_bet: u64,
    pub bet_direction: BetDirection,
    pub bet_placed_at: SystemTime,
    pub outcome_received: BetOutcomeForBetMaker,
}

impl From<PlacedBetDetail> for PlacedBetDetailV1 {
    fn from(detail: PlacedBetDetail) -> Self {
        PlacedBetDetailV1 {
            canister_id: detail.canister_id,
            post_id: detail.post_id,
            slot_id: detail.slot_id.into(),
            room_id: detail.room_id,
            amount_bet: detail.amount_bet,
            bet_direction: detail.bet_direction,
            bet_placed_at: detail.bet_placed_at,
            outcome_received: detail.outcome_received,
        }
    }
}

#[derive(Deserialize, Serialize, Default, CandidType, PartialEq, Eq, Clone, Debug)]
pub enum BetOutcomeForBetMaker {
    #[default]
    AwaitingResult,
    Won(u64),
    Lost,
    Draw(u64),
}

impl Post {
    // pub fn get_hot_or_not_betting_status_for_this_post(
    //     &self,
    //     current_time_when_request_being_made: &SystemTime,
    //     bet_maker_principal_id: &Principal,
    // ) -> BettingStatus {
    //     let betting_status =
    //         match current_time_when_request_being_made
    //             .duration_since(self.created_at)
    //             .unwrap()
    //             .as_secs()
    //         {
    //             // * contest is still ongoing
    //             0..=TOTAL_DURATION_OF_ALL_SLOTS_IN_SECONDS => {
    //                 let started_at = self.created_at;
    //                 let numerator = current_time_when_request_being_made
    //                     .duration_since(started_at)
    //                     .unwrap()
    //                     .as_secs();

    //                 let denominator = DURATION_OF_EACH_SLOT_IN_SECONDS;
    //                 let currently_ongoing_slot = ((numerator / denominator) + 1) as u8;

    //                 let temp_hot_or_not_default = &HotOrNotDetails::default();
    //                 let temp_slot_details_default = &SlotDetails::default();
    //                 let room_details = &self
    //                     .hot_or_not_details
    //                     .as_ref()
    //                     .unwrap_or(temp_hot_or_not_default)
    //                     .slot_history
    //                     .get(&currently_ongoing_slot)
    //                     .unwrap_or(temp_slot_details_default)
    //                     .room_details;

    //                 let temp_room_details_default = &RoomDetails::default();
    //                 let currently_active_room = room_details
    //                     .last_key_value()
    //                     .unwrap_or((&1, temp_room_details_default));
    //                 let number_of_participants = currently_active_room.1.bets_made.len() as u8;
    //                 BettingStatus::BettingOpen {
    //                     started_at,
    //                     number_of_participants,
    //                     ongoing_slot: currently_ongoing_slot,
    //                     ongoing_room: *currently_active_room.0,
    //                     has_this_user_participated_in_this_post: if *bet_maker_principal_id
    //                         == Principal::anonymous()
    //                     {
    //                         None
    //                     } else {
    //                         Some(self.has_this_principal_already_bet_on_this_post(
    //                             bet_maker_principal_id,
    //                         ))
    //                     },
    //                 }
    //             }
    //             // * contest is over
    //             _ => BettingStatus::BettingClosed,
    //         };

    //     betting_status
    // }

    #[deprecated(note = "use get_hot_or_not_betting_status_for_this_post_v2 instead")]
    pub fn get_hot_or_not_betting_status_for_this_post_v1(
        &self,
        current_time_when_request_being_made: &SystemTime,
        bet_maker_principal_id: &Principal,
        room_details_map: &ic_stable_structures::btreemap::BTreeMap<
            GlobalRoomId,
            RoomDetailsV1,
            VirtualMemory<DefaultMemoryImpl>,
        >,
        post_principal_map: &ic_stable_structures::btreemap::BTreeMap<
            (PostId, StablePrincipal),
            (),
            VirtualMemory<DefaultMemoryImpl>,
        >,
        slot_details_map: &ic_stable_structures::btreemap::BTreeMap<
            (PostId, SlotId),
            SlotDetailsV1,
            VirtualMemory<DefaultMemoryImpl>,
        >,
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

                let temp_room_details_default = RoomDetailsV1::default();

                // get currently active room
                let active_room_id = slot_details_map
                    .get(&(self.id, currently_ongoing_slot))
                    .unwrap_or(SlotDetailsV1::default())
                    .active_room_id;
                let global_room_id = GlobalRoomId(self.id, currently_ongoing_slot, active_room_id);

                let room_details = room_details_map
                    .get(&global_room_id)
                    .unwrap_or(temp_room_details_default);

                let number_of_participants =
                    (room_details.total_hot_bets + room_details.total_not_bets) as u8;

                BettingStatus::BettingOpen {
                    started_at,
                    number_of_participants,
                    ongoing_slot: currently_ongoing_slot,
                    ongoing_room: active_room_id,
                    has_this_user_participated_in_this_post: if *bet_maker_principal_id
                        == Principal::anonymous()
                    {
                        None
                    } else {
                        Some(self.has_this_principal_already_bet_on_this_post_v1(
                            bet_maker_principal_id,
                            post_principal_map,
                        ))
                    },
                }
            }
            // * contest is over
            _ => BettingStatus::BettingClosed,
        };

        betting_status
    }

    pub fn get_hot_or_not_betting_status_for_this_post_v2(
        &self,
        current_time_when_request_being_made: &SystemTime,
        bet_maker_principal_id: &Principal,
        room_details_map: &ic_stable_structures::btreemap::BTreeMap<
            GlobalRoomIdV1,
            RoomDetailsV1,
            VirtualMemory<DefaultMemoryImpl>,
        >,
        post_principal_map: &ic_stable_structures::btreemap::BTreeMap<
            (PostId, StablePrincipal),
            (),
            VirtualMemory<DefaultMemoryImpl>,
        >,
        slot_details_map: &ic_stable_structures::btreemap::BTreeMap<
            (PostId, NewSlotType),
            SlotDetailsV1,
            VirtualMemory<DefaultMemoryImpl>,
        >,
    ) -> BettingStatusV1 {
        // betting status is always Open
        let started_at = self.created_at;
        let numerator = current_time_when_request_being_made
            .duration_since(started_at)
            .unwrap()
            .as_secs();

        let denominator = DURATION_OF_EACH_SLOT_IN_SECONDS;
        // Since slots are tied to rooms, it is ok to keep a slot number.
        // Since there are infinite slots, slot number's absolute value is irrelevant.
        let currently_ongoing_slot = NewSlotType(((numerator / denominator) + 1) as u64);

        let temp_room_details_default = RoomDetailsV1::default();

        // get currently active room
        let active_room_id = slot_details_map
            .get(&(self.id, currently_ongoing_slot))
            .unwrap_or(SlotDetailsV1::default())
            .active_room_id;

        let global_room_id = GlobalRoomIdV1(self.id, currently_ongoing_slot, active_room_id);

        let room_details = room_details_map
            .get(&global_room_id)
            .unwrap_or(temp_room_details_default);

        let number_of_participants =
            (room_details.total_hot_bets + room_details.total_not_bets) as u8;

        BettingStatusV1::BettingOpen {
            started_at,
            number_of_participants,
            ongoing_slot: currently_ongoing_slot,
            ongoing_room: active_room_id,
            has_this_user_participated_in_this_post: if *bet_maker_principal_id
                == Principal::anonymous()
            {
                None
            } else {
                Some(self.has_this_principal_already_bet_on_this_post_v1(
                    bet_maker_principal_id,
                    post_principal_map,
                ))
            },
        }
    }

    // pub fn has_this_principal_already_bet_on_this_post(
    //     &self,
    //     principal_making_bet: &Principal,
    // ) -> bool {
    //     self.hot_or_not_details
    //         .as_ref()
    //         .unwrap()
    //         .slot_history
    //         .values()
    //         .flat_map(|slot_details| slot_details.room_details.iter())
    //         .flat_map(|(_, room_details)| room_details.bets_made.iter())
    //         .any(|(principal, _)| principal == principal_making_bet)
    // }

    pub fn has_this_principal_already_bet_on_this_post_v1(
        &self,
        principal_making_bet: &Principal,
        post_principal_map: &ic_stable_structures::btreemap::BTreeMap<
            (PostId, StablePrincipal),
            (),
            VirtualMemory<DefaultMemoryImpl>,
        >,
    ) -> bool {
        post_principal_map.contains_key(&(self.id, StablePrincipal(*principal_making_bet)))
    }

    // pub fn place_hot_or_not_bet(
    //     &mut self,
    //     bet_maker_principal_id: &Principal,
    //     bet_maker_canister_id: &CanisterId,
    //     bet_amount: u64,
    //     bet_direction: &BetDirection,
    //     current_time_when_request_being_made: &SystemTime,
    // ) -> Result<BettingStatus, BetOnCurrentlyViewingPostError> {
    //     if *bet_maker_principal_id == Principal::anonymous() {
    //         return Err(BetOnCurrentlyViewingPostError::UserNotLoggedIn);
    //     }

    //     let betting_status = self.get_hot_or_not_betting_status_for_this_post(
    //         current_time_when_request_being_made,
    //         bet_maker_principal_id,
    //     );

    //     match betting_status {
    //         BettingStatus::BettingClosed => Err(BetOnCurrentlyViewingPostError::BettingClosed),
    //         BettingStatus::BettingOpen {
    //             ongoing_slot,
    //             ongoing_room,
    //             has_this_user_participated_in_this_post,
    //             ..
    //         } => {
    //             if has_this_user_participated_in_this_post.unwrap() {
    //                 return Err(BetOnCurrentlyViewingPostError::UserAlreadyParticipatedInThisPost);
    //             }

    //             let mut hot_or_not_details = self
    //                 .hot_or_not_details
    //                 .take()
    //                 .unwrap_or(HotOrNotDetails::default());
    //             let slot_history = hot_or_not_details
    //                 .slot_history
    //                 .entry(ongoing_slot)
    //                 .or_default();
    //             let room_detail = slot_history.room_details.entry(ongoing_room).or_default();
    //             let bets_made_currently = &mut room_detail.bets_made;

    //             // * Update bets_made currently
    //             if bets_made_currently.len() < 100 {
    //                 bets_made_currently.insert(
    //                     *bet_maker_principal_id,
    //                     BetDetails {
    //                         amount: bet_amount,
    //                         bet_direction: bet_direction.clone(),
    //                         payout: BetPayout::default(),
    //                         bet_maker_canister_id: *bet_maker_canister_id,
    //                     },
    //                 );
    //                 room_detail.room_bets_total_pot += bet_amount;
    //             } else {
    //                 let new_room_number = ongoing_room + 1;
    //                 let mut bets_made = BTreeMap::default();
    //                 bets_made.insert(
    //                     *bet_maker_principal_id,
    //                     BetDetails {
    //                         amount: bet_amount,
    //                         bet_direction: bet_direction.clone(),
    //                         payout: BetPayout::default(),
    //                         bet_maker_canister_id: *bet_maker_canister_id,
    //                     },
    //                 );
    //                 slot_history.room_details.insert(
    //                     new_room_number,
    //                     RoomDetails {
    //                         bets_made,
    //                         room_bets_total_pot: bet_amount,
    //                         ..Default::default()
    //                     },
    //                 );
    //             }

    //             // * Update aggregate stats
    //             hot_or_not_details.aggregate_stats.total_amount_bet += bet_amount;
    //             let mut last_room_entry = slot_history.room_details.last_entry().unwrap();
    //             match bet_direction {
    //                 BetDirection::Hot => {
    //                     hot_or_not_details.aggregate_stats.total_number_of_hot_bets += 1;
    //                     last_room_entry.get_mut().total_hot_bets += 1;
    //                 }
    //                 BetDirection::Not => {
    //                     hot_or_not_details.aggregate_stats.total_number_of_not_bets += 1;
    //                     last_room_entry.get_mut().total_not_bets += 1;
    //                 }
    //             }

    //             self.hot_or_not_details = Some(hot_or_not_details);

    //             let slot_history = &self.hot_or_not_details.as_ref().unwrap().slot_history;
    //             let started_at = self.created_at;
    //             let number_of_participants = slot_history
    //                 .last_key_value()
    //                 .unwrap()
    //                 .1
    //                 .room_details
    //                 .last_key_value()
    //                 .unwrap()
    //                 .1
    //                 .bets_made
    //                 .len() as u8;
    //             let ongoing_slot = *slot_history.last_key_value().unwrap().0;
    //             let ongoing_room = *slot_history
    //                 .last_key_value()
    //                 .unwrap()
    //                 .1
    //                 .room_details
    //                 .last_key_value()
    //                 .unwrap()
    //                 .0;
    //             Ok(BettingStatus::BettingOpen {
    //                 started_at,
    //                 number_of_participants,
    //                 ongoing_slot,
    //                 ongoing_room,
    //                 has_this_user_participated_in_this_post: Some(true),
    //             })
    //         }
    //     }
    // }

    #[deprecated(note = "use place_hot_or_not_bet_v2 instead")]
    pub fn place_hot_or_not_bet_v1(
        &mut self,
        bet_maker_principal_id: &Principal,
        bet_maker_canister_id: &CanisterId,
        bet_amount: u64,
        bet_direction: &BetDirection,
        current_time_when_request_being_made: &SystemTime,
        room_details_map: &mut ic_stable_structures::btreemap::BTreeMap<
            GlobalRoomId,
            RoomDetailsV1,
            VirtualMemory<DefaultMemoryImpl>,
        >,
        bet_details_map: &mut ic_stable_structures::btreemap::BTreeMap<
            GlobalBetId,
            BetDetails,
            VirtualMemory<DefaultMemoryImpl>,
        >,
        post_principal_map: &mut ic_stable_structures::btreemap::BTreeMap<
            (PostId, StablePrincipal),
            (),
            VirtualMemory<DefaultMemoryImpl>,
        >,
        slot_details_map: &mut ic_stable_structures::btreemap::BTreeMap<
            (PostId, SlotId),
            SlotDetailsV1,
            VirtualMemory<DefaultMemoryImpl>,
        >,
    ) -> Result<BettingStatus, BetOnCurrentlyViewingPostError> {
        if *bet_maker_principal_id == Principal::anonymous() {
            return Err(BetOnCurrentlyViewingPostError::UserNotLoggedIn);
        }

        let betting_status = self.get_hot_or_not_betting_status_for_this_post_v1(
            current_time_when_request_being_made,
            bet_maker_principal_id,
            room_details_map,
            post_principal_map,
            slot_details_map,
        );

        match betting_status {
            BettingStatus::BettingClosed => Err(BetOnCurrentlyViewingPostError::BettingClosed),
            BettingStatus::BettingOpen {
                ongoing_slot,
                ongoing_room,
                has_this_user_participated_in_this_post,
                ..
            } => {
                if has_this_user_participated_in_this_post.unwrap() {
                    return Err(BetOnCurrentlyViewingPostError::UserAlreadyParticipatedInThisPost);
                }

                let mut hot_or_not_details = self
                    .hot_or_not_details
                    .take()
                    .unwrap_or(HotOrNotDetails::default());
                let mut global_room_id = GlobalRoomId(self.id, ongoing_slot, ongoing_room);
                let mut global_bet_id =
                    GlobalBetId(global_room_id, StablePrincipal(*bet_maker_principal_id));

                let mut room_detail = room_details_map.get(&global_room_id).unwrap_or_default();
                let num_bets_made = room_detail.total_hot_bets + room_detail.total_not_bets;

                if num_bets_made < 100 {
                    room_detail.room_bets_total_pot += bet_amount;
                } else {
                    let new_room_number = ongoing_room + 1;
                    global_room_id = GlobalRoomId(self.id, ongoing_slot, new_room_number);
                    global_bet_id =
                        GlobalBetId(global_room_id, StablePrincipal(*bet_maker_principal_id));
                    room_detail = RoomDetailsV1 {
                        room_bets_total_pot: bet_amount,
                        ..Default::default()
                    };
                }

                bet_details_map.insert(
                    global_bet_id,
                    BetDetails {
                        amount: bet_amount,
                        bet_direction: bet_direction.clone(),
                        payout: BetPayout::default(),
                        bet_maker_canister_id: *bet_maker_canister_id,
                    },
                );

                // * Update aggregate stats
                hot_or_not_details.aggregate_stats.total_amount_bet += bet_amount;
                match bet_direction {
                    BetDirection::Hot => {
                        hot_or_not_details.aggregate_stats.total_number_of_hot_bets += 1;
                        room_detail.total_hot_bets += 1;
                    }
                    BetDirection::Not => {
                        hot_or_not_details.aggregate_stats.total_number_of_not_bets += 1;
                        room_detail.total_not_bets += 1;
                    }
                }

                room_details_map.insert(global_room_id, room_detail);
                if global_room_id.2 != ongoing_room {
                    slot_details_map.insert(
                        (self.id, ongoing_slot),
                        SlotDetailsV1 {
                            active_room_id: global_room_id.2,
                        },
                    );
                }

                self.hot_or_not_details = Some(hot_or_not_details);

                let started_at = self.created_at;
                let number_of_participants = (num_bets_made + 1) as u8;
                let ongoing_slot = global_room_id.1;
                let ongoing_room = global_room_id.2;

                post_principal_map.insert((self.id, StablePrincipal(*bet_maker_principal_id)), ());

                Ok(BettingStatus::BettingOpen {
                    started_at,
                    number_of_participants,
                    ongoing_slot,
                    ongoing_room,
                    has_this_user_participated_in_this_post: Some(true),
                })
            }
        }
    }

    pub fn place_hot_or_not_bet_v2(
        &mut self,
        bet_maker_principal_id: &Principal,
        bet_maker_canister_id: &CanisterId,
        bet_amount: u64,
        bet_direction: &BetDirection,
        current_time_when_request_being_made: &SystemTime,
        room_details_map: &mut ic_stable_structures::btreemap::BTreeMap<
            GlobalRoomIdV1,
            RoomDetailsV1,
            VirtualMemory<DefaultMemoryImpl>,
        >,
        bet_details_map: &mut ic_stable_structures::btreemap::BTreeMap<
            GlobalBetIdV1,
            BetDetails,
            VirtualMemory<DefaultMemoryImpl>,
        >,
        post_principal_map: &mut ic_stable_structures::btreemap::BTreeMap<
            (PostId, StablePrincipal),
            (),
            VirtualMemory<DefaultMemoryImpl>,
        >,
        slot_details_map: &mut ic_stable_structures::btreemap::BTreeMap<
            (PostId, NewSlotType),
            SlotDetailsV1,
            VirtualMemory<DefaultMemoryImpl>,
        >,
    ) -> Result<BettingStatusV1, BetOnCurrentlyViewingPostError> {
        if *bet_maker_principal_id == Principal::anonymous() {
            return Err(BetOnCurrentlyViewingPostError::UserNotLoggedIn);
        }

        let betting_status = self.get_hot_or_not_betting_status_for_this_post_v2(
            current_time_when_request_being_made,
            bet_maker_principal_id,
            room_details_map,
            post_principal_map,
            slot_details_map,
        );

        match betting_status {
            BettingStatusV1::BettingClosed => Err(BetOnCurrentlyViewingPostError::BettingClosed),
            BettingStatusV1::BettingOpen {
                ongoing_slot,
                ongoing_room,
                has_this_user_participated_in_this_post,
                ..
            } => {
                if has_this_user_participated_in_this_post.unwrap() {
                    return Err(BetOnCurrentlyViewingPostError::UserAlreadyParticipatedInThisPost);
                }

                let mut hot_or_not_details = self
                    .hot_or_not_details
                    .take()
                    .unwrap_or(HotOrNotDetails::default());
                let mut global_room_id = GlobalRoomIdV1(self.id, ongoing_slot, ongoing_room);
                let mut global_bet_id =
                    GlobalBetIdV1(global_room_id, StablePrincipal(*bet_maker_principal_id));

                let mut room_detail = room_details_map.get(&global_room_id).unwrap_or_default();
                let num_bets_made = room_detail.total_hot_bets + room_detail.total_not_bets;

                if num_bets_made < 100 {
                    room_detail.room_bets_total_pot += bet_amount;
                } else {
                    let new_room_number = ongoing_room + 1;
                    global_room_id = GlobalRoomIdV1(self.id, ongoing_slot, new_room_number);
                    global_bet_id =
                        GlobalBetIdV1(global_room_id, StablePrincipal(*bet_maker_principal_id));
                    room_detail = RoomDetailsV1 {
                        room_bets_total_pot: bet_amount,
                        ..Default::default()
                    };
                }

                bet_details_map.insert(
                    global_bet_id,
                    BetDetails {
                        amount: bet_amount,
                        bet_direction: bet_direction.clone(),
                        payout: BetPayout::default(),
                        bet_maker_canister_id: *bet_maker_canister_id,
                    },
                );

                // * Update aggregate stats
                hot_or_not_details.aggregate_stats.total_amount_bet += bet_amount;
                match bet_direction {
                    BetDirection::Hot => {
                        hot_or_not_details.aggregate_stats.total_number_of_hot_bets += 1;
                        room_detail.total_hot_bets += 1;
                    }
                    BetDirection::Not => {
                        hot_or_not_details.aggregate_stats.total_number_of_not_bets += 1;
                        room_detail.total_not_bets += 1;
                    }
                }

                room_details_map.insert(global_room_id, room_detail);
                if global_room_id.2 != ongoing_room {
                    slot_details_map.insert(
                        (self.id, ongoing_slot),
                        SlotDetailsV1 {
                            active_room_id: global_room_id.2,
                        },
                    );
                }

                self.hot_or_not_details = Some(hot_or_not_details);

                let started_at = self.created_at;
                let number_of_participants = (num_bets_made + 1) as u8;
                let ongoing_slot = global_room_id.1;
                let ongoing_room = global_room_id.2;

                post_principal_map.insert((self.id, StablePrincipal(*bet_maker_principal_id)), ());

                Ok(BettingStatusV1::BettingOpen {
                    started_at,
                    number_of_participants,
                    ongoing_slot,
                    ongoing_room,
                    has_this_user_participated_in_this_post: Some(true),
                })
            }
        }
    }

    // pub fn tabulate_hot_or_not_outcome_for_slot(
    //     &mut self,
    //     post_canister_id: &CanisterId,
    //     slot_id: &u8,
    //     token_balance: &mut TokenBalance,
    //     current_time: &SystemTime,
    // ) {
    //     let hot_or_not_details = self.hot_or_not_details.as_mut();

    //     if hot_or_not_details.is_none() {
    //         return;
    //     }

    //     let slot_history = hot_or_not_details.unwrap().slot_history.get_mut(slot_id);

    //     if slot_history.is_none() {
    //         return;
    //     }

    //     slot_history
    //         .unwrap()
    //         .room_details
    //         .iter_mut()
    //         .for_each(|(room_id, room_detail)| {
    //             if room_detail.bet_outcome == RoomBetPossibleOutcomes::BetOngoing {
    //                 // * Figure out which side won
    //                 match room_detail.total_hot_bets.cmp(&room_detail.total_not_bets) {
    //                     Ordering::Greater => {
    //                         room_detail.bet_outcome = RoomBetPossibleOutcomes::HotWon;
    //                     }
    //                     Ordering::Less => {
    //                         room_detail.bet_outcome = RoomBetPossibleOutcomes::NotWon;
    //                     }
    //                     Ordering::Equal => room_detail.bet_outcome = RoomBetPossibleOutcomes::Draw,
    //                 }

    //                 // * Reward creator with commission. Commission is 10% of total pot
    //                 token_balance.handle_token_event(TokenEvent::HotOrNotOutcomePayout {
    //                     amount: room_detail.room_bets_total_pot
    //                         * HOT_OR_NOT_BET_CREATOR_COMMISSION_PERCENTAGE
    //                         / 100,
    //                     details: HotOrNotOutcomePayoutEvent::CommissionFromHotOrNotBet {
    //                         post_canister_id: *post_canister_id,
    //                         post_id: self.id,
    //                         slot_id: *slot_id,
    //                         room_id: *room_id,
    //                         room_pot_total_amount: room_detail.room_bets_total_pot,
    //                     },
    //                     timestamp: *current_time,
    //                 });

    //                 // * Reward individual participants
    //                 room_detail
    //                     .bets_made
    //                     .iter_mut()
    //                     .for_each(|(_user_id, bet_details)| {
    //                         match &room_detail.bet_outcome {
    //                             RoomBetPossibleOutcomes::HotWon => {
    //                                 if bet_details.bet_direction == BetDirection::Hot {
    //                                     bet_details.payout = BetPayout::Calculated(
    //                                         bet_details.amount
    //                                             * HOT_OR_NOT_BET_WINNINGS_MULTIPLIER
    //                                             * (100
    //                                                 - HOT_OR_NOT_BET_CREATOR_COMMISSION_PERCENTAGE)
    //                                             / 100,
    //                                     );
    //                                 } else {
    //                                     bet_details.payout = BetPayout::Calculated(0);
    //                                 }
    //                             }
    //                             RoomBetPossibleOutcomes::NotWon => {
    //                                 if bet_details.bet_direction == BetDirection::Not {
    //                                     bet_details.payout = BetPayout::Calculated(
    //                                         bet_details.amount
    //                                             * HOT_OR_NOT_BET_WINNINGS_MULTIPLIER
    //                                             * (100
    //                                                 - HOT_OR_NOT_BET_CREATOR_COMMISSION_PERCENTAGE)
    //                                             / 100,
    //                                     );
    //                                 } else {
    //                                     bet_details.payout = BetPayout::Calculated(0);
    //                                 }
    //                             }
    //                             RoomBetPossibleOutcomes::Draw => {
    //                                 bet_details.payout = BetPayout::Calculated(
    //                                     bet_details.amount
    //                                         * (100 - HOT_OR_NOT_BET_CREATOR_COMMISSION_PERCENTAGE)
    //                                         / 100,
    //                                 );
    //                             }
    //                             RoomBetPossibleOutcomes::BetOngoing => {}
    //                         };
    //                     });
    //             }
    //         })
    // }

    #[deprecated(note = "use tabulate_hot_or_not_outcome_for_slot_v2 instead")]
    pub fn tabulate_hot_or_not_outcome_for_slot_v1(
        &mut self,
        post_canister_id: &CanisterId,
        slot_id: &u8,
        token_balance: &mut TokenBalance,
        current_time: &SystemTime,
        room_details_map: &mut ic_stable_structures::btreemap::BTreeMap<
            GlobalRoomId,
            RoomDetailsV1,
            VirtualMemory<DefaultMemoryImpl>,
        >,
        bet_details_map: &mut ic_stable_structures::btreemap::BTreeMap<
            GlobalBetId,
            BetDetails,
            VirtualMemory<DefaultMemoryImpl>,
        >,
    ) {
        let hot_or_not_details = self.hot_or_not_details.as_mut();

        if hot_or_not_details.is_none() {
            return;
        }

        let start_global_room_id = GlobalRoomId(self.id, *slot_id, 1);
        let end_global_room_id = GlobalRoomId(self.id, *slot_id + 1, 1);

        let room_details = room_details_map
            .range(start_global_room_id..end_global_room_id)
            .collect::<Vec<_>>();
        room_details.iter().for_each(|(groomid, room_detail)| {
            let mut room_detail = room_detail.clone();
            let room_id = groomid.2;

            if room_detail.bet_outcome == RoomBetPossibleOutcomes::BetOngoing {
                // * Figure out which side won
                match room_detail.total_hot_bets.cmp(&room_detail.total_not_bets) {
                    Ordering::Greater => {
                        room_detail.bet_outcome = RoomBetPossibleOutcomes::HotWon;
                    }
                    Ordering::Less => {
                        room_detail.bet_outcome = RoomBetPossibleOutcomes::NotWon;
                    }
                    Ordering::Equal => room_detail.bet_outcome = RoomBetPossibleOutcomes::Draw,
                }

                // * Reward creator with commission. Commission is 10% of total pot
                token_balance.handle_token_event(TokenEvent::HotOrNotOutcomePayout {
                    amount: room_detail.room_bets_total_pot
                        * HOT_OR_NOT_BET_CREATOR_COMMISSION_PERCENTAGE
                        / 100,
                    details: HotOrNotOutcomePayoutEvent::CommissionFromHotOrNotBet {
                        post_canister_id: *post_canister_id,
                        post_id: self.id,
                        slot_id: *slot_id,
                        room_id,
                        room_pot_total_amount: room_detail.room_bets_total_pot,
                    },
                    timestamp: *current_time,
                });

                room_details_map.insert(*groomid, room_detail.clone());
            }

            // * Reward individual participants
            let start_global_bet_id = GlobalBetId(start_global_room_id, StablePrincipal::default());
            let end_global_bet_id = GlobalBetId(end_global_room_id, StablePrincipal::default());
            let bet_details = bet_details_map
                .range(start_global_bet_id..end_global_bet_id)
                .collect::<Vec<_>>();
            bet_details.iter().for_each(|(gbetid, bet_detail)| {
                let mut bet_detail = bet_detail.clone();
                match &room_detail.bet_outcome {
                    RoomBetPossibleOutcomes::HotWon => {
                        if bet_detail.bet_direction == BetDirection::Hot {
                            bet_detail.payout = BetPayout::Calculated(
                                bet_detail.amount
                                    * HOT_OR_NOT_BET_WINNINGS_MULTIPLIER
                                    * (100 - HOT_OR_NOT_BET_CREATOR_COMMISSION_PERCENTAGE)
                                    / 100,
                            );
                        } else {
                            bet_detail.payout = BetPayout::Calculated(0);
                        }
                    }
                    RoomBetPossibleOutcomes::NotWon => {
                        if bet_detail.bet_direction == BetDirection::Not {
                            bet_detail.payout = BetPayout::Calculated(
                                bet_detail.amount
                                    * HOT_OR_NOT_BET_WINNINGS_MULTIPLIER
                                    * (100 - HOT_OR_NOT_BET_CREATOR_COMMISSION_PERCENTAGE)
                                    / 100,
                            );
                        } else {
                            bet_detail.payout = BetPayout::Calculated(0);
                        }
                    }
                    RoomBetPossibleOutcomes::Draw => {
                        bet_detail.payout = BetPayout::Calculated(
                            bet_detail.amount
                                * (100 - HOT_OR_NOT_BET_CREATOR_COMMISSION_PERCENTAGE)
                                / 100,
                        );
                    }
                    RoomBetPossibleOutcomes::BetOngoing => {}
                };

                bet_details_map.insert(gbetid.clone(), bet_detail);
            });
        });
    }


pub fn tabulate_hot_or_not_outcome_for_slot_v2(
    &mut self,
    post_canister_id: &CanisterId,
    slot_id_type: &NewSlotType,
    token_balance: &mut TokenBalanceV1,
    current_time: &SystemTime,
    room_details_map: &mut ic_stable_structures::btreemap::BTreeMap<
        GlobalRoomIdV1,
        RoomDetailsV1,
        VirtualMemory<DefaultMemoryImpl>,
    >,
    bet_details_map: &mut ic_stable_structures::btreemap::BTreeMap<
        GlobalBetIdV1,
        BetDetails,
        VirtualMemory<DefaultMemoryImpl>,
    >,
) {
    let slot_id_owned = slot_id_type.clone();

    let hot_or_not_details = self.hot_or_not_details.as_mut();

    // if hot_or_not_details.is_none() {
    //     return;
    // }

    let start_global_room_id = GlobalRoomIdV1(self.id, slot_id_owned, 1);
    let end_global_room_id = GlobalRoomIdV1(self.id, slot_id_owned.increment_by(1), 1);

    let room_details = room_details_map
        .range(start_global_room_id..end_global_room_id)
        .collect::<Vec<_>>();
    room_details.iter().for_each(|(groomid, room_detail)| {
        let mut room_detail = room_detail.clone();
        let room_id = groomid.2;

        if room_detail.bet_outcome == RoomBetPossibleOutcomes::BetOngoing {
            // * Figure out which side won
            match room_detail.total_hot_bets.cmp(&room_detail.total_not_bets) {
                Ordering::Greater => {
                    room_detail.bet_outcome = RoomBetPossibleOutcomes::HotWon;
                }
                Ordering::Less => {
                    room_detail.bet_outcome = RoomBetPossibleOutcomes::NotWon;
                }
                Ordering::Equal => room_detail.bet_outcome = RoomBetPossibleOutcomes::Draw,
            }

            // * Reward creator with commission. Commission is 10% of total pot
            token_balance.handle_token_event(TokenEventV1::HotOrNotOutcomePayout {
                amount: room_detail.room_bets_total_pot
                    * HOT_OR_NOT_BET_CREATOR_COMMISSION_PERCENTAGE
                    / 100,
                details: HotOrNotOutcomePayoutEventV1::CommissionFromHotOrNotBet {
                    post_canister_id: *post_canister_id,
                    post_id: self.id,
                    slot_id: slot_id_owned,
                    room_id,
                    room_pot_total_amount: room_detail.room_bets_total_pot,
                },
                timestamp: *current_time,
            });

            room_details_map.insert(*groomid, room_detail.clone());
        }

        // * Reward individual participants
        let start_global_bet_id = GlobalBetIdV1(start_global_room_id, StablePrincipal::default());
        let end_global_bet_id = GlobalBetIdV1(end_global_room_id, StablePrincipal::default());
        let bet_details = bet_details_map
            .range(start_global_bet_id..end_global_bet_id)
            .collect::<Vec<_>>();
        bet_details.iter().for_each(|(gbetid, bet_detail)| {
            let mut bet_detail = bet_detail.clone();
            match &room_detail.bet_outcome {
                RoomBetPossibleOutcomes::HotWon => {
                    if bet_detail.bet_direction == BetDirection::Hot {
                        bet_detail.payout = BetPayout::Calculated(
                            bet_detail.amount
                                * HOT_OR_NOT_BET_WINNINGS_MULTIPLIER
                                * (100 - HOT_OR_NOT_BET_CREATOR_COMMISSION_PERCENTAGE)
                                / 100,
                        );
                    } else {
                        bet_detail.payout = BetPayout::Calculated(0);
                    }
                }
                RoomBetPossibleOutcomes::NotWon => {
                    if bet_detail.bet_direction == BetDirection::Not {
                        bet_detail.payout = BetPayout::Calculated(
                            bet_detail.amount
                                * HOT_OR_NOT_BET_WINNINGS_MULTIPLIER
                                * (100 - HOT_OR_NOT_BET_CREATOR_COMMISSION_PERCENTAGE)
                                / 100,
                        );
                    } else {
                        bet_detail.payout = BetPayout::Calculated(0);
                    }
                }
                RoomBetPossibleOutcomes::Draw => {
                    bet_detail.payout = BetPayout::Calculated(
                        bet_detail.amount
                            * (100 - HOT_OR_NOT_BET_CREATOR_COMMISSION_PERCENTAGE)
                            / 100,
                    );
                }
                RoomBetPossibleOutcomes::BetOngoing => {}
            };

            bet_details_map.insert(gbetid.clone(), bet_detail);
        });
    });
}

}

#[cfg(test)]
pub mod test_hot_or_not {
    use std::{cell::RefCell, time::Duration};

    use ic_stable_structures::memory_manager::{MemoryId, MemoryManager};
    use test_utils::setup::test_constants::{
        get_mock_user_alice_canister_id, get_mock_user_alice_principal_id,
    };

    use crate::canister_specific::individual_user_template::types::post::PostDetailsFromFrontend;

    use super::*;
    pub type Memory = VirtualMemory<DefaultMemoryImpl>;

    // #[test]
    // fn test_get_hot_or_not_betting_status_for_this_post() {
    //     let mut post = Post::new(
    //         0,
    //         &PostDetailsFromFrontend {
    //             is_nsfw: false,
    //             description: "Doggos and puppers".into(),
    //             hashtags: vec!["doggo".into(), "pupper".into()],
    //             video_uid: "abcd#1234".into(),
    //             creator_consent_for_inclusion_in_hot_or_not: true,
    //         },
    //         &SystemTime::now(),
    //     );

    //     let result = post.get_hot_or_not_betting_status_for_this_post(
    //         &SystemTime::now()
    //             .checked_add(Duration::from_secs(
    //                 TOTAL_DURATION_OF_ALL_SLOTS_IN_SECONDS + 1,
    //             ))
    //             .unwrap(),
    //         &Principal::anonymous(),
    //     );

    //     assert_eq!(result, BettingStatus::BettingClosed);

    //     let current_time = SystemTime::now();

    //     let result = post
    //         .get_hot_or_not_betting_status_for_this_post(&current_time, &Principal::anonymous());

    //     assert_eq!(
    //         result,
    //         BettingStatus::BettingOpen {
    //             started_at: post.created_at,
    //             number_of_participants: 0,
    //             ongoing_slot: 1,
    //             ongoing_room: 1,
    //             has_this_user_participated_in_this_post: None,
    //         }
    //     );

    //     let result = post.get_hot_or_not_betting_status_for_this_post(
    //         &current_time
    //             .checked_add(Duration::from_secs(
    //                 DURATION_OF_EACH_SLOT_IN_SECONDS * 2 + 1,
    //             ))
    //             .unwrap(),
    //         &Principal::anonymous(),
    //     );

    //     assert_eq!(
    //         result,
    //         BettingStatus::BettingOpen {
    //             started_at: post.created_at,
    //             number_of_participants: 0,
    //             ongoing_slot: 3,
    //             ongoing_room: 1,
    //             has_this_user_participated_in_this_post: None,
    //         }
    //     );

    //     let result = post.place_hot_or_not_bet(
    //         &get_mock_user_alice_principal_id(),
    //         &get_mock_user_alice_canister_id(),
    //         100,
    //         &BetDirection::Hot,
    //         &current_time
    //             .checked_add(Duration::from_secs(
    //                 DURATION_OF_EACH_SLOT_IN_SECONDS * 2 + 1,
    //             ))
    //             .unwrap(),
    //     );

    //     assert!(result.is_ok());

    //     let result = post.get_hot_or_not_betting_status_for_this_post(
    //         &current_time
    //             .checked_add(Duration::from_secs(
    //                 DURATION_OF_EACH_SLOT_IN_SECONDS * 2 + 1,
    //             ))
    //             .unwrap(),
    //         &get_mock_user_alice_principal_id(),
    //     );

    //     assert_eq!(
    //         result,
    //         BettingStatus::BettingOpen {
    //             started_at: post.created_at,
    //             number_of_participants: 1,
    //             ongoing_slot: 3,
    //             ongoing_room: 1,
    //             has_this_user_participated_in_this_post: Some(true),
    //         }
    //     );

    //     (100..200).for_each(|num| {
    //         let result = post.place_hot_or_not_bet(
    //             &Principal::from_slice(&[num]),
    //             &Principal::from_slice(&[num]),
    //             100,
    //             &BetDirection::Hot,
    //             &current_time
    //                 .checked_add(Duration::from_secs(
    //                     DURATION_OF_EACH_SLOT_IN_SECONDS * 2 + 1,
    //                 ))
    //                 .unwrap(),
    //         );

    //         assert!(result.is_ok());
    //     });

    //     let result = post.place_hot_or_not_bet(
    //         &Principal::from_slice(&[200]),
    //         &Principal::from_slice(&[200]),
    //         100,
    //         &BetDirection::Hot,
    //         &current_time
    //             .checked_add(Duration::from_secs(
    //                 DURATION_OF_EACH_SLOT_IN_SECONDS * 2 + 1,
    //             ))
    //             .unwrap(),
    //     );

    //     assert!(result.is_ok());

    //     let result = post.get_hot_or_not_betting_status_for_this_post(
    //         &current_time
    //             .checked_add(Duration::from_secs(
    //                 DURATION_OF_EACH_SLOT_IN_SECONDS * 2 + 1,
    //             ))
    //             .unwrap(),
    //         &Principal::from_slice(&[100]),
    //     );

    //     assert_eq!(
    //         result,
    //         BettingStatus::BettingOpen {
    //             started_at: post.created_at,
    //             number_of_participants: 2,
    //             ongoing_slot: 3,
    //             ongoing_room: 2,
    //             has_this_user_participated_in_this_post: Some(true),
    //         }
    //     );

    //     let result = post.place_hot_or_not_bet(
    //         &get_mock_user_alice_principal_id(),
    //         &get_mock_user_alice_canister_id(),
    //         100,
    //         &BetDirection::Hot,
    //         &current_time
    //             .checked_add(Duration::from_secs(
    //                 DURATION_OF_EACH_SLOT_IN_SECONDS * 2 + 1,
    //             ))
    //             .unwrap(),
    //     );

    //     assert!(result.is_err());

    //     let result = post.get_hot_or_not_betting_status_for_this_post(
    //         &current_time
    //             .checked_add(Duration::from_secs(
    //                 DURATION_OF_EACH_SLOT_IN_SECONDS * 2 + 1,
    //             ))
    //             .unwrap(),
    //         &get_mock_user_alice_principal_id(),
    //     );

    //     assert_eq!(
    //         result,
    //         BettingStatus::BettingOpen {
    //             started_at: post.created_at,
    //             number_of_participants: 2,
    //             ongoing_slot: 3,
    //             ongoing_room: 2,
    //             has_this_user_participated_in_this_post: Some(true),
    //         }
    //     );

    //     let result = post.place_hot_or_not_bet(
    //         &get_mock_user_alice_principal_id(),
    //         &get_mock_user_alice_canister_id(),
    //         100,
    //         &BetDirection::Hot,
    //         &current_time
    //             .checked_add(Duration::from_secs(
    //                 DURATION_OF_EACH_SLOT_IN_SECONDS * 4 + 1,
    //             ))
    //             .unwrap(),
    //     );

    //     assert!(result.is_err());

    //     let result = post.get_hot_or_not_betting_status_for_this_post(
    //         &current_time
    //             .checked_add(Duration::from_secs(
    //                 DURATION_OF_EACH_SLOT_IN_SECONDS * 4 + 1,
    //             ))
    //             .unwrap(),
    //         &get_mock_user_alice_principal_id(),
    //     );

    //     assert_eq!(
    //         result,
    //         BettingStatus::BettingOpen {
    //             started_at: post.created_at,
    //             number_of_participants: 0,
    //             ongoing_slot: 5,
    //             ongoing_room: 1,
    //             has_this_user_participated_in_this_post: Some(true),
    //         }
    //     );
    // }

    // #[test]
    // fn test_has_this_principal_already_bet_on_this_post() {
    //     let mut post = Post::new(
    //         0,
    //         &PostDetailsFromFrontend {
    //             is_nsfw: false,
    //             description: "Doggos and puppers".into(),
    //             hashtags: vec!["doggo".into(), "pupper".into()],
    //             video_uid: "abcd#1234".into(),
    //             creator_consent_for_inclusion_in_hot_or_not: true,
    //         },
    //         &SystemTime::now(),
    //     );

    //     let result =
    //         post.has_this_principal_already_bet_on_this_post(&get_mock_user_alice_principal_id());

    //     assert!(!result);

    //     post.place_hot_or_not_bet(
    //         &get_mock_user_alice_principal_id(),
    //         &get_mock_user_alice_canister_id(),
    //         100,
    //         &BetDirection::Hot,
    //         &SystemTime::now(),
    //     )
    //     .ok();

    //     let result =
    //         post.has_this_principal_already_bet_on_this_post(&get_mock_user_alice_principal_id());

    //     assert!(result);
    // }

    // #[test]
    // fn test_place_hot_or_not_bet() {
    //     let mut post = Post::new(
    //         0,
    //         &PostDetailsFromFrontend {
    //             is_nsfw: false,
    //             description: "Doggos and puppers".into(),
    //             hashtags: vec!["doggo".into(), "pupper".into()],
    //             video_uid: "abcd#1234".into(),
    //             creator_consent_for_inclusion_in_hot_or_not: true,
    //         },
    //         &SystemTime::now(),
    //     );

    //     assert!(post.hot_or_not_details.is_some());

    //     let result = post.place_hot_or_not_bet(
    //         &get_mock_user_alice_principal_id(),
    //         &get_mock_user_alice_canister_id(),
    //         100,
    //         &BetDirection::Hot,
    //         &SystemTime::now()
    //             .checked_add(Duration::from_secs(
    //                 TOTAL_DURATION_OF_ALL_SLOTS_IN_SECONDS + 1,
    //             ))
    //             .unwrap(),
    //     );

    //     assert_eq!(result, Err(BetOnCurrentlyViewingPostError::BettingClosed));

    //     let result = post.place_hot_or_not_bet(
    //         &get_mock_user_alice_principal_id(),
    //         &get_mock_user_alice_canister_id(),
    //         100,
    //         &BetDirection::Hot,
    //         &SystemTime::now(),
    //     );

    //     assert_eq!(
    //         result,
    //         Ok(BettingStatus::BettingOpen {
    //             started_at: post.created_at,
    //             number_of_participants: 1,
    //             ongoing_slot: 1,
    //             ongoing_room: 1,
    //             has_this_user_participated_in_this_post: Some(true)
    //         })
    //     );
    //     let hot_or_not_details = post.hot_or_not_details.clone().unwrap();
    //     assert_eq!(hot_or_not_details.slot_history.len(), 1);
    //     let room_details = &hot_or_not_details
    //         .slot_history
    //         .get(&1)
    //         .unwrap()
    //         .room_details;
    //     assert_eq!(room_details.len(), 1);
    //     let room_detail = room_details.get(&1).unwrap();
    //     let bets_made = &room_detail.bets_made;
    //     assert_eq!(bets_made.len(), 1);
    //     assert_eq!(
    //         bets_made
    //             .get(&get_mock_user_alice_principal_id())
    //             .unwrap()
    //             .amount,
    //         100
    //     );
    //     assert_eq!(
    //         bets_made
    //             .get(&get_mock_user_alice_principal_id())
    //             .unwrap()
    //             .bet_direction,
    //         BetDirection::Hot
    //     );
    //     assert_eq!(room_detail.room_bets_total_pot, 100);
    //     assert_eq!(room_detail.total_hot_bets, 1);
    //     assert_eq!(room_detail.total_not_bets, 0);
    //     assert_eq!(hot_or_not_details.aggregate_stats.total_amount_bet, 100);
    //     assert_eq!(
    //         hot_or_not_details.aggregate_stats.total_number_of_hot_bets,
    //         1
    //     );
    //     assert_eq!(
    //         hot_or_not_details.aggregate_stats.total_number_of_not_bets,
    //         0
    //     );

    //     let result = post.place_hot_or_not_bet(
    //         &get_mock_user_alice_principal_id(),
    //         &get_mock_user_alice_canister_id(),
    //         100,
    //         &BetDirection::Hot,
    //         &SystemTime::now(),
    //     );
    //     assert!(result.is_err());
    // }

    // #[test]
    // fn test_tabulate_hot_or_not_outcome_for_slot_case_1() {
    //     let post_creation_time = SystemTime::now();
    //     let mut post = Post::new(
    //         0,
    //         &PostDetailsFromFrontend {
    //             is_nsfw: false,
    //             description: "Doggos and puppers".into(),
    //             hashtags: vec!["doggo".into(), "pupper".into()],
    //             video_uid: "abcd#1234".into(),
    //             creator_consent_for_inclusion_in_hot_or_not: true,
    //         },
    //         &post_creation_time,
    //     );
    //     let mut token_balance = TokenBalance::default();
    //     let tabulation_canister_id = get_mock_user_alice_canister_id();

    //     assert!(post.hot_or_not_details.is_some());

    //     let data_set: Vec<(u64, BetDirection, u64, u64)> = vec![
    //         (1, BetDirection::Not, 10, 18),
    //         (2, BetDirection::Hot, 100, 0),
    //         (3, BetDirection::Hot, 100, 0),
    //         (4, BetDirection::Not, 100, 180),
    //         (5, BetDirection::Hot, 10, 0),
    //         (6, BetDirection::Not, 100, 180),
    //         (7, BetDirection::Not, 50, 90),
    //         (8, BetDirection::Not, 100, 180),
    //         (9, BetDirection::Hot, 50, 0),
    //         (10, BetDirection::Not, 50, 90),
    //         (11, BetDirection::Not, 100, 180),
    //         (12, BetDirection::Not, 10, 18),
    //         (13, BetDirection::Hot, 100, 0),
    //         (14, BetDirection::Not, 10, 18),
    //         (15, BetDirection::Hot, 50, 0),
    //         (16, BetDirection::Hot, 10, 0),
    //         (17, BetDirection::Hot, 10, 0),
    //         (18, BetDirection::Hot, 100, 0),
    //         (19, BetDirection::Not, 10, 18),
    //         (20, BetDirection::Hot, 50, 0),
    //         (21, BetDirection::Hot, 10, 0),
    //         (22, BetDirection::Not, 50, 90),
    //         (23, BetDirection::Not, 50, 90),
    //         (24, BetDirection::Hot, 100, 0),
    //         (25, BetDirection::Not, 50, 90),
    //         (26, BetDirection::Not, 10, 18),
    //         (27, BetDirection::Not, 10, 18),
    //         (28, BetDirection::Not, 50, 90),
    //         (29, BetDirection::Hot, 50, 0),
    //         (30, BetDirection::Not, 100, 180),
    //         (31, BetDirection::Not, 50, 90),
    //         (32, BetDirection::Not, 50, 90),
    //         (33, BetDirection::Hot, 100, 0),
    //         (34, BetDirection::Not, 10, 18),
    //         (35, BetDirection::Not, 10, 18),
    //         (36, BetDirection::Not, 100, 180),
    //         (37, BetDirection::Hot, 10, 0),
    //         (38, BetDirection::Not, 100, 180),
    //         (39, BetDirection::Not, 50, 90),
    //         (40, BetDirection::Hot, 100, 0),
    //         (41, BetDirection::Hot, 50, 0),
    //         (42, BetDirection::Not, 10, 18),
    //         (43, BetDirection::Hot, 50, 0),
    //         (44, BetDirection::Not, 10, 18),
    //         (45, BetDirection::Not, 10, 18),
    //         (46, BetDirection::Hot, 100, 0),
    //         (47, BetDirection::Hot, 50, 0),
    //         (48, BetDirection::Hot, 50, 0),
    //         (49, BetDirection::Not, 100, 180),
    //         (50, BetDirection::Hot, 10, 0),
    //         (51, BetDirection::Not, 50, 90),
    //         (52, BetDirection::Hot, 10, 0),
    //         (53, BetDirection::Not, 50, 90),
    //         (54, BetDirection::Not, 10, 18),
    //         (55, BetDirection::Hot, 100, 0),
    //         (56, BetDirection::Hot, 50, 0),
    //         (57, BetDirection::Not, 50, 90),
    //         (58, BetDirection::Not, 10, 18),
    //         (59, BetDirection::Not, 50, 90),
    //         (60, BetDirection::Hot, 10, 0),
    //         (61, BetDirection::Not, 10, 18),
    //         (62, BetDirection::Not, 50, 90),
    //         (63, BetDirection::Not, 50, 90),
    //         (64, BetDirection::Not, 10, 18),
    //         (65, BetDirection::Not, 10, 18),
    //         (66, BetDirection::Not, 100, 180),
    //         (67, BetDirection::Hot, 100, 0),
    //         (68, BetDirection::Not, 10, 18),
    //         (69, BetDirection::Not, 10, 18),
    //         (70, BetDirection::Not, 50, 90),
    //         (71, BetDirection::Not, 100, 180),
    //         (72, BetDirection::Not, 10, 18),
    //         (73, BetDirection::Not, 10, 18),
    //         (74, BetDirection::Hot, 10, 0),
    //         (75, BetDirection::Not, 10, 18),
    //     ];

    //     data_set
    //         .iter()
    //         .for_each(|(user_id, bet_direction, bet_amount, _)| {
    //             let result = post.place_hot_or_not_bet(
    //                 &Principal::self_authenticating(user_id.to_ne_bytes()),
    //                 &Principal::self_authenticating(user_id.to_ne_bytes()),
    //                 *bet_amount,
    //                 bet_direction,
    //                 &post_creation_time,
    //             );
    //             assert!(result.is_ok());
    //         });

    //     let score_tabulation_time = post_creation_time
    //         .checked_add(Duration::from_secs(60 * 5))
    //         .unwrap();

    //     post.tabulate_hot_or_not_outcome_for_slot(
    //         &tabulation_canister_id,
    //         &1,
    //         &mut token_balance,
    //         &score_tabulation_time,
    //     );

    //     assert_eq!(token_balance.utility_token_transaction_history.len(), 1);
    //     assert_eq!(token_balance.utility_token_balance, 355);

    //     let room_detail = post
    //         .hot_or_not_details
    //         .as_ref()
    //         .unwrap()
    //         .slot_history
    //         .get(&1)
    //         .unwrap()
    //         .room_details
    //         .get(&1)
    //         .unwrap();

    //     assert_eq!(room_detail.bet_outcome, RoomBetPossibleOutcomes::NotWon);
    //     assert_eq!(room_detail.room_bets_total_pot, 3550);
    //     assert_eq!(room_detail.total_hot_bets, 28);
    //     assert_eq!(room_detail.total_not_bets, 47);

    //     data_set
    //         .iter()
    //         .for_each(|(user_id, bet_direction, bet_amount, amount_won)| {
    //             let bet_detail = room_detail
    //                 .bets_made
    //                 .get(&Principal::self_authenticating(user_id.to_ne_bytes()))
    //                 .unwrap();

    //             assert_eq!(bet_detail.bet_direction, *bet_direction);
    //             assert_eq!(bet_detail.amount, *bet_amount);
    //             assert_eq!(
    //                 match bet_detail.payout {
    //                     BetPayout::Calculated(n) => {
    //                         n
    //                     }
    //                     _ => {
    //                         0
    //                     }
    //                 },
    //                 *amount_won
    //             );
    //         });

    //     let data_set: Vec<(u64, BetDirection, u64, u64)> = vec![
    //         (1, BetDirection::Hot, 10, 18),
    //         (2, BetDirection::Hot, 50, 90),
    //         (3, BetDirection::Hot, 10, 18),
    //         (4, BetDirection::Not, 100, 0),
    //         (5, BetDirection::Hot, 100, 180),
    //         (6, BetDirection::Not, 100, 0),
    //         (7, BetDirection::Hot, 50, 90),
    //         (8, BetDirection::Hot, 100, 180),
    //         (9, BetDirection::Hot, 100, 180),
    //         (10, BetDirection::Not, 50, 0),
    //         (11, BetDirection::Not, 50, 0),
    //         (12, BetDirection::Hot, 50, 90),
    //         (13, BetDirection::Hot, 100, 180),
    //         (14, BetDirection::Hot, 100, 180),
    //         (15, BetDirection::Not, 50, 0),
    //         (16, BetDirection::Not, 50, 0),
    //         (17, BetDirection::Not, 100, 0),
    //         (18, BetDirection::Not, 100, 0),
    //         (19, BetDirection::Hot, 100, 180),
    //         (20, BetDirection::Not, 10, 0),
    //         (21, BetDirection::Hot, 100, 180),
    //         (22, BetDirection::Hot, 10, 18),
    //         (23, BetDirection::Hot, 10, 18),
    //         (24, BetDirection::Hot, 50, 90),
    //         (25, BetDirection::Not, 100, 0),
    //         (26, BetDirection::Hot, 10, 18),
    //         (27, BetDirection::Hot, 100, 180),
    //         (28, BetDirection::Hot, 50, 90),
    //         (29, BetDirection::Hot, 50, 90),
    //         (30, BetDirection::Hot, 10, 18),
    //         (31, BetDirection::Hot, 10, 18),
    //         (32, BetDirection::Hot, 100, 180),
    //         (33, BetDirection::Not, 100, 0),
    //         (34, BetDirection::Hot, 50, 90),
    //         (35, BetDirection::Hot, 100, 180),
    //         (36, BetDirection::Hot, 100, 180),
    //         (37, BetDirection::Hot, 50, 90),
    //         (38, BetDirection::Not, 10, 0),
    //         (39, BetDirection::Hot, 50, 90),
    //         (40, BetDirection::Not, 10, 0),
    //         (41, BetDirection::Hot, 50, 90),
    //         (42, BetDirection::Not, 10, 0),
    //         (43, BetDirection::Not, 100, 0),
    //         (44, BetDirection::Not, 100, 0),
    //         (45, BetDirection::Not, 100, 0),
    //         (46, BetDirection::Hot, 100, 180),
    //         (47, BetDirection::Not, 50, 0),
    //         (48, BetDirection::Hot, 100, 180),
    //         (49, BetDirection::Not, 100, 0),
    //         (50, BetDirection::Not, 50, 0),
    //         (51, BetDirection::Not, 10, 0),
    //         (52, BetDirection::Not, 100, 0),
    //         (53, BetDirection::Hot, 100, 180),
    //         (54, BetDirection::Hot, 10, 18),
    //         (55, BetDirection::Not, 100, 0),
    //         (56, BetDirection::Not, 100, 0),
    //         (57, BetDirection::Hot, 50, 90),
    //         (58, BetDirection::Not, 100, 0),
    //         (59, BetDirection::Not, 10, 0),
    //         (60, BetDirection::Hot, 10, 18),
    //         (61, BetDirection::Not, 10, 0),
    //         (62, BetDirection::Hot, 50, 90),
    //         (63, BetDirection::Hot, 10, 18),
    //         (64, BetDirection::Hot, 50, 90),
    //         (65, BetDirection::Not, 100, 0),
    //         (66, BetDirection::Not, 50, 0),
    //         (67, BetDirection::Not, 100, 0),
    //         (68, BetDirection::Hot, 10, 18),
    //         (69, BetDirection::Hot, 50, 90),
    //         (70, BetDirection::Not, 100, 0),
    //         (71, BetDirection::Hot, 50, 90),
    //         (72, BetDirection::Hot, 50, 90),
    //         (73, BetDirection::Not, 50, 0),
    //         (74, BetDirection::Not, 50, 0),
    //         (75, BetDirection::Not, 50, 0),
    //     ];

    //     // * 1 min into the 2nd hour/2nd slot
    //     let post_creation_time = post_creation_time
    //         .checked_add(Duration::from_secs(60 * (60 + 1)))
    //         .unwrap();

    //     data_set
    //         .iter()
    //         .for_each(|(user_id, bet_direction, bet_amount, _)| {
    //             let result = post.place_hot_or_not_bet(
    //                 &Principal::self_authenticating((user_id + 75).to_ne_bytes()),
    //                 &Principal::self_authenticating((user_id + 75).to_ne_bytes()),
    //                 *bet_amount,
    //                 bet_direction,
    //                 &post_creation_time,
    //             );
    //             assert!(result.is_ok());
    //         });

    //     let score_tabulation_time = post_creation_time
    //         .checked_add(Duration::from_secs(60 * 5))
    //         .unwrap();

    //     post.tabulate_hot_or_not_outcome_for_slot(
    //         &get_mock_user_alice_canister_id(),
    //         &2,
    //         &mut token_balance,
    //         &score_tabulation_time,
    //     );

    //     assert_eq!(token_balance.utility_token_transaction_history.len(), 2);
    //     assert_eq!(token_balance.utility_token_balance, 355 + 458);

    //     let room_detail = post
    //         .hot_or_not_details
    //         .as_ref()
    //         .unwrap()
    //         .slot_history
    //         .get(&2)
    //         .unwrap()
    //         .room_details
    //         .get(&1)
    //         .unwrap();

    //     assert_eq!(room_detail.bet_outcome, RoomBetPossibleOutcomes::HotWon);
    //     assert_eq!(room_detail.room_bets_total_pot, 4580);
    //     assert_eq!(room_detail.total_hot_bets, 41);
    //     assert_eq!(room_detail.total_not_bets, 34);

    //     data_set
    //         .iter()
    //         .for_each(|(user_id, bet_direction, bet_amount, amount_won)| {
    //             let bet_detail = room_detail
    //                 .bets_made
    //                 .get(&Principal::self_authenticating(
    //                     (user_id + 75).to_ne_bytes(),
    //                 ))
    //                 .unwrap();

    //             assert_eq!(bet_detail.bet_direction, *bet_direction);
    //             assert_eq!(bet_detail.amount, *bet_amount);
    //             assert_eq!(
    //                 match bet_detail.payout {
    //                     BetPayout::Calculated(n) => {
    //                         n
    //                     }
    //                     _ => {
    //                         0
    //                     }
    //                 },
    //                 *amount_won
    //             );
    //         });
    // }

    // #[test]
    // fn test_tabulate_hot_or_not_outcome_for_slot_case_2() {
    //     let post_creation_time = SystemTime::now();
    //     let mut post = Post::new(
    //         0,
    //         &PostDetailsFromFrontend {
    //             is_nsfw: false,
    //             description: "Doggos and puppers".into(),
    //             hashtags: vec!["doggo".into(), "pupper".into()],
    //             video_uid: "abcd#1234".into(),
    //             creator_consent_for_inclusion_in_hot_or_not: true,
    //         },
    //         &post_creation_time,
    //     );
    //     let mut token_balance = TokenBalance::default();

    //     assert!(post.hot_or_not_details.is_some());

    //     let data_set: Vec<(u64, BetDirection, u64, u64)> = vec![
    //         (1, BetDirection::Not, 10, 18),
    //         (2, BetDirection::Hot, 100, 0),
    //         (3, BetDirection::Hot, 100, 0),
    //         (4, BetDirection::Not, 100, 180),
    //         (5, BetDirection::Hot, 10, 0),
    //         (6, BetDirection::Not, 100, 180),
    //         (7, BetDirection::Not, 50, 90),
    //         (8, BetDirection::Not, 100, 180),
    //         (9, BetDirection::Hot, 50, 0),
    //         (10, BetDirection::Not, 50, 90),
    //         (11, BetDirection::Not, 100, 180),
    //         (12, BetDirection::Not, 10, 18),
    //         (13, BetDirection::Hot, 100, 0),
    //         (14, BetDirection::Not, 10, 18),
    //         (15, BetDirection::Hot, 50, 0),
    //         (16, BetDirection::Hot, 10, 0),
    //         (17, BetDirection::Hot, 10, 0),
    //         (18, BetDirection::Hot, 100, 0),
    //         (19, BetDirection::Not, 10, 18),
    //         (20, BetDirection::Hot, 50, 0),
    //         (21, BetDirection::Hot, 10, 0),
    //         (22, BetDirection::Not, 50, 90),
    //         (23, BetDirection::Not, 50, 90),
    //         (24, BetDirection::Hot, 100, 0),
    //         (25, BetDirection::Not, 50, 90),
    //         (26, BetDirection::Not, 10, 18),
    //         (27, BetDirection::Not, 10, 18),
    //         (28, BetDirection::Not, 50, 90),
    //         (29, BetDirection::Hot, 50, 0),
    //         (30, BetDirection::Not, 100, 180),
    //         (31, BetDirection::Not, 50, 90),
    //         (32, BetDirection::Not, 50, 90),
    //         (33, BetDirection::Hot, 100, 0),
    //         (34, BetDirection::Not, 10, 18),
    //         (35, BetDirection::Not, 10, 18),
    //         (36, BetDirection::Not, 100, 180),
    //         (37, BetDirection::Hot, 10, 0),
    //         (38, BetDirection::Not, 100, 180),
    //         (39, BetDirection::Not, 50, 90),
    //         (40, BetDirection::Hot, 100, 0),
    //         (41, BetDirection::Hot, 50, 0),
    //         (42, BetDirection::Not, 10, 18),
    //         (43, BetDirection::Hot, 50, 0),
    //         (44, BetDirection::Not, 10, 18),
    //         (45, BetDirection::Not, 10, 18),
    //         (46, BetDirection::Hot, 100, 0),
    //         (47, BetDirection::Hot, 50, 0),
    //         (48, BetDirection::Hot, 50, 0),
    //         (49, BetDirection::Not, 100, 180),
    //         (50, BetDirection::Hot, 10, 0),
    //         (51, BetDirection::Not, 50, 90),
    //         (52, BetDirection::Hot, 10, 0),
    //         (53, BetDirection::Not, 50, 90),
    //         (54, BetDirection::Not, 10, 18),
    //         (55, BetDirection::Hot, 100, 0),
    //         (56, BetDirection::Hot, 50, 0),
    //         (57, BetDirection::Not, 50, 90),
    //         (58, BetDirection::Not, 10, 18),
    //         (59, BetDirection::Not, 50, 90),
    //         (60, BetDirection::Hot, 10, 0),
    //         (61, BetDirection::Not, 10, 18),
    //         (62, BetDirection::Not, 50, 90),
    //         (63, BetDirection::Not, 50, 90),
    //         (64, BetDirection::Not, 10, 18),
    //         (65, BetDirection::Not, 10, 18),
    //         (66, BetDirection::Not, 100, 180),
    //         (67, BetDirection::Hot, 100, 0),
    //         (68, BetDirection::Not, 10, 18),
    //         (69, BetDirection::Not, 10, 18),
    //         (70, BetDirection::Not, 50, 90),
    //         (71, BetDirection::Not, 100, 180),
    //         (72, BetDirection::Not, 10, 18),
    //         (73, BetDirection::Not, 10, 18),
    //         (74, BetDirection::Hot, 10, 0),
    //         (75, BetDirection::Not, 10, 18),
    //         (76, BetDirection::Hot, 50, 0),
    //         (77, BetDirection::Hot, 50, 0),
    //         (78, BetDirection::Not, 100, 180),
    //         (79, BetDirection::Not, 100, 180),
    //         (80, BetDirection::Hot, 50, 0),
    //         (81, BetDirection::Hot, 10, 0),
    //         (82, BetDirection::Hot, 50, 0),
    //         (83, BetDirection::Not, 10, 18),
    //         (84, BetDirection::Not, 50, 90),
    //         (85, BetDirection::Not, 10, 18),
    //         (86, BetDirection::Not, 10, 18),
    //         (87, BetDirection::Hot, 100, 0),
    //         (88, BetDirection::Not, 10, 18),
    //         (89, BetDirection::Not, 50, 90),
    //         (90, BetDirection::Hot, 100, 0),
    //         (91, BetDirection::Hot, 100, 0),
    //         (92, BetDirection::Hot, 10, 0),
    //         (93, BetDirection::Hot, 10, 0),
    //         (94, BetDirection::Hot, 100, 0),
    //         (95, BetDirection::Hot, 50, 0),
    //         (96, BetDirection::Hot, 100, 0),
    //         (97, BetDirection::Hot, 50, 0),
    //         (98, BetDirection::Hot, 50, 0),
    //         (99, BetDirection::Hot, 50, 0),
    //         (100, BetDirection::Hot, 50, 0),
    //         (101, BetDirection::Not, 10, 18),
    //         (102, BetDirection::Not, 50, 90),
    //         (103, BetDirection::Not, 10, 18),
    //         (104, BetDirection::Hot, 100, 0),
    //         (105, BetDirection::Not, 100, 180),
    //         (106, BetDirection::Hot, 100, 0),
    //         (107, BetDirection::Not, 50, 90),
    //         (108, BetDirection::Not, 100, 180),
    //         (109, BetDirection::Not, 100, 180),
    //         (110, BetDirection::Hot, 50, 0),
    //         (111, BetDirection::Hot, 50, 0),
    //         (112, BetDirection::Not, 50, 90),
    //         (113, BetDirection::Not, 100, 180),
    //         (114, BetDirection::Not, 100, 180),
    //         (115, BetDirection::Hot, 50, 0),
    //         (116, BetDirection::Hot, 50, 0),
    //         (117, BetDirection::Hot, 100, 0),
    //         (118, BetDirection::Hot, 100, 0),
    //         (119, BetDirection::Not, 100, 180),
    //         (120, BetDirection::Hot, 10, 0),
    //         (121, BetDirection::Not, 100, 180),
    //         (122, BetDirection::Not, 10, 18),
    //         (123, BetDirection::Not, 10, 18),
    //         (124, BetDirection::Not, 50, 90),
    //         (125, BetDirection::Hot, 100, 0),
    //         (126, BetDirection::Not, 10, 18),
    //         (127, BetDirection::Not, 100, 180),
    //         (128, BetDirection::Not, 50, 90),
    //         (129, BetDirection::Not, 50, 90),
    //         (130, BetDirection::Not, 10, 18),
    //         (131, BetDirection::Not, 10, 18),
    //         (132, BetDirection::Not, 100, 180),
    //         (133, BetDirection::Hot, 100, 0),
    //         (134, BetDirection::Not, 50, 90),
    //         (135, BetDirection::Not, 100, 180),
    //         (136, BetDirection::Not, 100, 180),
    //         (137, BetDirection::Not, 50, 90),
    //         (138, BetDirection::Hot, 10, 0),
    //         (139, BetDirection::Not, 50, 90),
    //         (140, BetDirection::Hot, 10, 0),
    //         (141, BetDirection::Not, 50, 90),
    //         (142, BetDirection::Hot, 10, 0),
    //         (143, BetDirection::Hot, 100, 0),
    //         (144, BetDirection::Hot, 100, 0),
    //         (145, BetDirection::Hot, 100, 0),
    //         (146, BetDirection::Not, 100, 180),
    //         (147, BetDirection::Hot, 50, 0),
    //         (148, BetDirection::Not, 100, 180),
    //         (149, BetDirection::Hot, 100, 0),
    //         (150, BetDirection::Hot, 50, 0),
    //     ];

    //     data_set
    //         .iter()
    //         .for_each(|(user_id, bet_direction, bet_amount, _)| {
    //             let result = post.place_hot_or_not_bet(
    //                 &Principal::self_authenticating(user_id.to_ne_bytes()),
    //                 &Principal::self_authenticating(user_id.to_ne_bytes()),
    //                 *bet_amount,
    //                 bet_direction,
    //                 &post_creation_time,
    //             );
    //             assert!(result.is_ok());
    //         });

    //     let score_tabulation_time = post_creation_time
    //         .checked_add(Duration::from_secs(60 * 5))
    //         .unwrap();

    //     post.tabulate_hot_or_not_outcome_for_slot(
    //         &get_mock_user_alice_canister_id(),
    //         &1,
    //         &mut token_balance,
    //         &score_tabulation_time,
    //     );

    //     assert_eq!(token_balance.utility_token_transaction_history.len(), 2);
    //     assert_eq!(token_balance.utility_token_balance, 487 + 321);

    //     // * Room 1
    //     let room_detail = post
    //         .hot_or_not_details
    //         .as_ref()
    //         .unwrap()
    //         .slot_history
    //         .get(&1)
    //         .unwrap()
    //         .room_details
    //         .get(&1)
    //         .unwrap();

    //     assert_eq!(room_detail.bet_outcome, RoomBetPossibleOutcomes::NotWon);
    //     assert_eq!(room_detail.room_bets_total_pot, 4870);
    //     assert_eq!(room_detail.total_hot_bets, 45);
    //     assert_eq!(room_detail.total_not_bets, 55);

    //     data_set[0..100]
    //         .iter()
    //         .for_each(|(user_id, bet_direction, bet_amount, amount_won)| {
    //             let bet_detail = room_detail
    //                 .bets_made
    //                 .get(&Principal::self_authenticating(user_id.to_ne_bytes()))
    //                 .unwrap();

    //             assert_eq!(bet_detail.bet_direction, *bet_direction);
    //             assert_eq!(bet_detail.amount, *bet_amount);
    //             assert_eq!(
    //                 match bet_detail.payout {
    //                     BetPayout::Calculated(n) => {
    //                         n
    //                     }
    //                     _ => {
    //                         0
    //                     }
    //                 },
    //                 *amount_won
    //             );
    //         });

    //     // * Room 2
    //     let room_detail = post
    //         .hot_or_not_details
    //         .as_ref()
    //         .unwrap()
    //         .slot_history
    //         .get(&1)
    //         .unwrap()
    //         .room_details
    //         .get(&2)
    //         .unwrap();

    //     assert_eq!(room_detail.bet_outcome, RoomBetPossibleOutcomes::NotWon);
    //     assert_eq!(room_detail.room_bets_total_pot, 3210);
    //     assert_eq!(room_detail.total_hot_bets, 20);
    //     assert_eq!(room_detail.total_not_bets, 30);

    //     data_set[100..]
    //         .iter()
    //         .for_each(|(user_id, bet_direction, bet_amount, amount_won)| {
    //             let bet_detail = room_detail
    //                 .bets_made
    //                 .get(&Principal::self_authenticating(user_id.to_ne_bytes()))
    //                 .unwrap();

    //             assert_eq!(bet_detail.bet_direction, *bet_direction);
    //             assert_eq!(bet_detail.amount, *bet_amount);
    //             assert_eq!(
    //                 match bet_detail.payout {
    //                     BetPayout::Calculated(n) => {
    //                         n
    //                     }
    //                     _ => {
    //                         0
    //                     }
    //                 },
    //                 *amount_won
    //             );
    //         });
    // }

    // #[test]
    // fn test_tabulate_hot_or_not_outcome_for_slot_case_3() {
    //     let post_creation_time = SystemTime::now();
    //     let mut post = Post::new(
    //         0,
    //         &PostDetailsFromFrontend {
    //             is_nsfw: false,
    //             description: "Doggos and puppers".into(),
    //             hashtags: vec!["doggo".into(), "pupper".into()],
    //             video_uid: "abcd#1234".into(),
    //             creator_consent_for_inclusion_in_hot_or_not: true,
    //         },
    //         &post_creation_time,
    //     );
    //     let mut token_balance = TokenBalance::default();

    //     assert!(post.hot_or_not_details.is_some());

    //     let data_set: Vec<(u64, BetDirection, u64, u64)> = vec![
    //         (1, BetDirection::Not, 10, 9),
    //         (2, BetDirection::Hot, 100, 90),
    //         (3, BetDirection::Hot, 100, 90),
    //         (4, BetDirection::Hot, 100, 90),
    //         (5, BetDirection::Hot, 10, 9),
    //         (6, BetDirection::Hot, 100, 90),
    //         (7, BetDirection::Hot, 50, 45),
    //         (8, BetDirection::Not, 100, 90),
    //         (9, BetDirection::Hot, 50, 45),
    //         (10, BetDirection::Not, 50, 45),
    //         (11, BetDirection::Not, 100, 90),
    //         (12, BetDirection::Hot, 10, 9),
    //         (13, BetDirection::Hot, 100, 90),
    //         (14, BetDirection::Not, 10, 9),
    //         (15, BetDirection::Hot, 50, 45),
    //         (16, BetDirection::Hot, 10, 9),
    //         (17, BetDirection::Hot, 10, 9),
    //         (18, BetDirection::Hot, 100, 90),
    //         (19, BetDirection::Not, 10, 9),
    //         (20, BetDirection::Hot, 50, 45),
    //         (21, BetDirection::Hot, 10, 9),
    //         (22, BetDirection::Hot, 50, 45),
    //         (23, BetDirection::Not, 50, 45),
    //         (24, BetDirection::Hot, 100, 90),
    //         (25, BetDirection::Not, 50, 45),
    //         (26, BetDirection::Not, 10, 9),
    //         (27, BetDirection::Hot, 10, 9),
    //         (28, BetDirection::Hot, 50, 45),
    //         (29, BetDirection::Hot, 50, 45),
    //         (30, BetDirection::Not, 100, 90),
    //         (31, BetDirection::Hot, 50, 45),
    //         (32, BetDirection::Not, 50, 45),
    //         (33, BetDirection::Hot, 100, 90),
    //         (34, BetDirection::Hot, 10, 9),
    //         (35, BetDirection::Not, 10, 9),
    //         (36, BetDirection::Not, 100, 90),
    //         (37, BetDirection::Hot, 10, 9),
    //         (38, BetDirection::Not, 100, 90),
    //         (39, BetDirection::Not, 50, 45),
    //         (40, BetDirection::Hot, 100, 90),
    //         (41, BetDirection::Hot, 50, 45),
    //         (42, BetDirection::Not, 10, 9),
    //         (43, BetDirection::Hot, 50, 45),
    //         (44, BetDirection::Not, 10, 9),
    //         (45, BetDirection::Not, 10, 9),
    //         (46, BetDirection::Hot, 100, 90),
    //         (47, BetDirection::Hot, 50, 45),
    //         (48, BetDirection::Hot, 50, 45),
    //         (49, BetDirection::Not, 100, 90),
    //         (50, BetDirection::Hot, 10, 9),
    //         (51, BetDirection::Not, 50, 45),
    //         (52, BetDirection::Hot, 10, 9),
    //         (53, BetDirection::Not, 50, 45),
    //         (54, BetDirection::Not, 10, 9),
    //         (55, BetDirection::Hot, 100, 90),
    //         (56, BetDirection::Hot, 50, 45),
    //         (57, BetDirection::Not, 50, 45),
    //         (58, BetDirection::Not, 10, 9),
    //         (59, BetDirection::Not, 50, 45),
    //         (60, BetDirection::Hot, 10, 9),
    //         (61, BetDirection::Not, 10, 9),
    //         (62, BetDirection::Not, 50, 45),
    //         (63, BetDirection::Not, 50, 45),
    //         (64, BetDirection::Not, 10, 9),
    //         (65, BetDirection::Not, 10, 9),
    //         (66, BetDirection::Not, 100, 90),
    //         (67, BetDirection::Hot, 100, 90),
    //         (68, BetDirection::Not, 10, 9),
    //         (69, BetDirection::Not, 10, 9),
    //         (70, BetDirection::Not, 50, 45),
    //         (71, BetDirection::Not, 100, 90),
    //         (72, BetDirection::Not, 10, 9),
    //         (73, BetDirection::Not, 10, 9),
    //         (74, BetDirection::Hot, 10, 9),
    //         (75, BetDirection::Not, 10, 9),
    //         (76, BetDirection::Hot, 50, 45),
    //         (77, BetDirection::Hot, 50, 45),
    //         (78, BetDirection::Not, 100, 90),
    //         (79, BetDirection::Not, 100, 90),
    //         (80, BetDirection::Hot, 50, 45),
    //     ];

    //     data_set
    //         .iter()
    //         .for_each(|(user_id, bet_direction, bet_amount, _)| {
    //             let result = post.place_hot_or_not_bet(
    //                 &Principal::self_authenticating(user_id.to_ne_bytes()),
    //                 &Principal::self_authenticating(user_id.to_ne_bytes()),
    //                 *bet_amount,
    //                 bet_direction,
    //                 &post_creation_time,
    //             );
    //             assert!(result.is_ok());
    //         });

    //     let score_tabulation_time = post_creation_time
    //         .checked_add(Duration::from_secs(60 * 5))
    //         .unwrap();

    //     post.tabulate_hot_or_not_outcome_for_slot(
    //         &get_mock_user_alice_canister_id(),
    //         &1,
    //         &mut token_balance,
    //         &score_tabulation_time,
    //     );

    //     assert_eq!(token_balance.utility_token_transaction_history.len(), 1);
    //     assert_eq!(token_balance.utility_token_balance, 390);

    //     let room_detail = post
    //         .hot_or_not_details
    //         .as_ref()
    //         .unwrap()
    //         .slot_history
    //         .get(&1)
    //         .unwrap()
    //         .room_details
    //         .get(&1)
    //         .unwrap();

    //     assert_eq!(room_detail.bet_outcome, RoomBetPossibleOutcomes::Draw);
    //     assert_eq!(room_detail.room_bets_total_pot, 3900);
    //     assert_eq!(room_detail.total_hot_bets, 40);
    //     assert_eq!(room_detail.total_not_bets, 40);

    //     data_set
    //         .iter()
    //         .for_each(|(user_id, bet_direction, bet_amount, amount_won)| {
    //             let bet_detail = room_detail
    //                 .bets_made
    //                 .get(&Principal::self_authenticating(user_id.to_ne_bytes()))
    //                 .unwrap();

    //             assert_eq!(bet_detail.bet_direction, *bet_direction);
    //             assert_eq!(bet_detail.amount, *bet_amount);
    //             assert_eq!(
    //                 match bet_detail.payout {
    //                     BetPayout::Calculated(n) => {
    //                         n
    //                     }
    //                     _ => {
    //                         0
    //                     }
    //                 },
    //                 *amount_won
    //             );
    //         });
    // }

    pub fn setup_room_and_bet_details_map() -> (
        ic_stable_structures::btreemap::BTreeMap<GlobalRoomId, RoomDetailsV1, Memory>,
        ic_stable_structures::btreemap::BTreeMap<GlobalBetId, BetDetails, Memory>,
        ic_stable_structures::btreemap::BTreeMap<(PostId, StablePrincipal), (), Memory>,
        ic_stable_structures::btreemap::BTreeMap<(PostId, SlotId), SlotDetailsV1, Memory>,
    ) {
        const ROOM_DETAILS_MEMORY: MemoryId = MemoryId::new(0);
        const BET_DETAILS_MEMORY: MemoryId = MemoryId::new(1);
        const POST_PRINCIPAL_MEMORY: MemoryId = MemoryId::new(3);
        const SLOT_DETAILS_MEMORY: MemoryId = MemoryId::new(4);

        thread_local! {
            // The memory manager is used for simulating multiple memories. Given a `MemoryId` it can
            // return a memory that can be used by stable structures.
            static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
                RefCell::new(MemoryManager::init_with_bucket_size(DefaultMemoryImpl::default(), 1));
        }

        pub fn get_room_details_memory() -> Memory {
            MEMORY_MANAGER.with(|m| m.borrow_mut().get(ROOM_DETAILS_MEMORY))
        }
        pub fn get_bet_details_memory() -> Memory {
            MEMORY_MANAGER.with(|m| m.borrow_mut().get(BET_DETAILS_MEMORY))
        }
        pub fn get_post_principal_memory() -> Memory {
            MEMORY_MANAGER.with(|m| m.borrow_mut().get(POST_PRINCIPAL_MEMORY))
        }
        pub fn get_slot_details_memory() -> Memory {
            MEMORY_MANAGER.with(|m| m.borrow_mut().get(SLOT_DETAILS_MEMORY))
        }
        fn _default_room_details(
        ) -> ic_stable_structures::btreemap::BTreeMap<GlobalRoomId, RoomDetailsV1, Memory> {
            ic_stable_structures::btreemap::BTreeMap::init(get_room_details_memory())
        }

        fn _default_bet_details(
        ) -> ic_stable_structures::btreemap::BTreeMap<GlobalBetId, BetDetails, Memory> {
            ic_stable_structures::btreemap::BTreeMap::init(get_bet_details_memory())
        }

        fn _default_post_principal_map(
        ) -> ic_stable_structures::btreemap::BTreeMap<(PostId, StablePrincipal), (), Memory>
        {
            ic_stable_structures::btreemap::BTreeMap::init(get_post_principal_memory())
        }
        fn _default_slot_details_map(
        ) -> ic_stable_structures::btreemap::BTreeMap<(PostId, SlotId), SlotDetailsV1, Memory>
        {
            ic_stable_structures::btreemap::BTreeMap::init(get_slot_details_memory())
        }

        let room_details_map = _default_room_details();
        let bet_details_map = _default_bet_details();
        let post_principal_map = _default_post_principal_map();
        let slot_details_map = _default_slot_details_map();

        (
            room_details_map,
            bet_details_map,
            post_principal_map,
            slot_details_map,
        )
    }

    #[test]
    fn test_get_hot_or_not_betting_status_for_this_post_v1() {
        // A memory for the StableBTreeMap we're using. A new memory should be created for
        // every additional stable structure.

        let (
            mut room_details_map,
            mut bet_details_map,
            mut post_principal_map,
            mut slot_details_map,
        ) = setup_room_and_bet_details_map();
        let mut post = Post::new(
            0,
            &PostDetailsFromFrontend {
                is_nsfw: false,
                description: "Doggos and puppers".into(),
                hashtags: vec!["doggo".into(), "pupper".into()],
                video_uid: "abcd#1234".into(),
                creator_consent_for_inclusion_in_hot_or_not: true,
            },
            &SystemTime::now(),
        );

        let result = post.get_hot_or_not_betting_status_for_this_post_v1(
            &SystemTime::now()
                .checked_add(Duration::from_secs(
                    TOTAL_DURATION_OF_ALL_SLOTS_IN_SECONDS + 1,
                ))
                .unwrap(),
            &Principal::anonymous(),
            &room_details_map,
            &post_principal_map,
            &slot_details_map,
        );

        assert_eq!(result, BettingStatus::BettingClosed);
        let current_time = SystemTime::now();

        let result = post.get_hot_or_not_betting_status_for_this_post_v1(
            &current_time,
            &Principal::anonymous(),
            &room_details_map,
            &post_principal_map,
            &slot_details_map,
        );

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

        let result = post.get_hot_or_not_betting_status_for_this_post_v1(
            &current_time
                .checked_add(Duration::from_secs(
                    DURATION_OF_EACH_SLOT_IN_SECONDS * 2 + 1,
                ))
                .unwrap(),
            &Principal::anonymous(),
            &room_details_map,
            &post_principal_map,
            &slot_details_map,
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

        let result = post.place_hot_or_not_bet_v1(
            &get_mock_user_alice_principal_id(),
            &get_mock_user_alice_canister_id(),
            100,
            &BetDirection::Hot,
            &current_time
                .checked_add(Duration::from_secs(
                    DURATION_OF_EACH_SLOT_IN_SECONDS * 2 + 1,
                ))
                .unwrap(),
            &mut room_details_map,
            &mut bet_details_map,
            &mut post_principal_map,
            &mut slot_details_map,
        );

        assert!(result.is_ok());

        let result = post.get_hot_or_not_betting_status_for_this_post_v1(
            &current_time
                .checked_add(Duration::from_secs(
                    DURATION_OF_EACH_SLOT_IN_SECONDS * 2 + 1,
                ))
                .unwrap(),
            &get_mock_user_alice_principal_id(),
            &room_details_map,
            &post_principal_map,
            &slot_details_map,
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
            let result = post.place_hot_or_not_bet_v1(
                &Principal::from_slice(&[num]),
                &Principal::from_slice(&[num]),
                100,
                &BetDirection::Hot,
                &current_time
                    .checked_add(Duration::from_secs(
                        DURATION_OF_EACH_SLOT_IN_SECONDS * 2 + 1,
                    ))
                    .unwrap(),
                &mut room_details_map,
                &mut bet_details_map,
                &mut post_principal_map,
                &mut slot_details_map,
            );

            if result.is_err() {
                println!("Error: {:?}", result);
            }
            assert!(result.is_ok());
        });

        let result = post.place_hot_or_not_bet_v1(
            &Principal::from_slice(&[200]),
            &Principal::from_slice(&[200]),
            100,
            &BetDirection::Hot,
            &current_time
                .checked_add(Duration::from_secs(
                    DURATION_OF_EACH_SLOT_IN_SECONDS * 2 + 1,
                ))
                .unwrap(),
            &mut room_details_map,
            &mut bet_details_map,
            &mut post_principal_map,
            &mut slot_details_map,
        );

        assert!(result.is_ok());

        let result = post.get_hot_or_not_betting_status_for_this_post_v1(
            &current_time
                .checked_add(Duration::from_secs(
                    DURATION_OF_EACH_SLOT_IN_SECONDS * 2 + 1,
                ))
                .unwrap(),
            &Principal::from_slice(&[100]),
            &room_details_map,
            &post_principal_map,
            &slot_details_map,
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

        let result = post.place_hot_or_not_bet_v1(
            &get_mock_user_alice_principal_id(),
            &get_mock_user_alice_canister_id(),
            100,
            &BetDirection::Hot,
            &current_time
                .checked_add(Duration::from_secs(
                    DURATION_OF_EACH_SLOT_IN_SECONDS * 2 + 1,
                ))
                .unwrap(),
            &mut room_details_map,
            &mut bet_details_map,
            &mut post_principal_map,
            &mut slot_details_map,
        );

        assert!(result.is_err());

        let result = post.get_hot_or_not_betting_status_for_this_post_v1(
            &current_time
                .checked_add(Duration::from_secs(
                    DURATION_OF_EACH_SLOT_IN_SECONDS * 2 + 1,
                ))
                .unwrap(),
            &get_mock_user_alice_principal_id(),
            &room_details_map,
            &post_principal_map,
            &slot_details_map,
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

        let result = post.place_hot_or_not_bet_v1(
            &get_mock_user_alice_principal_id(),
            &get_mock_user_alice_canister_id(),
            100,
            &BetDirection::Hot,
            &current_time
                .checked_add(Duration::from_secs(
                    DURATION_OF_EACH_SLOT_IN_SECONDS * 4 + 1,
                ))
                .unwrap(),
            &mut room_details_map,
            &mut bet_details_map,
            &mut post_principal_map,
            &mut slot_details_map,
        );

        assert!(result.is_err());

        let result = post.get_hot_or_not_betting_status_for_this_post_v1(
            &current_time
                .checked_add(Duration::from_secs(
                    DURATION_OF_EACH_SLOT_IN_SECONDS * 4 + 1,
                ))
                .unwrap(),
            &get_mock_user_alice_principal_id(),
            &room_details_map,
            &post_principal_map,
            &slot_details_map,
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
    fn test_has_this_principal_already_bet_on_this_post_v1() {
        let (
            mut room_details_map,
            mut bet_details_map,
            mut post_principal_map,
            mut slot_details_map,
        ) = setup_room_and_bet_details_map();

        let mut post = Post::new(
            0,
            &PostDetailsFromFrontend {
                is_nsfw: false,
                description: "Doggos and puppers".into(),
                hashtags: vec!["doggo".into(), "pupper".into()],
                video_uid: "abcd#1234".into(),
                creator_consent_for_inclusion_in_hot_or_not: true,
            },
            &SystemTime::now(),
        );

        let result = post.has_this_principal_already_bet_on_this_post_v1(
            &get_mock_user_alice_principal_id(),
            &post_principal_map,
        );

        assert!(!result);

        post.place_hot_or_not_bet_v1(
            &get_mock_user_alice_principal_id(),
            &get_mock_user_alice_canister_id(),
            100,
            &BetDirection::Hot,
            &SystemTime::now(),
            &mut room_details_map,
            &mut bet_details_map,
            &mut post_principal_map,
            &mut slot_details_map,
        )
        .ok();

        let result = post.has_this_principal_already_bet_on_this_post_v1(
            &get_mock_user_alice_principal_id(),
            &post_principal_map,
        );
        assert!(result);
    }

    #[test]
    fn test_place_hot_or_not_bet_v1() {
        let (
            mut room_details_map,
            mut bet_details_map,
            mut post_principal_map,
            mut slot_details_map,
        ) = setup_room_and_bet_details_map();

        let mut post = Post::new(
            0,
            &PostDetailsFromFrontend {
                is_nsfw: false,
                description: "Doggos and puppers".into(),
                hashtags: vec!["doggo".into(), "pupper".into()],
                video_uid: "abcd#1234".into(),
                creator_consent_for_inclusion_in_hot_or_not: true,
            },
            &SystemTime::now(),
        );

        assert!(post.hot_or_not_details.is_some());

        let result = post.place_hot_or_not_bet_v1(
            &get_mock_user_alice_principal_id(),
            &get_mock_user_alice_canister_id(),
            100,
            &BetDirection::Hot,
            &SystemTime::now()
                .checked_add(Duration::from_secs(
                    TOTAL_DURATION_OF_ALL_SLOTS_IN_SECONDS + 1,
                ))
                .unwrap(),
            &mut room_details_map,
            &mut bet_details_map,
            &mut post_principal_map,
            &mut slot_details_map,
        );

        assert_eq!(result, Err(BetOnCurrentlyViewingPostError::BettingClosed));

        let result = post.place_hot_or_not_bet_v1(
            &get_mock_user_alice_principal_id(),
            &get_mock_user_alice_canister_id(),
            100,
            &BetDirection::Hot,
            &SystemTime::now(),
            &mut room_details_map,
            &mut bet_details_map,
            &mut post_principal_map,
            &mut slot_details_map,
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

        let start_global_room_id = GlobalRoomId(0, 1, 1);
        let end_global_room_id = GlobalRoomId(0, 2, 1);
        let start_global_bet_id = GlobalBetId(
            start_global_room_id,
            StablePrincipal(Principal::from_slice(&[1])),
        );
        let end_global_bet_id = GlobalBetId(
            end_global_room_id,
            StablePrincipal(Principal::from_slice(&[1])),
        );
        let rooms = room_details_map
            .range(start_global_room_id..end_global_room_id)
            .collect::<Vec<_>>();
        assert_eq!(rooms.len(), 1);
        let room_details = &rooms[0].1;

        let bets = bet_details_map
            .range(start_global_bet_id..end_global_bet_id)
            .collect::<Vec<_>>();
        assert_eq!(bets.len(), 1);
        let bet_details = &bets[0].1;
        assert_eq!(bet_details.amount, 100);
        assert_eq!(bet_details.bet_direction, BetDirection::Hot);

        assert_eq!(room_details.room_bets_total_pot, 100);
        assert_eq!(room_details.total_hot_bets, 1);
        assert_eq!(room_details.total_not_bets, 0);
        assert_eq!(hot_or_not_details.aggregate_stats.total_amount_bet, 100);
        assert_eq!(
            hot_or_not_details.aggregate_stats.total_number_of_hot_bets,
            1
        );
        assert_eq!(
            hot_or_not_details.aggregate_stats.total_number_of_not_bets,
            0
        );

        let result = post.place_hot_or_not_bet_v1(
            &get_mock_user_alice_principal_id(),
            &get_mock_user_alice_canister_id(),
            100,
            &BetDirection::Hot,
            &SystemTime::now(),
            &mut room_details_map,
            &mut bet_details_map,
            &mut post_principal_map,
            &mut slot_details_map,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_tabulate_hot_or_not_outcome_for_slot_case_1_v1() {
        let (
            mut room_details_map,
            mut bet_details_map,
            mut post_principal_map,
            mut slot_details_map,
        ) = setup_room_and_bet_details_map();

        let post_creation_time = SystemTime::now();
        let mut post = Post::new(
            0,
            &PostDetailsFromFrontend {
                is_nsfw: false,
                description: "Doggos and puppers".into(),
                hashtags: vec!["doggo".into(), "pupper".into()],
                video_uid: "abcd#1234".into(),
                creator_consent_for_inclusion_in_hot_or_not: true,
            },
            &post_creation_time,
        );
        let mut token_balance = TokenBalance::default();
        let tabulation_canister_id = get_mock_user_alice_canister_id();

        assert!(post.hot_or_not_details.is_some());

        let data_set: Vec<(u64, BetDirection, u64, u64)> = vec![
            (1, BetDirection::Not, 10, 18),
            (2, BetDirection::Hot, 100, 0),
            (3, BetDirection::Hot, 100, 0),
            (4, BetDirection::Not, 100, 180),
            (5, BetDirection::Hot, 10, 0),
            (6, BetDirection::Not, 100, 180),
            (7, BetDirection::Not, 50, 90),
            (8, BetDirection::Not, 100, 180),
            (9, BetDirection::Hot, 50, 0),
            (10, BetDirection::Not, 50, 90),
            (11, BetDirection::Not, 100, 180),
            (12, BetDirection::Not, 10, 18),
            (13, BetDirection::Hot, 100, 0),
            (14, BetDirection::Not, 10, 18),
            (15, BetDirection::Hot, 50, 0),
            (16, BetDirection::Hot, 10, 0),
            (17, BetDirection::Hot, 10, 0),
            (18, BetDirection::Hot, 100, 0),
            (19, BetDirection::Not, 10, 18),
            (20, BetDirection::Hot, 50, 0),
            (21, BetDirection::Hot, 10, 0),
            (22, BetDirection::Not, 50, 90),
            (23, BetDirection::Not, 50, 90),
            (24, BetDirection::Hot, 100, 0),
            (25, BetDirection::Not, 50, 90),
            (26, BetDirection::Not, 10, 18),
            (27, BetDirection::Not, 10, 18),
            (28, BetDirection::Not, 50, 90),
            (29, BetDirection::Hot, 50, 0),
            (30, BetDirection::Not, 100, 180),
            (31, BetDirection::Not, 50, 90),
            (32, BetDirection::Not, 50, 90),
            (33, BetDirection::Hot, 100, 0),
            (34, BetDirection::Not, 10, 18),
            (35, BetDirection::Not, 10, 18),
            (36, BetDirection::Not, 100, 180),
            (37, BetDirection::Hot, 10, 0),
            (38, BetDirection::Not, 100, 180),
            (39, BetDirection::Not, 50, 90),
            (40, BetDirection::Hot, 100, 0),
            (41, BetDirection::Hot, 50, 0),
            (42, BetDirection::Not, 10, 18),
            (43, BetDirection::Hot, 50, 0),
            (44, BetDirection::Not, 10, 18),
            (45, BetDirection::Not, 10, 18),
            (46, BetDirection::Hot, 100, 0),
            (47, BetDirection::Hot, 50, 0),
            (48, BetDirection::Hot, 50, 0),
            (49, BetDirection::Not, 100, 180),
            (50, BetDirection::Hot, 10, 0),
            (51, BetDirection::Not, 50, 90),
            (52, BetDirection::Hot, 10, 0),
            (53, BetDirection::Not, 50, 90),
            (54, BetDirection::Not, 10, 18),
            (55, BetDirection::Hot, 100, 0),
            (56, BetDirection::Hot, 50, 0),
            (57, BetDirection::Not, 50, 90),
            (58, BetDirection::Not, 10, 18),
            (59, BetDirection::Not, 50, 90),
            (60, BetDirection::Hot, 10, 0),
            (61, BetDirection::Not, 10, 18),
            (62, BetDirection::Not, 50, 90),
            (63, BetDirection::Not, 50, 90),
            (64, BetDirection::Not, 10, 18),
            (65, BetDirection::Not, 10, 18),
            (66, BetDirection::Not, 100, 180),
            (67, BetDirection::Hot, 100, 0),
            (68, BetDirection::Not, 10, 18),
            (69, BetDirection::Not, 10, 18),
            (70, BetDirection::Not, 50, 90),
            (71, BetDirection::Not, 100, 180),
            (72, BetDirection::Not, 10, 18),
            (73, BetDirection::Not, 10, 18),
            (74, BetDirection::Hot, 10, 0),
            (75, BetDirection::Not, 10, 18),
        ];

        data_set
            .iter()
            .for_each(|(user_id, bet_direction, bet_amount, _)| {
                let result = post.place_hot_or_not_bet_v1(
                    &Principal::self_authenticating(user_id.to_ne_bytes()),
                    &Principal::self_authenticating(user_id.to_ne_bytes()),
                    *bet_amount,
                    bet_direction,
                    &post_creation_time,
                    &mut room_details_map,
                    &mut bet_details_map,
                    &mut post_principal_map,
                    &mut slot_details_map,
                );
                assert!(result.is_ok());
            });

        let score_tabulation_time = post_creation_time
            .checked_add(Duration::from_secs(60 * 5))
            .unwrap();

        post.tabulate_hot_or_not_outcome_for_slot_v1(
            &tabulation_canister_id,
            &1,
            &mut token_balance,
            &score_tabulation_time,
            &mut room_details_map,
            &mut bet_details_map,
        );

        assert_eq!(token_balance.utility_token_transaction_history.len(), 1);
        assert_eq!(token_balance.utility_token_balance, 355);

        let global_room_id = GlobalRoomId(0, 1, 1);
        let room_detail = room_details_map.get(&global_room_id).unwrap();

        assert_eq!(room_detail.bet_outcome, RoomBetPossibleOutcomes::NotWon);
        assert_eq!(room_detail.room_bets_total_pot, 3550);
        assert_eq!(room_detail.total_hot_bets, 28);
        assert_eq!(room_detail.total_not_bets, 47);

        data_set
            .iter()
            .for_each(|(user_id, bet_direction, bet_amount, amount_won)| {
                let global_bet_id = GlobalBetId(
                    global_room_id,
                    StablePrincipal(Principal::self_authenticating(user_id.to_ne_bytes())),
                );

                let bet_detail = bet_details_map.get(&global_bet_id).unwrap();

                assert_eq!(bet_detail.bet_direction, *bet_direction);
                assert_eq!(bet_detail.amount, *bet_amount);
                assert_eq!(
                    match bet_detail.payout {
                        BetPayout::Calculated(n) => {
                            n
                        }
                        _ => {
                            0
                        }
                    },
                    *amount_won
                );
            });

        let data_set: Vec<(u64, BetDirection, u64, u64)> = vec![
            (1, BetDirection::Hot, 10, 18),
            (2, BetDirection::Hot, 50, 90),
            (3, BetDirection::Hot, 10, 18),
            (4, BetDirection::Not, 100, 0),
            (5, BetDirection::Hot, 100, 180),
            (6, BetDirection::Not, 100, 0),
            (7, BetDirection::Hot, 50, 90),
            (8, BetDirection::Hot, 100, 180),
            (9, BetDirection::Hot, 100, 180),
            (10, BetDirection::Not, 50, 0),
            (11, BetDirection::Not, 50, 0),
            (12, BetDirection::Hot, 50, 90),
            (13, BetDirection::Hot, 100, 180),
            (14, BetDirection::Hot, 100, 180),
            (15, BetDirection::Not, 50, 0),
            (16, BetDirection::Not, 50, 0),
            (17, BetDirection::Not, 100, 0),
            (18, BetDirection::Not, 100, 0),
            (19, BetDirection::Hot, 100, 180),
            (20, BetDirection::Not, 10, 0),
            (21, BetDirection::Hot, 100, 180),
            (22, BetDirection::Hot, 10, 18),
            (23, BetDirection::Hot, 10, 18),
            (24, BetDirection::Hot, 50, 90),
            (25, BetDirection::Not, 100, 0),
            (26, BetDirection::Hot, 10, 18),
            (27, BetDirection::Hot, 100, 180),
            (28, BetDirection::Hot, 50, 90),
            (29, BetDirection::Hot, 50, 90),
            (30, BetDirection::Hot, 10, 18),
            (31, BetDirection::Hot, 10, 18),
            (32, BetDirection::Hot, 100, 180),
            (33, BetDirection::Not, 100, 0),
            (34, BetDirection::Hot, 50, 90),
            (35, BetDirection::Hot, 100, 180),
            (36, BetDirection::Hot, 100, 180),
            (37, BetDirection::Hot, 50, 90),
            (38, BetDirection::Not, 10, 0),
            (39, BetDirection::Hot, 50, 90),
            (40, BetDirection::Not, 10, 0),
            (41, BetDirection::Hot, 50, 90),
            (42, BetDirection::Not, 10, 0),
            (43, BetDirection::Not, 100, 0),
            (44, BetDirection::Not, 100, 0),
            (45, BetDirection::Not, 100, 0),
            (46, BetDirection::Hot, 100, 180),
            (47, BetDirection::Not, 50, 0),
            (48, BetDirection::Hot, 100, 180),
            (49, BetDirection::Not, 100, 0),
            (50, BetDirection::Not, 50, 0),
            (51, BetDirection::Not, 10, 0),
            (52, BetDirection::Not, 100, 0),
            (53, BetDirection::Hot, 100, 180),
            (54, BetDirection::Hot, 10, 18),
            (55, BetDirection::Not, 100, 0),
            (56, BetDirection::Not, 100, 0),
            (57, BetDirection::Hot, 50, 90),
            (58, BetDirection::Not, 100, 0),
            (59, BetDirection::Not, 10, 0),
            (60, BetDirection::Hot, 10, 18),
            (61, BetDirection::Not, 10, 0),
            (62, BetDirection::Hot, 50, 90),
            (63, BetDirection::Hot, 10, 18),
            (64, BetDirection::Hot, 50, 90),
            (65, BetDirection::Not, 100, 0),
            (66, BetDirection::Not, 50, 0),
            (67, BetDirection::Not, 100, 0),
            (68, BetDirection::Hot, 10, 18),
            (69, BetDirection::Hot, 50, 90),
            (70, BetDirection::Not, 100, 0),
            (71, BetDirection::Hot, 50, 90),
            (72, BetDirection::Hot, 50, 90),
            (73, BetDirection::Not, 50, 0),
            (74, BetDirection::Not, 50, 0),
            (75, BetDirection::Not, 50, 0),
        ];

        // * 1 min into the 2nd hour/2nd slot
        let post_creation_time = post_creation_time
            .checked_add(Duration::from_secs(60 * (60 + 1)))
            .unwrap();

        data_set
            .iter()
            .for_each(|(user_id, bet_direction, bet_amount, _)| {
                let result = post.place_hot_or_not_bet_v1(
                    &Principal::self_authenticating((user_id + 75).to_ne_bytes()),
                    &Principal::self_authenticating((user_id + 75).to_ne_bytes()),
                    *bet_amount,
                    bet_direction,
                    &post_creation_time,
                    &mut room_details_map,
                    &mut bet_details_map,
                    &mut post_principal_map,
                    &mut slot_details_map,
                );
                assert!(result.is_ok());
            });

        let score_tabulation_time = post_creation_time
            .checked_add(Duration::from_secs(60 * 5))
            .unwrap();

        post.tabulate_hot_or_not_outcome_for_slot_v1(
            &get_mock_user_alice_canister_id(),
            &2,
            &mut token_balance,
            &score_tabulation_time,
            &mut room_details_map,
            &mut bet_details_map,
        );

        assert_eq!(token_balance.utility_token_transaction_history.len(), 2);
        assert_eq!(token_balance.utility_token_balance, 355 + 458);

        let global_room_id = GlobalRoomId(0, 2, 1);
        let room_detail = room_details_map.get(&global_room_id).unwrap();

        assert_eq!(room_detail.bet_outcome, RoomBetPossibleOutcomes::HotWon);
        assert_eq!(room_detail.room_bets_total_pot, 4580);
        assert_eq!(room_detail.total_hot_bets, 41);
        assert_eq!(room_detail.total_not_bets, 34);

        data_set
            .iter()
            .for_each(|(user_id, bet_direction, bet_amount, amount_won)| {
                let global_bet_id = GlobalBetId(
                    global_room_id,
                    StablePrincipal(Principal::self_authenticating((user_id + 75).to_ne_bytes())),
                );

                let bet_detail = bet_details_map.get(&global_bet_id).unwrap();

                assert_eq!(bet_detail.bet_direction, *bet_direction);
                assert_eq!(bet_detail.amount, *bet_amount);
                assert_eq!(
                    match bet_detail.payout {
                        BetPayout::Calculated(n) => {
                            n
                        }
                        _ => {
                            0
                        }
                    },
                    *amount_won
                );
            });
    }

    #[test]
    fn test_tabulate_hot_or_not_outcome_for_slot_case_2_v1() {
        let (
            mut room_details_map,
            mut bet_details_map,
            mut post_principal_map,
            mut slot_details_map,
        ) = setup_room_and_bet_details_map();

        let post_creation_time = SystemTime::now();
        let mut post = Post::new(
            0,
            &PostDetailsFromFrontend {
                is_nsfw: false,
                description: "Doggos and puppers".into(),
                hashtags: vec!["doggo".into(), "pupper".into()],
                video_uid: "abcd#1234".into(),
                creator_consent_for_inclusion_in_hot_or_not: true,
            },
            &post_creation_time,
        );
        let mut token_balance = TokenBalance::default();

        assert!(post.hot_or_not_details.is_some());

        let data_set: Vec<(u64, BetDirection, u64, u64)> = vec![
            (1, BetDirection::Not, 10, 18),
            (2, BetDirection::Hot, 100, 0),
            (3, BetDirection::Hot, 100, 0),
            (4, BetDirection::Not, 100, 180),
            (5, BetDirection::Hot, 10, 0),
            (6, BetDirection::Not, 100, 180),
            (7, BetDirection::Not, 50, 90),
            (8, BetDirection::Not, 100, 180),
            (9, BetDirection::Hot, 50, 0),
            (10, BetDirection::Not, 50, 90),
            (11, BetDirection::Not, 100, 180),
            (12, BetDirection::Not, 10, 18),
            (13, BetDirection::Hot, 100, 0),
            (14, BetDirection::Not, 10, 18),
            (15, BetDirection::Hot, 50, 0),
            (16, BetDirection::Hot, 10, 0),
            (17, BetDirection::Hot, 10, 0),
            (18, BetDirection::Hot, 100, 0),
            (19, BetDirection::Not, 10, 18),
            (20, BetDirection::Hot, 50, 0),
            (21, BetDirection::Hot, 10, 0),
            (22, BetDirection::Not, 50, 90),
            (23, BetDirection::Not, 50, 90),
            (24, BetDirection::Hot, 100, 0),
            (25, BetDirection::Not, 50, 90),
            (26, BetDirection::Not, 10, 18),
            (27, BetDirection::Not, 10, 18),
            (28, BetDirection::Not, 50, 90),
            (29, BetDirection::Hot, 50, 0),
            (30, BetDirection::Not, 100, 180),
            (31, BetDirection::Not, 50, 90),
            (32, BetDirection::Not, 50, 90),
            (33, BetDirection::Hot, 100, 0),
            (34, BetDirection::Not, 10, 18),
            (35, BetDirection::Not, 10, 18),
            (36, BetDirection::Not, 100, 180),
            (37, BetDirection::Hot, 10, 0),
            (38, BetDirection::Not, 100, 180),
            (39, BetDirection::Not, 50, 90),
            (40, BetDirection::Hot, 100, 0),
            (41, BetDirection::Hot, 50, 0),
            (42, BetDirection::Not, 10, 18),
            (43, BetDirection::Hot, 50, 0),
            (44, BetDirection::Not, 10, 18),
            (45, BetDirection::Not, 10, 18),
            (46, BetDirection::Hot, 100, 0),
            (47, BetDirection::Hot, 50, 0),
            (48, BetDirection::Hot, 50, 0),
            (49, BetDirection::Not, 100, 180),
            (50, BetDirection::Hot, 10, 0),
            (51, BetDirection::Not, 50, 90),
            (52, BetDirection::Hot, 10, 0),
            (53, BetDirection::Not, 50, 90),
            (54, BetDirection::Not, 10, 18),
            (55, BetDirection::Hot, 100, 0),
            (56, BetDirection::Hot, 50, 0),
            (57, BetDirection::Not, 50, 90),
            (58, BetDirection::Not, 10, 18),
            (59, BetDirection::Not, 50, 90),
            (60, BetDirection::Hot, 10, 0),
            (61, BetDirection::Not, 10, 18),
            (62, BetDirection::Not, 50, 90),
            (63, BetDirection::Not, 50, 90),
            (64, BetDirection::Not, 10, 18),
            (65, BetDirection::Not, 10, 18),
            (66, BetDirection::Not, 100, 180),
            (67, BetDirection::Hot, 100, 0),
            (68, BetDirection::Not, 10, 18),
            (69, BetDirection::Not, 10, 18),
            (70, BetDirection::Not, 50, 90),
            (71, BetDirection::Not, 100, 180),
            (72, BetDirection::Not, 10, 18),
            (73, BetDirection::Not, 10, 18),
            (74, BetDirection::Hot, 10, 0),
            (75, BetDirection::Not, 10, 18),
            (76, BetDirection::Hot, 50, 0),
            (77, BetDirection::Hot, 50, 0),
            (78, BetDirection::Not, 100, 180),
            (79, BetDirection::Not, 100, 180),
            (80, BetDirection::Hot, 50, 0),
            (81, BetDirection::Hot, 10, 0),
            (82, BetDirection::Hot, 50, 0),
            (83, BetDirection::Not, 10, 18),
            (84, BetDirection::Not, 50, 90),
            (85, BetDirection::Not, 10, 18),
            (86, BetDirection::Not, 10, 18),
            (87, BetDirection::Hot, 100, 0),
            (88, BetDirection::Not, 10, 18),
            (89, BetDirection::Not, 50, 90),
            (90, BetDirection::Hot, 100, 0),
            (91, BetDirection::Hot, 100, 0),
            (92, BetDirection::Hot, 10, 0),
            (93, BetDirection::Hot, 10, 0),
            (94, BetDirection::Hot, 100, 0),
            (95, BetDirection::Hot, 50, 0),
            (96, BetDirection::Hot, 100, 0),
            (97, BetDirection::Hot, 50, 0),
            (98, BetDirection::Hot, 50, 0),
            (99, BetDirection::Hot, 50, 0),
            (100, BetDirection::Hot, 50, 0),
            (101, BetDirection::Not, 10, 18),
            (102, BetDirection::Not, 50, 90),
            (103, BetDirection::Not, 10, 18),
            (104, BetDirection::Hot, 100, 0),
            (105, BetDirection::Not, 100, 180),
            (106, BetDirection::Hot, 100, 0),
            (107, BetDirection::Not, 50, 90),
            (108, BetDirection::Not, 100, 180),
            (109, BetDirection::Not, 100, 180),
            (110, BetDirection::Hot, 50, 0),
            (111, BetDirection::Hot, 50, 0),
            (112, BetDirection::Not, 50, 90),
            (113, BetDirection::Not, 100, 180),
            (114, BetDirection::Not, 100, 180),
            (115, BetDirection::Hot, 50, 0),
            (116, BetDirection::Hot, 50, 0),
            (117, BetDirection::Hot, 100, 0),
            (118, BetDirection::Hot, 100, 0),
            (119, BetDirection::Not, 100, 180),
            (120, BetDirection::Hot, 10, 0),
            (121, BetDirection::Not, 100, 180),
            (122, BetDirection::Not, 10, 18),
            (123, BetDirection::Not, 10, 18),
            (124, BetDirection::Not, 50, 90),
            (125, BetDirection::Hot, 100, 0),
            (126, BetDirection::Not, 10, 18),
            (127, BetDirection::Not, 100, 180),
            (128, BetDirection::Not, 50, 90),
            (129, BetDirection::Not, 50, 90),
            (130, BetDirection::Not, 10, 18),
            (131, BetDirection::Not, 10, 18),
            (132, BetDirection::Not, 100, 180),
            (133, BetDirection::Hot, 100, 0),
            (134, BetDirection::Not, 50, 90),
            (135, BetDirection::Not, 100, 180),
            (136, BetDirection::Not, 100, 180),
            (137, BetDirection::Not, 50, 90),
            (138, BetDirection::Hot, 10, 0),
            (139, BetDirection::Not, 50, 90),
            (140, BetDirection::Hot, 10, 0),
            (141, BetDirection::Not, 50, 90),
            (142, BetDirection::Hot, 10, 0),
            (143, BetDirection::Hot, 100, 0),
            (144, BetDirection::Hot, 100, 0),
            (145, BetDirection::Hot, 100, 0),
            (146, BetDirection::Not, 100, 180),
            (147, BetDirection::Hot, 50, 0),
            (148, BetDirection::Not, 100, 180),
            (149, BetDirection::Hot, 100, 0),
            (150, BetDirection::Hot, 50, 0),
        ];

        data_set
            .iter()
            .for_each(|(user_id, bet_direction, bet_amount, _)| {
                let result = post.place_hot_or_not_bet_v1(
                    &Principal::self_authenticating(user_id.to_ne_bytes()),
                    &Principal::self_authenticating(user_id.to_ne_bytes()),
                    *bet_amount,
                    bet_direction,
                    &post_creation_time,
                    &mut room_details_map,
                    &mut bet_details_map,
                    &mut post_principal_map,
                    &mut slot_details_map,
                );
                assert!(result.is_ok());
            });

        let score_tabulation_time = post_creation_time
            .checked_add(Duration::from_secs(60 * 5))
            .unwrap();

        post.tabulate_hot_or_not_outcome_for_slot_v1(
            &get_mock_user_alice_canister_id(),
            &1,
            &mut token_balance,
            &score_tabulation_time,
            &mut room_details_map,
            &mut bet_details_map,
        );

        assert_eq!(token_balance.utility_token_transaction_history.len(), 2);
        assert_eq!(token_balance.utility_token_balance, 487 + 321);

        // * Room 1
        let global_room_id = GlobalRoomId(0, 1, 1);
        let room_detail = room_details_map.get(&global_room_id).unwrap();

        assert_eq!(room_detail.bet_outcome, RoomBetPossibleOutcomes::NotWon);
        assert_eq!(room_detail.room_bets_total_pot, 4870);
        assert_eq!(room_detail.total_hot_bets, 45);
        assert_eq!(room_detail.total_not_bets, 55);

        data_set[0..100]
            .iter()
            .for_each(|(user_id, bet_direction, bet_amount, amount_won)| {
                let global_bet_id = GlobalBetId(
                    global_room_id,
                    StablePrincipal(Principal::self_authenticating(user_id.to_ne_bytes())),
                );
                let bet_detail = bet_details_map.get(&global_bet_id).unwrap();

                assert_eq!(bet_detail.bet_direction, *bet_direction);
                assert_eq!(bet_detail.amount, *bet_amount);
                assert_eq!(
                    match bet_detail.payout {
                        BetPayout::Calculated(n) => {
                            n
                        }
                        _ => {
                            0
                        }
                    },
                    *amount_won
                );
            });

        // * Room 2
        let global_room_id = GlobalRoomId(0, 1, 2);
        let room_detail = room_details_map.get(&global_room_id).unwrap();

        assert_eq!(room_detail.bet_outcome, RoomBetPossibleOutcomes::NotWon);
        assert_eq!(room_detail.room_bets_total_pot, 3210);
        assert_eq!(room_detail.total_hot_bets, 20);
        assert_eq!(room_detail.total_not_bets, 30);

        data_set[100..]
            .iter()
            .for_each(|(user_id, bet_direction, bet_amount, amount_won)| {
                let global_bet_id = GlobalBetId(
                    global_room_id,
                    StablePrincipal(Principal::self_authenticating(user_id.to_ne_bytes())),
                );
                let bet_detail = bet_details_map.get(&global_bet_id).unwrap();

                assert_eq!(bet_detail.bet_direction, *bet_direction);
                assert_eq!(bet_detail.amount, *bet_amount);
                assert_eq!(
                    match bet_detail.payout {
                        BetPayout::Calculated(n) => {
                            n
                        }
                        _ => {
                            0
                        }
                    },
                    *amount_won
                );
            });
    }

    #[test]
    fn test_tabulate_hot_or_not_outcome_for_slot_case_3_v1() {
        let (
            mut room_details_map,
            mut bet_details_map,
            mut post_principal_map,
            mut slot_details_map,
        ) = setup_room_and_bet_details_map();

        let post_creation_time = SystemTime::now();
        let mut post = Post::new(
            0,
            &PostDetailsFromFrontend {
                is_nsfw: false,
                description: "Doggos and puppers".into(),
                hashtags: vec!["doggo".into(), "pupper".into()],
                video_uid: "abcd#1234".into(),
                creator_consent_for_inclusion_in_hot_or_not: true,
            },
            &post_creation_time,
        );
        let mut token_balance = TokenBalance::default();

        assert!(post.hot_or_not_details.is_some());

        let data_set: Vec<(u64, BetDirection, u64, u64)> = vec![
            (1, BetDirection::Not, 10, 9),
            (2, BetDirection::Hot, 100, 90),
            (3, BetDirection::Hot, 100, 90),
            (4, BetDirection::Hot, 100, 90),
            (5, BetDirection::Hot, 10, 9),
            (6, BetDirection::Hot, 100, 90),
            (7, BetDirection::Hot, 50, 45),
            (8, BetDirection::Not, 100, 90),
            (9, BetDirection::Hot, 50, 45),
            (10, BetDirection::Not, 50, 45),
            (11, BetDirection::Not, 100, 90),
            (12, BetDirection::Hot, 10, 9),
            (13, BetDirection::Hot, 100, 90),
            (14, BetDirection::Not, 10, 9),
            (15, BetDirection::Hot, 50, 45),
            (16, BetDirection::Hot, 10, 9),
            (17, BetDirection::Hot, 10, 9),
            (18, BetDirection::Hot, 100, 90),
            (19, BetDirection::Not, 10, 9),
            (20, BetDirection::Hot, 50, 45),
            (21, BetDirection::Hot, 10, 9),
            (22, BetDirection::Hot, 50, 45),
            (23, BetDirection::Not, 50, 45),
            (24, BetDirection::Hot, 100, 90),
            (25, BetDirection::Not, 50, 45),
            (26, BetDirection::Not, 10, 9),
            (27, BetDirection::Hot, 10, 9),
            (28, BetDirection::Hot, 50, 45),
            (29, BetDirection::Hot, 50, 45),
            (30, BetDirection::Not, 100, 90),
            (31, BetDirection::Hot, 50, 45),
            (32, BetDirection::Not, 50, 45),
            (33, BetDirection::Hot, 100, 90),
            (34, BetDirection::Hot, 10, 9),
            (35, BetDirection::Not, 10, 9),
            (36, BetDirection::Not, 100, 90),
            (37, BetDirection::Hot, 10, 9),
            (38, BetDirection::Not, 100, 90),
            (39, BetDirection::Not, 50, 45),
            (40, BetDirection::Hot, 100, 90),
            (41, BetDirection::Hot, 50, 45),
            (42, BetDirection::Not, 10, 9),
            (43, BetDirection::Hot, 50, 45),
            (44, BetDirection::Not, 10, 9),
            (45, BetDirection::Not, 10, 9),
            (46, BetDirection::Hot, 100, 90),
            (47, BetDirection::Hot, 50, 45),
            (48, BetDirection::Hot, 50, 45),
            (49, BetDirection::Not, 100, 90),
            (50, BetDirection::Hot, 10, 9),
            (51, BetDirection::Not, 50, 45),
            (52, BetDirection::Hot, 10, 9),
            (53, BetDirection::Not, 50, 45),
            (54, BetDirection::Not, 10, 9),
            (55, BetDirection::Hot, 100, 90),
            (56, BetDirection::Hot, 50, 45),
            (57, BetDirection::Not, 50, 45),
            (58, BetDirection::Not, 10, 9),
            (59, BetDirection::Not, 50, 45),
            (60, BetDirection::Hot, 10, 9),
            (61, BetDirection::Not, 10, 9),
            (62, BetDirection::Not, 50, 45),
            (63, BetDirection::Not, 50, 45),
            (64, BetDirection::Not, 10, 9),
            (65, BetDirection::Not, 10, 9),
            (66, BetDirection::Not, 100, 90),
            (67, BetDirection::Hot, 100, 90),
            (68, BetDirection::Not, 10, 9),
            (69, BetDirection::Not, 10, 9),
            (70, BetDirection::Not, 50, 45),
            (71, BetDirection::Not, 100, 90),
            (72, BetDirection::Not, 10, 9),
            (73, BetDirection::Not, 10, 9),
            (74, BetDirection::Hot, 10, 9),
            (75, BetDirection::Not, 10, 9),
            (76, BetDirection::Hot, 50, 45),
            (77, BetDirection::Hot, 50, 45),
            (78, BetDirection::Not, 100, 90),
            (79, BetDirection::Not, 100, 90),
            (80, BetDirection::Hot, 50, 45),
        ];

        data_set
            .iter()
            .for_each(|(user_id, bet_direction, bet_amount, _)| {
                let result = post.place_hot_or_not_bet_v1(
                    &Principal::self_authenticating(user_id.to_ne_bytes()),
                    &Principal::self_authenticating(user_id.to_ne_bytes()),
                    *bet_amount,
                    bet_direction,
                    &post_creation_time,
                    &mut room_details_map,
                    &mut bet_details_map,
                    &mut post_principal_map,
                    &mut slot_details_map,
                );
                assert!(result.is_ok());
            });

        let score_tabulation_time = post_creation_time
            .checked_add(Duration::from_secs(60 * 5))
            .unwrap();

        post.tabulate_hot_or_not_outcome_for_slot_v1(
            &get_mock_user_alice_canister_id(),
            &1,
            &mut token_balance,
            &score_tabulation_time,
            &mut room_details_map,
            &mut bet_details_map,
        );

        assert_eq!(token_balance.utility_token_transaction_history.len(), 1);
        assert_eq!(token_balance.utility_token_balance, 390);

        let global_room_id = GlobalRoomId(0, 1, 1);
        let room_detail = room_details_map.get(&global_room_id).unwrap();

        assert_eq!(room_detail.bet_outcome, RoomBetPossibleOutcomes::Draw);
        assert_eq!(room_detail.room_bets_total_pot, 3900);
        assert_eq!(room_detail.total_hot_bets, 40);
        assert_eq!(room_detail.total_not_bets, 40);

        data_set
            .iter()
            .for_each(|(user_id, bet_direction, bet_amount, amount_won)| {
                let global_bet_id = GlobalBetId(
                    global_room_id,
                    StablePrincipal(Principal::self_authenticating(user_id.to_ne_bytes())),
                );
                let bet_detail = bet_details_map.get(&global_bet_id).unwrap();

                assert_eq!(bet_detail.bet_direction, *bet_direction);
                assert_eq!(bet_detail.amount, *bet_amount);
                assert_eq!(
                    match bet_detail.payout {
                        BetPayout::Calculated(n) => {
                            n
                        }
                        _ => {
                            0
                        }
                    },
                    *amount_won
                );
            });
    }
}
