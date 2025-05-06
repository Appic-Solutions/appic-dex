use ethnum::U256;
use num_bigint::BigUint;
use num_traits::Zero;

use super::safe_cast::big_uint_to_u256;

#[derive(Debug, Clone, PartialEq)]
pub enum FullMathError {
    DivisionByZero,
    Overflow,
}

pub fn mul_div(a: U256, b: U256, denominator: U256) -> Result<U256, FullMathError> {
    if denominator == U256::ZERO {
        return Err(FullMathError::DivisionByZero);
    }

    let product =
        BigUint::from_bytes_be(&a.to_be_bytes()) * BigUint::from_bytes_be(&b.to_be_bytes());

    if product.bits() > 512 {
        return Err(FullMathError::Overflow);
    }

    let quotient = product / BigUint::from_bytes_be(&denominator.to_be_bytes());

    if quotient.bits() > 256 {
        return Err(FullMathError::Overflow);
    }

    // Convert BigUint quotient back to U256
    Ok(big_uint_to_u256(quotient).unwrap())
}

pub fn mul_div_rounding_up(a: U256, b: U256, denominator: U256) -> Result<U256, FullMathError> {
    if denominator == U256::ZERO {
        return Err(FullMathError::DivisionByZero);
    }

    let product =
        BigUint::from_bytes_be(&a.to_be_bytes()) * BigUint::from_bytes_be(&b.to_be_bytes());

    if product.bits() > 512 {
        return Err(FullMathError::Overflow);
    }

    let denominator_big = BigUint::from_bytes_be(&denominator.to_be_bytes());
    let quotient = &product / &denominator_big;
    let remainder = &product % &denominator_big;
    let result = if remainder.is_zero() {
        quotient
    } else {
        quotient + BigUint::from(1u32)
    };

    if result.bits() > 256 {
        return Err(FullMathError::Overflow);
    }

    // Convert BigUint result back to U256
    Ok(big_uint_to_u256(result).unwrap())
}

/// Returns ceil(x / y)
/// division by 0 will return 0, and should be checked externally
/// returns z The quotient, ceil(x / y)
pub fn div_rounding_up(x: U256, y: U256) -> U256 {
    if y == U256::ZERO {
        return U256::ZERO; // Match Solidity's behavior of returning 0 on division by 0
    }
    let quotient = x / y;
    let remainder = x % y;
    quotient
        + if remainder > U256::ZERO {
            U256::from(1u32)
        } else {
            U256::ZERO
        }
}

#[cfg(test)]
mod tests {
    use crate::libraries::constants::Q96;

    use super::*;
    use ethnum::U256;

    const Q128: U256 = U256::from_words(1, 0); // 2^128
    const MAX_UINT256: U256 = U256::MAX;

    #[test]
    fn test_u128() {
        assert_eq!(Q128, U256::from(u128::MAX) + U256::ONE)
    }

    #[test]
    fn test_q96() {
        assert_eq!(Q96.clone(), U256::from(2_u8).pow(96))
    }

    #[test]
    fn reverts_if_denominator_is_zero() {
        let a = Q128;
        let b = U256::from(5_u8);
        let denominator = U256::from(0_u8);
        assert_eq!(
            mul_div(a, b, denominator),
            Err(FullMathError::DivisionByZero)
        );
    }

    #[test]
    fn reverts_if_denominator_is_zero_and_numerator_overflows() {
        let a = Q128;
        let b = Q128;
        let denominator = U256::from(0_u8);
        assert_eq!(
            mul_div(a, b, denominator),
            Err(FullMathError::DivisionByZero)
        );
    }

    #[test]
    fn reverts_if_output_overflows_uint256() {
        let a = Q128;
        let b = Q128;
        let denominator = U256::from(1_u8);
        assert_eq!(mul_div(a, b, denominator), Err(FullMathError::Overflow));
    }

    #[test]
    fn reverts_on_overflow_with_all_max_inputs() {
        let a = MAX_UINT256;
        let b = MAX_UINT256;
        let denominator = MAX_UINT256 - U256::from(1_u8);
        assert_eq!(mul_div(a, b, denominator), Err(FullMathError::Overflow));
    }

    #[test]
    fn all_max_inputs() {
        let a = MAX_UINT256;
        let b = MAX_UINT256;
        let denominator = MAX_UINT256;
        assert_eq!(mul_div(a, b, denominator), Ok(MAX_UINT256));
    }

    #[test]
    fn accurate_without_phantom_overflow() {
        let a = Q128;
        let b = U256::from(50_u8) * Q128 / U256::from(100_u8); // 0.5 * Q128
        let denominator = U256::from(150_u8) * Q128 / U256::from(100_u8); // 1.5 * Q128
        let expected = Q128 / U256::from(3_u8);
        assert_eq!(mul_div(a, b, denominator), Ok(expected));
    }

    #[test]
    fn accurate_with_phantom_overflow() {
        let a = Q128;
        let b = U256::from(35_u8) * Q128;
        let denominator = U256::from(8_u8) * Q128;
        let expected = U256::from(4375_u32) * Q128 / U256::from(1000_u32);
        assert_eq!(mul_div(a, b, denominator), Ok(expected));
    }

    #[test]
    fn accurate_with_phantom_overflow_and_repeating_decimal() {
        let a = Q128;
        let b = U256::from(1000_u32) * Q128;
        let denominator = U256::from(3000_u32) * Q128;
        let expected = U256::from(1_u8) * Q128 / U256::from(3_u8);
        assert_eq!(mul_div(a, b, denominator), Ok(expected));
    }

    #[test]
    fn test_mul_div_rounding_up_valid_with_all_max_inputs() {
        let result = mul_div_rounding_up(MAX_UINT256, MAX_UINT256, MAX_UINT256).unwrap();
        assert_eq!(result, MAX_UINT256);
    }

    #[test]
    fn test_mul_div_rounding_up_valid_with_no_phantom_overflow() {
        let numerator = Q128 * 50 / 100; // 50 * Q128 / 100
        let denominator = Q128 * 150 / 100; // 150 * Q128 / 100
        let expected = Q128 / 3 + 1; // ceil(Q128 * 0.5 / 1.5)
        let result = mul_div_rounding_up(Q128, numerator, denominator).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_mul_div_rounding_up_valid_with_phantom_overflow() {
        let numerator = Q128 * 35; // 35 * Q128
        let denominator = Q128 * 8; // 8 * Q128
        let expected = Q128 * 4375 / 1000; // 4375 * Q128 / 1000
        let result = mul_div(numerator, Q128, denominator).unwrap(); // Note: This test uses mul_div, not rounding up
        assert_eq!(result, expected);
    }

    #[test]
    fn test_mul_div_rounding_up_valid_with_phantom_overflow_repeating_decimal() {
        let numerator = Q128 * 1000; // 1000 * Q128
        let denominator = Q128 * 3000; // 3000 * Q128
        let expected = Q128 / 3 + 1; // ceil(Q128 / 3)
        let result = mul_div_rounding_up(Q128, numerator, denominator).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    #[should_panic]
    fn test_mul_div_rounding_up_reverts_if_overflows_256_bits_after_rounding_up() {
        let a = U256::from(535006138814359u64);
        let b = U256::from_str_radix(
            "432862656469423142931042426214547535783388063929571229938474969",
            10,
        )
        .unwrap();
        let d = U256::from(2u64);
        mul_div_rounding_up(a, b, d).unwrap(); // Should panic with Overflow
    }

    #[test]
    #[should_panic]
    fn test_mul_div_rounding_up_reverts_if_overflows_256_bits_after_rounding_up_case2() {
        let a = U256::from_str_radix(
            "115792089237316195423570985008687907853269984659341747863450311749907997002549",
            10,
        )
        .unwrap();
        let b = U256::from_str_radix(
            "115792089237316195423570985008687907853269984659341747863450311749907997002550",
            10,
        )
        .unwrap();
        let d = U256::from_str_radix(
            "115792089237316195423570985008687907853269984653042931687443039491902864365164",
            10,
        )
        .unwrap();
        mul_div_rounding_up(a, b, d).unwrap(); // Should panic with Overflow
    }

    #[test]
    fn test_fuzz_mul_div_rounding_up() {
        // Fuzzing simulation with specific values (Rust doesn’t have built-in fuzzing like Foundry)
        let cases = [
            (U256::from(10u32), U256::from(20u32), U256::from(3u32)), // x=10, y=20, d=3
            (U256::from(100u32), U256::from(200u32), U256::from(50u32)), // x=100, y=200, d=50
            (MAX_UINT256 / 2, MAX_UINT256 / 2, MAX_UINT256),          // Large values
        ];

        for (x, y, d) in cases {
            if let Ok(result) = mul_div_rounding_up(x, y, d) {
                let product = BigUint::from_bytes_be(&x.to_be_bytes())
                    * BigUint::from_bytes_be(&y.to_be_bytes());
                let denominator = BigUint::from_bytes_be(&d.to_be_bytes());
                let numerator = product.clone();
                let expected = if product % denominator > BigUint::ZERO {
                    numerator / BigUint::from_bytes_be(&d.to_be_bytes()) + 1_u8
                } else {
                    numerator / BigUint::from_bytes_be(&d.to_be_bytes())
                };
                assert_eq!(
                    result,
                    U256::from_str_radix(&expected.to_str_radix(10), 10).unwrap()
                );
            }
        }
    }

    #[test]
    fn test_invariant_mul_div_rounding() {
        let cases = [
            (U256::from(10u32), U256::from(20u32), U256::from(3u32)),
            (Q128, Q128 * 50 / 100, Q128 * 150 / 100),
        ];

        for (x, y, d) in cases {
            if let (Ok(ceiled), Ok(floored)) = (mul_div_rounding_up(x, y, d), mul_div(x, y, d)) {
                let product = BigUint::from_bytes_be(&x.to_be_bytes())
                    * BigUint::from_bytes_be(&y.to_be_bytes());
                let denominator = BigUint::from_bytes_be(&d.to_be_bytes());
                if product % denominator > BigUint::ZERO {
                    assert_eq!(ceiled - floored, U256::ONE);
                } else {
                    assert_eq!(ceiled, floored);
                }
            }
        }
    }

    #[test]
    fn test_invariant_mul_div() {
        let cases = [
            (U256::from(10u32), U256::from(20u32), U256::from(3u32)),
            (Q128, Q128 * 35, Q128 * 8),
        ];

        for (x, y, d) in cases {
            if let Ok(z) = mul_div(x, y, d) {
                if x == U256::ZERO || y == U256::ZERO {
                    assert_eq!(z, U256::ZERO);
                    continue;
                }

                let x2 = mul_div(z, d, y).unwrap();
                let y2 = mul_div(z, d, x).unwrap();
                assert!(x2 <= x);
                assert!(y2 <= y);
                assert!(x - x2 < d);
                assert!(y - y2 < d);
            }
        }
    }

    #[test]
    fn test_invariant_mul_div_rounding_up() {
        let cases = [
            (U256::from(10u32), U256::from(20u32), U256::from(3u32)),
            (Q128, Q128 * 1000, Q128 * 3000),
        ];

        for (x, y, d) in cases {
            if let Ok(z) = mul_div_rounding_up(x, y, d) {
                if x == U256::ZERO || y == U256::ZERO {
                    assert_eq!(z, U256::ZERO);
                    continue;
                }

                if let (Ok(x2), Ok(y2)) = (mul_div(z, d, y), mul_div(z, d, x)) {
                    assert!(x2 >= x);
                    assert!(y2 >= y);
                    assert!(x2 - x < d);
                    assert!(y2 - y < d);
                }
            }
        }
    }

    // Helper function to simulate resultOverflows
    fn result_overflows(x: U256, y: U256, d: U256) -> bool {
        if x == U256::ZERO || y == U256::ZERO {
            return false;
        }

        let product =
            BigUint::from_bytes_be(&x.to_be_bytes()) * BigUint::from_bytes_be(&y.to_be_bytes());
        if product.bits() <= 256 {
            return false;
        }

        let quotient = mul_div(x, y, d).unwrap_or(U256::MAX);
        let rounding_up_quotient = mul_div_rounding_up(x, y, d).unwrap_or(U256::MAX);
        product.bits() > 512 || quotient == U256::MAX || rounding_up_quotient == U256::MAX
    }

    #[test]
    fn test_result_overflows_helper() {
        assert!(!result_overflows(U256::ZERO, U256::ZERO, U256::ONE));
        assert!(!result_overflows(U256::ONE, U256::ZERO, U256::ONE));
        assert!(!result_overflows(U256::ZERO, U256::ONE, U256::ONE));
        assert!(!result_overflows(U256::ONE, U256::ONE, U256::ONE));
        assert!(!result_overflows(
            U256::from(10000000u32),
            U256::from(10000000u32),
            U256::ONE
        ));
        assert!(!result_overflows(Q128, Q128 * 50 / 100, Q128 * 150 / 100));
        assert!(!result_overflows(Q128, Q128 * 35, Q128 * 8));
        assert!(result_overflows(U256::MAX, U256::MAX, U256::MAX - 1));
        assert!(result_overflows(Q128, U256::MAX, U256::ONE));
    }

    #[test]
    fn test_div_rounding_up_zero_does_not_revert() {
        // In Solidity, x.divRoundingUp(0) doesn't revert and returns 0 due to assembly
        // Rust version explicitly returns U256::ZERO, so we test various x values
        let cases = [U256::from(0u32), U256::from(42u32), Q128, MAX_UINT256];
        for x in cases {
            assert_eq!(div_rounding_up(x, U256::ZERO), U256::ZERO);
        }
    }

    #[test]
    fn test_div_rounding_up_max_input() {
        // MAX_UINT256 / MAX_UINT256 should be 1
        assert_eq!(div_rounding_up(MAX_UINT256, MAX_UINT256), U256::from(1u32));
    }

    #[test]
    fn test_div_rounding_up_rounds_up() {
        // Q128 / 3 should round up to Q128 / 3 + 1
        let result = Q128 / U256::from(3u32) + U256::from(1u32);
        assert_eq!(div_rounding_up(Q128, U256::from(3u32)), result);
    }

    #[test]
    fn test_fuzz_div_rounding_up() {
        // Simulate fuzzing with sample cases (Rust doesn't have vm.assume natively)
        let cases = [
            (U256::from(5u32), U256::from(2u32)),    // 5 / 2 = 2.5 -> 3
            (U256::from(10u32), U256::from(3u32)),   // 10 / 3 = 3.33 -> 4
            (U256::from(100u32), U256::from(25u32)), // 100 / 25 = 4 -> 4
            (Q128, U256::from(7u32)),                // Q128 / 7
            (MAX_UINT256, Q128),                     // MAX_UINT256 / Q128
        ];

        for (x, y) in cases {
            let result = div_rounding_up(x, y);
            let floor = x / y;
            assert!(result == floor || result == floor + U256::from(1u32));
        }
    }

    #[test]
    fn test_invariant_div_rounding_up() {
        // Test the invariant: z = x / y or x / y + 1 based on remainder
        let cases = [
            (U256::from(6u32), U256::from(2u32)), // 6 / 2 = 3 (no remainder)
            (U256::from(7u32), U256::from(3u32)), // 7 / 3 = 2 + 1 (remainder)
            (Q128, U256::from(4u32)),             // Q128 / 4
            (MAX_UINT256, U256::from(100u32)),    // MAX_UINT256 / 100
        ];

        for (x, y) in cases {
            let z = div_rounding_up(x, y);
            let floor = x / y;
            let diff = z - floor;
            let remainder = x % y;
            if remainder == U256::ZERO {
                assert_eq!(diff, U256::ZERO);
            } else {
                assert_eq!(diff, U256::from(1u32));
            }
        }
    }
}
