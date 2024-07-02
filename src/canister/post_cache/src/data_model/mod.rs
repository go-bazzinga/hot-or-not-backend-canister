use std::time::SystemTime;

use candid::{CandidType, Deserialize};
use serde::Serialize;
use shared_utils::common::types::{
    known_principal::KnownPrincipalMap,
    top_posts::{
        post_score_home_index::PostScoreHomeIndex,
        post_score_hot_or_not_index::PostScoreHotOrNotIndex, post_score_index::PostScoreIndex,
    },
    version_details::VersionDetails,
};

#[derive(Default, CandidType, Deserialize, Serialize)]
pub struct CanisterData {
    pub known_principal_ids: KnownPrincipalMap,
    #[serde(skip)]
    pub posts_index_sorted_by_home_feed_score: PostScoreIndex,
    #[serde(skip)]
    pub posts_index_sorted_by_hot_or_not_feed_score: PostScoreIndex,
    #[serde(default)]
    pub posts_index_sorted_by_home_feed_score_v1: PostScoreHomeIndex,
    #[serde(default)]
    pub posts_index_sorted_by_hot_or_not_feed_score_v1: PostScoreHotOrNotIndex,
    #[serde(default)]
    pub posts_index_sorted_by_yral_feed_score_v1: PostScoreHomeIndex,
    #[serde(default)]
    pub metadata: Metadata,
    #[serde(default)]
    pub version_details: VersionDetails,
}

#[derive(Default, CandidType, Deserialize, Serialize)]
pub struct Metadata {
    pub last_updated_hot_or_not_timestamp_index: Option<SystemTime>,
    pub last_updated_reconcile_scores: Option<SystemTime>,
}
