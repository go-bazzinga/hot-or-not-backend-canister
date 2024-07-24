use ic_cdk_macros::update;
use std::time::SystemTime;

use candid::{types::internal, Principal};
use ic_cdk::{api::management_canister::provisional::CanisterId, println};
use shared_utils::{
    canister_specific::individual_user_template::types::{
        arg::PlaceBetArg,
        error::BetOnCurrentlyViewingPostError,
        hot_or_not::{BetDirection, BettingStatus},
    },
    common::utils::system_time,
};

use shared_utils::common::types::{
    app_primitive_type::PostId,
    utility_token::token_event::SystemTimeInMs
};
use shared_utils::constant::TIMER_DURATION;
use ic_stable_structures::btreemap::BTreeMap;
use ic_stable_structures::{Memory, Storable};
use std::fmt::Debug;
use crate::api::hot_or_not_bet::tabulate_hot_or_not_outcome_for_post_slot::tabulate_hot_or_not_outcome_for_post_slot;

use crate::{
    api::{
        canister_management::update_last_access_time::update_last_canister_functionality_access_time,
        post::update_scores_and_share_with_post_cache_if_difference_beyond_threshold::update_scores_and_share_with_post_cache_if_difference_beyond_threshold,
    },
    data_model::CanisterData,
    CANISTER_DATA,
};

#[update]
fn receive_bet_from_bet_makers_canister(
    place_bet_arg: PlaceBetArg,
    bet_maker_principal_id: Principal,
) -> Result<BettingStatus, BetOnCurrentlyViewingPostError> {
    let bet_maker_canister_id = ic_cdk::caller();
    update_last_canister_functionality_access_time();

    ic_cdk::println!("receive_bet_from_bet_makers_canister - creator_post: {:?}",&place_bet_arg.post_id);

    let current_time = system_time::get_current_system_time_from_ic();

    let status = CANISTER_DATA.with(|canister_data_ref_cell| {
        let canister_data = &mut canister_data_ref_cell.borrow_mut();
       
        let bet_result = receive_bet_from_bet_makers_canister_impl(
            canister_data,
            &bet_maker_principal_id,
            &bet_maker_canister_id,
            place_bet_arg.clone(),
            &current_time,
        );

        maybe_start_timer_based_on_bet_result(canister_data, current_time, place_bet_arg.clone(), bet_result.clone());

        bet_result
    })?;

    CANISTER_DATA.with(|canister_data_ref_cell| {
        update_profile_stats_with_bet_placed(
            &mut canister_data_ref_cell.borrow_mut(),
            &place_bet_arg.bet_direction,
        );
    });

    update_scores_and_share_with_post_cache_if_difference_beyond_threshold(&place_bet_arg.post_id);

    Ok(status)
}

fn maybe_start_timer_based_on_bet_result(canister_data: &mut CanisterData, current_time: SystemTime, place_bet_arg: PlaceBetArg, bet_result: Result<BettingStatus, BetOnCurrentlyViewingPostError>) {
    // let current_time = system_time::get_current_system_time_from_ic();

    if let Ok(ok_bet_result) = bet_result {
    match ok_bet_result {
        // this case should never match in yral game implementation
        BettingStatus::BettingClosed => {
            ();
        }
        BettingStatus::BettingOpen {
            ongoing_slot,
            ongoing_room,
            ..
        } => {
            // insert only the first bet in first_bet_placed_at_hashmap
            if !canister_data
            .first_bet_placed_at_hashmap
            .contains_key(&place_bet_arg.post_id)
        {
            ic_cdk::println!("current_time: {:?}", current_time);
            canister_data.first_bet_placed_at_hashmap.insert(
                place_bet_arg.post_id,
                (SystemTimeInMs::from_system_time(current_time), ongoing_slot),
            );
            ic_cdk::println!("maybe_start_timer_based_on_bet_result: canister_data.first_bet_placed_at_hashmap: {}", print_btree_map(&canister_data.first_bet_placed_at_hashmap));
            
            // also push the post_id to the queue
            let bet_timer_posts = &mut canister_data.bet_timer_posts;

            let _to_print = match bet_timer_posts.insert(
                (
                    SystemTimeInMs::from_system_time(current_time),
                    place_bet_arg.post_id,
                ),
                (),
            ) {
                Some(timer) => format!("Timer pushed to empty array: {:?}", timer),
                None => "Failed to push timer to empty array".to_string(),
            };

            ic_cdk::println!(
                "before maybe_enqueue_timer (maybe_start_timer_based_on_bet_result) - bet_timer_posts: {:?}",
                print_btree_map(&canister_data.bet_timer_posts)
            );

            maybe_enqueue_timer(canister_data);
        }
        ic_cdk::println!("second bet on same post: {:?}", place_bet_arg.post_id);
    }
}
    }
}

fn receive_bet_from_bet_makers_canister_impl(
    canister_data: &mut CanisterData,
    bet_maker_principal_id: &Principal,
    bet_maker_canister_id: &CanisterId,
    place_bet_arg: PlaceBetArg,
    current_time: &SystemTime,
) -> Result<BettingStatus, BetOnCurrentlyViewingPostError> {
    let PlaceBetArg {
        post_id,
        bet_amount,
        bet_direction,
        ..
    } = place_bet_arg;

    let post = canister_data.all_created_posts.get_mut(&post_id).unwrap();

    ic_cdk::println!("receive_bet_from_bet_makers_canister_impl - post: {:?}",&post);

    post.place_hot_or_not_bet_v2(
        bet_maker_principal_id,
        bet_maker_canister_id,
        bet_amount,
        &bet_direction,
        current_time,
        &mut canister_data.room_details_map,
        &mut canister_data.bet_details_map,
        &mut canister_data.post_principal_map,
        &mut canister_data.slot_details_map,
    )
}

// TIMER LOGIC STARTS HERE

fn maybe_enqueue_timer(canister_data: &mut CanisterData) {
    let should_start_timer = match canister_data.is_timer_running {
        Some(post_id) => process_running_timer(canister_data, post_id),
        None => !canister_data.first_bet_placed_at_hashmap.is_empty(),
    };

    ic_cdk::println!(
        "inside maybe_enqueue_timer: timer_running = {:?} ; should_start_timer = {:?} \n",
        &canister_data.is_timer_running,
        should_start_timer
    );

    if should_start_timer {
        start_timer(canister_data);
    }
}

fn process_running_timer(canister_data: &mut CanisterData, post_id: u64) -> bool {
    ic_cdk::println!("process_running_timer for post_id: {:?}", post_id);
    if !timer_expired(post_id, canister_data) {
        // don't start timer again if previous one has not expired yet
        ic_cdk::println!("process_running_timer for post_id: {:?} -  early_return as timer is not expired", post_id);
        return false;
    }

    ic_cdk::println!("process_running_timer - canister_data.first_bet_placed_at_hashmap for post_id: {:?}",canister_data.first_bet_placed_at_hashmap.get(&post_id));

    if let Some((_, ongoing_slot)) = canister_data.first_bet_placed_at_hashmap.get(&post_id) {
        tabulate_hot_or_not_outcome_for_post_slot(canister_data, post_id, ongoing_slot);

        ic_cdk::println!(
            "\n\n canister_data.bet_timer_posts: {:?}",
            print_btree_map(&canister_data.bet_timer_posts)
        );
        ic_cdk::println!("\n\n BEFORE PROCESSING TIMER {}", "//".repeat(400));
        // remove the post_id from the hashmap and bet_timer_posts
        let val = canister_data.first_bet_placed_at_hashmap.remove(&post_id);
        let same_post_id = canister_data.bet_timer_posts.pop_first();

        canister_data.is_timer_running = None;
        ic_cdk::println!(" after PROCESSING TIMER {}", "--".repeat(400));
        ic_cdk::println!(
            "\n\n processed_timer for -- post_id: {:?}, same_post_id: {:?}",
            post_id,
            same_post_id
        );
        ic_cdk::println!("\n\n canister_data.first_bet_placed_at_hashmap NOW: {:?}", print_btree_map(&canister_data.first_bet_placed_at_hashmap));
        ic_cdk::println!(
            "\n\n canister_data.bet_timer_posts: {:?}",
            print_btree_map(&canister_data.bet_timer_posts)
        );

        // return true to indicate that timer has been processed
        true
    } else {
        false
    }
}

pub fn print_btree_map<K, V, M>(btree: &BTreeMap<K, V, M>) -> String
where
    K: Ord + Debug + Storable + Clone,
    V: Debug + Storable,
    M: Memory,
{
    let mut result = String::from("{");
    let mut iter = btree.iter();

    if let Some((key, value)) = iter.next() {
        result.push_str(&format!("{:?}: {:?}", key, value));
    }

    for (key, value) in iter {
        result.push_str(&format!(", {:?}: {:?}", key, value));
    }

    result.push('}');
    result
}

fn start_timer(canister_data: &mut CanisterData) {
    if !canister_data.first_bet_placed_at_hashmap.is_empty() {
        // bet_timer_posts is a queue with head at the last element of array
        // and tail at the first element of array.
        // this is because `pop` removes the last entry from the vec in ic_stable_structures
        if let Some(((insertion_time, first_post_id), _)) = canister_data.bet_timer_posts.first_key_value() {
            // if let Some(composite_key) = canister_data.bet_timer_posts.pop_first() {
            // ic_cdk::println!("\n--------- post_id: {}, starting timer ------\n", first_post_id);
            ic_cdk::println!("\n--------- post_id: {:?}, starting timer ------ bet_placed_at: {:?} \n", first_post_id, insertion_time);
            if let Some((bet_placed_time, _ongoing_slot_for_post)) =
                canister_data.first_bet_placed_at_hashmap.get(&first_post_id)
            {
                let current_time = SystemTimeInMs::now();
                let interval_for_timer = current_time.calculate_remaining_interval(&bet_placed_time, TIMER_DURATION).unwrap();

                canister_data.is_timer_running = Some(first_post_id);

                ic_cdk_timers::set_timer(interval_for_timer, move || {

                    CANISTER_DATA.with(|canister_data_ref_cell| {
                        let canister_data = &mut canister_data_ref_cell.borrow_mut();
                        ic_cdk::println!("INSIDE start_timer - canister_data.first_bet_placed_at_hashmap: {}, post_id: {}, interval_for_timer: {:?} \n",print_btree_map(&canister_data.first_bet_placed_at_hashmap), first_post_id, interval_for_timer);

                        maybe_enqueue_timer(canister_data);
                    });
                });
            }
        }
    }
}

fn timer_expired(post_id: PostId, canister_data: &CanisterData) -> bool {
    if !canister_data.first_bet_placed_at_hashmap.is_empty() {
        if let Some(((first_time, first_post_id), _)) =
            canister_data.bet_timer_posts.first_key_value()
        {
            if let Some((bet_placed_time, _ongoing_slot_for_post)) =
                canister_data.first_bet_placed_at_hashmap.get(&post_id)
            {
                let current_time = SystemTimeInMs::now();
                let interval = current_time.duration_since(&bet_placed_time);
                let return_val = interval > TIMER_DURATION;

                ic_cdk::println!(
                    "post_id ({}) == last_post_id ({}), {}, \n bet_timer_posts: {:?}, \n bet_timer_posts.time: {:?} === first_bet_placed_at_hashmap.time: {:?}", 
                    post_id,
                    first_post_id,
                    post_id == first_post_id,
                    print_btree_map(&canister_data.bet_timer_posts),
                    first_time, 
                    bet_placed_time
                );

                ic_cdk::println!("post_id: {}, timer_expired: {:?}", post_id, return_val);

                return return_val;
            }
        }
    }
    ic_cdk::println!(
        "post_id: {}, timer_expired: FALSE (returning from outside)",
        post_id
    );

    false
}

// TIMER LOGIC ENDS HERE

fn update_profile_stats_with_bet_placed(
    canister_data: &mut CanisterData,
    bet_direction: &BetDirection,
) {
    match *bet_direction {
        BetDirection::Hot => {
            canister_data.profile.profile_stats.hot_bets_received += 1;
        }
        BetDirection::Not => {
            canister_data.profile.profile_stats.not_bets_received += 1;
        }
    }
}

#[cfg(test)]
mod test {
    use shared_utils::{
        canister_specific::individual_user_template::types::{
            hot_or_not::{BetDirection, GlobalBetId, GlobalRoomId, StablePrincipal},
            post::{Post, PostDetailsFromFrontend},
        },
        common::types::utility_token::token_event::NewSlotType,
    };
    use test_utils::setup::test_constants::{
        get_mock_user_alice_canister_id, get_mock_user_alice_principal_id,
    };

    use super::*;

    #[test]
    fn test_receive_bet_from_bet_makers_canister_impl() {
        let mut canister_data = CanisterData::default();
        canister_data.all_created_posts.insert(
            0,
            Post::new(
                0,
                &PostDetailsFromFrontend {
                    is_nsfw: false,
                    description: "Doggos and puppers".into(),
                    hashtags: vec!["doggo".into(), "pupper".into()],
                    video_uid: "abcd#1234".into(),
                    creator_consent_for_inclusion_in_hot_or_not: true,
                },
                &SystemTime::now(),
            ),
        );

        let result = receive_bet_from_bet_makers_canister_impl(
            &mut canister_data,
            &get_mock_user_alice_principal_id(),
            &get_mock_user_alice_canister_id(),
            PlaceBetArg {
                post_canister_id: get_mock_user_alice_canister_id(),
                post_id: 0,
                bet_amount: 100,
                bet_direction: BetDirection::Hot,
            },
            &SystemTime::now(),
        );

        let post = canister_data.all_created_posts.get(&0).unwrap();
        let global_room_id = GlobalRoomId(0, NewSlotType(1), 1);
        let global_bet_id = GlobalBetId(
            global_room_id,
            StablePrincipal(get_mock_user_alice_principal_id()),
        );

        let room_details = canister_data.room_details_map.get(&global_room_id).unwrap();

        let bet_details = canister_data.bet_details_map.get(&global_bet_id).unwrap();

        assert_eq!(
            result,
            Ok(BettingStatus::BettingOpen {
                started_at: post.created_at,
                number_of_participants: 1,
                ongoing_slot: NewSlotType(1),
                ongoing_room: 1,
                has_this_user_participated_in_this_post: Some(true)
            })
        );

        assert_eq!(room_details.room_bets_total_pot, 100);
        assert_eq!(room_details.total_hot_bets, 1);
        assert_eq!(room_details.total_not_bets, 0);

        assert_eq!(bet_details.amount, 100);
        assert_eq!(bet_details.bet_direction, BetDirection::Hot);
        assert_eq!(
            bet_details.bet_maker_canister_id,
            get_mock_user_alice_canister_id()
        );
    }
}
