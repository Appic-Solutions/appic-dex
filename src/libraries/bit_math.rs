use ethnum::U256;

// Returns the most significant bit's potions
// None if value == 0
pub fn get_msb_bit_position(value: &U256) -> Option<u8> {
    if value == &U256::ZERO {
        None // No set bits
    } else {
        Some((255 - value.leading_zeros()) as u8)
    }
}

// Returns the least significant bit's position
// None if value == 0
pub fn get_lsb_bit_position(value: &U256) -> Option<u8> {
    if value == &U256::ZERO {
        None // No set bits
    } else {
        Some((value.trailing_zeros()) as u8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn test_most_significant_bit_reverts_when_zero() {
        // In Rust, we'll simulate revert by panicking if None is returned
        let _result = get_msb_bit_position(&U256::ZERO).expect("Expected panic for zero");
    }

    #[test]
    fn test_most_significant_bit_one() {
        assert_eq!(get_msb_bit_position(&U256::from(1u128)), Some(0));
    }

    #[test]
    fn test_most_significant_bit_two() {
        assert_eq!(get_msb_bit_position(&U256::from(2u128)), Some(1));
    }

    #[test]
    fn test_most_significant_bit_powers_of_two() {
        for i in 0..=255 {
            let x = U256::from(1u128) << i;
            assert_eq!(get_msb_bit_position(&x), Some(i), "Failed at power 2^{}", i);
        }
    }

    #[test]
    fn test_most_significant_bit_max_uint256() {
        assert_eq!(get_msb_bit_position(&U256::MAX), Some(255));
    }

    #[test]
    fn test_invariant_most_significant_bit() {
        // Test with some non-zero values
        let values = [
            U256::from(1u128),
            U256::from(2u128),
            U256::from(3u128),
            U256::from(256u128),
            U256::MAX,
        ];

        for x in values.iter() {
            let msb = get_msb_bit_position(x).unwrap();
            // Assert x >= 2^msb
            assert!(*x >= U256::from(1u128) << msb, "Failed: {} < 2^{}", x, msb);
            // Assert x < 2^(msb+1) unless msb == 255
            if msb < 255 {
                assert!(
                    *x < U256::from(1u128) << (msb + 1),
                    "Failed: {} >= 2^{}",
                    x,
                    msb + 1
                );
            }
        }
    }

    // LSB Tests
    #[test]
    #[should_panic]
    fn test_least_significant_bit_reverts_when_zero() {
        let _result = get_lsb_bit_position(&U256::ZERO).expect("Expected panic for zero");
    }

    #[test]
    fn test_least_significant_bit_max_uint256() {
        assert_eq!(get_lsb_bit_position(&U256::MAX), Some(0));
    }

    #[test]
    fn test_least_significant_bit_one() {
        assert_eq!(get_lsb_bit_position(&U256::from(1u128)), Some(0));
    }

    #[test]
    fn test_least_significant_bit_two() {
        assert_eq!(get_lsb_bit_position(&U256::from(2u128)), Some(1));
    }

    #[test]
    fn test_least_significant_bit_powers_of_two() {
        for i in 0..=255 {
            let x = U256::from(1u128) << i;
            assert_eq!(get_lsb_bit_position(&x), Some(i), "Failed at power 2^{}", i);
        }
    }

    #[test]
    fn test_least_significant_bit_odd_numbers() {
        // Even numbers have LSB = 0
        let values = [U256::from(1u128), U256::from(3u128), U256::from(255u128)];
        for x in values.iter() {
            assert_eq!(get_lsb_bit_position(x), Some(0), "Failed for value {}", x);
        }
    }

    #[test]
    fn test_invariant_least_significant_bit() {
        let values = [
            U256::from(1u128),   // 0b1
            U256::from(2u128),   // 0b10
            U256::from(3u128),   // 0b11
            U256::from(256u128), // 0b100000000
            U256::MAX,
        ];

        for x in values.iter() {
            let lsb = get_lsb_bit_position(x).unwrap();
            // Assert x has a 1 at lsb position: x & (1 << lsb) != 0
            assert!(
                (*x & (U256::from(1u128) << lsb)) != U256::ZERO,
                "Failed: {} does not have 1 at bit {}",
                x,
                lsb
            );
            // Assert all bits below lsb are 0: x & ((1 << lsb) - 1) == 0
            if lsb > 0 {
                assert!(
                    (*x & ((U256::from(1u128) << lsb) - U256::from(1u128))) == U256::ZERO,
                    "Failed: {} has set bits below {}",
                    x,
                    lsb
                );
            }
        }
    }
}
