use std::borrow::Cow;
use std::time::SystemTime;

use candid::CandidType;
use candid::Decode;
use candid::Deserialize;
use candid::Encode;
use candid::Principal;
use ic_stable_structures::storable::Bound;
use ic_stable_structures::Storable;
use serde::Serialize;

#[derive(Deserialize, Serialize, PartialEq, Clone, CandidType, Debug)]
pub struct WatchHistoryItem {
    pub post_id: u64,
    pub publisher_canister_id: Principal,
    pub viewed_at: SystemTime,
    pub cf_video_id: String,
    pub percentage_watched: f32,
}

impl PartialOrd for WatchHistoryItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for WatchHistoryItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.viewed_at.cmp(&other.viewed_at)
    }
}

impl Eq for WatchHistoryItem {}

impl Storable for WatchHistoryItem {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 200,
        is_fixed_size: false,
    };
}

#[deprecated(note = "use SuccessHistoryItemV1 instead")]
#[derive(Deserialize, Serialize, PartialEq, Clone, CandidType, Debug)]
pub struct SuccessHistoryItem {
    pub post_id: u64,
    pub publisher_canister_id: Principal,
    pub interacted_at: SystemTime,
    pub cf_video_id: String,
}

impl PartialOrd for SuccessHistoryItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SuccessHistoryItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.interacted_at.cmp(&other.interacted_at)
    }
}

impl Eq for SuccessHistoryItem {}

impl Storable for SuccessHistoryItem {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 200,
        is_fixed_size: false,
    };
}

#[derive(Deserialize, Serialize, PartialEq, Clone, CandidType, Debug)]
pub struct SuccessHistoryItemV1 {
    pub post_id: u64,
    pub publisher_canister_id: Principal,
    pub interacted_at: SystemTime,
    pub cf_video_id: String,
    pub item_type: String,
    pub percentage_watched: f32,
}

impl PartialOrd for SuccessHistoryItemV1 {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SuccessHistoryItemV1 {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.interacted_at.cmp(&other.interacted_at)
    }
}

impl Eq for SuccessHistoryItemV1 {}

impl Storable for SuccessHistoryItemV1 {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 200,
        is_fixed_size: false,
    };
}

#[derive(Deserialize, Serialize, Clone, CandidType, Debug)]
pub struct MLFeedCacheItem {
    pub post_id: u64,
    pub canister_id: Principal,
    pub video_id: String,
    pub creator_principal_id: Option<Principal>,
}
