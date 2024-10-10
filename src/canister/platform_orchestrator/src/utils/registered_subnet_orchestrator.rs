use candid::Principal;
use ic_cdk::api::management_canister::main::{
    canister_status, deposit_cycles, update_settings, CanisterIdRecord, CanisterSettings,
    LogVisibility, UpdateSettingsArgument,
};
use shared_utils::{
    common::types::wasm::WasmType, constant::SUBNET_ORCHESTRATOR_CANISTER_CYCLES_THRESHOLD,
};

use crate::CANISTER_DATA;

pub struct RegisteredSubnetOrchestrator {
    canister_id: Principal,
}

impl RegisteredSubnetOrchestrator {
    pub fn new(canister_id: Principal) -> Result<RegisteredSubnetOrchestrator, String> {
        let contains = CANISTER_DATA.with_borrow(|canister_data| {
            canister_data
                .all_subnet_orchestrator_canisters_list
                .contains(&canister_id)
        });

        if contains {
            Ok(RegisteredSubnetOrchestrator { canister_id })
        } else {
            Err("Canister Id is not found in platform orchestrator".into())
        }
    }

    pub fn get_canister_id(&self) -> Principal {
        self.canister_id
    }

    pub async fn deposit_cycles(&self) -> Result<(), String> {
        let (subnet_orchestrator_status_res,) = canister_status(CanisterIdRecord {
            canister_id: self.canister_id,
        })
        .await
        .map_err(|e| e.1)?;

        let subnet_orchestrator_cycles = subnet_orchestrator_status_res.cycles;

        if subnet_orchestrator_cycles > SUBNET_ORCHESTRATOR_CANISTER_CYCLES_THRESHOLD {
            return Ok(());
        }

        deposit_cycles(
            CanisterIdRecord {
                canister_id: self.canister_id,
            },
            SUBNET_ORCHESTRATOR_CANISTER_CYCLES_THRESHOLD,
        )
        .await
        .map_err(|e| e.1)
    }

    pub async fn make_logs_public(&self) -> Result<(), String> {
        update_settings(UpdateSettingsArgument {
            canister_id: self.canister_id,
            settings: CanisterSettings {
                log_visibility: Some(LogVisibility::Public),
                ..Default::default()
            },
        })
        .await
        .map_err(|e| e.1)
    }

    pub async fn make_logs_private(&self) -> Result<(), String> {
        update_settings(UpdateSettingsArgument {
            canister_id: self.canister_id,
            settings: CanisterSettings {
                log_visibility: Some(LogVisibility::Controllers),
                ..Default::default()
            },
        })
        .await
        .map_err(|e| e.1)
    }

    pub async fn make_individual_canister_logs_public(
        &self,
        individual_canister_id: Principal,
    ) -> Result<(), String> {
        let (res,) = ic_cdk::call(
            self.canister_id,
            "make_individual_canister_logs_public",
            (individual_canister_id,),
        )
        .await
        .map_err(|e| e.1)?;

        res
    }

    pub async fn make_individual_canister_logs_private(
        &self,
        individual_canister_id: Principal,
    ) -> Result<(), String> {
        let (res,) = ic_cdk::call(
            self.canister_id,
            "make_individual_canister_logs_private",
            (individual_canister_id,),
        )
        .await
        .map_err(|e| e.1)?;

        res
    }

    pub async fn provision_empty_canisters(&self, number_of_canisters: u64) -> Result<(), String> {
        ic_cdk::call::<_, ()>(
            self.canister_id,
            "provision_empty_canisters",
            (number_of_canisters,),
        )
        .await
        .map_err(|e| e.1)
    }

    pub async fn upgrade_individual_canisters_in_subnet_with_latest_wasm(
        &self,
    ) -> Result<(), String> {
        let individual_canister_wasm = CANISTER_DATA
            .with_borrow(|canister_data| canister_data.wasms.get(&WasmType::IndividualUserWasm))
            .unwrap();
        self.deposit_cycles().await?;

        let res: Result<(String,), String> = ic_cdk::call(
            self.canister_id,
            "start_upgrades_for_individual_canisters",
            (
                individual_canister_wasm.version.clone(),
                individual_canister_wasm.wasm_blob.clone(),
            ),
        )
        .await
        .map_err(|e| {
            format!(
                "Failed to start upgrades on {}. Error {}",
                self.canister_id, e.1
            )
        });

        match res {
            Ok((_str,)) => Ok(()),
            Err(e) => Err(e),
        }
    }
}
