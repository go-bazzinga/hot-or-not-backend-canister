use candid::{Nat, Principal};
use futures::{stream::FuturesUnordered, StreamExt};
use ic_cdk::{query, update};
use ic_sns_root::pb::v1::{ListSnsCanistersRequest, ListSnsCanistersResponse};
use icrc_ledger_types::icrc1::{account::Account, transfer::{Memo, TransferArg, TransferError}};
use shared_utils::{canister_specific::individual_user_template::types::error::CdaoTokenError, pagination::{self, PaginationError}};

use crate::CANISTER_DATA;

/// Add a new token
/// returns true if new token is added
#[update]
async fn add_token(root_canister: Principal) -> Result<bool, CdaoTokenError> {
    let token_added = CANISTER_DATA.with(|cdata| {
        let cdata = cdata.borrow();
        cdata.token_roots.contains_key(&root_canister)
    });
    if token_added {
        return Ok(false);
    }

    if is_token_balance_zero(root_canister).await? {
        return Err(CdaoTokenError::NoBalance);
    }

    CANISTER_DATA.with(|cdata| {
        let mut cdata = cdata.borrow_mut();
        cdata.token_roots.insert(root_canister, ());
    });


    Ok(true)
}

async fn is_token_balance_zero(root_canister: Principal) -> Result<bool, CdaoTokenError> {
    let res: (ListSnsCanistersResponse,) = ic_cdk::call(root_canister, "list_sns_canisters", (ListSnsCanistersRequest {},)).await?;
    let cans = res.0;
    let ledger = cans.ledger.ok_or(CdaoTokenError::InvalidRoot)?;

    let my_principal_id = CANISTER_DATA
        .with(|canister_data_ref_cell| canister_data_ref_cell.borrow().profile.principal_id)
        .ok_or(CdaoTokenError::Unauthenticated)?;
    let acc = Account { owner: my_principal_id, subaccount: None };

    let balance: (Nat,) = ic_cdk::call(ledger.into(), "icrc1_balance_of", (acc,)).await?;
    Ok(balance.0 == 0u32)
}

/// Add multiple tokens
/// returns a list specifying which tokens were added (or failed to be added)
/// the list matches the order of the input list
#[update]
async fn add_tokens(root_canisters: Vec<Principal>) -> Vec<Result<bool, CdaoTokenError>> {
    let mut result = vec![];
    let filtered_roots: Vec<_> = CANISTER_DATA.with_borrow(|cdata| {
        root_canisters.into_iter().enumerate().filter(|(_, root)| {
            let already_added = cdata.token_roots.contains_key(root);
            result.push(Ok(!already_added));
            !already_added
        }).collect()
    });

    let mut is_balances_zero = filtered_roots
        .into_iter()
        .map(|(i, root)| async move {
            (i, root, is_token_balance_zero(root).await)
        })
        .collect::<FuturesUnordered<_>>();

    while let Some((i, root, is_balance_zero_res)) = is_balances_zero.next().await {
        let is_balance_zero = is_balance_zero_res.as_ref().copied().unwrap_or_default();
        result[i] = is_balance_zero_res;

        if is_balance_zero {
            continue;
        }

        CANISTER_DATA.with_borrow_mut(|cdata| {
            cdata.token_roots.insert(root, ());
        });
    }

    result
}

/// Transfer some tokens from this canister's balance to another user's principal
/// NOTE: target_canister must be the canister of the user. This check is not performed in this function.
/// token_ledger must be the ledger of the token root, this function does not perform this check.
pub(crate) async fn transfer_canister_token_to_user_principal(token_root: Principal, token_ledger: Principal, target_user_principal: Principal, target_canister: Principal, memo: Option<Memo>, amount: Nat) -> Result<(), CdaoTokenError> {
    let transfer_args = TransferArg {
        from_subaccount: None,
        to: Account { owner: target_user_principal, subaccount: None },
        fee: None,
        created_at_time: None,
        memo,
        amount,
    };
    let transfer_res: (Result<Nat, TransferError>,) = ic_cdk::call(token_ledger, "icrc1_transfer", (transfer_args,)).await?;
    transfer_res.0.map_err(CdaoTokenError::Transfer)?;

    let res: (Result<bool, CdaoTokenError>,) = ic_cdk::call(target_canister, "add_token", (token_root,)).await?;
    // Rollback transfer on failure
    res.0.unwrap();

    Ok(())
}

#[query]
fn get_token_roots_of_this_user_with_pagination_cursor(from_inclusive_index: u64, limit: u64) -> Result<Vec<Principal>, PaginationError> {
    CANISTER_DATA.with_borrow(|cdata| {
        let (from_inclusive_index, limit) = pagination::get_pagination_bounds_cursor(
            from_inclusive_index,
            limit,
            cdata.token_roots.len(),
        )?;
        let tokens = cdata.token_roots
            .iter()
            .skip(from_inclusive_index as usize)
            .take(limit as usize)
            .map(|(token, _)| token)
            .collect();
        Ok(tokens)
    })
}
