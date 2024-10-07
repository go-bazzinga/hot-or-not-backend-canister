pub(crate) mod token;

use std::collections::VecDeque;

use candid::{Encode, Principal};
use futures::{
    stream::{FuturesOrdered, FuturesUnordered},
    TryStreamExt,
};
use ic_base_types::PrincipalId;
use ic_cdk::{
    api::management_canister::main::{
        create_canister, install_code, update_settings, CanisterInstallMode, CanisterSettings,
        CreateCanisterArgument, InstallCodeArgument, UpdateSettingsArgument,
    }, notify, query, update
};
use ic_sns_init::{pb::v1::SnsInitPayload, SnsCanisterIds};
use ic_sns_wasm::pb::v1::{GetWasmRequest, GetWasmResponse};
// use ic_sns_swap::pb::v1::{SettleNeuronsFundParticipationRequest, SettleNeuronsFundParticipationResponse};
use ic_nns_governance::neurons_fund::NeuronsFundSnapshot;
use ic_nns_governance::pb::v1::{
    SettleNeuronsFundParticipationRequest, SettleNeuronsFundParticipationResponse,
};
use shared_utils::{
    canister_specific::individual_user_template::{
        consts::CDAO_TOKEN_LIMIT,
        types::{airdrop::TokenClaim, cdao::DeployedCdaoCanisters, error::CdaoDeployError, session::SessionType},
    },
    common::types::known_principal::KnownPrincipalType,
    constant::{NNS_LEDGER_CANISTER_ID, USER_SNS_CANISTER_INITIAL_CYCLES},
};
use token::transfer_canister_token_to_user_principal;

use crate::{util::cycles::request_cycles_from_subnet_orchestrator, CANISTER_DATA};

use super::airdrop::is_balance_enough_for_airdrop;

#[update]
pub async fn settle_neurons_fund_participation(
    request: SettleNeuronsFundParticipationRequest,
) -> SettleNeuronsFundParticipationResponse {
    let response = Ok(NeuronsFundSnapshot::empty());
    let intermediate = SettleNeuronsFundParticipationResponse::from(response);
    SettleNeuronsFundParticipationResponse::from(intermediate)
}

async fn create_empty_canister(
    arg: CreateCanisterArgument,
) -> Result<PrincipalId, CdaoDeployError> {
    let can = create_canister(arg, USER_SNS_CANISTER_INITIAL_CYCLES).await?;
    Ok(PrincipalId(can.0.canister_id))
}

async fn install_canister_wasm(
    wasm: Vec<u8>,
    arg: Vec<u8>,
    canister_id: PrincipalId,
) -> Result<(), CdaoDeployError> {
    let install_arg = InstallCodeArgument {
        mode: CanisterInstallMode::Install,
        canister_id: canister_id.0,
        wasm_module: wasm,
        arg,
    };
    install_code(install_arg).await?;
    Ok(())
}

async fn update_controllers(
    canister_id: PrincipalId,
    controllers: Vec<Principal>,
) -> Result<(), CdaoDeployError> {
    update_settings(UpdateSettingsArgument {
        canister_id: canister_id.0,
        settings: CanisterSettings {
            controllers: Some(controllers),
            ..Default::default()
        },
    })
    .await?;
    Ok(())
}

#[query]
async fn deployed_cdao_canisters() -> Vec<DeployedCdaoCanisters> {
    CANISTER_DATA.with(|cdata| cdata.borrow().cdao_canisters.clone())
}

#[update]
async fn deploy_cdao_sns(
    init_payload: SnsInitPayload,
    swap_time: u64,
) -> Result<DeployedCdaoCanisters, CdaoDeployError> {
    // * access control
    let current_caller = ic_cdk::caller();
    let my_principal_id = CANISTER_DATA
        .with(|canister_data_ref_cell| canister_data_ref_cell.borrow().profile.principal_id);
    if my_principal_id != Some(current_caller) {
        return Err(CdaoDeployError::Unauthenticated);
    };

    let (registered, limit_hit) = CANISTER_DATA.with(|cdata| {
        let cdata = cdata.borrow();
        let registered = matches!(cdata.session_type, Some(SessionType::RegisteredSession));
        (registered, cdata.cdao_canisters.len() == CDAO_TOKEN_LIMIT)
    });

    if limit_hit {
        return Err(CdaoDeployError::TokenLimit(CDAO_TOKEN_LIMIT));
    }

    // Alloting 0.5T more to the user canister to be on safer side while deploying canisters
    request_cycles_from_subnet_orchestrator(6 * USER_SNS_CANISTER_INITIAL_CYCLES)
        .await
        .map_err(|e| CdaoDeployError::CycleError(e))?;

    let creation_arg = CreateCanisterArgument {
        settings: Some(CanisterSettings {
            controllers: Some(vec![ic_cdk::id()]),
            ..Default::default()
        }),
    };

    let canisters: Vec<_> = (0..5)
        .map(|_| create_empty_canister(creation_arg.clone()))
        .collect::<FuturesOrdered<_>>()
        .try_collect()
        .await?;
    let governance = canisters[0];
    let ledger = canisters[1];
    let root = canisters[2];
    let swap = canisters[3];
    let index = canisters[4];

    let sns_canisters = SnsCanisterIds {
        governance,
        ledger,
        root,
        swap,
        index,
    };
    let mut payloads = init_payload
        .build_canister_payloads(&sns_canisters, None, true)
        .map_err(CdaoDeployError::InvalidInitPayload)?;
    let time_seconds = ic_cdk::api::time() / 1_000_000_000;
    payloads.swap.swap_start_timestamp_seconds = Some(time_seconds);
    payloads.swap.swap_due_timestamp_seconds = Some(time_seconds + swap_time);
    payloads.swap.icp_ledger_canister_id = NNS_LEDGER_CANISTER_ID.into();
    payloads.swap.nns_governance_canister_id = ic_cdk::id().to_string();

    let sns_wasm = CANISTER_DATA
        .with(|cdata| {
            cdata
                .borrow()
                .known_principal_ids
                .get(&KnownPrincipalType::CanisterIdSnsWasm)
                .copied()
        })
        .expect("SNS WASM not specified in config");

    let gov_hash =
        hex::decode("3feb8ff7b47f53da83235e4c68676bb6db54df1e62df3681de9425ad5cf43be5").unwrap();
    let ledger_hash =
        hex::decode("e8942f56f9439b89b13bd8037f357126e24f1e7932cf03018243347505959fd4").unwrap();
    let root_hash =
        hex::decode("495e31370b14fa61c76bd1483c9f9ba66733793ee2963e8e44a231436a60bcc6").unwrap();
    let swap_hash =
        hex::decode("3bb490d197b8cf2e7d9948bcb5d1fc46747a835294b3ffe47b882dbfa584555f").unwrap();
    let index_hash =
        hex::decode("08ae5042c8e413716d04a08db886b8c6b01bb610b8197cdbe052c59538b924f0").unwrap();

    ic_cdk::println!("gov_hash: {:?}", gov_hash);

    let mut wasm_bins: VecDeque<_> = [gov_hash, ledger_hash, root_hash, swap_hash, index_hash]
        .into_iter()
        .map(|hash| async move {
            let req = GetWasmRequest { hash };
            let wasm_res =
                ic_cdk::call::<_, (GetWasmResponse,)>(sns_wasm, "get_wasm", (req,)).await?;
            Ok::<_, CdaoDeployError>(wasm_res.0.wasm.unwrap().wasm)
        })
        .collect::<FuturesOrdered<_>>()
        .try_collect()
        .await?;

    let mut sns_install_futs = FuturesUnordered::new();
    sns_install_futs.push(install_canister_wasm(
        wasm_bins.pop_front().unwrap(),
        Encode!(&payloads.governance).unwrap(),
        governance,
    ));
    sns_install_futs.push(install_canister_wasm(
        wasm_bins.pop_front().unwrap(),
        Encode!(&payloads.ledger).unwrap(),
        ledger,
    ));
    sns_install_futs.push(install_canister_wasm(
        wasm_bins.pop_front().unwrap(),
        Encode!(&payloads.root).unwrap(),
        root,
    ));
    sns_install_futs.push(install_canister_wasm(
        wasm_bins.pop_front().unwrap(),
        Encode!(&payloads.swap).unwrap(),
        swap,
    ));

    sns_install_futs.push(install_canister_wasm(
        wasm_bins.pop_front().unwrap(),
        Encode!(&payloads.index_ng).unwrap(),
        index,
    ));
    while sns_install_futs.try_next().await?.is_some() {}

    let admin_canister = CANISTER_DATA
        .with(|cdata| {
            cdata
                .borrow()
                .known_principal_ids
                .get(&KnownPrincipalType::UserIdGlobalSuperAdmin)
                .copied()
        })
        .expect("Super admin not specified in config");

    let user_can = ic_cdk::id();
    let mut update_ctrls_futs = FuturesUnordered::new();
    update_ctrls_futs.push(update_controllers(
        governance,
        vec![admin_canister, user_can, root.0],
    ));
    update_ctrls_futs.push(update_controllers(
        root,
        vec![admin_canister, user_can, governance.0],
    ));
    update_ctrls_futs.push(update_controllers(
        ledger,
        vec![admin_canister, user_can, governance.0],
    ));
    update_ctrls_futs.push(update_controllers(
        swap,
        vec![
            admin_canister,
            user_can,
            swap.0,
            ic_nns_constants::ROOT_CANISTER_ID.into(),
        ],
    ));
    update_ctrls_futs.push(update_controllers(
        index,
        vec![admin_canister, user_can, root.0],
    ));
    while update_ctrls_futs.try_next().await?.is_some() {}

    let deployed_cans = DeployedCdaoCanisters {
        governance: governance.0,
        ledger: ledger.0,
        root: root.0,
        swap: swap.0,
        index: index.0,
    };
    CANISTER_DATA.with(|cdata| {
        let mut cdata = cdata.borrow_mut();
        cdata.cdao_canisters.push(deployed_cans);
        cdata.token_roots.insert(root.0, ());
    });

    Ok(deployed_cans)
}

/// Destributes the newly created to all users in the existing airdrop token chain
/// this must be called right after tokens have been swapped to this user canister
/// by the off-chain infrastructure
#[update]
async fn distribute_newly_created_token_to_token_chain(token: DeployedCdaoCanisters) -> Result<(), String> {
    let caller = ic_cdk::caller();
    let profile_owner = CANISTER_DATA.with_borrow(|cdata| cdata.profile.principal_id);
    if Some(caller) != profile_owner {
        return Err("unauthorized".into());
    }

    let token_root = token.root;
    let token_ledger = token.ledger;
    let (parent, token_chain, pop) = CANISTER_DATA.with_borrow(|cdata| {
        Ok::<_, String>((
            cdata.airdrop.parent,
            cdata.airdrop.token_chain.iter().map(|(k, _)| k).collect::<Vec<_>>(),
            cdata.proof_of_participation.clone().ok_or_else(|| "method is not available right now".to_string())?,
        ))
    })?;
    let amount = is_balance_enough_for_airdrop(token_ledger, token_chain.len())
        .await?
        .ok_or_else(move || "not token enough balance".to_string())?;

    for member in token_chain {
        let amount = amount.clone();
        if Some(member) != parent {
            ic_cdk::spawn(async move {
                // ignore the result in case this fails
                _ = transfer_canister_token_to_user_principal(
                        token_root,
                        token_ledger,
                        member.user_principal,
                        member.user_canister,
                        None,
                        amount,
                    ).await;
            });
            continue;
        }
        let token_claim = TokenClaim {
            sender_canister: ic_cdk::id(),
            amount,
            token_root,
            token_ledger
        };
        notify(
            member.user_canister,
            "receive_token_claim",
            (pop.clone(), token_claim)
        ).unwrap();
    }

    Ok(())
}
