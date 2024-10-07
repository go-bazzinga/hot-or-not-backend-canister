use candid::{Nat, Principal};
use futures::{future, stream::FuturesUnordered, StreamExt};
use ic_cdk::{notify, update};
use icrc_ledger_types::icrc1::{account::Account, transfer::{TransferArg, TransferError}};
use shared_utils::{canister_specific::individual_user_template::types::{airdrop::{AirdropMember, TokenClaim}, cdao::DeployedCdaoCanisters}, common::participant_crypto::ProofOfParticipation};

use crate::CANISTER_DATA;

use super::cdao::token::transfer_canister_token_to_user_principal;

#[update]
pub async fn add_user_to_airdrop_chain(pop: ProofOfParticipation, member: AirdropMember) -> Result<(), String> {
    pop.verify_caller_is_participant(&CANISTER_DATA).await?;

    let (pop, parent) = CANISTER_DATA.with_borrow(|cdata|
        Ok::<_, String>((
            cdata.proof_of_participation.clone().ok_or("not available")?,
            cdata.airdrop.parent,
        ))
    )?;
    if let Some(parent) = parent {
        notify(
            parent.user_canister,
            "add_user_to_airdrop_chain",
            (pop, member)
        ).unwrap();
    }

    add_user_to_airdrop_chain_inner(member).await;

    Ok(())
}

/// Returns the token amount to transfer for airdrop
/// returns None if not enough tokens are available 
/// returns Ok(Some(Nat)) if enough tokens are available, where Nat is the amount to transfer
pub(crate) async fn is_balance_enough_for_airdrop(ledger: Principal, transfer_cnt: usize) -> Result<Option<Nat>, String> {
    // SNS Tokens have 8 decimals
    // 1e8 e8s -> 1 token
    let base_amount = Nat::from(1e8 as usize);

    let fee: (Nat,) = ic_cdk::call(ledger, "icrc1_fee", ()).await.map_err(|(_, err)| err)?;
    let fee = fee.0;
    let transfer_amt = (base_amount.clone() + fee) * transfer_cnt; 

    let acc = Account { owner: ic_cdk::id(), subaccount: None };
    let bal: (Nat,) = ic_cdk::call(ledger, "icrc1_balance_of", (acc,)).await.map_err(|(_, err)| err)?;

    if transfer_amt > bal.0 {
        return Ok(None);
    }

    Ok(Some(base_amount))
}

/// Transfer tokens for airdrop to target user_canister
/// returns Ok(Some(Principal)) if the transfer was successful
/// returns Ok(None) if transfer is not available
async fn transfer_token_for_airdrop(canisters: DeployedCdaoCanisters, member: AirdropMember) -> Result<Option<Principal>, String> {
    let ledger = canisters.ledger;
    let Some(amount) = is_balance_enough_for_airdrop(ledger, 1).await? else {
        return Ok(None);
    };

    let transfer_args = TransferArg {
        from_subaccount: None,
        to: Account { owner: member.user_principal, subaccount: None },
        fee: None,
        created_at_time: None,
        memo: None,
        amount,
    };
    let transfer_res: (Result<Nat, TransferError>,) = ic_cdk::call(ledger, "icrc1_transfer", (transfer_args,))
        .await
        .map_err(|(_, err)| err)?;
    if transfer_res.0.is_err() {
        return Err("transfer failed".into());
    }

    Ok(Some(canisters.root))
}

/// Add The user to the airdrop chain
/// also airdrops all the created tokens to this user
pub(crate) async fn add_user_to_airdrop_chain_inner(member: AirdropMember) {
    let added = CANISTER_DATA.with_borrow_mut(|cdata| {
        cdata.airdrop.token_chain.insert(member, ()).is_some()
    });

    if added {
        return;
    }

    let my_tokens = CANISTER_DATA.with_borrow(|cdata| cdata.cdao_canisters.clone());
    let transferred_tokens = my_tokens
        .into_iter()
        .map(|canisters| transfer_token_for_airdrop(canisters, member))
        .collect::<FuturesUnordered<_>>()
        .filter_map(|res| {
            let Ok(Some(res)) = res else {
                return future::ready(None);
            };
            future::ready(Some(res))
        })
        .collect::<Vec<_>>()
        .await;

    // Rollback if notify fails
    ic_cdk::notify(
        member.user_canister,
        "add_tokens",
        (transferred_tokens,)
    ).unwrap();
}

#[update]
async fn receive_token_claim(pop: ProofOfParticipation, token_claim: TokenClaim) -> Result<(), String> {
    pop.verify_caller_is_participant(&CANISTER_DATA).await?;

    let (pop, user_principal) = CANISTER_DATA.with_borrow(|cdata| {
        Ok::<_, String>((
            cdata.proof_of_participation.clone().ok_or_else(|| "canister is not ready".to_string())?,
            cdata.profile.principal_id.ok_or_else(|| "canister is not ready".to_string())?,
        ))
    })?;
    notify(
        token_claim.sender_canister,
        "redeem_token_claim",
        (pop.clone(), user_principal, token_claim.clone())
    ).unwrap();

    let Some(parent) = CANISTER_DATA.with_borrow(|cdata| cdata.airdrop.parent) else {
        return Ok(());
    };

    notify(
        parent.user_canister,
        "receive_token_claim",
        (pop, token_claim,)
    ).unwrap();

    Ok(())
}

#[update]
async fn redeem_token_claim(pop: ProofOfParticipation, target_principal: Principal, token_claim: TokenClaim) -> Result<(), String> {
    pop.verify_caller_is_participant(&CANISTER_DATA).await?;

    transfer_canister_token_to_user_principal(
        token_claim.token_root,
        token_claim.token_ledger,
        target_principal,
        ic_cdk::caller(),
        None,
        token_claim.amount
    ).await.map_err(|e| format!("transfer failed: {e:?}"))?;

    Ok(())
}