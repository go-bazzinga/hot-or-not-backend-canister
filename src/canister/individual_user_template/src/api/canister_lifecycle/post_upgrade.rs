use candid::Principal;
use ciborium::de;
use ic_cdk_macros::post_upgrade;
use ic_stable_structures::Memory;
<<<<<<< HEAD
use std::{borrow::BorrowMut, time::{Duration, SystemTime}};
=======
use std::{borrow::BorrowMut, collections::HashSet, time::Duration};
>>>>>>> 50c53d8... reconcile winnings

use crate::{
    api::hot_or_not_bet::tabulate_hot_or_not_outcome_for_post_slot::inform_participants_of_outcome,
    data_model::memory,
};

use shared_utils::{canister_specific::individual_user_template::types::{
    arg::IndividualUserTemplateInitArgs,
    session::SessionType,
}, common::utils::system_time::get_current_system_time_from_ic};

use crate::{
    api::{
        hot_or_not_bet::reenqueue_timers_for_pending_bet_outcomes::reenqueue_timers_for_pending_bet_outcomes,
        well_known_principal::update_locally_stored_well_known_principals,
    },
    CANISTER_DATA,
};

#[post_upgrade]
fn post_upgrade() {
    restore_data_from_stable_memory();
    save_upgrade_args_to_memory();
    refetch_well_known_principals();
    reenqueue_timers_for_pending_bet_outcomes();
}

fn restore_data_from_stable_memory() {
    let heap_data = memory::get_upgrades_memory();
    let mut heap_data_len_bytes = [0; 4];
    heap_data.read(0, &mut heap_data_len_bytes);
    let heap_data_len = u32::from_le_bytes(heap_data_len_bytes) as usize;

    let mut canister_data_bytes = vec![0; heap_data_len];
    heap_data.read(4, &mut canister_data_bytes);

    let canister_data =
        de::from_reader(&*canister_data_bytes).expect("Failed to deserialize heap data");
    CANISTER_DATA.with(|canister_data_ref_cell| {
        *canister_data_ref_cell.borrow_mut() = canister_data;
    });
}

fn save_upgrade_args_to_memory() {
    let upgrade_args = ic_cdk::api::call::arg_data::<(IndividualUserTemplateInitArgs,)>().0;

    CANISTER_DATA.with(|canister_data_ref_cell| {
        let mut canister_data_ref_cell = canister_data_ref_cell.borrow_mut();

        if let Some(known_principal_ids) = upgrade_args.known_principal_ids {
            canister_data_ref_cell.known_principal_ids = known_principal_ids;
        }

        if let Some(profile_owner) = upgrade_args.profile_owner {
            canister_data_ref_cell.profile.principal_id = Some(profile_owner);
        }

        if let Some(upgrade_version_number) = upgrade_args.upgrade_version_number {
            canister_data_ref_cell.version_details.version_number = upgrade_version_number;
        }

        canister_data_ref_cell.borrow_mut().version_details.version = upgrade_args.version;

        if let Some(url_to_send_canister_metrics_to) = upgrade_args.url_to_send_canister_metrics_to
        {
            canister_data_ref_cell
                .configuration
                .url_to_send_canister_metrics_to = Some(url_to_send_canister_metrics_to);
        }
    });
}

const DELAY_FOR_REFETCHING_WELL_KNOWN_PRINCIPALS: Duration = Duration::from_secs(1);
fn refetch_well_known_principals() {
    ic_cdk_timers::set_timer(DELAY_FOR_REFETCHING_WELL_KNOWN_PRINCIPALS, || {
        ic_cdk::spawn(update_locally_stored_well_known_principals::update_locally_stored_well_known_principals())
    });
}

const DELAY_FOR_MIGRATING_DATA: Duration = Duration::from_secs(1);
fn reconcile_canister_winnings() {
    ic_cdk_timers::set_timer(DELAY_FOR_MIGRATING_DATA, || {
        reconcile_canister_winnings_impl()
    });
}

fn reconcile_canister_winnings_impl() {
    let posts = CANISTER_DATA.with(|canister_data_ref_cell| {
        let canister_data_ref_cell = canister_data_ref_cell.borrow_mut();

        canister_data_ref_cell.all_created_posts.clone()
    });

    let rooms_list = posts
        .iter()
        .filter_map(|(post_id, post)| {
            if let Some(hot_or_not_details) = &post.hot_or_not_details {
                Some((post_id, hot_or_not_details))
            } else {
                None
            }
        })
        .map(|(post_id, hot_or_not_details)| {
            hot_or_not_details
                .slot_history
                .iter()
                .map(move |(slot_id, slot_details)| (post_id, slot_id, slot_details))
                .map(|(post_id, slot_id, slot_details)| {
                    slot_details
                        .room_details
                        .iter()
                        .map(move |(room_id, room_details)| {
                            GlobalRoomId(post_id.clone(), slot_id.clone(), room_id.clone())
                        })
                })
        })
        .flatten()
        .flatten()
        .collect::<HashSet<GlobalRoomId>>();

    CANISTER_DATA.with(|canister_data_ref_cell| {
        let canister_data_ref_cell = canister_data_ref_cell.borrow();

        canister_data_ref_cell
            .room_details_map
            .iter()
            .for_each(|(room_id, _)| {
                if !rooms_list.contains(&room_id) {
                    let post_id = room_id.0;
                    let post_to_tabulate_results_for = canister_data_ref_cell
                        .all_created_posts
                        .get(&post_id)
                        .unwrap();
                    let slot_id = room_id.1;

                    inform_participants_of_outcome(
                        post_to_tabulate_results_for,
                        &slot_id,
                        &canister_data_ref_cell.room_details_map,
                        &canister_data_ref_cell.bet_details_map,
                    );
                };
            });
    });
}
