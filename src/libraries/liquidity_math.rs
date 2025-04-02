#[derive(Debug, PartialEq)]
pub enum AddDeltaError {
    Overflow,  // 'LA'
    Underflow, // 'LS'
}

/// Add a signed liquidity delta to liquidity and Err if it overflows or underflows
/// x The liquidity before change
/// y The delta by which liquidity should be changed
pub fn add_delta(x: u128, y: i128) -> Result<u128, AddDeltaError> {
    if y >= 0 {
        x.checked_add(y as u128).ok_or(AddDeltaError::Overflow)
    } else {
        x.checked_sub(y.wrapping_neg() as u128)
            .ok_or(AddDeltaError::Underflow)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_one_plus_zero() {
        assert_eq!(add_delta(1, 0).unwrap(), 1);
    }

    #[test]
    fn test_one_plus_negative_one() {
        assert_eq!(add_delta(1, -1).unwrap(), 0);
    }

    #[test]
    fn test_one_plus_one() {
        assert_eq!(add_delta(1, 1).unwrap(), 2);
    }

    #[test]
    fn test_max_minus_15_plus_15() {
        let x = u128::MAX - 15;
        let result = add_delta(x, 15).unwrap();
        assert_eq!(result, u128::MAX);
    }

    #[test]
    fn test_zero_plus_negative_one_underflows() {
        let result = add_delta(0, -1);
        assert!(matches!(result, Err(AddDeltaError::Underflow)));
    }

    #[test]
    fn test_three_plus_negative_four_underflows() {
        let result = add_delta(3, -4);
        assert!(matches!(result, Err(AddDeltaError::Underflow)));
    }

    #[test]
    fn test_max_plus_one_overflows() {
        let result = add_delta(u128::MAX, 1);
        assert!(matches!(result, Err(AddDeltaError::Overflow)));
    }

    #[test]
    fn test_max_plus_max_i128_overflows() {
        let result = add_delta(u128::MAX, i128::MAX);
        assert!(matches!(result, Err(AddDeltaError::Overflow)));
    }

    #[test]
    fn test_zero_plus_min_i128_underflows() {
        let result = add_delta(0, i128::MIN);
        assert!(matches!(result, Err(AddDeltaError::Underflow)));
    }

    #[test]
    fn test_large_value_plus_large_negative() {
        let x = u128::MAX / 2;
        let y = -(x as i128) - 1;
        let result = add_delta(x, y);
        assert!(matches!(result, Err(AddDeltaError::Underflow)));
    }

    #[test]
    fn test_max_minus_one_plus_zero() {
        assert_eq!(add_delta(u128::MAX - 1, 0).unwrap(), u128::MAX - 1);
    }

    #[test]
    fn test_half_max_plus_half_max() {
        let half_max = u128::MAX / 2;
        assert_eq!(add_delta(half_max, half_max as i128).unwrap(), half_max * 2);
    }
}
