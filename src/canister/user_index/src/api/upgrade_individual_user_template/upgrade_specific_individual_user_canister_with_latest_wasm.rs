use candid::Principal;
use ic_cdk::api::management_canister::main::{
    deposit_cycles, CanisterIdRecord, CanisterInstallMode,
};
use ic_cdk_macros::update;
use shared_utils::{
    canister_specific::individual_user_template::types::arg::IndividualUserTemplateInitArgs,
    common::{
        types::{known_principal::KnownPrincipalType, wasm::WasmType},
        utils::permissions::is_caller_controller,
    },
    constant::INDIVIDUAL_USER_CANISTER_RECHARGE_AMOUNT,
};

use crate::{util::canister_management, CANISTER_DATA};

// * dfx canister call user_index upgrade_specific_individual_user_canister_with_latest_wasm '(principal "", principal "", null)' --network ic

#[update(guard = "is_caller_controller")]
async fn upgrade_specific_individual_user_canister_with_latest_wasm(
    user_canister_id: Principal,
    user_principal_id: Option<Principal>,
    upgrade_mode: Option<CanisterInstallMode>,
) -> String {
    let known_principal_ids = CANISTER_DATA.with(|canister_data_ref_cell| {
        canister_data_ref_cell
            .borrow()
            .configuration
            .known_principal_ids
            .clone()
    });

    let saved_upgrade_status = CANISTER_DATA.with(|canister_data_ref_cell| {
        canister_data_ref_cell
            .borrow()
            .last_run_upgrade_status
            .clone()
    });

    let configuration = CANISTER_DATA
        .with(|canister_data_ref_cell| canister_data_ref_cell.borrow().configuration.clone());

    let individual_canister_wasm = CANISTER_DATA.with_borrow(|canister_data| {
        canister_data
            .wasms
            .get(&WasmType::IndividualUserWasm)
            .unwrap()
    });

    let mut proof_of_participation = CANISTER_DATA.with_borrow(|cdata| cdata.proof_of_participation.clone());
    if let Some(pop) = proof_of_participation {
        proof_of_participation = Some(pop.derive_for_child(user_canister_id).await.unwrap());
    }
    match canister_management::upgrade_individual_user_canister(
        user_canister_id,
        upgrade_mode.unwrap_or(CanisterInstallMode::Upgrade(None)),
        IndividualUserTemplateInitArgs {
            known_principal_ids: Some(known_principal_ids.clone()),
            profile_owner: user_principal_id,
            upgrade_version_number: Some(saved_upgrade_status.version_number + 1),
            url_to_send_canister_metrics_to: Some(configuration.url_to_send_canister_metrics_to),
            version: individual_canister_wasm.version,
            proof_of_participation,
        },
        individual_canister_wasm.wasm_blob,
    )
    .await
    {
        Ok(_) => "Success".to_string(),
        Err(e) => e.1,
    }
}
