use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::borrow::Cow;
use candid::{CandidType, Decode, Deserialize, Encode, Principal};
use ic_stable_structures::{storable::Bound,Storable};
use serde::Serialize;

use crate::canister_specific::individual_user_template::types::hot_or_not::{
    BetDirection, BetOutcomeForBetMaker,
};
use std::hash::{Hash, Hasher};

#[deprecated(note = "use TokenEventV1 instead")]
#[derive(Clone, CandidType, Deserialize, Debug, PartialEq, Eq, Serialize)]
pub enum TokenEvent {
    Mint {
        amount: u64,
        details: MintEvent,
        timestamp: SystemTime,
    },
    Burn,
    Transfer {
        amount: u64,
        to_account: Principal,
        timestamp: SystemTime,
    },
    Receive {
        amount: u64,
        from_account: Principal,
        timestamp: SystemTime,
    },
    Stake {
        amount: u64,
        details: StakeEvent,
        timestamp: SystemTime,
    },
    HotOrNotOutcomePayout {
        amount: u64,
        details: HotOrNotOutcomePayoutEvent,
        timestamp: SystemTime,
    },
}

impl TokenEvent {
    pub fn get_token_amount_for_token_event(&self) -> u64 {
        match self {
            TokenEvent::Mint { details, .. } => match details {
                MintEvent::NewUserSignup { .. } => 1000,
                MintEvent::Referral { .. } => 500,
            },
            _ => 0,
        }
    }
}


#[derive(Clone, CandidType, Deserialize, Debug, PartialEq, Eq, Serialize)]
pub enum TokenEventV1 {
    Mint {
        amount: u64,
        details: MintEvent,
        timestamp: SystemTime,
    },
    Burn,
    Transfer {
        amount: u64,
        to_account: Principal,
        timestamp: SystemTime,
    },
    Receive {
        amount: u64,
        from_account: Principal,
        timestamp: SystemTime,
    },
    Stake {
        amount: u64,
        details: StakeEvent,
        timestamp: SystemTime,
    },
    HotOrNotOutcomePayout {
        amount: u64,
        details: HotOrNotOutcomePayoutEventV1,
        timestamp: SystemTime,
    },
}

impl TokenEventV1 {
    pub fn get_token_amount_for_token_event(&self) -> u64 {
        match self {
            TokenEventV1::Mint { details, .. } => match details {
                MintEvent::NewUserSignup { .. } => 1000,
                MintEvent::Referral { .. } => 500,
            },
            _ => 0,
        }
    }
}

impl From<TokenEvent> for TokenEventV1 {
    fn from(event: TokenEvent) -> Self {
        match event {
            TokenEvent::Mint {
                amount,
                details,
                timestamp,
            } => TokenEventV1::Mint {
                amount,
                details,
                timestamp,
            },
            TokenEvent::Burn => TokenEventV1::Burn,
            TokenEvent::Transfer {
                amount,
                to_account,
                timestamp,
            } => TokenEventV1::Transfer {
                amount,
                to_account,
                timestamp,
            },
            TokenEvent::Receive {
                amount,
                from_account,
                timestamp,
            } => TokenEventV1::Receive {
                amount,
                from_account,
                timestamp,
            },
            TokenEvent::Stake {
                amount,
                details,
                timestamp,
            } => TokenEventV1::Stake {
                amount,
                details,
                timestamp,
            },
            TokenEvent::HotOrNotOutcomePayout {
                amount,
                details,
                timestamp,
            } => TokenEventV1::HotOrNotOutcomePayout {
                amount,
                details: details.into(), // Assuming HotOrNotOutcomePayoutEventV1 implements From<HotOrNotOutcomePayoutEvent>
                timestamp,
            },
        }
    }
}

#[derive(Clone, CandidType, Deserialize, Debug, PartialEq, Eq, Serialize)]
pub enum MintEvent {
    NewUserSignup {
        new_user_principal_id: Principal,
    },
    Referral {
        referee_user_principal_id: Principal,
        referrer_user_principal_id: Principal,
    },
}

#[derive(Clone, CandidType, Deserialize, Serialize, Debug, PartialEq, Eq)]
pub enum StakeEvent {
    BetOnHotOrNotPost {
        post_canister_id: Principal,
        post_id: u64,
        bet_amount: u64,
        bet_direction: BetDirection,
    },
}

#[deprecated(note = "use HotOrNotOutcomePayoutEventV1 instead")]
#[derive(Clone, CandidType, Deserialize, Serialize, Debug, PartialEq, Eq)]
pub enum HotOrNotOutcomePayoutEvent {
    CommissionFromHotOrNotBet {
        post_canister_id: Principal,
        post_id: u64,
        slot_id: u8,
        room_id: u64,
        room_pot_total_amount: u64,
    },
    WinningsEarnedFromBet {
        post_canister_id: Principal,
        post_id: u64,
        slot_id: u8,
        room_id: u64,
        event_outcome: BetOutcomeForBetMaker,
        winnings_amount: u64,
    },
}

#[derive(Clone, CandidType, Deserialize, Serialize, Debug, PartialEq, Eq)]
pub enum HotOrNotOutcomePayoutEventV1 {
    CommissionFromHotOrNotBet {
        post_canister_id: Principal,
        post_id: u64,
        slot_id: NewSlotType,
        room_id: u64,
        room_pot_total_amount: u64,
    },
    WinningsEarnedFromBet {
        post_canister_id: Principal,
        post_id: u64,
        slot_id: NewSlotType,
        room_id: u64,
        event_outcome: BetOutcomeForBetMaker,
        winnings_amount: u64,
    },
}

impl From<HotOrNotOutcomePayoutEvent> for HotOrNotOutcomePayoutEventV1 {
    fn from(event: HotOrNotOutcomePayoutEvent) -> Self {
        match event {
            HotOrNotOutcomePayoutEvent::CommissionFromHotOrNotBet {
                post_canister_id,
                post_id,
                slot_id,
                room_id,
                room_pot_total_amount,
            } => HotOrNotOutcomePayoutEventV1::CommissionFromHotOrNotBet {
                post_canister_id,
                post_id,
                slot_id: NewSlotType::from(slot_id), // Assuming NewSlotType implements From<u8>
                room_id,
                room_pot_total_amount,
            },
            HotOrNotOutcomePayoutEvent::WinningsEarnedFromBet {
                post_canister_id,
                post_id,
                slot_id,
                room_id,
                event_outcome,
                winnings_amount,
            } => HotOrNotOutcomePayoutEventV1::WinningsEarnedFromBet {
                post_canister_id,
                post_id,
                slot_id: NewSlotType::from(slot_id), // Assuming NewSlotType implements From<u8>
                room_id,
                event_outcome,
                winnings_amount,
            },
        }
    }
}


#[derive(
    Clone, Copy, Default, CandidType, Deserialize, Serialize, Debug, PartialEq, Eq, PartialOrd, Ord,
)]
pub struct NewSlotType(pub u64);

impl NewSlotType {
    pub fn increment_by(&self, by: u64) -> Self {
        NewSlotType(self.0 + by)
    }
}

impl From<u8> for NewSlotType {
    fn from(value: u8) -> Self {
        NewSlotType(value as u64)
    }
}

impl From<NewSlotType> for u8 {
    fn from(value: NewSlotType) -> Self {
        // this will not be accurate if the value is greater than u8
        // this is only for the backwards-compatibility reasons.
        // todo : remove this one week after upgrade
        value.0 as u8
    }
}

impl Storable for NewSlotType {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(&self).unwrap())
    }

    fn from_bytes(from_u64_bytes: Cow<[u8]>) -> Self {
        Decode!(&from_u64_bytes, Self).unwrap()
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 150,
        is_fixed_size: false,
    };
}

impl Hash for NewSlotType {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

/// `u64` = number of milliseconds that have elapsed since UNIX_EPOCH
/// # Note
///
/// The maximum value of `u64` allows this struct to represent dates up to the year 292277026596.
/// However, be cautious when performing operations that might exceed this range.
#[derive(
    Clone, Copy, CandidType, Deserialize, Serialize, Debug, PartialEq, Eq, PartialOrd, Ord,
)]
pub struct SystemTimeInMs(u64);

impl SystemTimeInMs {
    pub fn now() -> Self {
        let time = ic_cdk::api::time() / 1_000_000;
        SystemTimeInMs(time)
    }

    pub fn duration_since(&self, earlier: &SystemTimeInMs) -> Duration {
        if self.0 >= earlier.0 {
            Duration::from_millis(self.0 - earlier.0)
        } else {
            Duration::from_millis(0)
        }
    }

    pub fn from_system_time(system_time: SystemTime) -> Self {
        let duration = system_time
            .duration_since(UNIX_EPOCH)
            .expect("SystemTime before UNIX EPOCH!");

        let millis = duration.as_millis() as u64;
        SystemTimeInMs(millis)
    }

    /// canister_time is in nanoseconds
    pub fn from_canister_time(canister_time: u64) -> Self {
        SystemTimeInMs(canister_time / 1_000_000)
    }

    pub fn to_system_time(&self) -> Option<SystemTime> {
        let duration = Duration::from_millis(self.0);
        UNIX_EPOCH.checked_add(duration)
    }

    pub fn checked_add(&self, duration: Duration) -> Option<SystemTimeInMs> {
        let duration_ms = duration.as_millis() as u64;
        self.0.checked_add(duration_ms).map(SystemTimeInMs)
    }

    pub fn calculate_remaining_interval(
        &self,
        earlier_time: &SystemTimeInMs,
        future_duration: Duration,
    ) -> Result<Duration, &'static str> {
        let future_time = earlier_time
            .checked_add(future_duration)
            .ok_or("Overflow when calculating future time")?;

        if future_time.0 > self.0 {
            Ok(Duration::from_millis(future_time.0 - self.0))
        } else {
            // keeping default to be 3s instead of 0ms for concurrent bets on two posts of the same user
            // if the last best timer was just processed, wait for 3s to process another post.
            Ok(Duration::from_millis(3000))
        }
    }
}

impl Default for SystemTimeInMs {
    fn default() -> Self {
        SystemTimeInMs::now()
    }
}

impl Storable for SystemTimeInMs {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(&self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(&bytes, Self).unwrap()
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 100,
        is_fixed_size: false,
    };
}

pub const HOT_OR_NOT_BET_CREATOR_COMMISSION_PERCENTAGE: u64 = 10;
pub const HOT_OR_NOT_BET_WINNINGS_MULTIPLIER: u64 = 2;
