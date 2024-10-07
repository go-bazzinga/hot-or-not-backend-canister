use ic_stable_structures::StableBTreeMap;
use serde::{Serialize, Deserialize};
use shared_utils::canister_specific::individual_user_template::types::airdrop::AirdropMember;

use super::memory::{get_airdrop_token_chain_memory, Memory};

#[derive(Deserialize, Serialize)]
pub struct AirdropData {
    pub parent: Option<AirdropMember>,
    #[serde(skip, default = "_default_token_chain")]
    pub token_chain: StableBTreeMap<AirdropMember, (), Memory>,
}

pub fn _default_token_chain() -> StableBTreeMap<AirdropMember, (), Memory> {
    StableBTreeMap::init(get_airdrop_token_chain_memory())
}

impl Default for AirdropData {
    fn default() -> Self {
        Self {
            parent: None,
            token_chain: _default_token_chain(),
        }
    }
}