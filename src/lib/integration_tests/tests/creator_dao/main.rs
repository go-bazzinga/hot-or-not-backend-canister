pub mod types;
pub mod airdrop;

use ic_sns_governance::pb::v1::{
    manage_neuron, neuron, Account, GetMode, GetModeResponse, ListNeurons, ListNeuronsResponse, ManageNeuron, ManageNeuronResponse
};
use ic_sns_init::pb::v1::{
    sns_init_payload::InitialTokenDistribution, AirdropDistribution, DeveloperDistribution,
    FractionalDeveloperVotingPower, NeuronDistribution, SnsInitPayload, SwapDistribution,
    TreasuryDistribution,
};
use ic_sns_swap::pb::v1::{
    new_sale_ticket_response, NeuronBasketConstructionParameters, NewSaleTicketRequest, NewSaleTicketResponse, RefreshBuyerTokensRequest,
};
use sha2::{Digest, Sha256};
use shared_utils::canister_specific::individual_user_template::types::profile::UserCanisterDetails;
use shared_utils::canister_specific::individual_user_template::types::session::SessionType;
use shared_utils::constant::GLOBAL_SUPER_ADMIN_USER_ID;
use test_utils::setup::env::pocket_ic_env::{execute_query, execute_update, execute_update_multi, execute_update_no_res, execute_update_no_res_multi};
use std::time::{Duration, UNIX_EPOCH};
use std::{collections::HashMap, fmt::Debug, str::FromStr, time::SystemTime, vec};

use candid::{CandidType, Encode, Nat, Principal};
use ic_base_types::PrincipalId;
use ic_sns_wasm::init::SnsWasmCanisterInitPayload;
use icp_ledger::Subaccount;
use pocket_ic::PocketIc;
use serde::{Deserialize, Serialize};
use shared_utils::{
    canister_specific::individual_user_template::{
        types::cdao::DeployedCdaoCanisters, types::error::CdaoDeployError,
    },
    common::types::known_principal::KnownPrincipalType,
    constant::SNS_WASM_W_PRINCIPAL_ID,
};
use test_utils::setup::{
    env::pocket_ic_env::get_new_pocket_ic_env,
    test_constants::{
        get_global_super_admin_principal_id, get_mock_user_alice_principal_id,
        get_mock_user_charlie_principal_id,
    },
};

pub const ICP_LEDGER_CANISTER_ID: &str = "ryjl3-tyaaa-aaaaa-aaaba-cai";
pub const ICP_INDEX_CANISTER_ID: &str = "qhbym-qaaaa-aaaaa-aaafq-cai";

#[derive(CandidType, Deserialize, PartialEq, Eq, Hash, Serialize, Clone)]
struct Wasm {
    wasm: Vec<u8>,
    proposal_id: Option<u64>,
    canister_type: i32,
}

impl Debug for Wasm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Wasm")
            .field("proposal_id", &self.proposal_id)
            .field("canister_type", &self.canister_type)
            .finish()
    }
}

#[derive(CandidType, Deserialize, PartialEq, Eq, Hash, Serialize, Clone, Debug)]
struct AddWasmPayload {
    hash: Vec<u8>,
    wasm: Option<Wasm>,
}

#[derive(CandidType, Serialize, Deserialize, Debug, Clone)]
pub struct AddWasmResultRecord {
    pub result: Option<ResultVariant>,
}

#[derive(CandidType, Serialize, Deserialize, Debug, Clone)]
pub enum ResultVariant {
    Error(ErrorRecord),
    Hash(Vec<u8>),
}

#[derive(CandidType, Serialize, Deserialize, Debug, Clone)]
pub struct ErrorRecord {
    pub message: String,
}

fn tokens_to_e8s(tokens: u64) -> Nat {
    Nat::from(tokens) * 1e8 as u64
}

fn add_wasm(pic: &PocketIc, wasm_canister: Principal, wasm_file: &[u8], canister_type: u32) {
    let mut hasher = Sha256::new();
    hasher.update(wasm_file);
    let file_hash = hasher.finalize();

    let wasm_data = AddWasmPayload {
        hash: file_hash.to_vec(),
        wasm: Some(Wasm {
            wasm: wasm_file.to_vec(),
            proposal_id: None,
            canister_type: canister_type as i32,
        }),
    };

    ic_cdk::println!(
        "Wasm data: {:?}\nType: {}, Hash: {}",
        wasm_data,
        canister_type,
        hex::encode(file_hash)
    );

    let super_admin = get_global_super_admin_principal_id();

    let res: AddWasmResultRecord = execute_update(
        pic,
        super_admin,
        wasm_canister,
        "add_wasm",
        &wasm_data,
    );

    assert!(matches!(res, AddWasmResultRecord { result: Some(ResultVariant::Hash(_)) }));
}

struct CDaoHarness {
    pub pic: PocketIc,
    pub subnet_orchestrator: Principal,
}

impl CDaoHarness {
    pub fn init() -> Self {
        let (pic, known_principal) = get_new_pocket_ic_env();
        let platform_canister_id = known_principal
            .get(&KnownPrincipalType::CanisterIdPlatformOrchestrator)
            .cloned()
            .unwrap();

        let super_admin = get_global_super_admin_principal_id();
        let application_subnets = pic.topology().get_app_subnets();
        let charlie_global_admin = get_mock_user_charlie_principal_id();

        execute_update_no_res(
            &pic,
            super_admin,
            platform_canister_id,
            "add_principal_as_global_admin",
            &charlie_global_admin
        );

        execute_update_no_res_multi(
            &pic,
            super_admin,
            platform_canister_id,
            "update_global_known_principal",
            (
                KnownPrincipalType::CanisterIdSnsWasm,
                Principal::from_text(SNS_WASM_W_PRINCIPAL_ID).unwrap()
            )
        );

        let subnet_orchestrator: Principal = execute_update::<_, Result<_, String>>(
            &pic,
            charlie_global_admin,
            platform_canister_id,
            "provision_subnet_orchestrator_canister",
            &application_subnets[1]
        ).unwrap();

        for _ in 0..50 {
            pic.tick();
        }

        let sns_wasm_w_canister_wasm = include_bytes!("../../../../../wasms/sns-wasm-canister.wasm");
        let sns_wasm_w_canister_id = Principal::from_text(SNS_WASM_W_PRINCIPAL_ID).unwrap();

        let res_principal = pic.create_canister_with_id(
            Some(super_admin),
            None,
            sns_wasm_w_canister_id
        ).unwrap();

        assert_eq!(res_principal, sns_wasm_w_canister_id);

        let sns_wasm_init = SnsWasmCanisterInitPayload {
            sns_subnet_ids: vec![],
            access_controls_enabled: false,
            allowed_principals: vec![]
        };

        pic.install_canister(
            sns_wasm_w_canister_id,
            sns_wasm_w_canister_wasm.to_vec(),
            Encode!(&sns_wasm_init).unwrap(),
            Some(super_admin),
        );

        add_wasm(
            &pic,
            sns_wasm_w_canister_id,
            include_bytes!("../../../../../wasms/root.wasm.gz"),
            1,
        );

        add_wasm(
            &pic,
            sns_wasm_w_canister_id,
            include_bytes!("../../../../../wasms/governance.wasm.gz"),
            2,
        );

        add_wasm(
            &pic,
            sns_wasm_w_canister_id,
            include_bytes!("../../../../../wasms/ledger.wasm.gz"),
            3,
        );

        add_wasm(
            &pic,
            sns_wasm_w_canister_id,
            include_bytes!("../../../../../wasms/swap.wasm.gz"),
            4,
        );

        add_wasm(
            &pic,
            sns_wasm_w_canister_id,
            include_bytes!("../../../../../wasms/archive.wasm.gz"),
            5,
        );

        add_wasm(
            &pic,
            sns_wasm_w_canister_id,
            include_bytes!("../../../../../wasms/index.wasm.gz"),
            6,
        );

        for _ in 0..50 {
            pic.tick();
        }


        Self {
            pic,
            subnet_orchestrator,
        }
    }

    pub fn provision_individual_canister(&self, owner: Principal, referrer: Option<UserCanisterDetails>) -> UserCanisterDetails {
        let new_canister = execute_update::<_, Result<_, String>>(
            &self.pic,
            owner,
            self.subnet_orchestrator,
            "get_requester_principals_canister_id_create_if_not_exists",
            &()
        ).unwrap();
        let new_canister_details = UserCanisterDetails {
            profile_owner: owner,
            user_canister_id: new_canister,
        };
    
        // XX: hack, this is a canister bug, where individual user canister uses mainnet admin id, even in testing
        // for certain update calls
        let super_admin = Principal::from_str(GLOBAL_SUPER_ADMIN_USER_ID).unwrap();
        let update_res = execute_update::<_, Result<String, String>>(
            &self.pic,
            super_admin,
            new_canister,
            "update_session_type",
            &SessionType::RegisteredSession
        );
        match update_res {
            Ok(_) => (),
            Err(e) if e == "Session Already marked as Registered Session" => (),
            e => panic!("{e:?}"),
        };

        let Some(referrer) = referrer.as_ref() else {
            return new_canister_details;
        };

        execute_update::<_, Result<String, String>>(
            &self.pic,
            owner,
            new_canister,
            "update_referrer_details",
            &referrer
        ).unwrap();

        execute_update::<_, Result<(), String>>(
            &self.pic,
            owner,
            new_canister,
            "receive_reward_for_being_referred",
            &()
        ).unwrap();

        // ensure that the referrer also gets the reward
        for _ in 0..20 {
            self.pic.tick();
        }

        new_canister_details
    }

    pub fn icrc1_balance(&self, token_ledger: Principal, acc: Principal) -> Nat {
        execute_query(
            &self.pic,
            acc,
            token_ledger,
            "icrc1_balance_of",
            &types::Icrc1BalanceOfArg {
                owner: acc,
                subaccount: None,
            }
        )
    }

    pub fn create_new_token(&self, user: UserCanisterDetails) -> DeployedCdaoCanisters {
        let start = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap();
        let tx_fee = 1u64;

        let super_admin = get_global_super_admin_principal_id();
        let sns_init = SnsInitPayload {
            confirmation_text: Some("GET RICH QUICK".into()),
            transaction_fee_e8s: Some(tx_fee),
            token_name: Some("Simulation Governance".to_string()),
            token_symbol: Some("SIMG".to_string()),
            proposal_reject_cost_e8s: Some(1u64),
            neuron_minimum_stake_e8s: Some(2u64),
            fallback_controller_principal_ids: vec![super_admin.to_string()],
            logo: Some("data:image/png;base64,iVBORw0".to_string()),
            url: Some("https://google.com".to_string()),
            name: Some("Simulation Governance".to_string()),
            description: Some("Simulation gov desc".to_string()),
            neuron_minimum_dissolve_delay_to_vote_seconds: Some(1),
            initial_reward_rate_basis_points: Some(30u64),
            final_reward_rate_basis_points: Some(20u64),
            reward_rate_transition_duration_seconds: Some(1u64),
            max_dissolve_delay_seconds: Some(5u64),
            max_neuron_age_seconds_for_age_bonus: Some(1u64),
            max_dissolve_delay_bonus_percentage: Some(10u64),
            max_age_bonus_percentage: Some(10u64),
            initial_voting_period_seconds: Some(86401u64),
            wait_for_quiet_deadline_increase_seconds: Some(1u64),
            restricted_countries: None,
            dapp_canisters: None,
            min_participants: Some(1),
            min_icp_e8s: None,
            max_icp_e8s: None,
            min_direct_participation_icp_e8s: Some(15u64),
            min_participant_icp_e8s: Some(2000u64),
            max_direct_participation_icp_e8s: Some(100_000_000u64),
            max_participant_icp_e8s: Some(100_000_000u64),
            swap_start_timestamp_seconds: Some(start.as_secs()),
            swap_due_timestamp_seconds: Some(start.as_secs() + 300),
            neuron_basket_construction_parameters: Some(NeuronBasketConstructionParameters {
                count: 2,
                dissolve_delay_interval_seconds: 2,
            }),
            nns_proposal_id: Some(1),
            neurons_fund_participation: Some(false),
            neurons_fund_participants: None,
            token_logo: Some("data:image/png;base64,iVBORw0".to_string()),
            neurons_fund_participation_constraints: None,
            initial_token_distribution: Some(InitialTokenDistribution::FractionalDeveloperVotingPower(
                FractionalDeveloperVotingPower {
                    airdrop_distribution: Some(AirdropDistribution {
                        airdrop_neurons: vec![],
                    }),
                    developer_distribution: Some(DeveloperDistribution {
                        developer_neurons: vec![
                            NeuronDistribution {
                                controller: Some(
                                    user.profile_owner.into()
                                ),
                                stake_e8s: 1_250_001 * (1e8 as u64),
                                memo: 0,
                                dissolve_delay_seconds: 0,
                                vesting_period_seconds: None,
                            },
                            NeuronDistribution {
                                controller: Some(
                                    user.profile_owner.into()
                                ),
                                stake_e8s: 1000 * (1e8 as u64),
                                memo: 1,
                                dissolve_delay_seconds: 2,
                                vesting_period_seconds: None,
                            },
                        ],
                    }),
                    treasury_distribution: Some(TreasuryDistribution {
                        total_e8s: 0,
                    }),
                    swap_distribution: Some(SwapDistribution {
                        total_e8s: 1_251_004 * (1e8 as u64),
                        initial_swap_amount_e8s: 1_251_004 * (1e8 as u64),
                    }),
                },
            )),
        };

        let deploy_res: DeployedCdaoCanisters = execute_update_multi::<_, Result<_, CdaoDeployError>>(
            &self.pic,
            user.profile_owner,
            user.user_canister_id,
            "deploy_cdao_sns",
            (
                sns_init,
                300u64
            )
        ).unwrap();

        let res = execute_update::<_, NewSaleTicketResponse>(
            &self.pic,
            super_admin,
            deploy_res.swap,
            "new_sale_ticket",
            &NewSaleTicketRequest {
                amount_icp_e8s: 1000000,
                subaccount: None,
            }
        ).result.unwrap();
        assert!(matches!(res, new_sale_ticket_response::Result::Ok(_)));

        let super_adm_sub_acc = Subaccount::from(&PrincipalId(super_admin));
        let transfer_icp_args = types::Transaction {
            memo: Some(vec![0]),
            amount: Nat::from(1000000u64),
            fee: None,
            from_subaccount: None,
            to: types::Recipient {
                owner: deploy_res.swap,
                subaccount: Some(super_adm_sub_acc.to_vec()),
            },
            created_at_time: None,
        };

        let res = execute_update::<_, types::TransferResult>(
            &self.pic,
            super_admin,
            Principal::from_text(ICP_LEDGER_CANISTER_ID).unwrap(),
            "icrc1_transfer",
            &transfer_icp_args,
        );
        assert!(matches!(res, types::TransferResult::Ok(_)), "{res:?}");

        execute_update_no_res(
            &self.pic,
            super_admin,
            deploy_res.swap,
            "refresh_buyer_tokens",
            &RefreshBuyerTokensRequest {
                buyer: super_admin.to_string(),
                confirmation_text: Some("GET RICH QUICK".to_string())
            }
        );

        // wait for governance to initialize
        self.pic.advance_time(Duration::from_secs(301));
        loop {
            self.pic.tick();
            let GetModeResponse { mode } = execute_query(
                &self.pic,
                Principal::anonymous(),
                deploy_res.governance,
                "get_mode",
                &GetMode {}
            );
            let mode = mode.unwrap();
            assert!(mode > 0 && mode <= 2);
            // mode 1 == MODE_NORMAL
            if mode == 1 {
                break;
            }
        }

        let neurons: ListNeuronsResponse = execute_query(
            &self.pic,
            super_admin,
            deploy_res.governance,
            "list_neurons",
            &ListNeurons {
                of_principal: Some(PrincipalId(user.profile_owner)),
                limit: 2,
                start_page_at: None
            }
        );
    
        let neurons = neurons.neurons;
        let mut ix = 0;
        if neurons[1].dissolve_state.is_some() {
            if let Some(neuron::DissolveState::DissolveDelaySeconds(x)) =
                neurons[1].dissolve_state.as_ref()
            {
                if *x == 0 {
                    ix = 1;
                }
            }
        }
        let neuron_id = neurons[ix].id.as_ref().unwrap().id.clone();
        let amount = neurons[ix].cached_neuron_stake_e8s;
        let manage_neuron = ManageNeuron {
            subaccount: neuron_id,
            command: Some(manage_neuron::Command::Disburse(manage_neuron::Disburse {
                to_account: Some(Account {
                    owner: Some(PrincipalId(user.profile_owner)),
                    subaccount: None,
                }),
                amount: Some(manage_neuron::disburse::Amount { e8s: amount })
            }))
        };

        let res: ManageNeuronResponse = execute_update(
            &self.pic,
            user.profile_owner,
            deploy_res.governance,
            "manage_neuron",
            &manage_neuron,
        );
        res.command.unwrap();


        let bal = self.icrc1_balance(
            deploy_res.ledger,
            user.profile_owner
        );

        let expected_balance = tokens_to_e8s(1_250_001) - tx_fee;
        assert_eq!(bal, expected_balance);

        // pass 10% of the overall supply to the user's canister
        // i.e pass 20% of the user's share as user share is 50%
        let canister_share = (tokens_to_e8s(1_250_001) * 20u32) / 100u32;
        let res = execute_update::<_, types::TransferResult>(
            &self.pic,
            user.profile_owner,
            deploy_res.ledger,
            "icrc1_transfer",
            &types::TransferArg {
                to: types::Account {
                    owner: user.user_canister_id,
                    subaccount: None,
                },
                fee: None,
                memo: None,
                from_subaccount: None,
                created_at_time: None,
                amount: canister_share,
            }
        );
        assert!(matches!(res, types::TransferResult::Ok(_)));

        execute_update::<_, Result<(), String>>(
            &self.pic,
            user.profile_owner,
            user.user_canister_id,
            "distribute_newly_created_token_to_token_chain",
            &deploy_res
        ).unwrap();

        deploy_res
    }
}

#[test]
fn creator_dao_tests() {
    let harness = CDaoHarness::init();

    let alice_principal = get_mock_user_alice_principal_id();
    let alice = harness.provision_individual_canister(alice_principal, None);


    let sns_wasm_w_canister_id: Option<Principal> = execute_query(
        &harness.pic,
        alice_principal,
        alice.user_canister_id,
        "get_well_known_principal_value",
        &KnownPrincipalType::CanisterIdSnsWasm,
    );
    let res: HashMap<String, String> = execute_query(
        &harness.pic,
        Principal::anonymous(),
        sns_wasm_w_canister_id.unwrap(),
        "get_latest_sns_version_pretty",
        &()
    );
    assert_eq!(res.len(), 6);

    harness.create_new_token(alice);
}

