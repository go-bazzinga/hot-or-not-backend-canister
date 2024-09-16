use candid::Principal;
use ic_stable_structures::{memory_manager::VirtualMemory, DefaultMemoryImpl};
use shared_utils::{
    canister_specific::individual_user_template::types::{
        hot_or_not::{
            BetDetails, BetDirection, BetOutcomeForBetMaker, BetPayout, GlobalBetId, GlobalBetIdV1, GlobalRoomId,GlobalRoomIdV1, RoomBetPossibleOutcomes, RoomDetailsV1, SlotDetails, StablePrincipal
        },
        post::Post,
    },
    common::{types::utility_token::token_event::NewSlotType, utils::system_time},
};

use crate::data_model::CanisterData;

#[deprecated(note = "use tabulate_hot_or_not_outcome_for_post_slot_v1 instead")]
pub fn tabulate_hot_or_not_outcome_for_post_slot(
    canister_data: &mut CanisterData,
    post_id: u64,
    slot_id: u8,
) {
    let current_time = system_time::get_current_system_time_from_ic();
    let this_canister_id = ic_cdk::id();

    let post_to_tabulate_results_for = canister_data.all_created_posts.get_mut(&post_id).unwrap();
    let token_balance = &mut canister_data.my_token_balance;

    post_to_tabulate_results_for.tabulate_hot_or_not_outcome_for_slot_v1(
        &this_canister_id,
        &slot_id,
        token_balance,
        &current_time,
        &mut canister_data.room_details_map,
        &mut canister_data.bet_details_map,
    );

    inform_participants_of_outcome(
        post_to_tabulate_results_for,
        &slot_id,
        &canister_data.room_details_map,
        &canister_data.bet_details_map,
    );
}


pub fn tabulate_hot_or_not_outcome_for_post_slot_v1(
    canister_data: &mut CanisterData,
    post_id: u64,
    slot_id: NewSlotType,
) {
    let current_time = system_time::get_current_system_time_from_ic();
    let this_canister_id = ic_cdk::id();

    let post_to_tabulate_results_for = canister_data.all_created_posts.get_mut(&post_id).unwrap();
    let token_balance = &mut canister_data.my_token_balance_v1;

    post_to_tabulate_results_for.tabulate_hot_or_not_outcome_for_slot_v2(
        &this_canister_id,
        &slot_id,
        token_balance,
        &current_time,
        &mut canister_data.room_details_map_v1,
        &mut canister_data.bet_details_map_v1,
    );

    inform_participants_of_outcome_v1(
        post_to_tabulate_results_for,
        &slot_id,
        &canister_data.room_details_map_v1,
        &canister_data.bet_details_map_v1,
    );
}


pub fn inform_participants_of_outcome_v1(
    post: &Post,
    slot_id: &NewSlotType,
    room_details_map: &ic_stable_structures::btreemap::BTreeMap<
        GlobalRoomIdV1,
        RoomDetailsV1,
        VirtualMemory<DefaultMemoryImpl>,
    >,
    bet_details_map: &ic_stable_structures::btreemap::BTreeMap<
        GlobalBetIdV1,
        BetDetails,
        VirtualMemory<DefaultMemoryImpl>,
    >,
) {
    let slot_id_owned = slot_id.clone();
    // let hot_or_not_details = post.hot_or_not_details.as_ref();

    // if hot_or_not_details.is_none() {
    //     return;
    // }

    let start_global_room_id = GlobalRoomIdV1(post.id, slot_id_owned, 1);
    let end_global_room_id = GlobalRoomIdV1(post.id, slot_id_owned.increment_by(1), 1);

    let room_details = room_details_map
        .range(start_global_room_id..end_global_room_id)
        .collect::<Vec<_>>();

    room_details.iter().for_each(|(groomid, room_detail)| {
        let mut room_detail = room_detail.clone();
        let room_id = groomid.2;

        let start_global_bet_id = GlobalBetIdV1(start_global_room_id, StablePrincipal::default());
        let end_global_bet_id = GlobalBetIdV1(end_global_room_id, StablePrincipal::default());
        let bet_details = bet_details_map
            .range(start_global_bet_id..end_global_bet_id)
            .collect::<Vec<_>>();

        for (_, bet) in bet_details.iter() {
            let bet_outcome_for_bet_maker: BetOutcomeForBetMaker = match room_detail.bet_outcome {
                RoomBetPossibleOutcomes::BetOngoing => BetOutcomeForBetMaker::AwaitingResult,
                RoomBetPossibleOutcomes::Draw => BetOutcomeForBetMaker::Draw(match bet.payout {
                    BetPayout::Calculated(amount) => amount,
                    _ => 0,
                }),
                RoomBetPossibleOutcomes::HotWon => match bet.bet_direction {
                    BetDirection::Hot => BetOutcomeForBetMaker::Won(match bet.payout {
                        BetPayout::Calculated(amount) => amount,
                        _ => 0,
                    }),
                    BetDirection::Not => BetOutcomeForBetMaker::Lost,
                },
                RoomBetPossibleOutcomes::NotWon => match bet.bet_direction {
                    BetDirection::Hot => BetOutcomeForBetMaker::Lost,
                    BetDirection::Not => BetOutcomeForBetMaker::Won(match bet.payout {
                        BetPayout::Calculated(amount) => amount,
                        _ => 0,
                    }),
                },
            };

            if bet_outcome_for_bet_maker == BetOutcomeForBetMaker::AwaitingResult {
                continue;
            }

            ic_cdk::spawn(ic_call_receive_bet_winnings_when_distributed_v1(
                bet.bet_maker_canister_id,
                post.id,
                bet_outcome_for_bet_maker,
            ));
        }
    });
}


#[deprecated(note = "use inform_participants_of_outcome_v1 instead")]
pub fn inform_participants_of_outcome(
    post: &Post,
    slot_id: &u8,
    room_details_map: &ic_stable_structures::btreemap::BTreeMap<
        GlobalRoomId,
        RoomDetailsV1,
        VirtualMemory<DefaultMemoryImpl>,
    >,
    bet_details_map: &ic_stable_structures::btreemap::BTreeMap<
        GlobalBetId,
        BetDetails,
        VirtualMemory<DefaultMemoryImpl>,
    >,
) {
    let hot_or_not_details = post.hot_or_not_details.as_ref();

    if hot_or_not_details.is_none() {
        return;
    }

    let start_global_room_id = GlobalRoomId(post.id, *slot_id, 1);
    let end_global_room_id = GlobalRoomId(post.id, *slot_id + 1, 1);

    let room_details = room_details_map
        .range(start_global_room_id..end_global_room_id)
        .collect::<Vec<_>>();

    room_details.iter().for_each(|(groomid, room_detail)| {
        let mut room_detail = room_detail.clone();
        let room_id = groomid.2;

        let start_global_bet_id = GlobalBetId(start_global_room_id, StablePrincipal::default());
        let end_global_bet_id = GlobalBetId(end_global_room_id, StablePrincipal::default());
        let bet_details = bet_details_map
            .range(start_global_bet_id..end_global_bet_id)
            .collect::<Vec<_>>();

        for (_, bet) in bet_details.iter() {
            let bet_outcome_for_bet_maker: BetOutcomeForBetMaker = match room_detail.bet_outcome {
                RoomBetPossibleOutcomes::BetOngoing => BetOutcomeForBetMaker::AwaitingResult,
                RoomBetPossibleOutcomes::Draw => BetOutcomeForBetMaker::Draw(match bet.payout {
                    BetPayout::Calculated(amount) => amount,
                    _ => 0,
                }),
                RoomBetPossibleOutcomes::HotWon => match bet.bet_direction {
                    BetDirection::Hot => BetOutcomeForBetMaker::Won(match bet.payout {
                        BetPayout::Calculated(amount) => amount,
                        _ => 0,
                    }),
                    BetDirection::Not => BetOutcomeForBetMaker::Lost,
                },
                RoomBetPossibleOutcomes::NotWon => match bet.bet_direction {
                    BetDirection::Hot => BetOutcomeForBetMaker::Lost,
                    BetDirection::Not => BetOutcomeForBetMaker::Won(match bet.payout {
                        BetPayout::Calculated(amount) => amount,
                        _ => 0,
                    }),
                },
            };

            if bet_outcome_for_bet_maker == BetOutcomeForBetMaker::AwaitingResult {
                continue;
            }

            ic_cdk::spawn(receive_bet_winnings_when_distributed(
                bet.bet_maker_canister_id,
                post.id,
                bet_outcome_for_bet_maker,
            ));
        }
    });
}

async fn receive_bet_winnings_when_distributed(
    bet_maker_canister_id: Principal,
    post_id: u64,
    bet_outcome_for_bet_maker: BetOutcomeForBetMaker,
) {
    ic_cdk::call::<_, ()>(
        bet_maker_canister_id,
        "receive_bet_winnings_when_distributed",
        (post_id, bet_outcome_for_bet_maker),
    )
    .await
    .ok();
}

async fn ic_call_receive_bet_winnings_when_distributed_v1(
    bet_maker_canister_id: Principal,
    post_id: u64,
    bet_outcome_for_bet_maker: BetOutcomeForBetMaker,
) {
    ic_cdk::call::<_, ()>(
        bet_maker_canister_id,
        "receive_bet_winnings_when_distributed_v1",
        (post_id, bet_outcome_for_bet_maker),
    )
    .await
    .ok();
}
