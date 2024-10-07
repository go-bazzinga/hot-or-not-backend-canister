use candid::Principal;
use ic_cdk::{notify, update};
use shared_utils::{canister_specific::individual_user_template::types::{airdrop::AirdropMember, session::SessionType}, common::{participant_crypto::ProofOfParticipation, types::utility_token::token_event::{MintEvent, TokenEvent}, utils::system_time}};

use crate::{api::canister_management::update_last_access_time::update_last_canister_functionality_access_time, CANISTER_DATA};

use super::airdrop::add_user_to_airdrop_chain_inner;

pub(crate) fn coyn_token_reward_for_referral(referrer: Principal, referree: Principal) {
    let current_time = system_time::get_current_system_time_from_ic();

    CANISTER_DATA.with_borrow_mut(|cdata| {
        let my_token_balance = &mut cdata.my_token_balance;

        let referral_reward_amount = TokenEvent::get_token_amount_for_token_event(&TokenEvent::Mint {
            amount: 0,
            details: MintEvent::Referral {
                referee_user_principal_id: referree,
                referrer_user_principal_id: referrer,
            },
            timestamp: current_time,
        });

        my_token_balance.handle_token_event(TokenEvent::Mint {
            amount: referral_reward_amount,
            details: MintEvent::Referral {
                referrer_user_principal_id: referrer,
                referee_user_principal_id: referree,
            },
            timestamp: current_time,
        });
    })
}

#[update]
pub async fn receive_reward_for_being_referred() -> Result<(), String> {
    let (pop, user_principal, referrer_details, session_type, has_parent) = CANISTER_DATA.with_borrow(|cdata| {
        let profile = &cdata.profile;
        (
            cdata.proof_of_participation.clone(),
            profile.principal_id,
            profile.referrer_details.clone(),
            cdata.session_type,
            cdata.airdrop.parent.is_some()
        )
    });

    let Some(pop) = pop else {
        return Err("method is not available right now".into());
    };

    let Some(user_principal) = user_principal else {
        return Err("canister is not ready".into());
    };

    let Some(referrer_details) = referrer_details else {
        return Err("no referrer details found".into());
    };

    if session_type != Some(SessionType::RegisteredSession) {
        return Err("user not signed up".into());
    }

    if has_parent {
        return Err("User has already claimed the reward".into());
    }

    update_last_canister_functionality_access_time();

    coyn_token_reward_for_referral(referrer_details.profile_owner, user_principal);

    let referrer_member = AirdropMember {
        user_canister: referrer_details.user_canister_id,
        user_principal: referrer_details.profile_owner,
    };
    CANISTER_DATA.with_borrow_mut(|cdata| {
        cdata.airdrop.parent = Some(referrer_member);
    });
    add_user_to_airdrop_chain_inner(referrer_member).await;

    // Rollback if the notification fails
    notify(
        referrer_details.user_canister_id,
        "receive_reward_for_referring",
        (pop, user_principal)
    ).map_err(|_| "failed to reward referrer".to_string())
    .unwrap();

    Ok(())
}

#[update]
pub async fn receive_reward_for_referring(pop: ProofOfParticipation, referree_principal: Principal) -> Result<(), String> {
    pop.verify_caller_is_participant(&CANISTER_DATA).await?;

    let Some(profile_owner) = CANISTER_DATA.with_borrow(|cdata| cdata.profile.principal_id) else {
        return Err("canister is not ready".into());
    };

    coyn_token_reward_for_referral(profile_owner, referree_principal);

    let referree_canister = ic_cdk::caller();
    let member = AirdropMember {
        user_principal: referree_principal,
        user_canister: referree_canister
    };
    add_user_to_airdrop_chain_inner(member).await;

    let Some(parent) = CANISTER_DATA.with_borrow(|cdata| cdata.airdrop.parent) else {
        return Ok(())
    };

    let Some(pop) = CANISTER_DATA.with_borrow(|cdata| cdata.proof_of_participation.clone()) else {
        return Err("method is not available right now".into());
    };

    notify(
        parent.user_canister,
        "add_user_to_airdrop_chain",
        (pop, member)
    ).unwrap();

    Ok(())
}
