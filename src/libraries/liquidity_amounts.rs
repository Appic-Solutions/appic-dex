use ethnum::U256;
use std::cmp::min;

use super::{constants::Q96, full_math::mul_div};

#[derive(Debug, PartialEq)]
pub enum LiquidityAmountsError {
    Overflow,
    InvalidPrice,
}

/// Computes the amount of liquidity received for a given amount of token0 and price range.
/// Calculates amount0 * (sqrt(upper) * sqrt(lower)) / (sqrt(upper) - sqrt(lower)).
/// # Arguments
/// * `sqrt_price_a_x96` - A sqrt price representing the first tick boundary (Q96 format)
/// * `sqrt_price_b_x96` - A sqrt price representing the second tick boundary (Q96 format)
/// * `amount0` - The amount of token0 being sent in
/// # Returns
/// * `Result<u128, LiquidityAmountsError>` - The amount of liquidity
pub fn get_liquidity_for_amount0(
    sqrt_price_a_x96: U256,
    sqrt_price_b_x96: U256,
    amount0: U256,
) -> Result<u128, LiquidityAmountsError> {
    let (sqrt_price_a_x96, sqrt_price_b_x96) = if sqrt_price_a_x96 > sqrt_price_b_x96 {
        (sqrt_price_b_x96, sqrt_price_a_x96)
    } else {
        (sqrt_price_a_x96, sqrt_price_b_x96)
    };

    if sqrt_price_a_x96 == sqrt_price_b_x96 {
        return Err(LiquidityAmountsError::InvalidPrice);
    }

    // sqrtPriceAX96 * sqrtPriceBX96 / Q96
    let intermediate = mul_div(sqrt_price_a_x96, sqrt_price_b_x96, *Q96)
        .map_err(|_e| LiquidityAmountsError::Overflow)?;

    // amount0 * intermediate / (sqrtPriceBX96 - sqrtPriceAX96)
    let liquidity = mul_div(amount0, intermediate, sqrt_price_b_x96 - sqrt_price_a_x96)
        .map_err(|_e| LiquidityAmountsError::Overflow)?;

    u128::try_from(liquidity).map_err(|_e| LiquidityAmountsError::Overflow)
}

/// Computes the amount of liquidity received for a given amount of token1 and price range.
/// Calculates amount1 / (sqrt(upper) - sqrt(lower)).
/// # Arguments
/// * `sqrt_price_a_x96` - A sqrt price representing the first tick boundary (Q96 format)
/// * `sqrt_price_b_x96` - A sqrt price representing the second tick boundary (Q96 format)
/// * `amount1` - The amount of token1 being sent in
/// # Returns
/// * `Result<u128, LiquidityAmountsError>` - The amount of liquidity
pub fn get_liquidity_for_amount1(
    sqrt_price_a_x96: U256,
    sqrt_price_b_x96: U256,
    amount1: U256,
) -> Result<u128, LiquidityAmountsError> {
    let (sqrt_price_a_x96, sqrt_price_b_x96) = if sqrt_price_a_x96 > sqrt_price_b_x96 {
        (sqrt_price_b_x96, sqrt_price_a_x96)
    } else {
        (sqrt_price_a_x96, sqrt_price_b_x96)
    };

    if sqrt_price_a_x96 == sqrt_price_b_x96 {
        return Err(LiquidityAmountsError::InvalidPrice);
    }

    // amount1 * Q96 / (sqrtPriceBX96 - sqrtPriceAX96)
    let liquidity = mul_div(amount1, *Q96, sqrt_price_b_x96 - sqrt_price_a_x96)
        .map_err(|_e| LiquidityAmountsError::Overflow)?;

    liquidity
        .try_into()
        .map_err(|_| LiquidityAmountsError::Overflow)
}

/// Computes the maximum amount of liquidity received for a given amount of token0, token1,
/// the current pool price, and the prices at the tick boundaries.
/// # Arguments
/// * `sqrt_price_x96` - A sqrt price representing the current pool price (Q96 format)
/// * `sqrt_price_a_x96` - A sqrt price representing the first tick boundary (Q96 format)
/// * `sqrt_price_b_x96` - A sqrt price representing the second tick boundary (Q96 format)
/// * `amount0` - The amount of token0 being sent in
/// * `amount1` - The amount of token1 being sent in
/// # Returns
/// * `Result<u128, LiquidityAmountsError>` - The maximum amount of liquidity
pub fn get_liquidity_for_amounts(
    sqrt_price_x96: U256,
    sqrt_price_a_x96: U256,
    sqrt_price_b_x96: U256,
    amount0: U256,
    amount1: U256,
) -> Result<u128, LiquidityAmountsError> {
    let (sqrt_price_a_x96, sqrt_price_b_x96) = if sqrt_price_a_x96 > sqrt_price_b_x96 {
        (sqrt_price_b_x96, sqrt_price_a_x96)
    } else {
        (sqrt_price_a_x96, sqrt_price_b_x96)
    };

    if sqrt_price_a_x96 == sqrt_price_b_x96 {
        return Err(LiquidityAmountsError::InvalidPrice);
    }

    let liquidity = if sqrt_price_x96 <= sqrt_price_a_x96 {
        get_liquidity_for_amount0(sqrt_price_a_x96, sqrt_price_b_x96, amount0)?
    } else if sqrt_price_x96 < sqrt_price_b_x96 {
        let liquidity0 = get_liquidity_for_amount0(sqrt_price_x96, sqrt_price_b_x96, amount0)?;
        let liquidity1 = get_liquidity_for_amount1(sqrt_price_a_x96, sqrt_price_x96, amount1)?;
        min(liquidity0, liquidity1)
    } else {
        get_liquidity_for_amount1(sqrt_price_a_x96, sqrt_price_b_x96, amount1)?
    };

    Ok(liquidity)
}
