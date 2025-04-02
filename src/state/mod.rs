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
    pub static STATE: State = State {
        pools: RefCell::new(BTreeMap::init(pools_memory_id())),
        pool_balances: RefCell::new(BTreeMap::init(pool_balances_memory_id())),
        positions: RefCell::new(BTreeMap::init(positions_memory_id())),
        ticks: RefCell::new(BTreeMap::init(ticks_memory_id())),
        tick_bitmaps: RefCell::new(BTreeMap::init(ticks_memory_id())),
    };
}

pub struct State {
    pools: RefCell<BTreeMap<PoolId, PoolState, StableMemory>>,
    pool_balances: RefCell<BTreeMap<TokenId, TokenBalance, StableMemory>>,
    positions: RefCell<BTreeMap<PositionKey, PositionInfo, StableMemory>>,
    ticks: RefCell<BTreeMap<TickKey, TickInfo, StableMemory>>,
    tick_bitmaps: RefCell<BTreeMap<TickBitmapKey, BitmapWord, StableMemory>>,
}

impl State {}
