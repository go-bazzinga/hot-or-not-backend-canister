use candid::Principal;
use std::cell::RefCell;

use crate::data_model::{CanisterUpgradeStatus, UpgradeCanisterArg};
use crate::api::generic_proposal::{PlatformOrchestratorGenericArgumentType, PlatformOrchestratorGenericResultType};
use data_model::CanisterData;
use ic_cdk_macros::export_candid;
use shared_utils::{
    canister_specific::platform_orchestrator::types::args::PlatformOrchestratorInitArgs,
    common::types::http::{HttpRequest, HttpResponse},
    common::types::wasm::WasmType,
};

mod api;
mod data_model;

//TODO: add method to deposit cycle

thread_local! {
    pub static CANISTER_DATA: RefCell<CanisterData> = RefCell::default();
}

export_candid!();
