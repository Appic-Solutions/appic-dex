use candid::Principal;
use ethnum::{I256, U256};

use crate::{
    candid_types::position::{IncreaseLiquidity, IncreaseLiquidityArgs},
    libraries::{
        constants::{MAX_TICK, MIN_TICK},
        safe_cast::big_uint_to_u256,
    },
    pool::types::{PoolId, PoolTickSpacing},
    position::types::{PositionInfo, PositionKey},
    state::read_state,
};

pub struct ValidatedIncreaseLiquidityArgs {
    pub tick_spacing: PoolTickSpacing,
    pub lower_tick: i32,
    pub upper_tick: i32,
    pub pool_id: PoolId,
    pub position_key: PositionKey,
    pub position_info: PositionInfo,
    pub amount0_max: I256,
    pub amount1_max: I256,
}

pub fn validate_increase_liquidity_args(
    args: IncreaseLiquidityArgs,
    caller: Principal,
) -> Result<ValidatedIncreaseLiquidityArgs, IncreaseLiquidity> {
    // check pool
    let pool_id: PoolId = args
        .pool
        .try_into()
        .map_err(|_e| IncreaseLiquidity::InvalidPoolFee)?;

    let pool = read_state(|s| s.get_pool(&pool_id)).ok_or(IncreaseLiquidity::PoolNotInitialized)?;
    let tick_spacing = pool.tick_spacing;
    // check ticks
    let lower_tick: i32 = args
        .tick_lower
        .0
        .try_into()
        .map_err(|_e| IncreaseLiquidity::InvalidTick)?;

    let upper_tick: i32 = args
        .tick_upper
        .0
        .try_into()
        .map_err(|_e| IncreaseLiquidity::InvalidTick)?;
    if lower_tick < MIN_TICK || upper_tick > MAX_TICK || lower_tick >= upper_tick {
        return Err(IncreaseLiquidity::InvalidTick);
    };

    // position should not exist
    let position_key = PositionKey {
        owner: caller,
        pool_id: pool_id.clone(),
        tick_lower: lower_tick,
        tick_upper: upper_tick,
    };

    let position_info = read_state(|s| s.get_position(&position_key));
    if position_info.liquidity == 0 {
        return Err(IncreaseLiquidity::PositionDoesNotExist);
    }

    // check alignment with tick spacing
    if upper_tick % tick_spacing.0 != 0 || lower_tick % tick_spacing.0 != 0 {
        return Err(IncreaseLiquidity::TickNotAlignedWithTickSpacing);
    };

    let amount0_max: U256 =
        big_uint_to_u256(args.amount0_max.0).map_err(|_e| IncreaseLiquidity::InvalidAmount)?;
    let amount1_max: U256 =
        big_uint_to_u256(args.amount1_max.0).map_err(|_e| IncreaseLiquidity::InvalidAmount)?;

    // MAX amount should be I256 to prevent overflow
    let amount0_max: I256 = amount0_max
        .try_into()
        .map_err(|_e| IncreaseLiquidity::InvalidAmount)?;
    let amount1_max: I256 = amount1_max
        .try_into()
        .map_err(|_e| IncreaseLiquidity::InvalidAmount)?;

    Ok(ValidatedIncreaseLiquidityArgs {
        tick_spacing,
        lower_tick,
        upper_tick,
        pool_id,
        amount0_max,
        amount1_max,
        position_key,
        position_info,
    })
}
