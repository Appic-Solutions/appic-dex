use std::cell::RefCell;

use ic_stable_structures::{
    DefaultMemoryImpl,
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
};

pub type StableMemory = VirtualMemory<DefaultMemoryImpl>;

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );
}

const POOLS_MEMORY_ID: MemoryId = MemoryId::new(0);

pub fn pools_memory_id() -> StableMemory {
    MEMORY_MANAGER.with(|m| m.borrow().get(POOLS_MEMORY_ID))
}

const POSITIONS_MEMORY_ID: MemoryId = MemoryId::new(1);

pub fn positions_memory_id() -> StableMemory {
    MEMORY_MANAGER.with(|m| m.borrow().get(POSITIONS_MEMORY_ID))
}

const TICKS_MEMORY_ID: MemoryId = MemoryId::new(2);

pub fn ticks_memory_id() -> StableMemory {
    MEMORY_MANAGER.with(|m| m.borrow().get(TICKS_MEMORY_ID))
}

const TICK_BITMAPS_MEMORY_ID: MemoryId = MemoryId::new(3);

pub fn tick_bitmaps_memory_id() -> StableMemory {
    MEMORY_MANAGER.with(|m| m.borrow().get(TICK_BITMAPS_MEMORY_ID))
}

const USER_BALANCES_MEMORY_ID: MemoryId = MemoryId::new(4);

pub fn user_balances_memory_id() -> StableMemory {
    MEMORY_MANAGER.with(|m| m.borrow().get(USER_BALANCES_MEMORY_ID))
}

const PROTOCOL_BALANCE: MemoryId = MemoryId::new(5);

pub fn protocol_balance_memory_id() -> StableMemory {
    MEMORY_MANAGER.with(|m| m.borrow().get(PROTOCOL_BALANCE))
}

const TICK_SPACINGS_MEMORY_ID: MemoryId = MemoryId::new(6);

pub fn tick_spacings_memory_id() -> StableMemory {
    MEMORY_MANAGER.with(|m| m.borrow().get(TICK_SPACINGS_MEMORY_ID))
}

const POOL_HISTORY_MEMORY_ID: MemoryId = MemoryId::new(7);

pub fn pool_history_memory_id() -> StableMemory {
    MEMORY_MANAGER.with(|m| m.borrow().get(POOL_HISTORY_MEMORY_ID))
}

const EVENTS_INDEX_MEMORY_ID: MemoryId = MemoryId::new(8);

pub fn events_index_memory_id() -> StableMemory {
    MEMORY_MANAGER.with(|m| m.borrow().get(EVENTS_INDEX_MEMORY_ID))
}

const EVENTS_DATA_MEMORY_ID: MemoryId = MemoryId::new(9);

pub fn events_data_memory_id() -> StableMemory {
    MEMORY_MANAGER.with(|m| m.borrow().get(EVENTS_DATA_MEMORY_ID))
}
