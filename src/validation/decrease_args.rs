use candid::Principal;
use ethnum::{I256, U256};

use crate::{
    candid_types::position::{DecreaseLiquidityArgs, DecreaseLiquidityError},
    libraries::{
        constants::{MAX_TICK, MIN_TICK},
        safe_cast::big_uint_to_u256,
    },
    pool::types::{PoolId, PoolTickSpacing},
    position::types::{PositionInfo, PositionKey},
    state::read_state,
};

pub struct ValidatedDecreaseLiquidityArgs {
    pub tick_spacing: PoolTickSpacing,
    pub lower_tick: i32,
    pub upper_tick: i32,
    pub position_key: PositionKey,
    pub position_info: PositionInfo,
    pub pool_id: PoolId,
    pub amount0_min: I256,
    pub amount1_min: I256,
    pub liquidity_delta: i128,
}

pub fn validate_decrease_liquidity_args(
    args: DecreaseLiquidityArgs,
    caller: Principal,
) -> Result<ValidatedDecreaseLiquidityArgs, DecreaseLiquidityError> {
    // check pool
    let pool_id: PoolId = args
        .pool
        .try_into()
        .map_err(|_e| DecreaseLiquidityError::InvalidPoolFee)?;
    let pool =
        read_state(|s| s.get_pool(&pool_id)).ok_or(DecreaseLiquidityError::PoolNotInitialized)?;
    let tick_spacing = pool.tick_spacing;
    // check ticks
    let lower_tick: i32 = args
        .tick_lower
        .0
        .try_into()
        .map_err(|_e| DecreaseLiquidityError::InvalidTick)?;
    let upper_tick: i32 = args
        .tick_upper
        .0
        .try_into()
        .map_err(|_e| DecreaseLiquidityError::InvalidTick)?;
    if lower_tick < MIN_TICK || upper_tick > MAX_TICK || lower_tick >= upper_tick {
        return Err(DecreaseLiquidityError::InvalidTick);
    };

    let liquidity_delta: u128 = args
        .liquidity
        .0
        .try_into()
        .map_err(|_| DecreaseLiquidityError::InvalidLiquidity)?;

    // position should not exist
    let position_key = PositionKey {
        owner: caller,
        pool_id: pool_id.clone(),
        tick_lower: lower_tick,
        tick_upper: upper_tick,
    };
    let position_info = read_state(|s| s.get_position(&position_key));
    if position_info.liquidity == 0 {
        return Err(DecreaseLiquidityError::PositionNotFound);
    }

    if position_info.liquidity < liquidity_delta {
        return Err(DecreaseLiquidityError::InvalidLiquidity);
    }

    let amount0_min: U256 =
        big_uint_to_u256(args.amount0_min.0).map_err(|_e| DecreaseLiquidityError::InvalidAmount)?;
    let amount1_min: U256 =
        big_uint_to_u256(args.amount1_min.0).map_err(|_e| DecreaseLiquidityError::InvalidAmount)?;

    // MIN amount should be I256 to prevent overflow
    let amount0_min: I256 = amount0_min
        .try_into()
        .map_err(|_e| DecreaseLiquidityError::InvalidAmount)?;
    let amount1_min: I256 = amount1_min
        .try_into()
        .map_err(|_e| DecreaseLiquidityError::InvalidAmount)?;

    let liquidity_delta = i128::try_from(liquidity_delta)
        .map_err(|_e| DecreaseLiquidityError::LiquidityOverflow)?
        .checked_mul(-1i128)
        .ok_or(DecreaseLiquidityError::LiquidityOverflow)?;

    Ok(ValidatedDecreaseLiquidityArgs {
        tick_spacing,
        lower_tick,
        upper_tick,
        position_key,
        position_info,
        pool_id,
        amount0_min,
        amount1_min,
        liquidity_delta,
    })
}
