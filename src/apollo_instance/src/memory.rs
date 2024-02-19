use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::DefaultMemoryImpl;
use std::cell::RefCell;

// A memory for upgrades, where data from the heap can be serialized/deserialized.
const UPGRADES_MEMORY_ID: MemoryId = MemoryId::new(0);
// A memory for canister metadata
const METADATA_MEMORY_ID: MemoryId = MemoryId::new(1);
// A memory for balances of users
const BALANCES_MEMORY_ID: MemoryId = MemoryId::new(2);
// A memory for withdraw requests
const WITHDRAW_REQUESTS_MEMORY_ID: MemoryId = MemoryId::new(3);
// A memory for allowed contracts
const ALLOWANCES_MEMORY_ID: MemoryId = MemoryId::new(4);

pub type VMemory = VirtualMemory<DefaultMemoryImpl>;

thread_local! {
    // The memory manager is used for simulating multiple memories. Given a `MemoryId` it can
    // return a memory that can be used by stable structures.
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
}

pub fn get_upgrades_memory() -> VMemory {
    MEMORY_MANAGER.with(|m| m.borrow().get(UPGRADES_MEMORY_ID))
}

pub fn get_metadata_memory() -> VMemory {
    MEMORY_MANAGER.with(|m| m.borrow().get(METADATA_MEMORY_ID))
}

pub fn get_balances_memory() -> VMemory {
    MEMORY_MANAGER.with(|m| m.borrow().get(BALANCES_MEMORY_ID))
}

pub fn get_withdraw_requests_memory() -> VMemory {
    MEMORY_MANAGER.with(|m| m.borrow().get(WITHDRAW_REQUESTS_MEMORY_ID))
}

pub fn get_allowances_memory() -> VMemory {
    MEMORY_MANAGER.with(|m| m.borrow().get(ALLOWANCES_MEMORY_ID))
}
