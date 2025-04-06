// [DEX Canister State]
//  ├── Stable Memory
//  │   ├── POOL_REGISTRY: StableBTreeMap<PoolId, PoolMetadata>
//  │   ├── TICK_DATA: StableBTreeMap<TickKey, TickInfo>
//  │   ├── POSITION_DATA: StableBTreeMap<PositionId, PositionInfo>
//  │   ├── TOKEN_BALANCES: StableBTreeMap<TokenId, u128>
//  │   └── TICK_BITMAP: StableBTreeMap<TickBitmapKey, U256>
//  └──

use crate::{
    pool::types::{PoolId, PoolState, TokenBalance, TokenId},
    position::{PositionInfo, PositionKey},
    tick::types::{BitmapWord, TickBitmapKey, TickInfo, TickKey},
};

use ic_stable_structures::BTreeMap;
use memory_manager::{
    pool_balances_memory_id, pools_memory_id, positions_memory_id, ticks_memory_id, StableMemory,
};
use std::cell::RefCell;

pub mod memory_manager;
pub mod storable_impl;

thread_local! {
    pub static STATE: RefCell<Option<State>> = RefCell::new(Some(State {
        pools: BTreeMap::init(pools_memory_id()),
        pool_balances: BTreeMap::init(pool_balances_memory_id()),
        positions: BTreeMap::init(positions_memory_id()),
        ticks: BTreeMap::init(ticks_memory_id()),
        tick_bitmaps: BTreeMap::init(ticks_memory_id()),
    }));
}

pub struct State {
    pools: BTreeMap<PoolId, PoolState, StableMemory>,
    pool_balances: BTreeMap<TokenId, TokenBalance, StableMemory>,
    positions: BTreeMap<PositionKey, PositionInfo, StableMemory>,
    ticks: BTreeMap<TickKey, TickInfo, StableMemory>,
    tick_bitmaps: BTreeMap<TickBitmapKey, BitmapWord, StableMemory>,
}

impl State {
    pub fn get_tick(&self, tick: &TickKey) -> TickInfo {
        self.ticks.get(tick).unwrap_or(TickInfo::default())
    }

    pub fn update_tick(&mut self, tick: TickKey, info: TickInfo) {
        self.ticks.insert(tick, info);
    }

    pub fn clear_tick(&mut self, tick: &TickKey) {
        self.ticks.remove(tick);
    }

    pub fn get_position(&self, positions_key: &PositionKey) -> Option<PositionInfo> {
        self.positions.get(positions_key)
    }
}

pub fn read_state<R>(f: impl FnOnce(&State) -> R) -> R {
    STATE.with(|cell| {
        f(cell
            .borrow()
            .as_ref()
            .expect("BUG: state is not initialized"))
    })
}

// / Mutates (part of) the current state using `f`.
// /
// / Panics if there is no state.
pub fn mutate_state<F, R>(f: F) -> R
where
    F: FnOnce(&mut State) -> R,
{
    STATE.with(|cell| {
        f(cell
            .borrow_mut()
            .as_mut()
            .expect("BUG: state is not initialized"))
    })
}
