
// use crate::CANISTER_DATA;
// use super::receive_bet_from_bet_makers_canister::maybe_enqueue_timer;

// pub fn reenqueue_timers_for_pending_bet_outcomes_v1() {
//     CANISTER_DATA.with(|canister_data_ref_cell| {
//         let canister_data = &mut canister_data_ref_cell.borrow_mut();
//         maybe_enqueue_timer(canister_data);
//     });
// }