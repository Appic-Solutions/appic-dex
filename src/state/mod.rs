// [DEX Canister State]
//  ├── Stable Memory
//  │   ├── POOL_REGISTRY: StableBTreeMap<PoolId, PoolMetadata>
//  │   ├── TICK_DATA: StableBTreeMap<TickKey, TickInfo>
//  │   ├── POSITION_DATA: StableBTreeMap<PositionId, PositionInfo>
//  │   ├── TOKEN_BALANCES: StableBTreeMap<TokenId, u128>
//  │   └── TICK_BITMAP: StableBTreeMap<TickBitmapKey, U256>
//  └──

use crate::{
    balances::types::{UserBalance, UserBalanceKey},
    pool::{
        modify_liquidity::ModifyLiquidityBufferState,
        swap::SwapBufferState,
        types::{PoolFee, PoolId, PoolState, PoolTickSpacing},
    },
    position::types::{PositionInfo, PositionKey},
    tick::types::{BitmapWord, TickBitmapKey, TickInfo, TickKey},
};

use candid::Principal;
use ethnum::U256;
use ic_stable_structures::BTreeMap;
use memory_manager::{
    pools_memory_id, positions_memory_id, protocol_balance_memory_id, tick_bitmaps_memory_id,
    tick_spacings_memory_id, ticks_memory_id, user_balances_memory_id, StableMemory,
};
use std::cell::RefCell;

pub mod memory_manager;
pub mod storable_impl;

thread_local! {
    pub static STATE: RefCell<Option<State>> = RefCell::new(Some(State {
        pools: BTreeMap::init(pools_memory_id()),
        user_balances: BTreeMap::init(user_balances_memory_id()),
        protocol_balance:BTreeMap::init(protocol_balance_memory_id()),
        positions: BTreeMap::init(positions_memory_id()),
        ticks: BTreeMap::init(ticks_memory_id()),
        tick_bitmaps: BTreeMap::init(tick_bitmaps_memory_id()),
        tick_spacings:BTreeMap::init(tick_spacings_memory_id())
    }));
}

pub struct State {
    pools: BTreeMap<PoolId, PoolState, StableMemory>,
    user_balances: BTreeMap<UserBalanceKey, UserBalance, StableMemory>,
    protocol_balance: BTreeMap<Principal, UserBalance, StableMemory>, // protocol accumulated from protocol-fee
    positions: BTreeMap<PositionKey, PositionInfo, StableMemory>,
    ticks: BTreeMap<TickKey, TickInfo, StableMemory>,
    tick_bitmaps: BTreeMap<TickBitmapKey, BitmapWord, StableMemory>,
    tick_spacings: BTreeMap<PoolFee, PoolTickSpacing, StableMemory>,
}

impl State {
    pub fn get_tick(&self, tick: &TickKey) -> TickInfo {
        self.ticks.get(tick).unwrap_or(TickInfo::default())
    }

    pub fn update_tick(&mut self, tick: TickKey, info: TickInfo) {
        self.ticks.insert(tick, info);
    }

    pub fn revert_tick(&mut self, tick: TickKey, previous_info: TickInfo) {
        self.ticks.insert(tick, previous_info);
    }

    pub fn clear_tick(&mut self, tick: &TickKey) {
        self.ticks.remove(tick);
    }

    pub fn get_position(&self, key: &PositionKey) -> PositionInfo {
        self.positions.get(key).unwrap_or_default()
    }

    pub fn update_position(&mut self, key: PositionKey, info: PositionInfo) {
        self.positions.insert(key, info);
    }

    pub fn revert_position(&mut self, key: PositionKey, previous_info: PositionInfo) {
        self.positions.insert(key, previous_info);
    }

    pub fn get_tick_spacing(&self, fee: &PoolFee) -> Option<PoolTickSpacing> {
        self.tick_spacings.get(fee)
    }

    pub fn set_tick_spacing(&mut self, fee: PoolFee, tick_spacing: PoolTickSpacing) {
        self.tick_spacings.insert(fee, tick_spacing);
    }

    pub fn get_pool(&self, pool_id: &PoolId) -> Option<PoolState> {
        self.pools.get(pool_id)
    }

    pub fn get_pools(&self) -> Vec<(PoolId, PoolState)> {
        self.pools.iter().collect()
    }

    pub fn set_pool(&mut self, pool_id: PoolId, pool_state: PoolState) {
        self.pools.insert(pool_id, pool_state);
    }

    pub fn get_bitmap_word(&self, bitmap_key: &TickBitmapKey) -> BitmapWord {
        self.tick_bitmaps
            .get(bitmap_key)
            .unwrap_or(BitmapWord(U256::ZERO))
    }

    pub fn set_bitmap_word(&mut self, bitmap_key: TickBitmapKey, bitmap_word: BitmapWord) {
        self.tick_bitmaps.insert(bitmap_key, bitmap_word);
    }

    pub fn get_user_balance(&self, key: &UserBalanceKey) -> UserBalance {
        self.user_balances
            .get(key)
            .unwrap_or(UserBalance(U256::ZERO))
    }

    pub fn update_user_balance(&mut self, key: UserBalanceKey, value: UserBalance) {
        self.user_balances.insert(key, value);
    }

    pub fn get_protocol_fee_for_token(&mut self, token: &Principal) -> UserBalance {
        self.protocol_balance
            .get(token)
            .unwrap_or(UserBalance(U256::ZERO))
    }

    pub fn update_protocol_fee_for_token(&mut self, token: Principal, value: UserBalance) {
        self.protocol_balance.insert(token, value);
    }

    pub fn apply_modify_liquidity_buffer_state(
        &mut self,
        buffer_state: ModifyLiquidityBufferState,
    ) {
        // pool state transition
        let pool_id = buffer_state.pool.0;
        self.pools.insert(pool_id, buffer_state.pool.1);

        //ticks state transition
        self.ticks
            .insert(buffer_state.tick_lower.0, buffer_state.tick_lower.1);
        self.ticks
            .insert(buffer_state.tick_upper.0, buffer_state.tick_upper.1);

        // position state transition
        if let Some((position_key, position_info)) = buffer_state.position {
            self.positions.insert(position_key, position_info);
        }

        // tickbitmaps state transition
        if let Some((bitmap_key, bitmap_word)) = buffer_state.flipped_lower_tick_bitmap {
            self.tick_bitmaps.insert(bitmap_key, bitmap_word);
        }
        if let Some((bitmap_key, bitmap_word)) = buffer_state.flipped_upper_tick_bitmap {
            self.tick_bitmaps.insert(bitmap_key, bitmap_word);
        }
    }

    pub fn apply_swap_buffer_state(&mut self, buffer_state: SwapBufferState) {
        // pool state transition
        let pool_id = buffer_state.pool.0;
        self.pools.insert(pool_id, buffer_state.pool.1);

        for tick in buffer_state.shifted_ticks.into_iter() {
            self.ticks.insert(tick.0, tick.1);
        }
    }

    pub fn update_token_trnasfer_fee_across_all_pools(
        &mut self,
        token: Principal,
        transfer_fee: U256,
    ) {
        let pools_to_update = self
            .get_pools()
            .into_iter()
            .filter(|(pool_id, _pool_state)| pool_id.token0 == token || pool_id.token1 == token);

        for pool in pools_to_update {
            let mut new_pool_state = pool.1;
            if pool.0.token0 == token {
                new_pool_state.token0_transfer_fee = transfer_fee
            } else {
                new_pool_state.token1_transfer_fee = transfer_fee
            };
            self.pools.insert(pool.0, new_pool_state);
        }
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
