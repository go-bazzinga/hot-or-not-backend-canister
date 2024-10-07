use std::{collections::HashMap, env, path::Path};

use candid::Principal;
use ic_test_state_machine_client::{CanisterSettings, StateMachine};
use shared_utils::{
    access_control::UserAccessRole,
    canister_specific::{
        post_cache::types::arg::PostCacheInitArgs, user_index::types::args::UserIndexInitArgs,
    },
    common::types::known_principal::{KnownPrincipalMap, KnownPrincipalType},
};

use crate::setup::test_constants::{
    get_canister_wasm, get_global_super_admin_principal_id, get_mock_canister_id_sns,
    v1::{
        CANISTER_INITIAL_CYCLES_FOR_NON_SPAWNING_CANISTERS,
        CANISTER_INITIAL_CYCLES_FOR_SPAWNING_CANISTERS,
    },
};

/// The path to the state machine binary to run the tests with
pub static STATE_MACHINE_BINARY: &str = "../../../ic-test-state-machine";

pub fn get_new_state_machine() -> StateMachine {
    let path = match env::var_os("DFX_IC_STATE_MACHINE_TESTS_PATH") {
        None => STATE_MACHINE_BINARY.to_string(),
        Some(path) => path
            .clone()
            .into_string()
            .unwrap_or_else(|_| panic!("Invalid string path for {path:?}")),
    };

    if !Path::new(&path).exists() {
        println!("
        Could not find state machine binary to run canister integration tests.

        I looked for it at {:?}. You can specify another path with the environment variable STATE_MACHINE_BINARY (note that I run from {:?}).

        Run the following command to get the binary:
            curl -sLO https://download.dfinity.systems/ic/a8da3aa23dc6f8f4708cb0cb8edce84c5bd8f225/binaries/x86_64-linux/ic-test-state-machine.gz
            gzip -d ic-test-state-machine.gz
            chmod +x ic-test-state-machine
        where $commit can be read from `.ic-commit` and $platform is 'x86_64-linux' for Linux and 'x86_64-darwin' for Intel/rosetta-enabled Darwin.
        ", &path, &env::current_dir().map(|x| x.display().to_string()).unwrap_or_else(|_| "an unknown directory".to_string()));
    }

    StateMachine::new(&path, false)
}

pub fn get_initialized_env_with_provisioned_known_canisters(
    state_machine: &StateMachine,
) -> KnownPrincipalMap {
    let canister_provisioner = |cycle_amount: u128| {
        let settings = Some(CanisterSettings {
            controllers: Some(vec![get_global_super_admin_principal_id()]),
            ..Default::default()
        });
        let canister_id = state_machine
            .create_canister_with_settings(settings, Some(get_global_super_admin_principal_id()));
        state_machine.add_cycles(canister_id, cycle_amount);
        canister_id
    };

    // * Provision canisters
    let mut known_principal_map_with_all_canisters = KnownPrincipalMap::default();
    known_principal_map_with_all_canisters.insert(
        KnownPrincipalType::UserIdGlobalSuperAdmin,
        get_global_super_admin_principal_id(),
    );
    known_principal_map_with_all_canisters.insert(
        KnownPrincipalType::CanisterIdPostCache,
        canister_provisioner(CANISTER_INITIAL_CYCLES_FOR_NON_SPAWNING_CANISTERS),
    );
    known_principal_map_with_all_canisters.insert(
        KnownPrincipalType::CanisterIdUserIndex,
        canister_provisioner(CANISTER_INITIAL_CYCLES_FOR_SPAWNING_CANISTERS),
    );

    known_principal_map_with_all_canisters.insert(
        KnownPrincipalType::CanisterIdSnsGovernance,
        get_mock_canister_id_sns(),
    );

    // * Install canisters
    let canister_installer = |canister_id: Principal, wasm_module: Vec<u8>, arg: Vec<u8>| {
        state_machine.install_canister(
            canister_id,
            wasm_module,
            arg,
            Some(get_global_super_admin_principal_id()),
        );
    };

    canister_installer(
        *known_principal_map_with_all_canisters
            .get(&KnownPrincipalType::CanisterIdPostCache)
            .unwrap(),
        get_canister_wasm(KnownPrincipalType::CanisterIdPostCache),
        candid::encode_one(PostCacheInitArgs {
            known_principal_ids: Some(known_principal_map_with_all_canisters.clone()),
            ..Default::default()
        })
        .unwrap(),
    );

    let mut user_index_access_control_map = HashMap::new();
    user_index_access_control_map.insert(
        get_global_super_admin_principal_id(),
        vec![
            UserAccessRole::CanisterAdmin,
            UserAccessRole::CanisterController,
        ],
    );

    canister_installer(
        *known_principal_map_with_all_canisters
            .get(&KnownPrincipalType::CanisterIdUserIndex)
            .unwrap(),
        get_canister_wasm(KnownPrincipalType::CanisterIdUserIndex),
        candid::encode_one(UserIndexInitArgs {
            known_principal_ids: Some(known_principal_map_with_all_canisters.clone()),
            access_control_map: Some(user_index_access_control_map),
            version: String::from("v1.0.0"),
            // V1 tests dont rely on PoP
            proof_of_participation: None,
        })
        .unwrap(),
    );

    let user_index_canister_id = known_principal_map_with_all_canisters
        .get(&KnownPrincipalType::CanisterIdUserIndex)
        .unwrap();

    provision_individual_user_canisters(state_machine, user_index_canister_id);

    known_principal_map_with_all_canisters
}

pub fn provision_individual_user_canisters(
    state_machine: &StateMachine,
    user_index_canister_id: &Principal,
) {
    state_machine.add_cycles(*user_index_canister_id, 10_000_000_000_000_000);
    let individual_user_template_wasm = include_bytes!(
        "../../../../../../target/wasm32-unknown-unknown/release/individual_user_template.wasm.gz"
    );
    state_machine
        .update_call(
            *user_index_canister_id,
            get_global_super_admin_principal_id(),
            "create_pool_of_individual_user_available_canisters",
            candid::encode_args(("v1.0.0", individual_user_template_wasm.to_vec())).unwrap(),
        )
        .unwrap();

    for _ in 0..100 {
        state_machine.tick();
    }
}

pub fn get_canister_id_of_specific_type_from_principal_id_map(
    principal_id_map: &KnownPrincipalMap,
    canister_type: KnownPrincipalType,
) -> Principal {
    *principal_id_map
        .get(&canister_type)
        .expect("Canister type not found in principal id map")
}
