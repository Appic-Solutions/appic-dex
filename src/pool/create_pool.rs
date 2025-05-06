use ethnum::U256;

use crate::{
    candid_types::pool::{CreatePoolArgs, CreatePoolError},
    libraries::{
        constants::{DEFAULT_PROTOCOL_FEE, MAX_SQRT_RATIO, MIN_SQRT_RATIO},
        safe_cast::big_uint_to_u256,
        tick_math::TickMath,
    },
    state::{mutate_state, read_state},
    tick::tick_spacing_to_max_liquidity_per_tick,
};

use super::types::{PoolFee, PoolId, PoolState};

pub fn create_pool_inner(args: CreatePoolArgs) -> Result<PoolId, CreatePoolError> {
    let sqrt_price_x96 = big_uint_to_u256(args.sqrt_price_x96.0)
        .map_err(|_e| CreatePoolError::InvalidSqrtPriceX96)?;

    if sqrt_price_x96 >= *MAX_SQRT_RATIO || sqrt_price_x96 <= *MIN_SQRT_RATIO {
        return Err(CreatePoolError::InvalidSqrtPriceX96);
    }

    // sort token_a and b, token 0 is always the smaller token
    let (token0, token1) = if args.token_a < args.token_b {
        (args.token_a, args.token_b)
    } else {
        (args.token_b, args.token_a)
    };

    let fee = PoolFee::try_from(args.fee).map_err(|_e| CreatePoolError::InvalidFeeAmount)?;

    let pool_id = PoolId {
        token0,
        token1,
        fee: fee.clone(),
    };

    if read_state(|s| s.get_pool(&pool_id)).is_some() {
        return Err(CreatePoolError::PoolAlreadyExists);
    }
    let tick_spacing =
        read_state(|s| s.get_tick_spacing(&fee)).ok_or(CreatePoolError::InvalidFeeAmount)?;

    let tick = TickMath::get_tick_at_sqrt_ratio(sqrt_price_x96);

    let max_liquidity_per_tick = tick_spacing_to_max_liquidity_per_tick(tick_spacing.0);
    let pool_state = PoolState {
        sqrt_price_x96,
        tick,
        fee_growth_global_0_x128: U256::ZERO,
        fee_growth_global_1_x128: U256::ZERO,
        liquidity: 0,
        tick_spacing,
        max_liquidity_per_tick,
        fee_protocol: *DEFAULT_PROTOCOL_FEE,
    };
    mutate_state(|s| s.set_pool(pool_id.clone(), pool_state));

    return Ok(pool_id);
}
