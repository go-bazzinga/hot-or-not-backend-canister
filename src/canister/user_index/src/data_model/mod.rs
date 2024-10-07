use std::collections::{BTreeMap, HashSet};

use candid::{Deserialize, Principal};
use ic_stable_structures::StableBTreeMap;
use serde::Serialize;
use shared_utils::canister_specific::user_index::types::{
    BroadcastCallStatus, RecycleStatus, UpgradeStatus,
};
use shared_utils::common::participant_crypto::ProofOfParticipation;
use shared_utils::common::types::wasm::{CanisterWasm, WasmType};

use self::memory::get_wasm_memory;
use self::{configuration::Configuration, memory::Memory};

pub mod configuration;
pub mod memory;

const fn _default_true() -> bool {
    return true;
}

fn _default_vec_principal() -> HashSet<Principal> {
    return HashSet::new();
}

#[derive(Serialize, Deserialize)]
pub struct CanisterData {
    pub configuration: Configuration,
    pub last_run_upgrade_status: UpgradeStatus,
    pub allow_upgrades_for_individual_canisters: bool,
    pub available_canisters: HashSet<Principal>,
    #[serde(default)]
    pub backup_canister_pool: HashSet<Principal>,
    pub user_principal_id_to_canister_id_map: BTreeMap<Principal, Principal>,
    pub unique_user_name_to_user_principal_id_map: BTreeMap<String, Principal>,
    #[serde(skip, default = "_empty_wasms")]
    pub wasms: StableBTreeMap<WasmType, CanisterWasm, Memory>,
    #[serde(default)]
    pub recycle_status: RecycleStatus,
    #[serde(default)]
    pub last_broadcast_call_status: BroadcastCallStatus,
    #[serde(default)]
    pub proof_of_participation: Option<ProofOfParticipation>,
}

impl Default for CanisterData {
    fn default() -> Self {
        Self {
            configuration: Default::default(),
            last_run_upgrade_status: Default::default(),
            allow_upgrades_for_individual_canisters: Default::default(),
            available_canisters: Default::default(),
            user_principal_id_to_canister_id_map: Default::default(),
            unique_user_name_to_user_principal_id_map: Default::default(),
            wasms: _empty_wasms(),
            backup_canister_pool: Default::default(),
            recycle_status: Default::default(),
            last_broadcast_call_status: Default::default(),
            proof_of_participation: None,
        }
    }
}

fn _empty_wasms() -> StableBTreeMap<WasmType, CanisterWasm, Memory> {
    StableBTreeMap::init(get_wasm_memory())
}
