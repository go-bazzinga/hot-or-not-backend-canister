use ic_cdk::call;
use shared_utils::common::types::known_principal::KnownPrincipalType;

use crate::CANISTER_DATA;

pub async fn request_cycles_from_subnet_orchestrator(amount: u128) -> Result<(), String> {
    let subnet_orchestrator_canister_id = CANISTER_DATA
        .with_borrow(|canister_data| {
            canister_data
                .known_principal_ids
                .get(&KnownPrincipalType::CanisterIdUserIndex)
                .copied()
        })
        .ok_or("Subnet Orchestrator Canister Id not found".to_owned())?;

    let result = call::<_, (Result<(), String>,)>(
        subnet_orchestrator_canister_id,
        "request_cycles",
        (amount,),
    )
    .await
    .map_err(|e| e.1)?
    .0;

    result
}
