use appic_dex::{
    endpoints::{CreatePoolArgs, CreatePoolError},
    libraries::tick_math::{self, TickMath, TickMathError},
    pool::types::{PoolFee, PoolId, PoolState},
    state::{mutate_state, read_state},
    tick,
};
use candid::{Nat, Principal};
use ethnum::U256;
use ic_cdk::{query, update};

//#[update]
//pub fn create_pool(
//    CreatePoolArgs {
//        token_a,
//        token_b,
//        fee,
//        sqrt_price_x96,
//    }: CreatePoolArgs,
//) -> Result<(), CreatePoolError> {
//    // sort token_a and b, token 0 is always the smaller token
//    let (token0, token1) = if token_a < token_b {
//        (token_a, token_b)
//    } else {
//        (token_b, token_a)
//    };
//
//    let fee = PoolFee::try_from(fee).map_err(|_e| CreatePoolError::InvalidFeeAmount)?;
//
//    let pool_id = PoolId {
//        token0,
//        token1,
//        fee: fee.clone(),
//    };
//
//    if read_state(|s| s.get_pool(&pool_id)).is_some() {
//        return Err(CreatePoolError::PoolAlreadyExists);
//    }
//    let tick_spacing =
//        read_state(|s| s.get_tick_spacing(&fee)).ok_or(CreatePoolError::InvalidFeeAmount)?;
//
//    let sqrt_price_x96 = U256::from_str_radix(&sqrt_price_x96.0.to_str_radix(10), 10)
//        .map_err(|_e| CreatePoolError::InvalidSqrtPriceX96)?;
//
//    let tick = TickMath::get_tick_at_sqrt_ratio(sqrt_price_x96)
//        .map_err(|_e| CreatePoolError::InvalidSqrtPriceX96)?;
//
//    let max_liquidity_per_tick = tick::tick_spacing_to_max_liquidity_per_tick(tick_spacing.0);
//    let pool_state = PoolState {
//        sqrt_price_x96,
//        tick,
//        fee_growth_global_0_x128: U256::ZERO,
//        fee_growth_global_1_x128: U256::ZERO,
//        liquidity: 0,
//        tick_spacing,
//        max_liquidity_per_tick,
//        fee_protocol: todo!(),
//    };
//    mutate_state(|s| s.set_pool(pool_id, pool_state));
//
//    return Ok(());
//}
//
fn main() {
    println!("Hello, world!");
}
