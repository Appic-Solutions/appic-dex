use candid::{Principal, types::principal};
use ethnum::{I256, U256};

use crate::{
    endpoints::{BurnPositionArgs, BurnPositionError, MintPositionArgs, MintPositionError},
    libraries::{
        constants::{MAX_TICK, MIN_TICK},
        safe_cast::big_uint_to_u256,
    },
    pool::types::{PoolId, PoolTickSpacing},
    position::types::{PositionInfo, PositionKey},
    state::read_state,
};

pub struct ValidatedMintPositionArgs {
    pub tick_spacing: PoolTickSpacing,
    pub lower_tick: i32,
    pub upper_tick: i32,
    pub pool_id: PoolId,
    pub amount0_max: I256,
    pub amount1_max: I256,
}
pub fn validate_mint_position_args(
    args: MintPositionArgs,
    caller: Principal,
) -> Result<ValidatedMintPositionArgs, MintPositionError> {
    // check pool
    let pool_id: PoolId = args
        .pool
        .try_into()
        .map_err(|_e| MintPositionError::InvalidPoolFee)?;

    let pool = read_state(|s| s.get_pool(&pool_id)).ok_or(MintPositionError::PoolNotInitialized)?;
    let tick_spacing = pool.tick_spacing;
    // check ticks
    let lower_tick: i32 = args
        .tick_lower
        .0
        .try_into()
        .map_err(|_e| MintPositionError::InvalidTick)?;
    let upper_tick: i32 = args
        .tick_upper
        .0
        .try_into()
        .map_err(|_e| MintPositionError::InvalidTick)?;
    if lower_tick < MIN_TICK || upper_tick > MAX_TICK || lower_tick >= upper_tick {
        return Err(MintPositionError::InvalidTick);
    };

    // position should not exist
    let position_key = PositionKey {
        owner: caller,
        pool_id: pool_id.clone(),
        tick_lower: lower_tick,
        tick_upper: upper_tick,
    };
    if read_state(|s| s.get_position(&position_key)).liquidity != 0 {
        return Err(MintPositionError::PositionAlreadyExists);
    }

    // check alignment with tick spacing
    if upper_tick % tick_spacing.0 != 0 || lower_tick % tick_spacing.0 != 0 {
        return Err(MintPositionError::TickNotAlignedWithTickSpacing);
    };

    let amount0_max: U256 =
        big_uint_to_u256(args.amount0_max.0).map_err(|_e| MintPositionError::InvalidAmount)?;
    let amount1_max: U256 =
        big_uint_to_u256(args.amount1_max.0).map_err(|_e| MintPositionError::InvalidAmount)?;

    // MAX amount should be I256 to prevent overflow
    let amount0_max: I256 = amount0_max
        .try_into()
        .map_err(|_e| MintPositionError::InvalidAmount)?;
    let amount1_max: I256 = amount1_max
        .try_into()
        .map_err(|_e| MintPositionError::InvalidAmount)?;

    Ok(ValidatedMintPositionArgs {
        tick_spacing,
        lower_tick,
        upper_tick,
        pool_id,
        amount0_max,
        amount1_max,
    })
}

pub struct ValidatedBurnPositionArgs {
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

pub fn validate_burn_position_args(
    args: BurnPositionArgs,
    caller: Principal,
) -> Result<ValidatedBurnPositionArgs, BurnPositionError> {
    // check pool
    let pool_id: PoolId = args
        .pool
        .try_into()
        .map_err(|_e| BurnPositionError::InvalidPoolFee)?;
    let pool = read_state(|s| s.get_pool(&pool_id)).ok_or(BurnPositionError::PoolNotInitialized)?;
    let tick_spacing = pool.tick_spacing;
    // check ticks
    let lower_tick: i32 = args
        .tick_lower
        .0
        .try_into()
        .map_err(|_e| BurnPositionError::InvalidTick)?;
    let upper_tick: i32 = args
        .tick_upper
        .0
        .try_into()
        .map_err(|_e| BurnPositionError::InvalidTick)?;
    if lower_tick < MIN_TICK || upper_tick > MAX_TICK || lower_tick >= upper_tick {
        return Err(BurnPositionError::InvalidTick);
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
        return Err(BurnPositionError::PositionNotFound);
    }

    let amount0_min: U256 =
        big_uint_to_u256(args.amount0_min.0).map_err(|_e| BurnPositionError::InvalidAmount)?;
    let amount1_min: U256 =
        big_uint_to_u256(args.amount1_min.0).map_err(|_e| BurnPositionError::InvalidAmount)?;

    // MIN amount should be I256 to prevent overflow
    let amount0_min: I256 = amount0_min
        .try_into()
        .map_err(|_e| BurnPositionError::InvalidAmount)?;
    let amount1_min: I256 = amount1_min
        .try_into()
        .map_err(|_e| BurnPositionError::InvalidAmount)?;

    let liquidity_delta = i128::try_from(position_info.liquidity)
        .map_err(|_e| BurnPositionError::LiquidityOverflow)?
        .checked_mul(-1i128)
        .ok_or(BurnPositionError::LiquidityOverflow)?;

    Ok(ValidatedBurnPositionArgs {
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
