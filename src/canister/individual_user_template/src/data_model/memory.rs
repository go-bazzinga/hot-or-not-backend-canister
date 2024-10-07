use ic_stable_structures::{
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
    DefaultMemoryImpl,
};
use std::cell::RefCell;

// A memory for upgrades, where data from the heap can be serialized/deserialized.
const UPGRADES: MemoryId = MemoryId::new(0);

// A memory for the StableBTreeMap we're using. A new memory should be created for
// every additional stable structure.

const ROOM_DETAILS_MEMORY: MemoryId = MemoryId::new(1);
const BET_DETAILS_MEMORY: MemoryId = MemoryId::new(2);
const POST_PRINCIPAL_MEMORY: MemoryId = MemoryId::new(3);
const SLOT_DETAILS_MEMORY: MemoryId = MemoryId::new(4);
const KV_STORAGE_NAMESPACE_MEMORY: MemoryId = MemoryId::new(5);
const KV_STORAGE_NAMESPACE_KEY_VALUE_MEMORY: MemoryId = MemoryId::new(6);
const WATCH_HISTORY_MEMORY: MemoryId = MemoryId::new(7);
const SUCCESS_HISTORY_MEMORY: MemoryId = MemoryId::new(8);
const TOKEN_LIST_MEMORY: MemoryId = MemoryId::new(9);
const PUBKEY_CACHE_MEMORY: MemoryId = MemoryId::new(10);
const AIRDROP_TOKEN_CHAIN_MEMORY: MemoryId = MemoryId::new(11);

pub type Memory = VirtualMemory<DefaultMemoryImpl>;

thread_local! {
    // The memory manager is used for simulating multiple memories. Given a `MemoryId` it can
    // return a memory that can be used by stable structures.
    pub static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init_with_bucket_size(DefaultMemoryImpl::default(), 1));
}

pub fn get_upgrades_memory() -> Memory {
    MEMORY_MANAGER.with(|m| m.borrow_mut().get(UPGRADES))
}

pub fn get_room_details_memory() -> Memory {
    MEMORY_MANAGER.with(|m| m.borrow_mut().get(ROOM_DETAILS_MEMORY))
}

pub fn get_bet_details_memory() -> Memory {
    MEMORY_MANAGER.with(|m| m.borrow_mut().get(BET_DETAILS_MEMORY))
}

pub fn get_post_principal_memory() -> Memory {
    MEMORY_MANAGER.with(|m| m.borrow_mut().get(POST_PRINCIPAL_MEMORY))
}

pub fn get_slot_details_memory() -> Memory {
    MEMORY_MANAGER.with(|m| m.borrow_mut().get(SLOT_DETAILS_MEMORY))
}

pub fn get_kv_storage_namespace_memory() -> Memory {
    MEMORY_MANAGER.with(|m| m.borrow_mut().get(KV_STORAGE_NAMESPACE_MEMORY))
}

pub fn get_kv_storage_namespace_key_value_memory() -> Memory {
    MEMORY_MANAGER.with(|m| m.borrow_mut().get(KV_STORAGE_NAMESPACE_KEY_VALUE_MEMORY))
}

pub fn get_watch_history_memory() -> Memory {
    MEMORY_MANAGER.with(|m| m.borrow_mut().get(WATCH_HISTORY_MEMORY))
}

pub fn get_success_history_memory() -> Memory {
    MEMORY_MANAGER.with(|m| m.borrow_mut().get(SUCCESS_HISTORY_MEMORY))
}

pub fn get_token_list_memory() -> Memory {
    MEMORY_MANAGER.with(|m| m.borrow_mut().get(TOKEN_LIST_MEMORY))
}

pub fn get_pubkey_cache_memory() -> Memory {
    MEMORY_MANAGER.with(|m| m.borrow_mut().get(PUBKEY_CACHE_MEMORY))
}

pub fn get_airdrop_token_chain_memory() -> Memory {
    MEMORY_MANAGER.with(|m| m.borrow_mut().get(AIRDROP_TOKEN_CHAIN_MEMORY))
}

pub fn init_memory_manager() {
    MEMORY_MANAGER.with(|m| {
        *m.borrow_mut() = MemoryManager::init_with_bucket_size(DefaultMemoryImpl::default(), 1);
    })
}
