use candid::Principal;
use ic_cdk::api::{
    self,
    call::RejectionCode,
    management_canister::{
        main::{self, CanisterInstallMode, CreateCanisterArgument, InstallCodeArgument},
        provisional::{CanisterIdRecord, CanisterSettings},
    },
};
use shared_utils::canister_specific::individual_user_template::types::arg::IndividualUserTemplateInitArgs;

use crate::CANISTER_DATA;

const INDIVIDUAL_USER_TEMPLATE_CANISTER_WASM: &[u8] = include_bytes!(
    "../../../../../target/wasm32-unknown-unknown/release/individual_user_template.wasm.gz"
);

pub async fn create_users_canister(profile_owner: Principal) -> Principal {
    // * config for provisioning canister
    let arg = CreateCanisterArgument {
        settings: Some(CanisterSettings {
            controllers: Some(vec![
                // this canister
                api::id(),
            ]),
            compute_allocation: None,
            memory_allocation: None,
            freezing_threshold: None,
        }),
    };

    // * provisioned canister
    let canister_id: Principal = main::create_canister(arg).await.unwrap().0.canister_id;

    // * deposit an additional 1T cycles
    main::deposit_cycles(CanisterIdRecord { canister_id }, 1_000_000_000_000)
        .await
        .unwrap();

    let individual_user_tempalate_init_args = IndividualUserTemplateInitArgs {
        profile_owner: Some(profile_owner),
        known_principal_ids: Some(CANISTER_DATA.with(|canister_data_ref_cell| {
            canister_data_ref_cell.borrow().known_principal_ids.clone()
        })),
        upgrade_version_number: Some(0),
    };

    // * encode argument for user canister init lifecycle method
    let arg = candid::encode_one(individual_user_tempalate_init_args)
        .expect("Failed to serialize the install argument.");

    // * install wasm to provisioned canister
    main::install_code(InstallCodeArgument {
        mode: CanisterInstallMode::Install,
        canister_id,
        wasm_module: INDIVIDUAL_USER_TEMPLATE_CANISTER_WASM.into(),
        arg,
    })
    .await
    .unwrap();

    canister_id
}

pub async fn upgrade_individual_user_canister(
    canister_id: Principal,
    install_mode: CanisterInstallMode,
    arg: IndividualUserTemplateInitArgs,
) -> Result<(), (RejectionCode, String)> {
    let serialized_arg =
        candid::encode_args((arg,)).expect("Failed to serialize the install argument.");

    main::install_code(InstallCodeArgument {
        mode: install_mode,
        canister_id,
        wasm_module: INDIVIDUAL_USER_TEMPLATE_CANISTER_WASM.into(),
        arg: serialized_arg,
    })
    .await
}
