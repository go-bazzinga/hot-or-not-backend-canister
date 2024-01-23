use core::time;
use std::{collections::{HashMap, HashSet}, time::SystemTime};

use candid::{CandidType, Principal};
use ic_cdk::api::{management_canister::provisional::CanisterSettings, time};
use pocket_ic::{PocketIcBuilder, WasmResult};
use serde::{Deserialize, Serialize};
use ic_ledger_types::{AccountIdentifier, BlockIndex, Tokens, DEFAULT_SUBACCOUNT};
use shared_utils::{canister_specific::platform_orchestrator::{self, types::args::PlatformOrchestratorInitArgs}, common::{types::known_principal::KnownPrincipalMap, utils::system_time}, constant::{NNS_CYCLE_MINTING_CANISTER, NNS_LEDGER_CANISTER_ID}};
use test_utils::setup::test_constants::{get_global_super_admin_principal_id, v1::CANISTER_INITIAL_CYCLES_FOR_SPAWNING_CANISTERS};

pub type CanisterId = Principal;

#[derive(CandidType, Serialize)]
pub enum ChangeIndexId {
    Unset,
    SetTo(Principal),
}

#[derive(CandidType, Serialize)]
pub struct Config {
    /// The maximum number of transactions
    /// returned by the [icrc3_get_transactions]
    /// endpoint
    pub max_transactions_per_request: u64,

    /// The principal of the index canister
    /// for this ledger
    pub index_id: Option<Principal>,
}
#[derive(CandidType, Serialize)]
pub struct UpgradeArgs {
    pub max_transactions_per_request: Option<u64>,
    pub change_index_id: Option<ChangeIndexId>,
}

#[derive(CandidType,Serialize)]
enum LedgerArgs {
    Init(Config),
    Upgrade(Option<UpgradeArgs>)
}

#[derive(CandidType)]
struct NnsLedgerCanisterInitPayload {
    minting_account: String,
    initial_values: HashMap<String, Tokens>,
    send_whitelist: HashSet<CanisterId>,
    transfer_fee: Option<Tokens>,
}

// pub struct CyclesCanisterInitPayload {
//     pub ledger_canister_id: Option<CanisterId>,
//     pub governance_canister_id: Option<CanisterId>,
//     pub minting_account_id: Option<AccountIdentifier>,
//     pub last_purged_notification: Option<BlockIndex>,
//     pub exchange_rate_canister: Option<ExchangeRateCanister>,
//     pub cycles_ledger_canister_id: Option<CanisterId>,                                                              
// }

#[derive(CandidType)]
struct AuthorizedSubnetWorks {
    who: Option<Principal>,
    subnets: Vec<Principal>
}


#[derive(CandidType)]
struct CyclesMintingCanisterInitPayload {
    ledger_canister_id: CanisterId,
    governance_canister_id: CanisterId,
    minting_account_id: Option<String>,
    last_purged_notification: Option<BlockIndex>,
}

#[derive(CandidType, Deserialize, Clone, Serialize, Debug)]
pub struct UpgradeStatus {
    pub version_number: u64,
    pub last_run_on: SystemTime,
    pub successful_upgrade_count: u32,
    pub failed_canister_ids: Vec<(Principal, Principal, String)>,
    #[serde(default)]
    pub version: String,
}

#[test]
fn provision_subnet_orchestrator_canister() {
    let pocket_ic = PocketIcBuilder::new()
        .with_nns_subnet()
        .with_application_subnet()
        .with_system_subnet()
        .build();

   
    let super_admin = get_global_super_admin_principal_id();

    let application_subnets = pocket_ic.topology().get_app_subnets();

    let platform_canister_id = pocket_ic.create_canister_with_settings(Some(super_admin), Some(CanisterSettings {controllers: Some(vec![super_admin]), compute_allocation: None, memory_allocation: None, freezing_threshold: None}));
    pocket_ic.add_cycles(platform_canister_id, CANISTER_INITIAL_CYCLES_FOR_SPAWNING_CANISTERS);
    let platform_orchestrator_wasm = include_bytes!("../../../../../target/wasm32-unknown-unknown/release/platform_orchestrator.wasm.gz");
    let subnet_orchestrator_canister_wasm = include_bytes!("../../../../../target/wasm32-unknown-unknown/release/user_index.wasm.gz");
    let platform_orchestrator_init_args = PlatformOrchestratorInitArgs {
        version: "v1.0.0".into()
    };
    pocket_ic.install_canister(platform_canister_id, platform_orchestrator_wasm.into(), candid::encode_one(platform_orchestrator_init_args).unwrap(), Some(super_admin));
    pocket_ic.add_cycles(platform_canister_id, CANISTER_INITIAL_CYCLES_FOR_SPAWNING_CANISTERS);



    //Ledger Canister
    let minting_account = AccountIdentifier::new(&super_admin, &DEFAULT_SUBACCOUNT);
    let ledger_canister_wasm = include_bytes!("../../ledger-canister.wasm");
    let ledger_canister_id = pocket_ic.create_canister_with_id(Some(super_admin), None, Principal::from_text(NNS_LEDGER_CANISTER_ID).unwrap()).unwrap();
    let icp_ledger_init_args = NnsLedgerCanisterInitPayload {
        minting_account: minting_account.to_string(),
        initial_values: HashMap::new(),
        send_whitelist: HashSet::new(),
        transfer_fee: Some(Tokens::from_e8s(10_000)),
    };
    pocket_ic.install_canister(ledger_canister_id, ledger_canister_wasm.into(), candid::encode_one(icp_ledger_init_args).unwrap(), Some(super_admin));
    

    //Cycle Minting Canister
    let cycle_minting_canister_wasm = include_bytes!("../../cycles-minting-canister.wasm");
    let cycle_minting_canister_id = pocket_ic.create_canister_with_id(Some(super_admin), None, Principal::from_text(NNS_CYCLE_MINTING_CANISTER).unwrap()).unwrap();
    pocket_ic.add_cycles(cycle_minting_canister_id, CANISTER_INITIAL_CYCLES_FOR_SPAWNING_CANISTERS);
    let cycles_minting_canister_init_args = CyclesMintingCanisterInitPayload {
        ledger_canister_id: ledger_canister_id,
        governance_canister_id: CanisterId::anonymous(),
        minting_account_id: Some(minting_account.to_string()),
        last_purged_notification: Some(0),
    };

    pocket_ic.install_canister(
        cycle_minting_canister_id,
        cycle_minting_canister_wasm.into(),
        candid::encode_one(cycles_minting_canister_init_args).unwrap(),
        Some(super_admin)
    );
    

    let authorized_subnetwork_list_args = AuthorizedSubnetWorks {
        who: Some(platform_canister_id),
        subnets: application_subnets.clone()
    };
    pocket_ic.update_call(
        cycle_minting_canister_id,
        CanisterId::anonymous(),
        "set_authorized_subnetwork_list",
        candid::encode_one(authorized_subnetwork_list_args).unwrap()
    ).unwrap();

    for i in 0..50 {
        pocket_ic.tick();
    }

    let subnet_orchestrator_canister_id: Principal = pocket_ic.update_call(
        platform_canister_id,
        super_admin,
        "provision_subnet_orchestrator_canister",
        candid::encode_args((application_subnets[0], subnet_orchestrator_canister_wasm)).unwrap()
    )
    .map(|res| {
        let canister_id: Principal = match res {
            WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
            _ => panic!("Canister call failed")
        };
        canister_id
    })
    .unwrap();

    for i in 0..30 {
        pocket_ic.tick();
    }


    //Check version Installed
    let last_upgrade_status: UpgradeStatus = pocket_ic.query_call(subnet_orchestrator_canister_id, Principal::anonymous(), "get_index_details_last_upgrade_status", candid::encode_one(()).unwrap())
    .map(|res| {
        let upgrade_status: UpgradeStatus = match res {
            WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
            _ => panic!("Canister call failed")
        };
        upgrade_status
    })
    .unwrap();

    assert_eq!(last_upgrade_status.version, "v1.0.1")


}