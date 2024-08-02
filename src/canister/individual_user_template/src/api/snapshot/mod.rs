use std::{
    borrow::Borrow,
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    time::SystemTime,
};

use candid::{CandidType, Deserialize, Principal};
use ic_cdk::api::management_canister::provisional::CanisterId;
use serde::Serialize;
use serde_json_any_key::*;
use shared_utils::{
    canister_specific::individual_user_template::types::{
        configuration::IndividualUserConfiguration,
        follow::{FollowData, FollowEntryDetail, FollowEntryId, FollowList},
        hot_or_not::{
            AggregateStats, BetDetails, BetMaker, BetMakerPrincipal, GlobalBetId, GlobalRoomId,
            HotOrNotDetails, PlacedBetDetail, RoomDetailsV1, RoomId, SlotDetailsV1, SlotId,
            StablePrincipal,
        },
        migration::MigrationInfo,
        post::{FeedScore, Post, PostViewStatistics},
        profile::UserProfile,
        session::SessionType,
        token::TokenBalance,
    },
    common::types::{
        app_primitive_type::PostId,
        known_principal::KnownPrincipalMap,
        top_posts::{
            post_score_index::PostScoreIndex,
            post_score_index_item::{PostScoreIndexItem, PostStatus},
            PublisherCanisterId, Score,
        },
        utility_token::token_event::TokenEvent,
        version_details::VersionDetails,
    },
};

use crate::data_model::_default_room_details;
use crate::data_model::{
    CanisterData, _default_bet_details, _default_post_principal_map, _default_slot_details_map,
};

pub mod get_snapshot;
pub mod serde_json_snapshot_test;

#[derive(Deserialize, Serialize)]
pub struct CanisterDataForSnapshot {
    // Key is Post ID
    pub all_created_posts: BTreeMap<u64, PostForSnapshot>,
    #[serde(with = "any_key_map")]
    pub room_details_map: BTreeMap<GlobalRoomId, RoomDetailsV1>,
    #[serde(with = "any_key_map")]
    pub bet_details_map: BTreeMap<GlobalBetId, BetDetails>,
    #[serde(with = "any_key_map")]
    pub post_principal_map: BTreeMap<(PostId, StablePrincipal), ()>,
    #[serde(with = "any_key_map")]
    pub slot_details_map: BTreeMap<(PostId, SlotId), SlotDetailsV1>,
    #[serde(with = "any_key_map")]
    pub all_hot_or_not_bets_placed: BTreeMap<(CanisterId, PostId), PlacedBetDetail>,
    // #[serde(with = "any_key_map")]
    // pub all_hot_or_not_bets_placed_v1: BTreeMap<(CanisterId, PostId), PlacedBetDetailV1>,
    pub configuration: IndividualUserConfiguration,
    pub follow_data: FollowDataForSnapshot,
    #[serde(with = "any_key_map")]
    pub known_principal_ids: KnownPrincipalMap,
    pub my_token_balance: TokenBalanceForSnapshot,
    pub posts_index_sorted_by_home_feed_score: PostScoreIndexForSnapshot,
    pub posts_index_sorted_by_hot_or_not_feed_score: PostScoreIndexForSnapshot,
    pub principals_i_follow: BTreeSet<Principal>,
    pub principals_that_follow_me: BTreeSet<Principal>,
    pub profile: UserProfile,
    pub version_details: VersionDetails,
    pub session_type: Option<SessionType>,
    pub last_access_time: Option<SystemTime>,
    pub last_canister_functionality_access_time: Option<SystemTime>,
    pub migration_info: MigrationInfo,
}

#[derive(CandidType, Clone, Deserialize, Debug, Serialize)]
pub struct PostForSnapshot {
    pub id: u64,
    pub description: String,
    pub hashtags: Vec<String>,
    pub video_uid: String,
    pub status: PostStatus,
    pub created_at: SystemTime,
    pub likes: HashSet<Principal>,
    pub share_count: u64,
    pub view_stats: PostViewStatistics,
    pub home_feed_score: FeedScore,
    pub creator_consent_for_inclusion_in_hot_or_not: bool,
    pub hot_or_not_details: Option<HotOrNotDetailsForSnapshot>,
    #[serde(default)]
    pub is_nsfw: bool,
}

#[derive(CandidType, Clone, Deserialize, Debug, Serialize, Default)]
pub struct HotOrNotDetailsForSnapshot {
    pub hot_or_not_feed_score: FeedScore,
    pub aggregate_stats: AggregateStats,
}

#[derive(Default, Clone, Deserialize, CandidType, Debug, Serialize)]
pub struct TokenBalanceForSnapshot {
    pub utility_token_balance: u64,
    #[serde(with = "any_key_map")]
    pub utility_token_transaction_history: BTreeMap<u64, TokenEvent>,
    pub lifetime_earnings: u64,
}

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct FollowDataForSnapshot {
    pub follower: FollowListForSnapshot,
    pub following: FollowListForSnapshot,
}

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct FollowListForSnapshot {
    #[serde(with = "any_key_map")]
    pub sorted_index: BTreeMap<FollowEntryId, FollowEntryDetail>,
    #[serde(with = "any_key_map")]
    pub members: HashMap<FollowEntryDetail, FollowEntryId>,
}

#[derive(Default, Debug, Clone, CandidType, Deserialize, Serialize)]
pub struct PostScoreIndexForSnapshot {
    #[serde(with = "any_key_map")]
    pub items_sorted_by_score: BTreeMap<Score, Vec<PostScoreIndexItem>>,
    #[serde(with = "any_key_map")]
    pub item_presence_index: HashMap<(PublisherCanisterId, PostId), Score>,
}

impl From<&CanisterData> for CanisterDataForSnapshot {
    fn from(canister_data: &CanisterData) -> Self {
        let mut all_created_posts: BTreeMap<u64, PostForSnapshot> = BTreeMap::new();
        canister_data.all_created_posts.iter().for_each(|(k, v)| {
            let hot_or_not_details = v.hot_or_not_details.clone();
            let hot_or_not_details_snapshot =
                hot_or_not_details.map(|hot_or_not_details| HotOrNotDetailsForSnapshot {
                    hot_or_not_feed_score: hot_or_not_details.hot_or_not_feed_score,
                    aggregate_stats: hot_or_not_details.aggregate_stats.clone(),
                });

            let post_details = PostForSnapshot {
                id: v.id,
                description: v.description.clone(),
                hashtags: v.hashtags.clone(),
                video_uid: v.video_uid.clone(),
                status: v.status,
                created_at: v.created_at,
                likes: v.likes.clone(),
                share_count: v.share_count,
                view_stats: v.view_stats.clone(),
                home_feed_score: v.home_feed_score.clone(),
                creator_consent_for_inclusion_in_hot_or_not: v
                    .creator_consent_for_inclusion_in_hot_or_not,
                hot_or_not_details: hot_or_not_details_snapshot,
                is_nsfw: v.is_nsfw,
            };

            all_created_posts.insert(k.clone(), post_details);
        });

        let mut room_details_map: BTreeMap<GlobalRoomId, RoomDetailsV1> = BTreeMap::new();
        canister_data.room_details_map.iter().for_each(|(k, v)| {
            room_details_map.insert(k, v.clone());
        });

        let mut bet_details_map: BTreeMap<GlobalBetId, BetDetails> = BTreeMap::new();
        canister_data.bet_details_map.iter().for_each(|(k, v)| {
            bet_details_map.insert(k, v.clone());
        });

        let mut post_principal_map: BTreeMap<(PostId, StablePrincipal), ()> = BTreeMap::new();
        canister_data.post_principal_map.iter().for_each(|(k, v)| {
            post_principal_map.insert(k, v.clone());
        });

        let mut slot_details_map: BTreeMap<(PostId, SlotId), SlotDetailsV1> = BTreeMap::new();
        canister_data.slot_details_map.iter().for_each(|(k, v)| {
            slot_details_map.insert(k, v.clone());
        });

        let my_token_balance = TokenBalanceForSnapshot {
            utility_token_balance: canister_data.my_token_balance.utility_token_balance,
            utility_token_transaction_history: canister_data
                .my_token_balance
                .utility_token_transaction_history
                .clone(),
            lifetime_earnings: canister_data.my_token_balance.lifetime_earnings,
        };

        let follow_data = FollowDataForSnapshot {
            follower: FollowListForSnapshot {
                sorted_index: canister_data.follow_data.follower.sorted_index.clone(),
                members: canister_data.follow_data.follower.members.clone(),
            },
            following: FollowListForSnapshot {
                sorted_index: canister_data.follow_data.following.sorted_index.clone(),
                members: canister_data.follow_data.following.members.clone(),
            },
        };

        let posts_index_sorted_by_home_feed_score = PostScoreIndexForSnapshot {
            items_sorted_by_score: canister_data
                .posts_index_sorted_by_home_feed_score
                .items_sorted_by_score
                .clone(),
            item_presence_index: canister_data
                .posts_index_sorted_by_home_feed_score
                .item_presence_index
                .clone(),
        };

        let posts_index_sorted_by_hot_or_not_feed_score = PostScoreIndexForSnapshot {
            items_sorted_by_score: canister_data
                .posts_index_sorted_by_hot_or_not_feed_score
                .items_sorted_by_score
                .clone(),
            item_presence_index: canister_data
                .posts_index_sorted_by_hot_or_not_feed_score
                .item_presence_index
                .clone(),
        };

        Self {
            all_created_posts,
            room_details_map,
            bet_details_map,
            post_principal_map,
            slot_details_map,
            all_hot_or_not_bets_placed: canister_data.all_hot_or_not_bets_placed.clone(),
            // all_hot_or_not_bets_placed_v1: canister_data.all_hot_or_not_bets_placed_v1.clone(),
            configuration: canister_data.configuration.clone(),
            follow_data,
            known_principal_ids: canister_data.known_principal_ids.clone(),
            my_token_balance,
            posts_index_sorted_by_home_feed_score,
            posts_index_sorted_by_hot_or_not_feed_score,
            principals_i_follow: canister_data.principals_i_follow.clone(),
            principals_that_follow_me: canister_data.principals_that_follow_me.clone(),
            profile: canister_data.profile.clone(),
            version_details: canister_data.version_details.clone(),
            session_type: canister_data.session_type,
            last_access_time: canister_data.last_access_time,
            last_canister_functionality_access_time: canister_data
                .last_canister_functionality_access_time,
            migration_info: canister_data.migration_info,
        }
    }
}

impl From<CanisterDataForSnapshot> for CanisterData {
    fn from(canister_data: CanisterDataForSnapshot) -> Self {
        let mut all_created_posts: BTreeMap<u64, Post> = BTreeMap::new();
        canister_data.all_created_posts.iter().for_each(|(k, v)| {
            let hot_or_not_details_snapshot = v.hot_or_not_details.clone();
            let hot_or_not_details =
                hot_or_not_details_snapshot.map(|hot_or_not_details_snapshot| HotOrNotDetails {
                    hot_or_not_feed_score: hot_or_not_details_snapshot.hot_or_not_feed_score,
                    aggregate_stats: hot_or_not_details_snapshot.aggregate_stats.clone(),
                    slot_history: BTreeMap::new(),
                });

            let post_details = Post {
                id: v.id,
                description: v.description.clone(),
                hashtags: v.hashtags.clone(),
                video_uid: v.video_uid.clone(),
                status: v.status,
                created_at: v.created_at,
                likes: v.likes.clone(),
                share_count: v.share_count,
                view_stats: v.view_stats.clone(),
                home_feed_score: v.home_feed_score.clone(),
                creator_consent_for_inclusion_in_hot_or_not: v
                    .creator_consent_for_inclusion_in_hot_or_not,
                hot_or_not_details: hot_or_not_details,
                is_nsfw: v.is_nsfw,
            };

            all_created_posts.insert(k.clone(), post_details);
        });

        let mut room_details_map = _default_room_details();
        canister_data.room_details_map.iter().for_each(|(k, v)| {
            room_details_map.insert(*k, v.clone());
        });

        let mut bet_details_map = _default_bet_details();
        canister_data.bet_details_map.iter().for_each(|(k, v)| {
            bet_details_map.insert(k.clone(), v.clone());
        });

        let mut post_principal_map = _default_post_principal_map();
        canister_data.post_principal_map.iter().for_each(|(k, v)| {
            post_principal_map.insert(k.clone(), v.clone());
        });

        let mut slot_details_map = _default_slot_details_map();
        canister_data.slot_details_map.iter().for_each(|(k, v)| {
            slot_details_map.insert(*k, v.clone());
        });

        let my_token_balance = TokenBalance {
            utility_token_balance: canister_data.my_token_balance.utility_token_balance,
            utility_token_transaction_history: canister_data
                .my_token_balance
                .utility_token_transaction_history
                .clone(),
            lifetime_earnings: canister_data.my_token_balance.lifetime_earnings,
        };

        let follow_data = FollowData {
            follower: FollowList {
                sorted_index: canister_data.follow_data.follower.sorted_index.clone(),
                members: canister_data.follow_data.follower.members.clone(),
            },
            following: FollowList {
                sorted_index: canister_data.follow_data.following.sorted_index.clone(),
                members: canister_data.follow_data.following.members.clone(),
            },
        };

        let posts_index_sorted_by_home_feed_score = PostScoreIndex {
            items_sorted_by_score: canister_data
                .posts_index_sorted_by_home_feed_score
                .items_sorted_by_score
                .clone(),
            item_presence_index: canister_data
                .posts_index_sorted_by_home_feed_score
                .item_presence_index
                .clone(),
        };

        let posts_index_sorted_by_hot_or_not_feed_score = PostScoreIndex {
            items_sorted_by_score: canister_data
                .posts_index_sorted_by_hot_or_not_feed_score
                .items_sorted_by_score
                .clone(),
            item_presence_index: canister_data
                .posts_index_sorted_by_hot_or_not_feed_score
                .item_presence_index
                .clone(),
        };

        Self {
            all_created_posts,
            room_details_map,
            bet_details_map,
            post_principal_map,
            slot_details_map,
            all_hot_or_not_bets_placed: canister_data.all_hot_or_not_bets_placed,
            // all_hot_or_not_bets_placed_v1: canister_data.all_hot_or_not_bets_placed_v1,
            configuration: canister_data.configuration,
            follow_data,
            known_principal_ids: canister_data.known_principal_ids,
            my_token_balance,
            posts_index_sorted_by_home_feed_score,
            posts_index_sorted_by_hot_or_not_feed_score,
            principals_i_follow: canister_data.principals_i_follow,
            principals_that_follow_me: canister_data.principals_that_follow_me,
            profile: canister_data.profile,
            version_details: canister_data.version_details,
            session_type: canister_data.session_type,
            last_access_time: canister_data.last_access_time,
            last_canister_functionality_access_time: canister_data
                .last_canister_functionality_access_time,
            migration_info: canister_data.migration_info,
            ..Default::default()
        }
    }
}
