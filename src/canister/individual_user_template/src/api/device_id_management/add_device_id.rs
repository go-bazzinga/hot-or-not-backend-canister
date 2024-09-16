use ic_cdk::{api, query};
use ic_cdk_macros::update;
use shared_utils::canister_specific::individual_user_template::types::device_id::DeviceIdentity;

use crate::{data_model::CanisterData, CANISTER_DATA};

/// #### Access Control
/// Only the user whose profile details are stored in this canister can add the device identity.
#[update]
fn add_device_id(identity_token: String) -> Result<bool, (String)> {
    // * access control
    let current_caller = ic_cdk::caller();
    let my_principal_id = CANISTER_DATA
        .with(|canister_data_ref_cell| canister_data_ref_cell.borrow().profile.principal_id);
    if my_principal_id != Some(current_caller) {
        return Err(
            "Only the user whose profile details are stored in this canister can add a device id."
                .to_string(),
        );
    };

    let device_id = DeviceIdentity {
        device_id: identity_token,
        timestamp: api::time() / 1_000_000,
    };

    let response = CANISTER_DATA.with(|canister_data_ref_cell| {
        add_device_id_to_memory(
            &mut canister_data_ref_cell.borrow_mut(),
            device_id
        )
    });

    if response {
        Ok(true)
    } else {
        Err(("Failed to add device id.".to_string()))
    }
}

fn add_device_id_to_memory(canister_data: &mut CanisterData, device_id: DeviceIdentity) -> bool {
    canister_data.device_identities.push(device_id);
    true
}

#[query]
fn get_device_identities() -> Vec<DeviceIdentity> {
    CANISTER_DATA.with(|canister_data_ref_cell| {
        canister_data_ref_cell.borrow().device_identities.clone()
    })
}
