use crate::{
    api::canister_management::update_last_access_time::update_last_canister_functionality_access_time,
    CANISTER_DATA,
};
use ic_cdk_macros::query;
use shared_utils::canister_specific::individual_user_template::types::profile::UserProfileDetailsForFrontend;

#[deprecated(note = "Please use get_profile_details_v3")]
#[query]
fn get_profile_details() -> UserProfileDetailsForFrontend {
    update_last_canister_functionality_access_time();

    CANISTER_DATA.with(|canister_data_ref_cell| {
        let profile = canister_data_ref_cell.borrow().profile.clone();
        let token_balance = &canister_data_ref_cell.borrow().my_token_balance;

        UserProfileDetailsForFrontend {
            principal_id: profile.principal_id.unwrap(),
            display_name: profile.display_name.clone(),
            unique_user_name: profile.unique_user_name.clone(),
            profile_picture_url: profile.profile_picture_url.clone(),
            profile_stats: profile.profile_stats,
            followers_count: canister_data_ref_cell.borrow().follow_data.follower.len() as u64,
            following_count: canister_data_ref_cell.borrow().follow_data.following.len() as u64,
            lifetime_earnings: token_balance.lifetime_earnings,
            referrer_details: profile.referrer_details,
        }
    })
}



#[query]
fn get_profile_details_v3() -> UserProfileDetailsForFrontend {
    update_last_canister_functionality_access_time();

    CANISTER_DATA.with(|canister_data_ref_cell| {
        let profile = canister_data_ref_cell.borrow().profile.clone();
        let token_balance = &canister_data_ref_cell.borrow().my_token_balance_v1;

        UserProfileDetailsForFrontend {
            principal_id: profile.principal_id.unwrap(),
            display_name: profile.display_name.clone(),
            unique_user_name: profile.unique_user_name.clone(),
            profile_picture_url: profile.profile_picture_url.clone(),
            profile_stats: profile.profile_stats,
            followers_count: canister_data_ref_cell.borrow().follow_data.follower.len() as u64,
            following_count: canister_data_ref_cell.borrow().follow_data.following.len() as u64,
            lifetime_earnings: token_balance.lifetime_earnings,
            referrer_details: profile.referrer_details,
        }
    })
}
