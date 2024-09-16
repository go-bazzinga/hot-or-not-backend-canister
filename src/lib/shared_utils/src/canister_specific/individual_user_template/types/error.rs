use candid::{CandidType, Deserialize};
use ic_cdk::api::call::RejectionCode;
use icrc_ledger_types::icrc1::transfer::TransferError;

#[derive(CandidType, Deserialize, PartialEq, Eq, Debug)]
pub enum GetPostsOfUserProfileError {
    InvalidBoundsPassed,
    ReachedEndOfItemsList,
    ExceededMaxNumberOfItemsAllowedInOneRequest,
}

#[derive(CandidType, Deserialize, PartialEq, Eq, Debug)]
pub enum GetFollowerOrFollowingError {
    InvalidBoundsPassed,
    ReachedEndOfItemsList,
    ExceededMaxNumberOfItemsAllowedInOneRequest,
}

#[derive(CandidType, Deserialize, PartialEq, Eq, Debug)]
pub enum GetFollowerOrFollowingPageError {
    Unauthenticated,
    Unauthorized,
}

#[derive(CandidType, PartialEq, Eq, Debug, Deserialize, Clone)]
pub enum BetOnCurrentlyViewingPostError {
    BettingClosed,
    InsufficientBalance,
    Unauthorized,
    UserAlreadyParticipatedInThisPost,
    UserNotLoggedIn,
    UserPrincipalNotSet,
    PostCreatorCanisterCallFailed,
}

#[derive(CandidType, Deserialize, PartialEq, Eq, Debug)]
pub enum FollowAnotherUserProfileError {
    Unauthenticated,
    Unauthorized,
    UsersICanFollowListIsFull,
    UserITriedToFollowCrossCanisterCallFailed,
    UserITriedToFollowHasTheirFollowersListFull,
}

#[derive(CandidType, Deserialize, PartialEq, Eq, Debug)]
pub enum CdaoDeployError {
    Unregistered,
    TokenLimit(usize),
    CallError(RejectionCode, String),
    InvalidInitPayload(String),
    Unauthenticated,
    CycleError(String),
}

impl From<(RejectionCode, String)> for CdaoDeployError {
    fn from((rejection_code, error_message): (RejectionCode, String)) -> Self {
        CdaoDeployError::CallError(rejection_code, error_message)
    }
}

#[derive(CandidType, Deserialize, PartialEq, Eq, Debug)]
pub enum CdaoTokenError {
    InvalidRoot,
    Transfer(TransferError),
    NoBalance,
    CallError(RejectionCode, String),
    Unauthenticated,
}

impl From<(RejectionCode, String)> for CdaoTokenError {
    fn from(value: (RejectionCode, String)) -> Self {
        CdaoTokenError::CallError(value.0, value.1)
    }
}
