use std::cell::RefCell;
use std::{collections::BTreeMap, time::SystemTime};

use api::{
    follow::update_profiles_that_follow_me_toggle_list_with_specified_profile::FollowerArg,
    profile::update_profile_display_details::UpdateProfileDetailsError,
};
use candid::Principal;
use data_model::CanisterData;
use ic_cdk::api::management_canister::provisional::CanisterId;
use ic_cdk_macros::export_candid;
use ic_nns_governance::pb::v1::{
    SettleNeuronsFundParticipationRequest, SettleNeuronsFundParticipationResponse,
};
use ic_sns_init::pb::v1::SnsInitPayload;
use shared_utils::{
    canister_specific::individual_user_template::types::{
        airdrop::{AirdropMember, TokenClaim},
        arg::{FolloweeArg, IndividualUserTemplateInitArgs, PlaceBetArg},
        cdao::DeployedCdaoCanisters,
        device_id::DeviceIdentity,
        error::{
            BetOnCurrentlyViewingPostError, CdaoDeployError, CdaoTokenError,
            FollowAnotherUserProfileError, GetPostsOfUserProfileError,
        },
        follow::{FollowEntryDetail, FollowEntryId},
        hot_or_not::{BetDetails, BetOutcomeForBetMaker, BettingStatus, PlacedBetDetail},
        kv_storage::{NamespaceErrors, NamespaceForFrontend},
        migration::MigrationErrors,
        ml_data::{MLFeedCacheItem, SuccessHistoryItemV1, WatchHistoryItem},
        post::{
            Post, PostDetailsForFrontend, PostDetailsFromFrontend, PostViewDetailsFromFrontend,
        },
        profile::{
            UserCanisterDetails, UserProfileDetailsForFrontend,
            UserProfileDetailsForFrontendV2, UserProfileUpdateDetailsFromFrontend,
        },
        session::SessionType,
    },
    common::{
        participant_crypto::ProofOfParticipation,
        types::{
            app_primitive_type::PostId,
            http::{HttpRequest, HttpResponse},
            known_principal::KnownPrincipalType,
            top_posts::post_score_index_item::PostStatus,
            utility_token::token_event::TokenEvent,
        }
    },
    pagination::PaginationError,
    types::canister_specific::individual_user_template::error_types::{
        GetUserUtilityTokenTransactionHistoryError, UpdateProfileSetUniqueUsernameError,
    },
};

mod api;
pub mod data_model;
mod util;

thread_local! {
    static CANISTER_DATA: RefCell<CanisterData> = RefCell::default();
    static SNAPSHOT_DATA: RefCell<Vec<u8>> = RefCell::default();
}

export_candid!();
