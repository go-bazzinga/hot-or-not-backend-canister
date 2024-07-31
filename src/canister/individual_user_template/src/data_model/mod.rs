use std::{
    collections::{BTreeMap, BTreeSet},
    time::SystemTime,
};

use candid::{Deserialize, Principal};
use ic_cdk::api::management_canister::provisional::CanisterId;
use memory::{get_bet_details_memory_v1, get_bet_timer_first_bet_at_memory, get_bet_timer_memory, get_room_details_memory_v1, get_slot_details_memory_v1,get_success_history_memory, get_watch_history_memory};
use serde::Serialize;
use shared_utils::{
    canister_specific::individual_user_template::types::{
        configuration::IndividualUserConfiguration,
        follow::FollowData,
        hot_or_not::{
            BetDetails, GlobalBetId, GlobalBetIdV1, GlobalRoomId, GlobalRoomIdV1, PlacedBetDetail, PlacedBetDetailV1, RoomDetailsV1, RoomId, SlotDetailsV1, SlotId, StablePrincipal
        },
        migration::MigrationInfo,
        ml_data::{SuccessHistoryItem, SuccessHistoryItemV1, WatchHistoryItem},
        post::{FeedScore, Post, PostViewStatistics},
        profile::UserProfile,
        session::SessionType,
        token::{TokenBalance, TokenBalanceV1},
    },
    common::types::{
        app_primitive_type::PostId, known_principal::KnownPrincipalMap, top_posts::{post_score_index::PostScoreIndex, post_score_index_item::PostStatus}, utility_token::token_event::{NewSlotType, SystemTimeInMs}, version_details::VersionDetails
    },
};

use self::memory::{
    get_bet_details_memory, get_post_principal_memory, get_room_details_memory,
    get_slot_details_memory, Memory,
};

use kv_storage::AppStorage;

pub mod kv_storage;
pub mod memory;

pub type RoomDetailsMapOld = ic_stable_structures::btreemap::BTreeMap<GlobalRoomId, RoomDetailsV1, Memory>;
pub type BetDetailsMapOld = ic_stable_structures::btreemap::BTreeMap<GlobalBetId, BetDetails, Memory>;
pub type SlotDetailsMapOld = ic_stable_structures::btreemap::BTreeMap<(PostId, SlotId), SlotDetailsV1, Memory>;

pub type RoomDetailsMap = ic_stable_structures::btreemap::BTreeMap<GlobalRoomIdV1, RoomDetailsV1, Memory>;
pub type BetDetailsMap =ic_stable_structures::btreemap::BTreeMap<GlobalBetIdV1, BetDetails, Memory>;
pub type SlotDetailsMap = ic_stable_structures::btreemap::BTreeMap<(PostId, NewSlotType), SlotDetailsV1, Memory>;
pub type PostPrincipalMap = ic_stable_structures::btreemap::BTreeMap<(PostId, StablePrincipal), (), Memory>;

#[derive(Deserialize, Serialize)]
pub struct CanisterData {
    // Key is Post ID
    pub all_created_posts: BTreeMap<u64, Post>,
    
    #[serde(skip, default = "_default_room_details")]
    pub room_details_map: RoomDetailsMapOld,
    #[serde(skip, default = "_default_room_details_v1")]
    pub room_details_map_v1: RoomDetailsMap, 

    #[serde(skip, default = "_default_bet_details")]
    pub bet_details_map: BetDetailsMapOld,
    #[serde(skip, default = "_default_bet_details_v1")]
    pub bet_details_map_v1: BetDetailsMap,
    
    #[serde(skip, default = "_default_post_principal_map")]
    pub post_principal_map: PostPrincipalMap,
    
    #[serde(skip, default = "_default_slot_details_map")]
    pub slot_details_map: SlotDetailsMapOld,
    #[serde(skip, default = "_default_slot_details_map_v1")]
    pub slot_details_map_v1: SlotDetailsMap,

    // #[serde(skip)]
    pub all_hot_or_not_bets_placed: BTreeMap<(CanisterId, PostId), PlacedBetDetail>,

    // this is a temp field. this will be manually migrated after the upgrade
    #[serde(skip_deserializing)]
    pub all_hot_or_not_bets_placed_v1: BTreeMap<(CanisterId, PostId), PlacedBetDetailV1>,

    pub configuration: IndividualUserConfiguration,
    pub follow_data: FollowData,
    pub known_principal_ids: KnownPrincipalMap,
    pub my_token_balance: TokenBalance,

    #[serde(skip_deserializing)]
    pub my_token_balance_v1: TokenBalanceV1,
    pub posts_index_sorted_by_home_feed_score: PostScoreIndex,
    pub posts_index_sorted_by_hot_or_not_feed_score: PostScoreIndex,
    pub principals_i_follow: BTreeSet<Principal>,
    pub principals_that_follow_me: BTreeSet<Principal>,
    pub profile: UserProfile,
    pub version_details: VersionDetails,
    #[serde(default)]
    pub session_type: Option<SessionType>,
    #[serde(default)]
    pub last_access_time: Option<SystemTime>,
    #[serde(default)]
    pub last_canister_functionality_access_time: Option<SystemTime>,
    #[serde(default)]
    pub migration_info: MigrationInfo,
    #[serde(default)]
    pub app_storage: AppStorage,
    #[serde(skip, default = "_default_watch_history")]
    pub watch_history: ic_stable_structures::btreemap::BTreeMap<WatchHistoryItem, (), Memory>,
    #[serde(skip, default = "_default_success_history_v1")]
    pub success_history: ic_stable_structures::btreemap::BTreeMap<SuccessHistoryItemV1, (), Memory>,
     // u64 is post_id, SystemTime refers to time when first_bet is placed
    // #[serde(skip, default = "_default_bet_timer_vec")]
    // pub bet_timer_posts: ic_stable_structures::vec::Vec<PostId, Memory>,
    #[serde(skip, default = "_default_bet_timer_posts_queue")]
    pub bet_timer_posts:
        ic_stable_structures::btreemap::BTreeMap<(SystemTimeInMs, PostId), (), Memory>,
    // this keeps track of when the first bet was placed.
    #[serde(skip, default = "_default_bet_timer_first_bet_placed_at_map")]
    pub first_bet_placed_at_hashmap:
        ic_stable_structures::btreemap::BTreeMap<PostId, (SystemTimeInMs, NewSlotType), Memory>,
    // #{serde(skip, default = "_default_global_bet_timer")}
    // there is one global timer for processing bets
    pub is_timer_running: Option<PostId>,
}

pub fn _default_room_details() -> RoomDetailsMapOld {
// ) -> ic_stable_structures::btreemap::BTreeMap<GlobalRoomIdV1, RoomDetailsV1, Memory> {
    ic_stable_structures::btreemap::BTreeMap::init(get_room_details_memory())
}

pub fn _default_room_details_v1()-> RoomDetailsMap {
    ic_stable_structures::btreemap::BTreeMap::init(get_room_details_memory_v1())
}

pub fn _default_bet_details() -> BetDetailsMapOld {
    ic_stable_structures::btreemap::BTreeMap::init(get_bet_details_memory())
}

pub fn _default_bet_details_v1() ->  BetDetailsMap {
    ic_stable_structures::btreemap::BTreeMap::init(get_bet_details_memory_v1())
}

pub fn _default_post_principal_map()-> PostPrincipalMap {
    ic_stable_structures::btreemap::BTreeMap::init(get_post_principal_memory())
}

pub fn _default_slot_details_map() ->  SlotDetailsMapOld {
    ic_stable_structures::btreemap::BTreeMap::init(get_slot_details_memory())
}


pub fn _default_watch_history(
) -> ic_stable_structures::btreemap::BTreeMap<WatchHistoryItem, (), Memory> {
    ic_stable_structures::btreemap::BTreeMap::init(get_watch_history_memory())
}

#[deprecated(note = "Use _default_success_history_v1 instead")]
pub fn _default_success_history(
) -> ic_stable_structures::btreemap::BTreeMap<SuccessHistoryItem, (), Memory> {
    ic_stable_structures::btreemap::BTreeMap::init(get_success_history_memory())
}
    pub fn _default_slot_details_map_v1() ->  SlotDetailsMap {
    ic_stable_structures::btreemap::BTreeMap::init(get_slot_details_memory_v1())
}


pub fn _default_bet_timer_first_bet_placed_at_map(
) -> ic_stable_structures::btreemap::BTreeMap<PostId, (SystemTimeInMs, NewSlotType), Memory> {
    ic_stable_structures::btreemap::BTreeMap::init(get_bet_timer_first_bet_at_memory())
}

pub fn _default_bet_timer_posts_queue(
) -> ic_stable_structures::btreemap::BTreeMap<(SystemTimeInMs, PostId), (), Memory> {
    ic_stable_structures::btreemap::BTreeMap::init(get_bet_timer_memory())
}

pub fn _default_success_history_v1(
) -> ic_stable_structures::btreemap::BTreeMap<SuccessHistoryItemV1, (), Memory> {
    ic_stable_structures::btreemap::BTreeMap::init(get_success_history_memory())
}

impl Default for CanisterData {
    fn default() -> Self {
        Self {
            all_created_posts: BTreeMap::new(),
            // todo migration
            room_details_map: _default_room_details(),
            room_details_map_v1: _default_room_details_v1(),
            
            // todo migration
            bet_details_map: _default_bet_details(),
            bet_details_map_v1: _default_bet_details_v1(),
            
            post_principal_map: _default_post_principal_map(),
            
            // todo migration
            slot_details_map: _default_slot_details_map(),
            slot_details_map_v1: _default_slot_details_map_v1(),
            
            // todo migration
            all_hot_or_not_bets_placed: BTreeMap::new(),
            all_hot_or_not_bets_placed_v1: BTreeMap::new(),

            configuration: IndividualUserConfiguration::default(),
            follow_data: FollowData::default(),
            known_principal_ids: KnownPrincipalMap::default(),
            // todo migration
            my_token_balance: TokenBalance::default(),
            my_token_balance_v1: TokenBalanceV1::default(),

            posts_index_sorted_by_home_feed_score: PostScoreIndex::default(),
            posts_index_sorted_by_hot_or_not_feed_score: PostScoreIndex::default(),
            principals_i_follow: BTreeSet::new(),
            principals_that_follow_me: BTreeSet::new(),
            profile: UserProfile::default(),
            version_details: VersionDetails::default(),
            session_type: None,
            last_access_time: None,
            last_canister_functionality_access_time: None,
            migration_info: MigrationInfo::NotMigrated,
            app_storage: AppStorage::default(),
            watch_history: _default_watch_history(),
            success_history: _default_success_history_v1(),
            // these two fields together help with infinite slots in yral game
            first_bet_placed_at_hashmap: _default_bet_timer_first_bet_placed_at_map(),
            // bet_timer_posts: _default_bet_timer_vec(),
            bet_timer_posts: _default_bet_timer_posts_queue(),
            is_timer_running: None,
        }
    }
}
