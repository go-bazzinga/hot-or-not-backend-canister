use candid::{CandidType, Deserialize};

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
