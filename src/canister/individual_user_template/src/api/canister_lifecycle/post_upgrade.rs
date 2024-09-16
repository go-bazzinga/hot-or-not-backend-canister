use ciborium::de;
use ic_cdk::api::call::ArgDecoderConfig;
use ic_cdk_macros::post_upgrade;
use ic_stable_structures::Memory;
use std::borrow::BorrowMut;
use std::time::Duration;

use crate::{
    api::hot_or_not_bet::reenqueue_timers_for_pending_bet_outcomes_v1::reenqueue_timers_for_pending_bet_outcomes_v1,
    data_model::memory,
};

use shared_utils::{
    canister_specific::individual_user_template::types::{
        arg::IndividualUserTemplateInitArgs,
        hot_or_not::{GlobalBetIdV1, GlobalRoomIdV1, PlacedBetDetailV1, SlotDetailsV1},
    },
    common::types::utility_token::token_event::NewSlotType,
};

use crate::CANISTER_DATA;

#[post_upgrade]
fn post_upgrade() {
    restore_data_from_stable_memory();
    save_upgrade_args_to_memory();
    migrate_data();

    reenqueue_timers_for_pending_bet_outcomes_v1();
    migrate_excessive_tokens();
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
    let upgrade_args = ic_cdk::api::call::arg_data::<(IndividualUserTemplateInitArgs,)>(
        ArgDecoderConfig::default(),
    )
    .0;

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

const DELAY_FOR_MIGRATING_DATA: Duration = Duration::from_secs(3);
fn migrate_data() {
    ic_cdk_timers::set_timer(DELAY_FOR_MIGRATING_DATA, || {
        ic_cdk::spawn(migrate_data_impl());
    });
}

async fn migrate_data_impl() {
    // Migrate Slot Details Map
    CANISTER_DATA.with(|canister_data_ref_cell| {
        let mut canister_data = canister_data_ref_cell.borrow_mut();
        let old_slot_details_map = canister_data
            .slot_details_map
            .iter()
            .map(|(k, v)| (k, v.clone()))
            .collect::<std::collections::BTreeMap<_, _>>();

        let new_slot_details_map = &mut canister_data.slot_details_map_v1;

        for ((post_id, old_slot_id), old_slot_details) in old_slot_details_map.iter() {
            let new_slot_id: NewSlotType = (*old_slot_id).into();
            new_slot_details_map.insert(((*post_id), new_slot_id), old_slot_details.clone());
        }
    });

    // Migrate Room Details Map
    CANISTER_DATA.with(|canister_data_ref_cell| {
        let mut canister_data = canister_data_ref_cell.borrow_mut();
        let old_room_details_map = canister_data
            .room_details_map
            .iter()
            .map(|(k, v)| (k, v.clone()))
            .collect::<std::collections::BTreeMap<_, _>>();
        let new_room_details_map = &mut canister_data.room_details_map_v1;

        for (old_global_room_id, room_details) in old_room_details_map.iter() {
            let new_global_room_id: GlobalRoomIdV1 = (*old_global_room_id).into();
            new_room_details_map.insert(new_global_room_id, room_details.clone());
        }
    });

    // Migrate Bet Details Map
    CANISTER_DATA.with(|canister_data_ref_cell| {
        let mut canister_data = canister_data_ref_cell.borrow_mut();
        let old_bet_details_map = canister_data
            .bet_details_map
            .iter()
            .map(|(k, v)| (k, v.clone()))
            .collect::<std::collections::BTreeMap<_, _>>();
        let new_bet_details_map = &mut canister_data.bet_details_map_v1;

        for (old_global_bet_id, bet_details) in old_bet_details_map.iter() {
            let new_global_bet_id: GlobalBetIdV1 = old_global_bet_id.clone().into();
            new_bet_details_map.insert(new_global_bet_id, bet_details.clone());
        }
    });

    // Migrate all_hot_or_not_bets_placed
    CANISTER_DATA.with(|canister_data_ref_cell| {
        let mut canister_data = canister_data_ref_cell.borrow_mut();
        let old_hot_or_not_bets = canister_data.all_hot_or_not_bets_placed.clone();
        // iter().map(|(k, v)| (k, v.clone())).collect::<std::collections::BTreeMap<_, _>>();
        // let new_hot_or_not_bets = &mut canister_data.all_hot_or_not_bets_placed_v1;

        for ((canister_id, post_id), placed_bet_detail) in old_hot_or_not_bets.iter() {
            let placed_bet_detail_v1: PlacedBetDetailV1 = placed_bet_detail.clone().into();
            // new_hot_or_not_bets.insert((*canister_id, *post_id), placed_bet_detail_v1);
            canister_data
                .all_hot_or_not_bets_placed_v1
                .insert((*canister_id, *post_id), placed_bet_detail_v1);
        }
    });

    // Assuming `TokenBalanceV1` implements `From<TokenBalance>`
    CANISTER_DATA.with(|canister_data_ref_cell| {
        let mut canister_data = canister_data_ref_cell.borrow_mut();

        // Migrate my_token_balance to my_token_balance_v1
        canister_data.my_token_balance_v1 = canister_data.my_token_balance.clone().into();
    });
}
fn migrate_excessive_tokens(){
    CANISTER_DATA.with(|canister_data_ref_cell| {
        let mut canister_data_ref_cell = canister_data_ref_cell.borrow_mut();
        if canister_data_ref_cell.my_token_balance.utility_token_balance > 18_00_00_00_00_00_00_00_00_00 {
            canister_data_ref_cell.my_token_balance.utility_token_balance = 1000;
        }
    });
}
