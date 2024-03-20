use candid::Principal;
use ic_cdk_macros::update;
use shared_utils::common::utils::permissions::is_caller_controller_or_global_admin;

use crate::CANISTER_DATA;


#[update(guard = "is_caller_controller_or_global_admin")]
pub fn remove_subnet_orchestrators_from_available_list(subnet_orchestrator: Principal) -> Result<String, String> {
    CANISTER_DATA.with_borrow_mut(|canister_data| {
        let remove_result = canister_data.subet_orchestrator_with_capacity_left.remove(&subnet_orchestrator);
        match remove_result {
            true => Ok("Success".into()),
            false => Err("Subnet not found in available list".into())
        }
    })    
}