use std::cell::RefCell;
use std::time::Duration;
use crate::common::types::known_principal::{KnownPrincipalMap, KnownPrincipalType};
use candid::Principal;
pub const INDIVIDUAL_USER_CANISTER_RECHARGE_AMOUNT: u128 = 1_000_000_000_000; // 1T Cycles
pub const CYCLES_THRESHOLD_TO_INITIATE_RECHARGE: u128 = 100_000_000_000; // 0.1T Cycles

pub const SUBNET_ORCHESTRATOR_CANISTER_INITIAL_CYCLES: u128 = 2_500_000_000_000_000; //2.5kT Cycles
pub const SUBNET_ORCHESTRATOR_CANISTER_CYCLES_THRESHOLD: u128 = 1_000_000_000_000_000; //1kT Cycles
pub const POST_CACHE_CANISTER_CYCLES_RECHARGE_AMOUMT: u128 = 5_000_000_000_000; //5T Cycles
pub const POST_CACHE_CANISTER_CYCLES_THRESHOLD: u128 = 2_000_000_000_000; //2T Cycles
pub const USER_SNS_CANISTER_INITIAL_CYCLES: u128 = 500_000_000_000; //0.5T Cycles
pub const PAGE_SIZE_RECHARGE_DIVIDER: u128 = 500; // 500 pages (recharge by page_size/page_size_recharge_divider * recharge_amout)

pub const MAX_USERS_IN_FOLLOWER_FOLLOWING_LIST: u64 = 10000;
pub const MAX_POSTS_IN_ONE_REQUEST: u64 = 100;
pub const HOME_FEED_DIFFERENCE_TO_INITIATE_SYNCHRONISATION: u64 = 100;
pub const HOT_OR_NOT_FEED_DIFFERENCE_TO_INITIATE_SYNCHRONISATION: u64 = 100;

const BACKUP_INDIVIDUAL_USER_CANISTER_BATCH_SIZE: u64 = 7_000;
const BACKUP_INDIVIDUAL_USER_CANISTER_THRESHOLD: u64 = 3_000;
const INDIVIDUAL_USER_CANISTER_SUBNET_BATCH_SIZE: u64 = 3_000;
const INDIVIDUAL_USER_CANISTER_SUBNET_THRESHOLD: u64 = 1_000;

const TEST_BACKUP_INDIVIDUAL_USER_CANISTER_BATCH_SIZE: u64 = 10;
const TEST_INDIVIDUAL_USER_CANISTER_SUBNET_BATCH_SIZE: u64 = 20;
const TEST_BACKUP_INDIVIDUAL_USER_CANISTER_THRESHOLD: u64 = 1;
const TEST_INDIVIDUAL_USER_CANISTER_SUBNET_THRESHOLD: u64 = 1;

pub const INDIVIDUAL_USER_CANISTER_SUBNET_MAX_CAPACITY: u64 = 50_000;

//  Cycles costs of IC
pub const DEFAULT_FREEZING_THRESHOLD: u128 = 30; // 30 days
pub const BASE_COST_FOR_INGRESS_MESSAGE: u128 = 1_200_000; //1.2M cycles
pub const BASE_COST_FOR_EXECUTION: u128 = 590_000; // 590K cycles
pub const COST_PER_BYTE_FOR_INGRESS_MESSAGE: u128 = 2_000; //2k cycles
pub const COST_PER_BILLION_INSTRUCTION_EXECUTED: u128 = 400_000_000; //400M cycles for per 1B instructions
pub const ASSUMED_NUMBER_OF_INGRESS_CALL_PER_SEC: u128 = 1;
pub const ASSUMED_BYTES_PER_INGRESS_CALL: u128 = 10; // 10 bytes
pub const ASSUMED_NUMBER_OF_INSTRUCTIONS_PER_INGRESS_CALL: u128 = 500_000; //0.5M instructions
pub const RESERVED_NUMBER_OF_INSTRUCTIONS_FOR_INSTALL_CODE: u128 = 200_000_000_000; //200B instructions
pub const THRESHOLD_NUMBER_OF_DAYS_TO_KEEP_CANISTER_RUNNING: u128 = 1;
pub const MAX_NUMBER_OF_DAYS_TO_KEEP_CANISTER_RUNNING: u128 = 3;

pub const TIMER_DURATION: Duration = Duration::from_secs(60 * 60); // 60 minutes

pub fn get_backup_individual_user_canister_batch_size() -> u64 {
    match option_env!("DFX_NETWORK") {
        Some(val) => {
            if val == "local" {
                TEST_BACKUP_INDIVIDUAL_USER_CANISTER_BATCH_SIZE
            } else {
                BACKUP_INDIVIDUAL_USER_CANISTER_BATCH_SIZE
            }
        }
        None => BACKUP_INDIVIDUAL_USER_CANISTER_BATCH_SIZE,
    }
}

pub fn get_backup_individual_user_canister_threshold() -> u64 {
    match option_env!("DFX_NETWORK") {
        Some(val) => {
            if val == "local" {
                TEST_BACKUP_INDIVIDUAL_USER_CANISTER_THRESHOLD
            } else {
                BACKUP_INDIVIDUAL_USER_CANISTER_THRESHOLD
            }
        }
        None => BACKUP_INDIVIDUAL_USER_CANISTER_THRESHOLD,
    }
}

pub fn get_individual_user_canister_subnet_threshold() -> u64 {
    match option_env!("DFX_NETWORK") {
        Some(val) => {
            if val == "local" {
                TEST_INDIVIDUAL_USER_CANISTER_SUBNET_THRESHOLD
            } else {
                INDIVIDUAL_USER_CANISTER_SUBNET_THRESHOLD
            }
        }
        None => INDIVIDUAL_USER_CANISTER_SUBNET_THRESHOLD,
    }
}

pub fn get_individual_user_canister_subnet_batch_size() -> u64 {
    match option_env!("DFX_NETWORK") {
        Some(val) => {
            if val == "local" {
                TEST_INDIVIDUAL_USER_CANISTER_SUBNET_BATCH_SIZE
            } else {
                INDIVIDUAL_USER_CANISTER_SUBNET_BATCH_SIZE
            }
        }
        None => INDIVIDUAL_USER_CANISTER_SUBNET_BATCH_SIZE,
    }
}

// * Important Principal IDs
pub const YRAL_POST_CACHE_CANISTER_ID: &str = "zyajx-3yaaa-aaaag-acoga-cai";
pub const GOVERNANCE_CANISTER_ID: &str = "6wcax-haaaa-aaaaq-aaava-cai";
pub const NNS_CYCLE_MINTING_CANISTER: &str = "rkp4c-7iaaa-aaaaa-aaaca-cai";
pub const NNS_LEDGER_CANISTER_ID: &str = "ryjl3-tyaaa-aaaaa-aaaba-cai";
pub const SNS_WASM_W_PRINCIPAL_ID: &'static str = "qaa6y-5yaaa-aaaaa-aaafa-cai";
pub const GLOBAL_SUPER_ADMIN_USER_ID: &str =
    "7gaq2-4kttl-vtbt4-oo47w-igteo-cpk2k-57h3p-yioqe-wkawi-wz45g-jae";
pub const RECLAIM_CANISTER_PRINCIPAL_ID: &str =
    "7gaq2-4kttl-vtbt4-oo47w-igteo-cpk2k-57h3p-yioqe-wkawi-wz45g-jae";

pub fn get_global_super_admin_principal_id_v1(
    well_known_canisters: KnownPrincipalMap,
) -> Principal {
    match option_env!("DFX_NETWORK") {
        Some("ic") => {
            Principal::from_text("7gaq2-4kttl-vtbt4-oo47w-igteo-cpk2k-57h3p-yioqe-wkawi-wz45g-jae")
                .unwrap()
        }
        _ => *well_known_canisters
            .get(&KnownPrincipalType::UserIdGlobalSuperAdmin)
            .expect("USER ID for global super admin not found"),
    }
}
