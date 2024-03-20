use ic_cdk_macros::query;

use crate::CANISTER_DATA;


pub mod provision_subnet_orchestrator;
pub mod upgrade_canisters_in_network;
pub mod upload_wasms;
mod subnet_orchestrator_maxed_out;
mod get_last_subnet_upgrade_status;
mod get_all_available_subnet_orchestrators;
mod get_all_subnet_orchestrators;
mod stop_upgrades_for_individual_user_canisters;
mod upgrade_specific_individual_canister;
mod update_profile_owner_for_individual_users;
pub mod remove_subnet_orchestrator_from_available_list;

#[query]
pub fn get_version() -> String {
    CANISTER_DATA.with_borrow(|canister_data| {
        canister_data.version_detail.version.clone()
    })
}