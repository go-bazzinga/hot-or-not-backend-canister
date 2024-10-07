use std::{
    collections::{BTreeMap, BTreeSet},
    time::SystemTime,
};

use airdrop::AirdropData;
use candid::{Deserialize, Principal};
use ic_cdk::api::management_canister::provisional::CanisterId;
use memory::{get_pubkey_cache_memory, get_success_history_memory, get_token_list_memory, get_watch_history_memory};
use serde::Serialize;
use shared_utils::{
    canister_specific::individual_user_template::types::{
        cdao::DeployedCdaoCanisters,
        configuration::IndividualUserConfiguration,
        device_id::DeviceIdentity,
        follow::FollowData,
        hot_or_not::{
            BetDetails, GlobalBetId, GlobalRoomId, PlacedBetDetail, RoomDetailsV1, RoomId,
            SlotDetailsV1, SlotId, StablePrincipal,
        },
        migration::MigrationInfo,
        ml_data::{MLFeedCacheItem, SuccessHistoryItem, SuccessHistoryItemV1, WatchHistoryItem},
        post::{FeedScore, Post, PostViewStatistics},
        profile::UserProfile,
        session::SessionType,
        token::TokenBalance,
    },
    common::{participant_crypto::{PoPStore, ProofOfParticipation, PubKeyCache}, types::{
        app_primitive_type::PostId,
        known_principal::{KnownPrincipalMap, KnownPrincipalType},
        top_posts::{post_score_index::PostScoreIndex, post_score_index_item::PostStatus},
        version_details::VersionDetails,
    }},
};

use self::memory::{
    get_bet_details_memory, get_post_principal_memory, get_room_details_memory,
    get_slot_details_memory, Memory,
};

use kv_storage::AppStorage;

pub mod kv_storage;
pub mod memory;
pub mod airdrop;

#[derive(Deserialize, Serialize)]
pub struct CanisterData {
    // Key is Post ID
    pub all_created_posts: BTreeMap<u64, Post>,
    #[serde(skip, default = "_default_room_details")]
    pub room_details_map:
        ic_stable_structures::btreemap::BTreeMap<GlobalRoomId, RoomDetailsV1, Memory>,
    #[serde(skip, default = "_default_bet_details")]
    pub bet_details_map: ic_stable_structures::btreemap::BTreeMap<GlobalBetId, BetDetails, Memory>,
    #[serde(skip, default = "_default_post_principal_map")]
    pub post_principal_map:
        ic_stable_structures::btreemap::BTreeMap<(PostId, StablePrincipal), (), Memory>,
    #[serde(skip, default = "_default_slot_details_map")]
    pub slot_details_map:
        ic_stable_structures::btreemap::BTreeMap<(PostId, SlotId), SlotDetailsV1, Memory>,
    pub all_hot_or_not_bets_placed: BTreeMap<(CanisterId, PostId), PlacedBetDetail>,
    pub configuration: IndividualUserConfiguration,
    pub follow_data: FollowData,
    pub known_principal_ids: KnownPrincipalMap,
    pub my_token_balance: TokenBalance,
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
    #[serde(default)]
    pub device_identities: Vec<DeviceIdentity>,
    #[serde(default)]
    pub ml_feed_cache: Vec<MLFeedCacheItem>,
    #[serde(default)]
    pub cdao_canisters: Vec<DeployedCdaoCanisters>,
    // list of root token canisters
    #[serde(skip, default = "_default_token_list")]
    pub token_roots: ic_stable_structures::btreemap::BTreeMap<Principal, (), Memory>,
    #[serde(default)]
    pub proof_of_participation: Option<ProofOfParticipation>,
    #[serde(skip, default = "_default_pubkey_cache")]
    pub pubkey_cache: PubKeyCache<Memory>,
    #[serde(default)]
    pub airdrop: AirdropData,
}

pub fn _default_room_details(
) -> ic_stable_structures::btreemap::BTreeMap<GlobalRoomId, RoomDetailsV1, Memory> {
    ic_stable_structures::btreemap::BTreeMap::init(get_room_details_memory())
}

pub fn _default_bet_details(
) -> ic_stable_structures::btreemap::BTreeMap<GlobalBetId, BetDetails, Memory> {
    ic_stable_structures::btreemap::BTreeMap::init(get_bet_details_memory())
}

pub fn _default_post_principal_map(
) -> ic_stable_structures::btreemap::BTreeMap<(PostId, StablePrincipal), (), Memory> {
    ic_stable_structures::btreemap::BTreeMap::init(get_post_principal_memory())
}

pub fn _default_slot_details_map(
) -> ic_stable_structures::btreemap::BTreeMap<(PostId, SlotId), SlotDetailsV1, Memory> {
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

pub fn _default_token_list() -> ic_stable_structures::btreemap::BTreeMap<Principal, (), Memory> {
    ic_stable_structures::btreemap::BTreeMap::init(get_token_list_memory())
}

pub fn _default_success_history_v1(
) -> ic_stable_structures::btreemap::BTreeMap<SuccessHistoryItemV1, (), Memory> {
    ic_stable_structures::btreemap::BTreeMap::init(get_success_history_memory())
}

pub fn _default_pubkey_cache() -> PubKeyCache<Memory> {
    PubKeyCache::init(get_pubkey_cache_memory())
}

impl Default for CanisterData {
    fn default() -> Self {
        Self {
            all_created_posts: BTreeMap::new(),
            room_details_map: _default_room_details(),
            bet_details_map: _default_bet_details(),
            post_principal_map: _default_post_principal_map(),
            slot_details_map: _default_slot_details_map(),
            all_hot_or_not_bets_placed: BTreeMap::new(),
            configuration: IndividualUserConfiguration::default(),
            follow_data: FollowData::default(),
            known_principal_ids: KnownPrincipalMap::default(),
            my_token_balance: TokenBalance::default(),
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
            device_identities: Vec::new(),
            ml_feed_cache: Vec::new(),
            cdao_canisters: Vec::new(),
            token_roots: _default_token_list(),
            proof_of_participation: None,
            pubkey_cache: _default_pubkey_cache(),
            airdrop: AirdropData::default(),
        }
    }
}

impl PoPStore<Memory> for CanisterData {
    fn pubkey_cache(&self) -> &PubKeyCache<Memory> {
        &self.pubkey_cache
    }

    fn pubkey_cache_mut(&mut self) -> &mut PubKeyCache<Memory> {
        &mut self.pubkey_cache
    }

    fn platform_orchestrator(&self) -> Principal {
        self.known_principal_ids[&KnownPrincipalType::CanisterIdPlatformOrchestrator]
    }
}
