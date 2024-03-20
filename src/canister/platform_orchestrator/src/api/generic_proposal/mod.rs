use std::{future::Future, pin::Pin};

use ic_cdk_macros::update;
use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};
use shared_utils::common::{types::wasm::WasmType, utils::permissions::is_caller_governance_canister};

use crate::data_model::UpgradeCanisterArg;

use super::canister_management::{provision_subnet_orchestrator::provision_subnet_orchestrator_canister, remove_subnet_orchestrator_from_available_list::{self, remove_subnet_orchestrators_from_available_list}, upgrade_canisters_in_network::{self, upgrade_canisters_in_network}, upload_wasms::upload_wasms};

#[derive(CandidType, Serialize, Deserialize)]
pub enum PlatformOrchestratorGenericArgumentType {
    RemoveSubnetOrchestratorFromAvailableList(Principal),
    ProvisionSubnetOrchestrator(Principal),
    UpgradeSubnetCanisters(UpgradeCanisterArg),
    UploadWasm(WasmType, Vec<u8>)

}



#[derive(CandidType, Deserialize, Serialize)]
pub enum PlatformOrchestratorGenericResultType {
    RemoveSubnetOrchestratorFromAvailableListResult(Result<String, String>),
    ProvisionSubnetOrchestrator(Result<Principal, String>),
    UpgradeSubnetCanisters(Result<String, String>),
    UploadWasm(Result<String, String>)

}


#[update(guard = "is_caller_governance_canister")]
pub fn validate_platform_orchestrator_generic_function(arg: PlatformOrchestratorGenericArgumentType) {
}


#[update(guard = "is_caller_governance_canister")]
pub async fn platform_orchestrator_generic_function(arg: PlatformOrchestratorGenericArgumentType) -> PlatformOrchestratorGenericResultType {
    match arg {
        PlatformOrchestratorGenericArgumentType::RemoveSubnetOrchestratorFromAvailableList(subnet_orchestrator) => {
            let res = remove_subnet_orchestrators_from_available_list(subnet_orchestrator);
            PlatformOrchestratorGenericResultType::RemoveSubnetOrchestratorFromAvailableListResult(res)
        },
        PlatformOrchestratorGenericArgumentType::ProvisionSubnetOrchestrator(subnet_principal) =>  {
            let res = Box::pin(provision_subnet_orchestrator_canister(subnet_principal)).await;
            
            PlatformOrchestratorGenericResultType::ProvisionSubnetOrchestrator(res)
        },
        PlatformOrchestratorGenericArgumentType::UpgradeSubnetCanisters(upgrade_canister_arg) => {
            let res = Box::pin(upgrade_canisters_in_network(upgrade_canister_arg)).await;
            
            PlatformOrchestratorGenericResultType::UpgradeSubnetCanisters(res)
        }
        PlatformOrchestratorGenericArgumentType::UploadWasm(wasm_type, wasm_blob) => {
            let res = upload_wasms(wasm_type, wasm_blob);
            
            PlatformOrchestratorGenericResultType::UploadWasm(res)
        }
    }
}