use ethnum::U256;

use super::{
    constants::{Q160, Q96, U160_MAX},
    full_math::{div_rounding_up, mul_div_rounding_up},
};

#[derive(Debug, PartialEq)]
pub enum SqrtPriceMathError {
    PriceOverflow,
    NotEnoughLiquidity,
    InvalidPriceOrLiquidity,
    InvalidFee,
}

/// Gets the next sqrt price given a delta of currency0, rounding up.
///
/// Always rounds up to ensure the price moves far enough for exact output (increasing price)
/// or moves less for exact input (decreasing price) to avoid over-sending output.
/// Uses the formula: liquidity * sqrtPX96 / (liquidity ± amount * sqrtPX96).
/// If overflow occurs, uses: liquidity / (liquidity / sqrtPX96 ± amount).
///
/// # Arguments
/// * `sqrt_px96` - The starting price (Q96 fixed-point).
/// * `liquidity` - The amount of usable liquidity.
/// * `amount` - How much of currency0 to add or remove from virtual reserves.
/// * `add` - Whether to add (true) or remove (false) the amount of currency0.
///
/// # Returns
/// The next sqrt price as a `U256`.
pub fn get_next_sqrt_price_from_amount0_rounding_up(
    sqrt_px96: U256,
    liquidity: u128,
    amount: U256,
    add: bool,
) -> Result<U256, SqrtPriceMathError> {
    if amount == 0 {
        return Ok(sqrt_px96);
    }

    let numerator1: U256 = U256::from(liquidity) << 96;

    if add {
        let product = amount.wrapping_mul(sqrt_px96);
        if product / amount == sqrt_px96 {
            let denominator = numerator1.wrapping_add(product);
            if denominator >= numerator1 {
                let result = mul_div_rounding_up(numerator1, sqrt_px96, denominator)
                    .map_err(|_| SqrtPriceMathError::PriceOverflow)?;
                if result > *U160_MAX {
                    return Err(SqrtPriceMathError::PriceOverflow);
                }
                return Ok(result);
            }
        }
        // Use alternative formula to avoid overflow
        let result = div_rounding_up(numerator1, (numerator1 / sqrt_px96).wrapping_add(amount));
        if result > *U160_MAX {
            return Err(SqrtPriceMathError::PriceOverflow);
        }
        Ok(result)
    } else {
        let product = amount.wrapping_mul(sqrt_px96);
        // Check for overflow or underflow
        if product / amount != sqrt_px96 || numerator1 <= product {
            return Err(SqrtPriceMathError::PriceOverflow);
        }
        let denominator = numerator1.wrapping_sub(product);
        let result = mul_div_rounding_up(numerator1, U256::from(sqrt_px96), denominator)
            .map_err(|_| SqrtPriceMathError::PriceOverflow)?;

        if result > *U160_MAX {
            return Err(SqrtPriceMathError::PriceOverflow);
        }
        Ok(result)
    }
}

/// Gets the next sqrt price given a delta of currency1, rounding down.
///
/// Always rounds down to ensure the price moves far enough for exact output (decreasing price)
/// or moves less for exact input (increasing price) to avoid over-sending output.
/// Uses the formula: sqrtPX96 ± amount / liquidity.
///
/// # Arguments
/// * `sqrt_px96` - The starting price (Q96 fixed-point).
/// * `liquidity` - The amount of usable liquidity.
/// * `amount` - How much of currency1 to add or remove from virtual reserves.
/// * `add` - Whether to add (true) or remove (false) the amount of currency1.
///
/// # Returns
/// The next sqrt price as a `u160` (fits in 160 bits).
pub fn get_next_sqrt_price_from_amount1_rounding_down(
    sqrt_px96: U256,
    liquidity: u128,
    amount: U256,
    add: bool,
) -> Result<U256, SqrtPriceMathError> {
    if add {
        let quotient = if amount <= *Q160 - 1 {
            (amount << 96) / U256::from(liquidity)
        } else {
            mul_div_rounding_up(amount, *Q96, U256::from(liquidity))
                .map_err(|_| SqrtPriceMathError::PriceOverflow)?
        };

        let result = sqrt_px96.wrapping_add(quotient);

        if result > *U160_MAX {
            return Err(SqrtPriceMathError::PriceOverflow);
        }

        Ok(result)
    } else {
        let quotient = if amount <= *Q160 - 1 {
            div_rounding_up(amount << 96, U256::from(liquidity))
        } else {
            mul_div_rounding_up(amount, *Q96, U256::from(liquidity))
                .map_err(|_| SqrtPriceMathError::PriceOverflow)?
        };
        if U256::from(sqrt_px96) <= quotient {
            return Err(SqrtPriceMathError::NotEnoughLiquidity);
        }

        let result = sqrt_px96.wrapping_sub(quotient);

        if result > *U160_MAX {
            return Err(SqrtPriceMathError::PriceOverflow);
        }

        Ok(result)
    }
}

/// Gets the next sqrt price given an input amount of currency0 or currency1.
///
/// Throws if price or liquidity are 0. Rounds to avoid passing the target price.
/// If `zeroForOne` is true, uses currency0 as input; otherwise, uses currency1.
///
/// # Arguments
/// * `sqrt_px96` - The starting price (Q96 fixed-point).
/// * `liquidity` - The amount of usable liquidity.
/// * `amount_in` - How much of currency0 or currency1 is being swapped in.
/// * `zero_for_one` - Whether the input is currency0 (true) or currency1 (false).
///
/// # Returns
/// The next sqrt price as a `u160`.
pub fn get_next_sqrt_price_from_input(
    sqrt_px96: U256,
    liquidity: u128,
    amount_in: U256,
    zero_for_one: bool,
) -> Result<U256, SqrtPriceMathError> {
    if sqrt_px96 == 0 || liquidity == 0 {
        return Err(SqrtPriceMathError::InvalidPriceOrLiquidity);
    }

    if zero_for_one {
        get_next_sqrt_price_from_amount0_rounding_up(sqrt_px96, liquidity, amount_in, true)
    } else {
        get_next_sqrt_price_from_amount1_rounding_down(sqrt_px96, liquidity, amount_in, true)
    }
}

/// Gets the next sqrt price given an output amount of currency0 or currency1.
///
/// Throws if price or liquidity are 0. Rounds to ensure passing the target price.
/// If `zeroForOne` is true, outputs currency1; otherwise, outputs currency0.
///
/// # Arguments
/// * `sqrt_px96` - The starting price (Q96 fixed-point).
/// * `liquidity` - The amount of usable liquidity.
/// * `amount_out` - How much of currency0 or currency1 is being swapped out.
/// * `zero_for_one` - Whether the output is currency1 (true) or currency0 (false).
///
/// # Returns
/// The next sqrt price as a `u160`.
pub fn get_next_sqrt_price_from_output(
    sqrt_px96: U256,
    liquidity: u128,
    amount_out: U256,
    zero_for_one: bool,
) -> Result<U256, SqrtPriceMathError> {
    if sqrt_px96 == 0 || liquidity == 0 {
        return Err(SqrtPriceMathError::InvalidPriceOrLiquidity);
    }

    if zero_for_one {
        get_next_sqrt_price_from_amount1_rounding_down(sqrt_px96, liquidity, amount_out, false)
    } else {
        get_next_sqrt_price_from_amount0_rounding_up(sqrt_px96, liquidity, amount_out, false)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use ethnum::U256;
    use lazy_static::lazy_static;

    // Constants for testing, matching Solidity Constants.sol
    pub const SQRT_PRICE_121_100: u128 = 87150978765690771352898345369; // sqrt(1.21/1) * 2^96
    lazy_static! {
            pub static ref SQRT_PRICE_1_1: U256 = U256::from(79228162514264337593543950336_u128); // sqrt(1/1) * 2^96

        pub static ref ONE_ETHER: U256 = U256::from(1_000_000_000_000_000_000u128); // 10^18
        pub static ref POINT_ONE_ETHER: U256 = U256::from(100_000_000_000_000_000u128); // 0.1 * 10^18
    }

    #[test]
    fn test_get_next_sqrt_price_from_input_reverts_if_price_is_zero() {
        let result = get_next_sqrt_price_from_input(U256::ZERO, 1, *POINT_ONE_ETHER, false);
        assert_eq!(result, Err(SqrtPriceMathError::InvalidPriceOrLiquidity));
    }

    #[test]
    fn test_get_next_sqrt_price_from_input_reverts_if_liquidity_is_zero() {
        let result = get_next_sqrt_price_from_input(U256::ONE, 0, *POINT_ONE_ETHER, true);
        assert_eq!(result, Err(SqrtPriceMathError::InvalidPriceOrLiquidity));
    }

    #[test]
    fn test_get_next_sqrt_price_from_input_reverts_if_input_amount_overflows_the_price() {
        let price = *U160_MAX - U256::ONE;
        let liquidity = 1024;
        let amount_in = U256::from(1024_u32);
        let result = get_next_sqrt_price_from_input(price, liquidity, amount_in, false);
        assert!(result.is_err()); // Expect any error (likely PriceOverflow or NotEnoughLiquidity)
    }

    #[test]
    fn test_get_next_sqrt_price_from_input_any_input_amount_cannot_underflow_the_price() {
        let price = U256::ONE;
        let liquidity = 1;
        let amount_in = U256::ONE << 255; // 2^255
        let sqrt_q = get_next_sqrt_price_from_input(price, liquidity, amount_in, true).unwrap();
        assert_eq!(sqrt_q, 1);
    }

    #[test]
    fn test_get_next_sqrt_price_from_input_returns_input_price_if_amount_in_is_zero_and_zero_for_one_true(
    ) {
        let price = *SQRT_PRICE_1_1;
        let liquidity = 1;
        let result = get_next_sqrt_price_from_input(price, liquidity, U256::ZERO, true).unwrap();
        assert_eq!(result, price);
    }

    #[test]
    fn test_get_next_sqrt_price_from_input_returns_input_price_if_amount_in_is_zero_and_zero_for_one_false(
    ) {
        let price = *SQRT_PRICE_1_1;
        let liquidity = 1;
        let result = get_next_sqrt_price_from_input(price, liquidity, U256::ZERO, false).unwrap();
        assert_eq!(result, price);
    }

    #[test]
    fn test_get_next_sqrt_price_from_input_returns_the_minimum_price_for_max_inputs() {
        let sqrt_p = *U160_MAX - U256::ONE;
        let liquidity = u128::MAX;
        let max_amount_no_overflow = U256::MAX - (U256::from(u128::MAX) << 96) / U256::from(sqrt_p);
        let result =
            get_next_sqrt_price_from_input(sqrt_p, liquidity, max_amount_no_overflow, true)
                .unwrap();
        assert_eq!(result, 1);
    }

    #[test]
    fn test_get_next_sqrt_price_from_input_input_amount_of_0_1_currency1() {
        let sqrt_p = *SQRT_PRICE_1_1;
        let sqrt_q = get_next_sqrt_price_from_input(
            sqrt_p,
            ONE_ETHER.as_u128() as u128,
            *POINT_ONE_ETHER,
            false,
        )
        .unwrap();
        assert_eq!(sqrt_q, SQRT_PRICE_121_100);
    }

    #[test]
    fn test_get_next_sqrt_price_from_input_input_amount_of_0_1_currency0() {
        let sqrt_p = *SQRT_PRICE_1_1;
        let sqrt_q = get_next_sqrt_price_from_input(
            sqrt_p,
            ONE_ETHER.as_u128() as u128,
            *POINT_ONE_ETHER,
            true,
        )
        .unwrap();
        assert_eq!(sqrt_q, 72025602285694852357767227579);
    }

    #[test]
    fn test_get_next_sqrt_price_from_input_amount_in_greater_than_uint96_max_and_zero_for_one_true()
    {
        let sqrt_p = *SQRT_PRICE_1_1;
        let liquidity = (10 * *ONE_ETHER).as_u128() as u128;
        let amount_in = U256::ONE << 100;
        let sqrt_q = get_next_sqrt_price_from_input(sqrt_p, liquidity, amount_in, true).unwrap();
        assert_eq!(sqrt_q, 624999999995069620);
    }

    #[test]
    fn test_get_next_sqrt_price_from_input_can_return_1_with_enough_amount_in_and_zero_for_one_true(
    ) {
        let sqrt_p = *SQRT_PRICE_1_1;
        let amount_in = U256::MAX / 2;
        let sqrt_q = get_next_sqrt_price_from_input(sqrt_p, 1, amount_in, true).unwrap();
        assert_eq!(sqrt_q, 1);
    }

    #[test]
    fn test_get_next_sqrt_price_from_output_reverts_if_price_is_zero() {
        let result = get_next_sqrt_price_from_output(U256::ZERO, 1, *POINT_ONE_ETHER, false);
        assert_eq!(result, Err(SqrtPriceMathError::InvalidPriceOrLiquidity));
    }

    #[test]
    fn test_get_next_sqrt_price_from_output_reverts_if_liquidity_is_zero() {
        let result = get_next_sqrt_price_from_output(U256::ONE, 0, *POINT_ONE_ETHER, true);
        assert_eq!(result, Err(SqrtPriceMathError::InvalidPriceOrLiquidity));
    }

    #[test]
    fn test_get_next_sqrt_price_from_output_reverts_if_output_amount_is_exactly_the_virtual_reserves_of_currency0(
    ) {
        let price = U256::from(20282409603651670423947251286016_u128);
        let liquidity = 1024;
        let amount_out = U256::from(4_u128);
        let result = get_next_sqrt_price_from_output(price, liquidity, amount_out, false);
        assert_eq!(result, Err(SqrtPriceMathError::PriceOverflow));
    }

    #[test]
    fn test_get_next_sqrt_price_from_output_reverts_if_output_amount_is_greater_than_the_virtual_reserves_of_currency0(
    ) {
        let price = U256::from(20282409603651670423947251286016_u128);
        let liquidity = 1024;
        let amount_out = U256::from(5_u128);
        let result = get_next_sqrt_price_from_output(price, liquidity, amount_out, false);
        assert_eq!(result, Err(SqrtPriceMathError::PriceOverflow));
    }

    #[test]
    fn test_get_next_sqrt_price_from_output_reverts_if_output_amount_is_greater_than_the_virtual_reserves_of_currency1(
    ) {
        let price = U256::from(20282409603651670423947251286016_u128);
        let liquidity = 1024;
        let amount_out = U256::from(262145_u128);
        let result = get_next_sqrt_price_from_output(price, liquidity, amount_out, true);
        assert_eq!(result, Err(SqrtPriceMathError::NotEnoughLiquidity));
    }

    #[test]
    fn test_get_next_sqrt_price_from_output_reverts_if_output_amount_is_exactly_the_virtual_reserves_of_currency1(
    ) {
        let price = U256::from(20282409603651670423947251286016_u128);
        let liquidity = 1024;
        let amount_out = U256::from(262144_u128);
        let result = get_next_sqrt_price_from_output(price, liquidity, amount_out, true);
        assert_eq!(result, Err(SqrtPriceMathError::NotEnoughLiquidity));
    }

    #[test]
    fn test_get_next_sqrt_price_from_output_succeeds_if_output_amount_is_just_less_than_the_virtual_reserves_of_currency1(
    ) {
        let price = U256::from(20282409603651670423947251286016_u128);
        let liquidity = 1024;
        let amount_out = U256::from(262143_u128);
        let sqrt_q = get_next_sqrt_price_from_output(price, liquidity, amount_out, true).unwrap();
        assert_eq!(sqrt_q, 77371252455336267181195264);
    }

    #[test]
    fn test_get_next_sqrt_price_from_output_puzzling_echidna_test() {
        let price = U256::from(20282409603651670423947251286016_u128);
        let liquidity = 1024;
        let amount_out = U256::from(4_u128);
        let result = get_next_sqrt_price_from_output(price, liquidity, amount_out, false);
        assert_eq!(result, Err(SqrtPriceMathError::PriceOverflow));
    }

    #[test]
    fn test_get_next_sqrt_price_from_output_returns_input_price_if_amount_in_is_zero_and_zero_for_one_true(
    ) {
        let sqrt_p = *SQRT_PRICE_1_1;
        let sqrt_q =
            get_next_sqrt_price_from_output(sqrt_p, POINT_ONE_ETHER.as_u128(), U256::ZERO, true)
                .unwrap();
        assert_eq!(sqrt_p, sqrt_q);
    }

    #[test]
    fn test_get_next_sqrt_price_from_output_returns_input_price_if_amount_in_is_zero_and_zero_for_one_false(
    ) {
        let sqrt_p = *SQRT_PRICE_1_1;
        let sqrt_q =
            get_next_sqrt_price_from_output(sqrt_p, POINT_ONE_ETHER.as_u128(), U256::ZERO, false)
                .unwrap();
        assert_eq!(sqrt_p, sqrt_q);
    }

    #[test]
    fn test_get_next_sqrt_price_from_output_output_amount_of_0_1_currency1() {
        let sqrt_p = *SQRT_PRICE_1_1;
        let sqrt_q = get_next_sqrt_price_from_output(
            sqrt_p,
            ONE_ETHER.as_u128() as u128,
            *POINT_ONE_ETHER,
            false,
        )
        .unwrap();
        assert_eq!(sqrt_q, 88031291682515930659493278152);
    }

    #[test]
    fn test_get_next_sqrt_price_from_output_output_amount_of_0_1_currency0() {
        let sqrt_p = *SQRT_PRICE_1_1;
        let sqrt_q = get_next_sqrt_price_from_output(
            sqrt_p,
            ONE_ETHER.as_u128() as u128,
            *POINT_ONE_ETHER,
            true,
        )
        .unwrap();
        assert_eq!(sqrt_q, 71305346262837903834189555302);
    }

    #[test]
    fn test_get_next_sqrt_price_from_output_reverts_if_amount_out_is_impossible_in_zero_for_one_direction(
    ) {
        let sqrt_p = *SQRT_PRICE_1_1;
        let result = get_next_sqrt_price_from_output(sqrt_p, 1, U256::MAX, true);
        assert_eq!(result, Err(SqrtPriceMathError::PriceOverflow));
    }

    #[test]
    fn test_get_next_sqrt_price_from_output_reverts_if_amount_out_is_impossible_in_one_for_zero_direction(
    ) {
        let sqrt_p = *SQRT_PRICE_1_1;
        let result = get_next_sqrt_price_from_output(sqrt_p, 1, U256::MAX, false);
        assert_eq!(result, Err(SqrtPriceMathError::PriceOverflow));
    }
}
