use ethnum::{I256, U256};

use super::types::{PoolId, PoolTickSpacing};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct SwapParams {
    pub pool_id: PoolId,
    pub amount_specified: I256,
    pub tick_spacing: PoolTickSpacing,
    pub zero_for_one: bool,
    pub sqrt_price_limit_x96: U256,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct StepComputations {
    // the price at the beginning of the step
    pub sqrt_price_start_x96: U256,
    // the next tick to swap to from the current tick in the swap direction
    pub tick_next: i32,
    // whether tickNext is initialized or not
    pub initialized: bool,
    // sqrt(price) for the next tick (1/0)
    pub sqrt_price_next_x96: U256,
    // how much is being swapped in in this step
    pub amount_in: U256,
    // how much is being swapped out
    pub amount_out: U256,
    // how much fee is being paid in
    pub fee_amount: U256,
    // the global fee growth of the input token. updated in storage at the end of swap
    pub fee_growth_global_x128: U256,
}

/// Keeps state changes, in case of success, state transition will be applied using this buffer
/// state, In case of failure no state transition will be triggered
/// Buffer for state changes to apply only on successful modification.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct SwapBufferState {}

// Tracks the state of a pool throughout a swap, and returns these values at the end of the swap
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct SwapResult {
    // the current sqrt(price)
    pub sqrt_price_x96: U256,
    // the tick associated with the current price
    pub tick: i32,
    // the current liquidity in range
    pub liquidity: u128,
    // buffer state after swap
    pub buffer_state: SwapBufferState,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum SwapError {}

pub fn swap(params: SwapParams) -> Result<SwapResult, SwapError> {
    todo!()
}
