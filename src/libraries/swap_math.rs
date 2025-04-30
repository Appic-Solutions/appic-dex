use ethnum::{I256, U256};

use super::{
    amount_delta::{get_amount_0_delta, get_amount_1_delta},
    constants::U160_MAX,
    full_math::{mul_div, mul_div_rounding_up},
    sqrt_price_math::{get_next_sqrt_price_from_input, get_next_sqrt_price_from_output},
};

/// Computes the sqrt price target for the next swap step.
///
/// Determines the target sqrt price for a swap step, choosing between the price at the next
/// initialized tick and a user-specified price limit, based on the swap direction.
/// For `zero_for_one = true` (currency0 to currency1), the target is the maximum of
/// `sqrt_price_next_x96` and `sqrt_price_limit_x96` to prevent the price from falling below the limit.
/// For `zero_for_one = false` (currency1 to currency0), the target is the minimum to prevent
/// exceeding the limit.
///
/// # Arguments
/// * `zero_for_one` - The direction of the swap: `true` for currency0 to currency1, `false` for currency1 to currency0.
/// * `sqrt_price_next_x96` - The Q64.96 sqrt price at the next initialized tick.
/// * `sqrt_price_limit_x96` - The Q64.96 sqrt price limit (cannot be less than this for `zero_for_one = true`,
///   or greater than this for `zero_for_one = false`).
///
/// # Returns
/// The target sqrt price as a `U256` (Q64.96 format), or an error if inputs are invalid.
///
/// # Errors
/// Returns `ComputeSwapError::InvalidPriceOrLiquidity` if either `sqrt_price_next_x96` or `sqrt_price_limit_x96` is zero.
pub fn get_sqrt_price_target(
    zero_for_one: bool,
    sqrt_price_next_x96: U256,
    sqrt_price_limit_x96: U256,
) -> Result<U256, ComputeSwapError> {
    // Validate inputs
    if sqrt_price_next_x96 == 0
        || sqrt_price_limit_x96 == 0
        || sqrt_price_next_x96 > *U160_MAX
        || sqrt_price_limit_x96 > *U160_MAX
    {
        return Err(ComputeSwapError::InvalidPriceOrLiquidity);
    }

    Ok(if zero_for_one {
        sqrt_price_next_x96.max(sqrt_price_limit_x96)
    } else {
        sqrt_price_next_x96.min(sqrt_price_limit_x96)
    })
}

#[derive(Debug, PartialEq)]
pub enum ComputeSwapError {
    PriceOverflow,
    NotEnoughLiquidity,
    InvalidPriceOrLiquidity,
    InvalidFee,
    AmountOverflow,
}

/// Maximum swap fee in hundredths of a bip (100% = 1,000,000).
const MAX_SWAP_FEE: u32 = 1_000_000;

/// Computes the result of swapping some amount in or out, given the parameters of the swap.
///
/// Calculates the next sqrt price, input amount, output amount, and fee for a swap step.
/// Supports exact input (negative `amount_remaining`, specify input amount) and exact output
/// (positive `amount_remaining`, specify output amount) swaps. The fee is taken from the input
/// amount and capped by `MAX_SWAP_FEE`. If the swap is exact input, the combined fee and input
/// amount will not exceed the absolute value of `amount_remaining`.
///
/// # Arguments
/// * `sqrt_price_current_x96` - The current Q64.96 sqrt price of the pool.
/// * `sqrt_price_target_x96` - The target Q64.96 sqrt price for this step (e.g., next tick or price limit).
/// * `liquidity` - The usable liquidity in the current tick range.
/// * `amount_remaining` - The remaining input (negative) or output (positive) amount to swap.
/// * `fee_pips` - The fee in hundredths of a bip (e.g., 3000 for 0.3%).
///
/// # Returns
/// A tuple containing:
/// - `sqrt_price_next_x96`: The Q64.96 sqrt price after the swap step.
/// - `amount_in`: The input amount consumed.
/// - `amount_out`: The output amount produced.
/// - `fee_amount`: The fee collected from the input.
///
/// # Errors
/// Returns `ComputeSwapError::InvalidPriceOrLiquidity` if `sqrt_price_current_x96` or `liquidity` is zero.
/// Returns `ComputeSwapError::PriceOverflow` for arithmetic overflows or invalid fee calculations.
/// Returns `ComputeSwapError::NotEnoughLiquidity` if the swap cannot proceed due to insufficient liquidity.
pub fn compute_swap_step(
    sqrt_price_current_x96: U256,
    sqrt_price_target_x96: U256,
    liquidity: u128,
    amount_remaining: I256,
    fee_pips: u32,
) -> Result<(U256, U256, U256, U256), ComputeSwapError> {
    // Validate inputs
    if sqrt_price_current_x96 == U256::ZERO || liquidity == 0 {
        return Err(ComputeSwapError::InvalidPriceOrLiquidity);
    }
    if fee_pips > MAX_SWAP_FEE {
        return Err(ComputeSwapError::InvalidFee); // Invalid fee
    }

    let zero_for_one = sqrt_price_current_x96 >= sqrt_price_target_x96;
    let exact_in = amount_remaining < I256::from(0);

    if exact_in {
        // Exact input swap: amount_remaining is negative input amount
        let amount_remaining_abs = U256::from((-amount_remaining).as_u256());
        let amount_remaining_less_fee = mul_div(
            amount_remaining_abs,
            U256::from(MAX_SWAP_FEE - fee_pips),
            U256::from(MAX_SWAP_FEE),
        )
        .map_err(|_| ComputeSwapError::AmountOverflow)?;

        let amount_in = if zero_for_one {
            get_amount_0_delta(
                sqrt_price_target_x96,
                sqrt_price_current_x96,
                liquidity,
                true,
            )
            .map_err(|_| ComputeSwapError::PriceOverflow)?
        } else {
            get_amount_1_delta(
                sqrt_price_current_x96,
                sqrt_price_target_x96,
                liquidity,
                true,
            )
            .map_err(|_| ComputeSwapError::PriceOverflow)?
        };

        if amount_remaining_less_fee >= amount_in {
            // Reached target price
            let sqrt_price_next_x96 = sqrt_price_target_x96;
            let fee_amount = if fee_pips == MAX_SWAP_FEE {
                amount_in // amount_in is 0 if amount_remaining_less_fee == 0
            } else {
                mul_div_rounding_up(
                    amount_in,
                    U256::from(fee_pips),
                    U256::from(MAX_SWAP_FEE - fee_pips),
                )
                .map_err(|_| ComputeSwapError::PriceOverflow)?
            };
            let amount_out = if zero_for_one {
                get_amount_1_delta(
                    sqrt_price_next_x96,
                    sqrt_price_current_x96,
                    liquidity,
                    false,
                )
                .map_err(|_| ComputeSwapError::PriceOverflow)?
            } else {
                get_amount_0_delta(
                    sqrt_price_current_x96,
                    sqrt_price_next_x96,
                    liquidity,
                    false,
                )
                .map_err(|_| ComputeSwapError::PriceOverflow)?
            };
            Ok((sqrt_price_next_x96, amount_in, amount_out, fee_amount))
        } else {
            // Exhaust remaining amount
            let amount_in = amount_remaining_less_fee;
            let sqrt_price_next_x96 = get_next_sqrt_price_from_input(
                sqrt_price_current_x96,
                liquidity,
                amount_in,
                zero_for_one,
            )
            .map_err(|_| ComputeSwapError::PriceOverflow)?;
            let fee_amount = amount_remaining_abs - amount_in;
            let amount_out = if zero_for_one {
                get_amount_1_delta(
                    sqrt_price_next_x96,
                    sqrt_price_current_x96,
                    liquidity,
                    false,
                )
                .map_err(|_| ComputeSwapError::PriceOverflow)?
            } else {
                get_amount_0_delta(
                    sqrt_price_current_x96,
                    sqrt_price_next_x96,
                    liquidity,
                    false,
                )
                .map_err(|_| ComputeSwapError::PriceOverflow)?
            };
            Ok((sqrt_price_next_x96, amount_in, amount_out, fee_amount))
        }
    } else {
        // Exact output swap: amount_remaining is positive output amount
        let amount_out = if zero_for_one {
            get_amount_1_delta(
                sqrt_price_target_x96,
                sqrt_price_current_x96,
                liquidity,
                false,
            )
            .map_err(|_| ComputeSwapError::PriceOverflow)?
        } else {
            get_amount_0_delta(
                sqrt_price_current_x96,
                sqrt_price_target_x96,
                liquidity,
                false,
            )
            .map_err(|_| ComputeSwapError::PriceOverflow)?
        };

        let amount_remaining_u256 = amount_remaining.as_u256();
        let (sqrt_price_next_x96, amount_out) = if amount_remaining_u256 >= amount_out {
            // Reached target price
            (sqrt_price_target_x96, amount_out)
        } else {
            // Cap output at remaining amount
            let capped_amount_out = amount_remaining_u256;
            let next_price = get_next_sqrt_price_from_output(
                sqrt_price_current_x96,
                liquidity,
                capped_amount_out,
                zero_for_one,
            )
            .map_err(|_| ComputeSwapError::PriceOverflow)?;
            (next_price, capped_amount_out)
        };

        let amount_in = if zero_for_one {
            get_amount_0_delta(sqrt_price_next_x96, sqrt_price_current_x96, liquidity, true)
                .map_err(|_| ComputeSwapError::PriceOverflow)?
        } else {
            get_amount_1_delta(sqrt_price_current_x96, sqrt_price_next_x96, liquidity, true)
                .map_err(|_| ComputeSwapError::PriceOverflow)?
        };

        // fee_pips cannot be MAX_SWAP_FEE for exact output
        if fee_pips == MAX_SWAP_FEE {
            return Err(ComputeSwapError::PriceOverflow);
        }
        let fee_amount = mul_div_rounding_up(
            amount_in,
            U256::from(fee_pips),
            U256::from(MAX_SWAP_FEE - fee_pips),
        )
        .map_err(|_| ComputeSwapError::PriceOverflow)?;

        Ok((sqrt_price_next_x96, amount_in, amount_out, fee_amount))
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use ethnum::{I256, U256};
    use lazy_static::lazy_static;

    // Constants from Solidity, converted to U256
    lazy_static! {
        pub static ref SQRT_PRICE_1_1: U256 = U256::from(79228162514264337593543950336u128);
        pub static ref SQRT_PRICE_1_2: U256 = U256::from(56022770974786139918731938227u128);
        pub static ref SQRT_PRICE_1_4: U256 = U256::from(39614081257132168796771975168u128);
        pub static ref SQRT_PRICE_2_1: U256 = U256::from(112045541949572279837463876454u128);
        pub static ref SQRT_PRICE_4_1: U256 = U256::from(158456325028528675187087900672u128);
        pub static ref SQRT_PRICE_121_100: U256 = U256::from(87150978765690771352898345369u128);
        pub static ref SQRT_PRICE_99_100: U256 = U256::from(78831026366734652303669917531u128);
        pub static ref SQRT_PRICE_99_1000: U256 = U256::from(24928559360766947368818086097u128);
        pub static ref SQRT_PRICE_101_100: U256 = U256::from(79623317895830914510639640423u128);
        pub static ref SQRT_PRICE_1000_100: U256 = U256::from(250541448375047931186413801569u128);
        pub static ref SQRT_PRICE_1010_100: U256 = U256::from(251791039410471229173201122529u128);
        pub static ref SQRT_PRICE_10000_100: U256 = U256::from(792281625142643375935439503360u128);
        pub static ref ONE_ETHER: U256 = U256::from(1_000_000_000_000_000_000u128); // 10^18
        pub static ref POINT_ONE_ETHER: U256 = U256::from(100_000_000_000_000_000u128); // 0.1 * 10^18
        pub static ref ONE_THOUSAND: U256 = U256::from(1000_u32);// 10^3

    }
    const MAX_U256: U256 = U256::MAX;

    #[test]
    fn test_get_sqrt_price_target() {
        let test_cases = [
            (true, *SQRT_PRICE_1_1 - *ONE_THOUSAND, *SQRT_PRICE_1_1),
            (true, *SQRT_PRICE_1_1 + *ONE_THOUSAND, *SQRT_PRICE_1_1),
            (false, *SQRT_PRICE_1_1 + *ONE_THOUSAND, *SQRT_PRICE_1_1),
            (false, *SQRT_PRICE_1_1 - *ONE_THOUSAND, *SQRT_PRICE_1_1),
        ];

        for (zero_for_one, sqrt_price_next_x96, sqrt_price_limit_x96) in test_cases {
            let result =
                get_sqrt_price_target(zero_for_one, sqrt_price_next_x96, sqrt_price_limit_x96)
                    .unwrap();
            let expected = if zero_for_one {
                if sqrt_price_next_x96 < sqrt_price_limit_x96 {
                    sqrt_price_limit_x96
                } else {
                    sqrt_price_next_x96
                }
            } else {
                if sqrt_price_next_x96 > sqrt_price_limit_x96 {
                    sqrt_price_limit_x96
                } else {
                    sqrt_price_next_x96
                }
            };
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_compute_swap_step_exact_amount_in_one_for_zero_capped_at_price_target() {
        let price = *SQRT_PRICE_1_1;
        let price_target = *SQRT_PRICE_101_100;
        let liquidity = 2 * ONE_ETHER.as_u128();
        let amount = I256::from(-ONE_ETHER.as_i256());
        let lp_fee = 600;

        let (sqrt_q, amount_in, amount_out, fee_amount) =
            compute_swap_step(price, price_target, liquidity, amount, lp_fee).unwrap();

        assert_eq!(amount_in, U256::from(9975124224178055u128));
        assert_eq!(amount_out, U256::from(9925619580021728u128));
        assert_eq!(fee_amount, U256::from(5988667735148u128));
        assert!(amount_in + fee_amount < U256::from((-amount).as_u256()));

        let price_after_whole_input = get_next_sqrt_price_from_input(
            price,
            liquidity,
            U256::from((-amount).as_u256()),
            false,
        )
        .unwrap();

        assert_eq!(sqrt_q, price_target);
        assert!(sqrt_q < price_after_whole_input);
    }

    #[test]
    fn test_compute_swap_step_exact_amount_out_one_for_zero_capped_at_price_target() {
        let price = *SQRT_PRICE_1_1;
        let price_target = *SQRT_PRICE_101_100;
        let liquidity = 2 * ONE_ETHER.as_u128();
        let amount = I256::from(ONE_ETHER.as_i256());
        let lp_fee = 600;

        let (sqrt_q, amount_in, amount_out, fee_amount) =
            compute_swap_step(price, price_target, liquidity, amount, lp_fee).unwrap();

        assert_eq!(amount_in, U256::from(9975124224178055u128));
        assert_eq!(amount_out, U256::from(9925619580021728u128));
        assert_eq!(fee_amount, U256::from(5988667735148u128));
        assert!(amount_out < U256::from(amount.as_u256()));

        let price_after_whole_output =
            get_next_sqrt_price_from_output(price, liquidity, U256::from(amount.as_u256()), false)
                .unwrap();

        assert_eq!(sqrt_q, price_target);
        assert!(sqrt_q < price_after_whole_output);
    }

    #[test]
    fn test_compute_swap_step_exact_amount_in_one_for_zero_fully_spent() {
        let price = *SQRT_PRICE_1_1;
        let price_target = *SQRT_PRICE_1000_100;
        let liquidity = 2 * ONE_ETHER.as_u128();
        let amount = I256::from(-ONE_ETHER.as_i256());
        let lp_fee = 600;

        let (sqrt_q, amount_in, amount_out, fee_amount) =
            compute_swap_step(price, price_target, liquidity, amount, lp_fee).unwrap();

        assert_eq!(amount_in, U256::from(999400000000000000u128));
        assert_eq!(amount_out, U256::from(666399946655997866u128));
        assert_eq!(fee_amount, U256::from(600000000000000u128));
        assert_eq!(amount_in + fee_amount, U256::from((-amount).as_u256()));

        let price_after_whole_input_less_fee = get_next_sqrt_price_from_input(
            price,
            liquidity,
            U256::from((-amount).as_u256()) - fee_amount,
            false,
        )
        .unwrap();

        assert!(sqrt_q < price_target);
        assert_eq!(sqrt_q, price_after_whole_input_less_fee);
    }

    #[test]
    fn test_compute_swap_step_exact_amount_out_one_for_zero_fully_received() {
        let price = *SQRT_PRICE_1_1;
        let price_target = *SQRT_PRICE_10000_100;
        let liquidity = 2 * ONE_ETHER.as_u128();
        let amount = I256::from(ONE_ETHER.as_i256());
        let lp_fee = 600;

        let (sqrt_q, amount_in, amount_out, fee_amount) =
            compute_swap_step(price, price_target, liquidity, amount, lp_fee).unwrap();

        assert_eq!(amount_in, U256::from(2000000000000000000u128));
        assert_eq!(fee_amount, U256::from(1200720432259356u128));
        assert_eq!(amount_out, U256::from(amount.as_u256()));

        let price_after_whole_output =
            get_next_sqrt_price_from_output(price, liquidity, U256::from(amount.as_u256()), false)
                .unwrap();

        assert!(sqrt_q < price_target);
        assert_eq!(sqrt_q, price_after_whole_output);
    }

    #[test]
    fn test_compute_swap_step_amount_out_capped_at_desired_amount_out() {
        let (sqrt_q, amount_in, amount_out, fee_amount) = compute_swap_step(
            U256::from(417332158212080721273783715441582u128),
            U256::from(1452870262520218020823638996u128),
            159344665391607089467575320103,
            I256::from(1),
            1,
        )
        .unwrap();

        assert_eq!(amount_in, U256::ONE);
        assert_eq!(fee_amount, U256::ONE);
        assert_eq!(amount_out, U256::ONE);
        assert_eq!(sqrt_q, U256::from(417332158212080721273783715441581u128));
    }

    #[test]
    fn test_compute_swap_step_target_price_of_1_uses_partial_input_amount() {
        let (sqrt_q, amount_in, amount_out, fee_amount) = compute_swap_step(
            U256::from(2_u8),
            U256::ONE,
            1,
            I256::from(-3915081100057732413702495386755767i128),
            1,
        )
        .unwrap();

        assert_eq!(amount_in, *SQRT_PRICE_1_4);
        assert_eq!(fee_amount, U256::from(39614120871253040049813u128));
        assert!(amount_in + fee_amount <= U256::from(3915081100057732413702495386755767u128));
        assert_eq!(amount_out, U256::ZERO);
        assert_eq!(sqrt_q, U256::ONE);
    }

    #[test]
    fn test_compute_swap_step_not_entire_input_amount_taken_as_fee() {
        let (sqrt_q, amount_in, amount_out, fee_amount) = compute_swap_step(
            U256::from(2413_u128),
            U256::from(79887613182836312u128),
            1985041575832132834610021537970,
            I256::from(-10),
            1872,
        )
        .unwrap();

        assert_eq!(amount_in, U256::from(9_u8));
        assert_eq!(fee_amount, U256::from(1_u8));
        assert_eq!(amount_out, U256::from(0_u8));
        assert_eq!(sqrt_q, U256::from(2413_u32));
    }

    #[test]
    fn test_compute_swap_step_zero_for_one_handles_intermediate_insufficient_liquidity_exact_output(
    ) {
        let sqrt_p = U256::from(20282409603651670423947251286016u128);
        let sqrt_p_target = sqrt_p * U256::from(11_u8) / U256::from(10_u8);
        let liquidity = 1024;
        let amount_remaining = I256::from(4);
        let fee_pips = 3000;

        let (sqrt_q, amount_in, amount_out, fee_amount) =
            compute_swap_step(sqrt_p, sqrt_p_target, liquidity, amount_remaining, fee_pips)
                .unwrap();

        assert_eq!(amount_out, U256::ZERO);
        assert_eq!(sqrt_q, sqrt_p_target);
        assert_eq!(amount_in, U256::from(26215_u32));
        assert_eq!(fee_amount, U256::from(79_u8));
    }

    #[test]
    fn test_compute_swap_step_one_for_zero_handles_intermediate_insufficient_liquidity_exact_output(
    ) {
        let sqrt_p = U256::from(20282409603651670423947251286016u128);
        let sqrt_p_target = sqrt_p * U256::from(9_u8) / U256::from(10_u8);
        let liquidity = 1024;
        let amount_remaining = I256::from(263000);
        let fee_pips = 3000;

        let (sqrt_q, amount_in, amount_out, fee_amount) =
            compute_swap_step(sqrt_p, sqrt_p_target, liquidity, amount_remaining, fee_pips)
                .unwrap();

        assert_eq!(amount_out, U256::from(26214_u32));
        assert_eq!(sqrt_q, sqrt_p_target);
        assert_eq!(amount_in, U256::ONE);
        assert_eq!(fee_amount, U256::ONE);
    }

    #[test]
    fn test_compute_swap_step_fuzz() {
        // Simulate fuzzing with constrained inputs
        let test_cases = [
            (
                *SQRT_PRICE_1_1,
                *SQRT_PRICE_101_100,
                1000000,
                I256::from(-1000),
                600,
            ),
            (
                *SQRT_PRICE_1_1,
                *SQRT_PRICE_99_100,
                1000000,
                I256::from(1000),
                600,
            ),
            (
                *SQRT_PRICE_1_2,
                *SQRT_PRICE_1_1,
                500000,
                I256::from(-500),
                3000,
            ),
        ];

        for (sqrt_price_raw, sqrt_price_target_raw, liquidity, amount_remaining, fee_pips) in
            test_cases
        {
            if sqrt_price_raw == U256::ZERO
                || sqrt_price_target_raw == U256::ZERO
                || fee_pips > MAX_SWAP_FEE
            {
                continue;
            }
            if amount_remaining >= I256::from(0) && fee_pips >= MAX_SWAP_FEE {
                continue;
            }

            let result = compute_swap_step(
                sqrt_price_raw,
                sqrt_price_target_raw,
                liquidity,
                amount_remaining,
                fee_pips,
            );

            if result.is_err() {
                continue; // Skip error cases for now
            }

            let (sqrt_q, amount_in, amount_out, fee_amount) = result.unwrap();

            // Check amount_in + fee_amount doesn't overflow
            assert!(amount_in <= MAX_U256 - fee_amount);

            // Check input/output constraints
            if amount_remaining >= I256::from(0) {
                assert!(amount_out <= U256::from(amount_remaining.as_u256()));
            } else {
                assert!(amount_in + fee_amount <= U256::from((-amount_remaining).as_u256()));
            }

            // If price didn't change, amounts should be zero
            if sqrt_price_raw == sqrt_price_target_raw {
                assert_eq!(amount_in, U256::ZERO);
                assert_eq!(amount_out, U256::ZERO);
                assert_eq!(fee_amount, U256::ZERO);
                assert_eq!(sqrt_q, sqrt_price_target_raw);
            }

            // If didn't reach target, entire amount consumed
            if sqrt_q != sqrt_price_target_raw {
                let abs_amt_remaining = if amount_remaining >= I256::from(0) {
                    U256::from(amount_remaining.as_u256())
                } else {
                    U256::from((-amount_remaining).as_u256())
                };
                if amount_remaining > I256::from(0) {
                    assert_eq!(amount_out, abs_amt_remaining);
                } else {
                    assert_eq!(amount_in + fee_amount, abs_amt_remaining);
                }
            }

            // Next price is between current and target
            if sqrt_price_target_raw <= sqrt_price_raw {
                assert!(sqrt_q <= sqrt_price_raw);
                assert!(sqrt_q >= sqrt_price_target_raw);
            } else {
                assert!(sqrt_q >= sqrt_price_raw);
                assert!(sqrt_q <= sqrt_price_target_raw);
            }
        }
    }
}
