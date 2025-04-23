use ethnum::{I256, U256};

use crate::{
    endpoints::{MintPositionArgs, MintPositionError},
    libraries::{
        constants::{MAX_TICK, MIN_TICK},
        safe_cast::big_uint_to_u256,
    },
    pool::types::{PoolId, PoolTickSpacing},
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
) -> Result<ValidatedMintPositionArgs, MintPositionError> {
    // check pool
    let pool_id: PoolId = args.pool.try_into()?;
    let pool = read_state(|s| s.get_pool(&pool_id)).ok_or(MintPositionError::PoolNotInitialized)?;
    let tick_spacing = pool.tick_spacing;

    // check ticks
    let lower_tick: i32 = args
        .tick_lower
        .0
        .try_into()
        .map_err(|_e| MintPositionError::InvalidTick)?;
    let upper_tick: i32 = args
        .tick_higher
        .0
        .try_into()
        .map_err(|_e| MintPositionError::InvalidTick)?;
    if lower_tick < MIN_TICK || upper_tick > MAX_TICK || lower_tick >= upper_tick {
        return Err(MintPositionError::InvalidTick);
    };

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
