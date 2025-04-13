use super::{
    constants::Q96,
    full_math::{div_rounding_up, mul_div, mul_div_rounding_up},
};

use ethnum::{I256, U256};

pub fn abs_diff(a: U256, b: U256) -> U256 {
    // diff = a - b
    let diff = a.wrapping_sub(b); // Use wrapping_sub to handle underflow

    // mask = diff >> 255 (arithmetic shift)
    // If diff is negative, mask = U256::MAX (-1); if positive, mask = U256::ZERO (0)
    let mask = if diff >> 255 != U256::ZERO {
        U256::MAX
    } else {
        U256::ZERO
    };

    // res = mask ^ (diff + mask)
    mask ^ (diff.wrapping_add(mask))
}

#[derive(Debug, Clone, PartialEq)]
pub enum AmountDeltaError {
    InvalidPrice,
    Overflow,
}

const FIXED_POINT_96_RESOLUTION: u8 = 96; // 2^96 shift

/// Gets the amount0 delta between two prices
/// Calculates liquidity / sqrt(lower) - liquidity / sqrt(upper),
/// i.e. liquidity * (sqrt(upper) - sqrt(lower)) / (sqrt(upper) * sqrt(lower))
/// sqrtPriceAX96 A sqrt price
/// sqrtPriceBX96 Another sqrt price
/// liquidity The amount of usable liquidity
/// roundUp Whether to round the amount up or down
/// returns uint256 Amount of currency0 required to cover a position of size liquidity between the two passed prices
pub fn get_amount_0_delta(
    sqrt_price_a_x96: U256,
    sqrt_price_b_x96: U256,
    liquidity: u128,
    round_up: bool,
) -> Result<U256, AmountDeltaError> {
    let (sqrt_lower, sqrt_upper) = if sqrt_price_a_x96 > sqrt_price_b_x96 {
        (sqrt_price_b_x96, sqrt_price_a_x96)
    } else {
        (sqrt_price_a_x96, sqrt_price_b_x96)
    };

    // Check for invalid price (sqrt_lower == 0)
    if sqrt_lower == U256::ZERO {
        return Err(AmountDeltaError::InvalidPrice);
    }

    // numerator1 = liquidity << 96
    let numerator1 = U256::from(liquidity) << FIXED_POINT_96_RESOLUTION;
    // numerator2 = sqrt(upper) - sqrt(lower)
    let numerator2 = sqrt_upper.wrapping_sub(sqrt_lower); // Safe since sqrt_upper >= sqrt_lower

    if round_up {
        // UnsafeMath.divRoundingUp(FullMath.mulDivRoundingUp(numerator1, numerator2, sqrt_upper), sqrt_lower)
        let mult_div_roundup_result = mul_div_rounding_up(numerator1, numerator2, sqrt_upper)
            .map_err(|_e| AmountDeltaError::Overflow)?;
        Ok(div_rounding_up(mult_div_roundup_result, sqrt_lower))
    } else {
        // FullMath.mulDiv(numerator1, numerator2, sqrt_upper) / sqrt_lower
        let mul_div_result =
            mul_div(numerator1, numerator2, sqrt_upper).map_err(|_e| AmountDeltaError::Overflow)?;
        Ok(mul_div_result / sqrt_lower) // Safe due to sqrt_lower != 0 check
    }
}

/// Helper that gets signed currency0 delta
/// sqrtPriceAX96 A sqrt price
/// sqrtPriceBX96 Another sqrt price
/// liquidity The change in liquidity for which to compute the amount0 delta
/// returns int256 Amount of currency0 corresponding to the passed liquidityDelta between the two prices
pub fn get_amount_0_delta_signed(
    sqrt_price_a_x96: U256,
    sqrt_price_b_x96: U256,
    liquidity: i128,
) -> Result<I256, AmountDeltaError> {
    if liquidity < 0 {
        let abs_liquidity = liquidity
            .checked_neg()
            .ok_or(AmountDeltaError::Overflow)? // Handle i128::MIN
            .try_into()
            .map_err(|_e| AmountDeltaError::Overflow)?; // Convert to u128
        I256::try_from(get_amount_0_delta(
            sqrt_price_a_x96,
            sqrt_price_b_x96,
            abs_liquidity,
            false,
        )?)
        .map_err(|_e| AmountDeltaError::Overflow)
    } else {
        let abs_liquidity = u128::try_from(liquidity).map_err(|_e| AmountDeltaError::Overflow)?; // Handle > u128::MAX
        I256::try_from(get_amount_0_delta(
            sqrt_price_a_x96,
            sqrt_price_b_x96,
            abs_liquidity,
            true,
        )?)
        .map(|amount_delta| -amount_delta)
        .map_err(|_e| AmountDeltaError::Overflow)
    }
}

/// Gets the amount1 delta between two prices
/// Calculates liquidity * (sqrt(upper) - sqrt(lower))
/// sqrtPriceAX96 A sqrt price
/// sqrtPriceBX96 Another sqrt price
/// liquidity The amount of usable liquidity
/// roundUp Whether to round the amount up, or down
/// returns amount1 Amount of currency1 required to cover a position of size liquidity between the two passed prices
pub fn get_amount_1_delta(
    sqrt_price_a_x96: U256,
    sqrt_price_b_x96: U256,
    liquidity: u128,
    round_up: bool,
) -> Result<U256, AmountDeltaError> {
    let numerator = abs_diff(sqrt_price_a_x96, sqrt_price_b_x96);
    let denominator = Q96.clone();
    let liquidity_u256 = U256::from(liquidity);

    // Base calculation: liquidity * numerator / Q96
    let amount1 =
        mul_div(liquidity_u256, numerator, denominator).map_err(|_e| AmountDeltaError::Overflow)?;

    // If round_up, add 1 if there's a remainder
    if round_up {
        let remainder = (liquidity_u256 * numerator) % denominator;
        if remainder > U256::ZERO {
            amount1
                .checked_add(U256::ONE)
                .ok_or(AmountDeltaError::Overflow)
        } else {
            Ok(amount1)
        }
    } else {
        Ok(amount1)
    }
}

/// Helper that gets signed currency1 delta
/// sqrtPriceAX96 A sqrt price
/// sqrtPriceBX96 Another sqrt price
/// liquidity The change in liquidity for which to compute the amount1 delta
/// returns int256 Amount of currency1 corresponding to the passed liquidityDelta between the two prices
pub fn get_amount_1_delta_signed(
    sqrt_price_a_x96: U256,
    sqrt_price_b_x96: U256,
    liquidity: i128,
) -> Result<I256, AmountDeltaError> {
    if liquidity < 0 {
        let abs_liquidity = liquidity
            .checked_neg()
            .ok_or(AmountDeltaError::Overflow)? // Handle i128::MIN
            .try_into()
            .map_err(|_e| AmountDeltaError::Overflow)?; // Convert to u128
        I256::try_from(get_amount_1_delta(
            sqrt_price_a_x96,
            sqrt_price_b_x96,
            abs_liquidity,
            false,
        )?)
        .map_err(|_e| AmountDeltaError::Overflow)
    } else {
        let abs_liquidity = u128::try_from(liquidity).map_err(|_e| AmountDeltaError::Overflow)?; // Handle > u128::MAX
        I256::try_from(get_amount_1_delta(
            sqrt_price_a_x96,
            sqrt_price_b_x96,
            abs_liquidity,
            true,
        )?)
        .map(|amount_delta| -amount_delta)
        .map_err(|_e| AmountDeltaError::Overflow)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    mod abs_diff {
        use super::*;
        #[test]
        fn test_abs_diff() {
            // a >= b: a - b
            assert_eq!(
                abs_diff(U256::from(10u32), U256::from(7u32)),
                U256::from(3u32)
            ); // 10 - 7 = 3
            assert_eq!(
                abs_diff(U256::from(100u32), U256::from(100u32)),
                U256::from(0u32)
            ); // 100 - 100 = 0

            // a < b: b - a
            assert_eq!(
                abs_diff(U256::from(7u32), U256::from(10u32)),
                U256::from(3u32)
            ); // 10 - 7 = 3

            // Large values (beyond u128, within uint160)
            let max_160 = U256::from(1u128) << 160 - 1; // 2^160 - 1
            assert_eq!(abs_diff(max_160, U256::ZERO), max_160); // max - 0
            assert_eq!(abs_diff(U256::ZERO, max_160), max_160); // max - 0

            // Edge cases with full U256 range
            assert_eq!(abs_diff(U256::ZERO, U256::ZERO), U256::from(0u32)); // 0 - 0 = 0
            assert_eq!(abs_diff(U256::MAX, U256::MAX), U256::from(0u32)); // max - max = 0

            // Beyond uint160, within U256
            let large = U256::MAX / U256::from(2u32); // ~2^255
            assert_eq!(abs_diff(large, U256::ZERO), large); // large - 0
            assert_eq!(abs_diff(U256::ZERO, large), large); // 0 - large
        }
    }

    mod amount_delta {
        use super::*;

        use lazy_static::lazy_static;
        lazy_static! {
             // Constants from Solidity
        static ref SQRT_PRICE_1_1: U256 =
            U256::from_str_radix("79228162514264337593543950336", 10).unwrap();
        static ref SQRT_PRICE_2_1: U256 = U256::from_str_radix("112045541949572279837463876454",10).unwrap();
        static ref SQRT_PRICE_121_100: U256 =
            U256::from_str_radix("87150978765690771352898345369",10).unwrap();
        static ref ONE_ETHER: u128 = 1_000_000_000_000_000_000; // 1e18


        }
        #[test]
        fn test_get_amount_0_delta_returns_0_if_liquidity_is_0() {
            let amount0 =
                get_amount_0_delta(SQRT_PRICE_1_1.clone(), SQRT_PRICE_2_1.clone(), 0, true)
                    .unwrap();
            assert_eq!(amount0, U256::ZERO);
        }

        #[test]
        fn test_get_amount_0_delta_returns_0_if_prices_are_equal() {
            let amount0 =
                get_amount_0_delta(SQRT_PRICE_1_1.clone(), SQRT_PRICE_1_1.clone(), 0, true)
                    .unwrap();
            assert_eq!(amount0, U256::ZERO);
        }

        #[test]
        fn test_get_amount_0_delta_reverts_if_price_is_zero() {
            let result = get_amount_0_delta(U256::ZERO, U256::ONE, 1, true);
            assert_eq!(result, Err(AmountDeltaError::InvalidPrice));
        }

        #[test]
        fn test_get_amount_0_delta_1_amount_1_for_price_of_1_to_1_21() {
            let amount0 = get_amount_0_delta(
                SQRT_PRICE_1_1.clone(),
                SQRT_PRICE_121_100.clone(),
                ONE_ETHER.clone(),
                true,
            )
            .unwrap();
            assert_eq!(
                amount0,
                U256::from_str_radix("90909090909090910", 10).unwrap()
            );

            let amount0_rounded_down = get_amount_0_delta(
                SQRT_PRICE_1_1.clone(),
                SQRT_PRICE_121_100.clone(),
                ONE_ETHER.clone(),
                false,
            )
            .unwrap();
            assert_eq!(amount0_rounded_down, amount0 - U256::ONE);
        }

        #[test]
        fn test_get_amount_0_delta_works_for_prices_that_overflow() {
            let sqrt_p_1 =
                U256::from_str_radix("2787593149816327892691964784081045188247552", 10).unwrap();
            let sqrt_p_2 =
                U256::from_str_radix("22300745198530623141535718272648361505980416", 10).unwrap();

            let amount0_up =
                get_amount_0_delta(sqrt_p_1, sqrt_p_2, ONE_ETHER.clone(), true).unwrap();
            let amount0_down =
                get_amount_0_delta(sqrt_p_1, sqrt_p_2, ONE_ETHER.clone(), false).unwrap();

            assert_eq!(amount0_up, amount0_down + U256::ONE);
        }

        #[test]
        fn test_get_amount_1_delta_returns_0_if_liquidity_is_0() {
            let amount1 =
                get_amount_1_delta(SQRT_PRICE_1_1.clone(), SQRT_PRICE_2_1.clone(), 0, true)
                    .unwrap();
            assert_eq!(amount1, U256::ZERO);
        }

        #[test]
        fn test_get_amount_1_delta_returns_0_if_prices_are_equal() {
            let amount1 =
                get_amount_1_delta(SQRT_PRICE_1_1.clone(), SQRT_PRICE_1_1.clone(), 0, true)
                    .unwrap();
            assert_eq!(amount1, U256::ZERO);
        }

        #[test]
        fn test_get_amount_1_delta_1_amount_1_for_price_of_1_to_1_21() {
            let amount1 = get_amount_1_delta(
                SQRT_PRICE_1_1.clone(),
                SQRT_PRICE_121_100.clone(),
                ONE_ETHER.clone(),
                true,
            )
            .unwrap();
            assert_eq!(
                amount1,
                U256::from_str_radix("100000000000000000", 10).unwrap()
            );

            let amount1_rounded_down = get_amount_1_delta(
                SQRT_PRICE_1_1.clone(),
                SQRT_PRICE_121_100.clone(),
                ONE_ETHER.clone(),
                false,
            )
            .unwrap();
            assert_eq!(amount1_rounded_down, amount1 - U256::ONE);
        }

        #[test]
        fn test_swap_computation_sqrt_p_times_sqrt_q_overflows() {
            let sqrt_p =
                U256::from_str_radix("1025574284609383690408304870162715216695788925244", 10)
                    .unwrap();
            let liquidity = 50_015_962_439_936_049_619_261_659_728_067_971_248u128;
            let amount_in = 406u128;

            // Placeholder for getNextSqrtPriceFromInput (not implemented here)
            let sqrt_q =
                U256::from_str_radix("1025574284609383582644711336373707553698163132913", 10)
                    .unwrap();

            let amount0_delta = get_amount_0_delta(sqrt_q, sqrt_p, liquidity, true).unwrap();
            assert_eq!(amount0_delta, U256::from(amount_in));
        }

        #[test]
        fn test_mul_div_for_amount_1_delta() {
            let liquidity_u256 = U256::from(ONE_ETHER.clone());
            let numerator = abs_diff(SQRT_PRICE_1_1.clone(), SQRT_PRICE_121_100.clone()); // 7922816251426433759354395033

            let denominator = Q96.clone(); // 79228162514264337593543950336

            // Expected results from the test_get_amount_1_delta_1_amount_1_for_price_of_1_to_1_21
            let expected_rounded_up = U256::from_str_radix("100000000000000000", 10).unwrap(); // 0.1 ether
            let expected_rounded_down = expected_rounded_up - U256::ONE; // 99999999999999999

            // Test mul_div (round down)
            let result_down = mul_div(liquidity_u256, numerator, denominator).unwrap();
            assert_eq!(
                result_down, expected_rounded_down,
                "mul_div should return 99999999999999999"
            );

            // Test mul_div_rounding_up (round up)
            let result_up = mul_div_rounding_up(liquidity_u256, numerator, denominator).unwrap();
            assert_eq!(
                result_up, expected_rounded_up,
                "mul_div_rounding_up should return 100000000000000000"
            );
        }
    }
}
