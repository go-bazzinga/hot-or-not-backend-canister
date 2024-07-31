use candid::Principal;
use ic_cdk::api::management_canister::provisional::CanisterId;
use shared_utils::common::types::known_principal::KnownPrincipalType;
use std::{fs::File, io::Read, path::PathBuf};

pub mod v1;

pub fn get_global_super_admin_principal_id() -> Principal {
    Principal::self_authenticating([0])
}

pub fn get_mock_user_alice_principal_id() -> Principal {
    Principal::self_authenticating([1])
}

pub fn get_mock_user_bob_principal_id() -> Principal {
    Principal::self_authenticating([2])
}

pub fn get_mock_user_charlie_principal_id() -> Principal {
    Principal::self_authenticating([3])
}

pub fn get_mock_user_dan_principal_id() -> Principal {
    Principal::self_authenticating([4])
}

pub fn get_mock_user_dholak_principal_id() -> Principal {
    Principal::self_authenticating([5])
}

pub fn get_mock_canister_id_post_cache() -> Principal {
    CanisterId::from_slice(&0_usize.to_ne_bytes())
}

pub fn get_mock_canister_id_root() -> Principal {
    CanisterId::from_slice(&2_usize.to_ne_bytes())
}

pub fn get_mock_canister_id_sns() -> Principal {
    CanisterId::from_slice(&3_usize.to_ne_bytes())
}

pub fn get_mock_canister_id_topic_cache() -> Principal {
    CanisterId::from_slice(&4_usize.to_ne_bytes())
}

pub fn get_mock_canister_id_user_index() -> Principal {
    CanisterId::from_slice(&5_usize.to_ne_bytes())
}

pub fn get_mock_canister_id_configuration() -> Principal {
    CanisterId::from_slice(&6_usize.to_ne_bytes())
}

pub fn get_mock_canister_id_data_backup() -> Principal {
    CanisterId::from_slice(&7_usize.to_ne_bytes())
}

pub fn get_mock_user_alice_canister_id() -> Principal {
    CanisterId::from_slice(&8_usize.to_ne_bytes())
}

pub fn get_mock_user_bob_canister_id() -> Principal {
    CanisterId::from_slice(&9_usize.to_ne_bytes())
}

pub fn get_mock_user_charlie_canister_id() -> Principal {
    CanisterId::from_slice(&10_usize.to_ne_bytes())
}

pub fn get_mock_user_dan_canister_id() -> Principal {
    CanisterId::from_slice(&11_usize.to_ne_bytes())
}

pub fn get_user_index_canister_wasm() -> Vec<u8> {
    let mut file_path = PathBuf::from(
        std::env::var("CARGO_MANIFEST_DIR")
            .expect("Failed to read CARGO_MANIFEST_DIR env variable"),
    );
    file_path.push("../../../../target/wasm32-unknown-unknown/release/user_index.wasm.gz");

    let mut file = File::open(&file_path)
        .unwrap_or_else(|_| panic!("Failed to open file: {}", file_path.to_str().unwrap()));
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).expect("Failed to read file");
    bytes
}

pub fn get_configuration_canister_wasm() -> Vec<u8> {
    let mut file_path = PathBuf::from(
        std::env::var("CARGO_MANIFEST_DIR")
            .expect("Failed to read CARGO_MANIFEST_DIR env variable"),
    );
    file_path.push("../../../../target/wasm32-unknown-unknown/release/configuration.wasm.gz");

    let mut file = File::open(&file_path)
        .unwrap_or_else(|_| panic!("Failed to open file: {}", file_path.to_str().unwrap()));
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).expect("Failed to read file");
    bytes
}

pub fn get_post_cache_canister_wasm() -> Vec<u8> {
    let mut file_path = PathBuf::from(
        std::env::var("CARGO_MANIFEST_DIR")
            .expect("Failed to read CARGO_MANIFEST_DIR env variable"),
    );
    file_path.push("../../../../target/wasm32-unknown-unknown/release/post_cache.wasm.gz");

    let mut file = File::open(&file_path)
        .unwrap_or_else(|_| panic!("Failed to open file: {}", file_path.to_str().unwrap()));
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).expect("Failed to read file");
    bytes
}

pub fn get_canister_wasm(canister_type: KnownPrincipalType) -> Vec<u8> {
    let mut bytes = Vec::new();

    let mut file_path = PathBuf::from(
        std::env::var("CARGO_MANIFEST_DIR")
            .expect("Failed to read CARGO_MANIFEST_DIR env variable"),
    );
    file_path.push("../../../target/wasm32-unknown-unknown/release");

    match canister_type {
        KnownPrincipalType::CanisterIdConfiguration => {
            file_path.push("configuration.wasm.gz");
            let mut file = File::open(&file_path)
                .unwrap_or_else(|_| panic!("Failed to open file: {}", file_path.to_str().unwrap()));
            file.read_to_end(&mut bytes).expect("Failed to read file");
        }
        KnownPrincipalType::CanisterIdDataBackup => {
            file_path.push("data_backup.wasm.gz");
            let mut file = File::open(&file_path)
                .unwrap_or_else(|_| panic!("Failed to open file: {}", file_path.to_str().unwrap()));
            file.read_to_end(&mut bytes).expect("Failed to read file");
        }
        KnownPrincipalType::CanisterIdPostCache => {
            file_path.push("post_cache.wasm.gz");
            let mut file = File::open(&file_path)
                .unwrap_or_else(|_| panic!("Failed to open file: {}", file_path.to_str().unwrap()));
            file.read_to_end(&mut bytes).expect("Failed to read file");
        }
        KnownPrincipalType::CanisterIdUserIndex => {
            file_path.push("user_index.wasm.gz");
            let mut file = File::open(&file_path)
                .unwrap_or_else(|_| panic!("Failed to open file: {}", file_path.to_str().unwrap()));
            file.read_to_end(&mut bytes).expect("Failed to read file");
        }

        _ => panic!("Canister type not supported"),
    };
    bytes
}
