use crate::{data_model::CanisterData, CANISTER_DATA};
use ic_cdk_macros::init;
use shared_utils::{
    canister_specific::individual_user_template::types::arg::IndividualUserTemplateInitArgs,
    common::timer::send_metrics::enqueue_timer_for_calling_metrics_rest_api,
};

#[init]
fn init(init_args: IndividualUserTemplateInitArgs) {
    CANISTER_DATA.with(|canister_data_ref_cell| {
        let mut data = canister_data_ref_cell.borrow_mut();
        init_impl(init_args, &mut data);
    });

    send_canister_metrics();
}

fn init_impl(init_args: IndividualUserTemplateInitArgs, data: &mut CanisterData) {
    init_args
        .known_principal_ids
        .unwrap_or_default()
        .iter()
        .for_each(|(principal_belongs_to, principal_id)| {
            data.known_principal_ids
                .insert(*principal_belongs_to, *principal_id);
        });

    data.profile.principal_id = init_args.profile_owner;

    data.configuration.url_to_send_canister_metrics_to = init_args.url_to_send_canister_metrics_to;

    data.version_details.version_number = init_args.upgrade_version_number.unwrap_or_default();
    data.version_details.version = init_args.version;
    if let Some(pop) = init_args.proof_of_participation {
        data.proof_of_participation = Some(pop);
    }
}

pub fn send_canister_metrics() {
    let url_to_send_canister_metrics_to = CANISTER_DATA.with(|canister_data_ref_cell| {
        canister_data_ref_cell
            .borrow()
            .configuration
            .url_to_send_canister_metrics_to
            .clone()
    });

    if let Some(url_to_send_canister_metrics_to) = url_to_send_canister_metrics_to {
        enqueue_timer_for_calling_metrics_rest_api(url_to_send_canister_metrics_to);
    }
}

#[cfg(test)]
mod test {
    use shared_utils::common::types::known_principal::{KnownPrincipalMap, KnownPrincipalType};
    use test_utils::setup::test_constants::{
        get_global_super_admin_principal_id,
        get_mock_canister_id_user_index, get_mock_user_alice_principal_id,
    };

    use super::*;

    #[test]
    fn test_init_impl() {
        // * Add some known principals
        let mut known_principal_ids = KnownPrincipalMap::new();
        known_principal_ids.insert(
            KnownPrincipalType::UserIdGlobalSuperAdmin,
            get_global_super_admin_principal_id(),
        );
        known_principal_ids.insert(
            KnownPrincipalType::CanisterIdUserIndex,
            get_mock_canister_id_user_index(),
        );

        // * Create the init args
        let init_args = IndividualUserTemplateInitArgs {
            known_principal_ids: Some(known_principal_ids),
            profile_owner: Some(get_mock_user_alice_principal_id()),
            upgrade_version_number: Some(0),
            url_to_send_canister_metrics_to: Some(
                "http://metrics-url.com/receive-metrics".to_string(),
            ),
            version: String::from("v1.0.0"),
            proof_of_participation: None,
        };
        let mut data = CanisterData::default();

        // * Run the init impl
        init_impl(init_args, &mut data);

        // * Check the data
        assert_eq!(
            data.known_principal_ids
                .get(&KnownPrincipalType::UserIdGlobalSuperAdmin)
                .unwrap(),
            &get_global_super_admin_principal_id()
        );
        assert_eq!(
            data.known_principal_ids
                .get(&KnownPrincipalType::CanisterIdUserIndex)
                .unwrap(),
            &get_mock_canister_id_user_index()
        );

        assert_eq!(
            data.profile.principal_id,
            Some(get_mock_user_alice_principal_id())
        );

        assert_eq!(
            data.configuration.url_to_send_canister_metrics_to,
            Some("http://metrics-url.com/receive-metrics".to_string())
        );

        assert!(data.version_details.version.eq("v1.0.0"));
    }
}
