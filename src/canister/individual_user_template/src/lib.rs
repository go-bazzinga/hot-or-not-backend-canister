use std::cell::RefCell;
use std::{collections::BTreeMap, time::SystemTime};

use api::{
    follow::update_profiles_that_follow_me_toggle_list_with_specified_profile::FollowerArg,
    profile::update_profile_display_details::UpdateProfileDetailsError,
};
use candid::{Principal, Nat};
use data_model::CanisterData;
use ic_cdk::api::management_canister::provisional::CanisterId;
use ic_cdk_macros::export_candid;
use shared_utils::{
    canister_specific::individual_user_template::types::{
        arg::{FolloweeArg, IndividualUserTemplateInitArgs, PlaceBetArg},
        device_id::DeviceIdentity,
        error::{
            BetOnCurrentlyViewingPostError, FollowAnotherUserProfileError,
            GetPostsOfUserProfileError, CdaoDeployError, CdaoTokenError,
        },
        follow::{FollowEntryDetail, FollowEntryId},
        hot_or_not::{BetOutcomeForBetMaker, BettingStatus,BettingStatusV1, PlacedBetDetail, PlacedBetDetailV1},
        kv_storage::{NamespaceErrors, NamespaceForFrontend},
        migration::MigrationErrors,
        ml_data::{MLFeedCacheItem, SuccessHistoryItemV1, WatchHistoryItem},
        post::{
            Post, PostDetailsForFrontend, PostDetailsFromFrontend, PostViewDetailsFromFrontend,
        },
        profile::{
            UserCanisterDetails, UserProfile, UserProfileDetailsForFrontend,
            UserProfileDetailsForFrontendV2, UserProfileUpdateDetailsFromFrontend,
        },
        cdao::DeployedCdaoCanisters,
        session::SessionType,
    },
    common::types::{
        app_primitive_type::PostId,
        http::{HttpRequest, HttpResponse},
        known_principal::KnownPrincipalType,
        top_posts::post_score_index_item::PostStatus,
        utility_token::token_event::TokenEvent,
    },
    types::canister_specific::individual_user_template::error_types::{
        GetUserUtilityTokenTransactionHistoryError, UpdateProfileSetUniqueUsernameError,
    },
    pagination::PaginationError,
};
use shared_utils::canister_specific::individual_user_template::types::post::PostDetailsForFrontendV1;
use shared_utils::common::types::utility_token::token_event::TokenEventV1;
use ic_sns_init::pb::v1::SnsInitPayload;
use icrc_ledger_types::icrc1::transfer::Memo;
use ic_nns_governance::pb::v1::{SettleNeuronsFundParticipationRequest, SettleNeuronsFundParticipationResponse};

mod api;
pub mod data_model;
mod util;

thread_local! {
    static CANISTER_DATA: RefCell<CanisterData> = RefCell::default();
    static SNAPSHOT_DATA: RefCell<Vec<u8>> = RefCell::default();
}

export_candid!();
