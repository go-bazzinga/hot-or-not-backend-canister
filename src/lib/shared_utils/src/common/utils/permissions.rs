use ic_cdk::{api::is_controller, caller};
use candid::Principal;
use crate::constant::{GLOBAL_SUPER_ADMIN_USER_ID, GOVERNANCE_CANISTER_ID, RECLAIM_CANISTER_PRINCIPAL_ID};

pub fn is_reclaim_canister_id() -> Result<(), String> {
    let caller = ic_cdk::caller();
    let reclaim_canister_principal = Principal::from_text(RECLAIM_CANISTER_PRINCIPAL_ID).unwrap();

    // Here accessing the args ???

    if caller == reclaim_canister_principal {
        Ok(())
    } else {
        Err("Caller is not allowed.".to_string())
    }
}


pub fn is_caller_global_admin() -> Result<(), String> {
    if !caller().to_string().eq(GLOBAL_SUPER_ADMIN_USER_ID) {
        return Err("Unauthorize".into())
    }
    Ok(())
}


pub fn is_caller_controller() -> Result<(), String> {
    if !is_controller(&caller()) {
        return Err("Unauthorize".into());
    }
    Ok(())
}


pub fn is_caller_controller_or_global_admin() -> Result<(), String> {
    if !is_controller(&caller()) && !caller().to_string().eq(GLOBAL_SUPER_ADMIN_USER_ID) {
        return Err("Unauthorize".into())
    }

    Ok(())
}


pub fn is_caller_governance_canister() -> Result<(), String> {
    if !caller().to_string().eq(GOVERNANCE_CANISTER_ID) {
        return Err("Unauthorized".into());
    }

    Ok(())
}