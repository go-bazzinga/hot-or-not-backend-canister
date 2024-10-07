use candid::{CandidType, Deserialize, Principal};

use crate::common::{participant_crypto::ProofOfParticipation, types::known_principal::KnownPrincipalMap};

use super::hot_or_not::BetDirection;

#[derive(Deserialize, CandidType)]
pub struct IndividualUserTemplateInitArgs {
    pub known_principal_ids: Option<KnownPrincipalMap>,
    pub profile_owner: Option<Principal>,
    pub upgrade_version_number: Option<u64>,
    pub url_to_send_canister_metrics_to: Option<String>,
    pub version: String,
    pub proof_of_participation: Option<ProofOfParticipation>,
}

#[derive(Deserialize, CandidType, Clone)]
pub struct PlaceBetArg {
    pub post_canister_id: Principal,
    pub post_id: u64,
    pub bet_amount: u64,
    pub bet_direction: BetDirection,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct FolloweeArg {
    pub followee_principal_id: Principal,
    pub followee_canister_id: Principal,
}
