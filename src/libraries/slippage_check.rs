use crate::logs::DEBUG;

use super::balance_delta::BalanceDelta;
use ethnum::{I256, U256};
use ic_canister_log::log;

#[derive(Debug, PartialEq)]
pub enum BalanceDeltaValidationError {
    MinimumAmountInsufficient { required: U256, received: U256 },
    MaximumAmountExceeded { maximum: U256, received: U256 },
    NegativeDeltaNotAllowed,
}

/// Validates that the balance delta meets minimum output requirements for liquidity removal.
/// Fails if amount0 or amount1 is negative or less than the minimum specified.
/// # Arguments
/// * `delta` - The balance delta containing token0 and token1 amounts
/// * `amount0_min` - The minimum amount of token0 to receive
/// * `amount1_min` - The minimum amount of token1 to receive
/// # Returns
/// * `Result<(), BalanceDeltaValidationError>` - Ok if valid, Err if constraints are not met
pub fn validate_min_out(
    delta: BalanceDelta,
    amount0_min: I256,
    amount1_min: I256,
) -> Result<(), BalanceDeltaValidationError> {
    let amount0 = delta.amount0();
    let amount1 = delta.amount1();

    // Check for negative deltas (not supported, as in Solidity)
    if amount0 < I256::ZERO {
        return Err(BalanceDeltaValidationError::NegativeDeltaNotAllowed);
    }
    if amount1 < I256::ZERO {
        return Err(BalanceDeltaValidationError::NegativeDeltaNotAllowed);
    }

    // Check minimum outputs
    if amount0 < amount0_min {
        log!(
            DEBUG,
            "[slippage_check]: failed to meet minimum of {:?} token0, with amount0 {:?}",
            amount1_min,
            amount0
        );
        return Err(BalanceDeltaValidationError::MinimumAmountInsufficient {
            required: amount0_min.as_u256(),
            received: amount0.as_u256(),
        });
    }
    if amount1 < amount1_min {
        log!(
            DEBUG,
            "[slippage_check]: failed to meet minimum of {:?} token1, with amount1 {:?}",
            amount1_min,
            amount1
        );

        return Err(BalanceDeltaValidationError::MinimumAmountInsufficient {
            required: amount1_min.as_u256(),
            received: amount1.as_u256(),
        });
    }

    Ok(())
}

/// Validates that the balance delta does not exceed maximum input amounts for liquidity addition.
/// Only checks negative deltas, ignoring positive ones (no slippage check).
/// # Arguments
/// * `delta` - The balance delta containing token0 and token1 amounts
/// * `amount0_max` - The maximum amount of token0 to spend
/// * `amount1_max` - The maximum amount of token1 to spend
/// # Returns
/// * `Result<(), BalanceDeltaValidationError>` - Ok if valid, Err if constraints are exceeded
pub fn validate_max_in(
    delta: BalanceDelta,
    amount0_max: I256,
    amount1_max: I256,
) -> Result<(), BalanceDeltaValidationError> {
    let amount0 = delta.amount0();
    let amount1 = delta.amount1();

    // Only validate negative deltas (positive deltas are ignored, as in Solidity)
    if amount0 < I256::ZERO {
        // Convert -amount0 to U256
        let amount0_abs = -amount0;
        if amount0_max < amount0_abs {
            return Err(BalanceDeltaValidationError::MaximumAmountExceeded {
                maximum: amount0_max.as_u256(),
                received: amount0_abs.as_u256(),
            });
        }
    }

    if amount1 < I256::ZERO {
        // Convert -amount1 to U256
        let amount1_abs = -amount1;
        if amount1_max < amount1_abs {
            return Err(BalanceDeltaValidationError::MaximumAmountExceeded {
                maximum: amount1_max.as_u256(),
                received: amount1_abs.as_u256(),
            });
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_min_out_positive_deltas_sufficient() {
        let delta = BalanceDelta::new(I256::from(1000), I256::from(2000));
        let amount0_min = I256::from(500_u32);
        let amount1_min = I256::from(1500_u32);
        assert_eq!(validate_min_out(delta, amount0_min, amount1_min), Ok(()));
    }

    #[test]
    fn test_validate_min_out_insufficient_amount0() {
        let delta = BalanceDelta::new(I256::from(400), I256::from(2000));
        let amount0_min = I256::from(500_u32);
        let amount1_min = I256::from(1500_u32);
        assert_eq!(
            validate_min_out(delta, amount0_min, amount1_min),
            Err(BalanceDeltaValidationError::MinimumAmountInsufficient {
                required: U256::from(500_u32),
                received: U256::from(400_u32),
            })
        );
    }

    #[test]
    fn test_validate_min_out_insufficient_amount1() {
        let delta = BalanceDelta::new(I256::from(1000_u32), I256::from(1000));
        let amount0_min = I256::from(500_u32);
        let amount1_min = I256::from(1500_u32);
        assert_eq!(
            validate_min_out(delta, amount0_min, amount1_min),
            Err(BalanceDeltaValidationError::MinimumAmountInsufficient {
                required: U256::from(1500_u32),
                received: U256::from(1000_u32),
            })
        );
    }

    #[test]
    fn test_validate_min_out_negative_delta() {
        let delta = BalanceDelta::new(I256::from(-100), I256::from(2000));
        let amount0_min = I256::from(500_u32);
        let amount1_min = I256::from(1500_u32);
        assert_eq!(
            validate_min_out(delta, amount0_min, amount1_min),
            Err(BalanceDeltaValidationError::NegativeDeltaNotAllowed)
        );
    }

    #[test]
    fn test_validate_max_in_negative_deltas_within_limits() {
        let delta = BalanceDelta::new(I256::from(-1000), I256::from(-2000));
        let amount0_max = I256::from(1500_u32);
        let amount1_max = I256::from(2500_u32);
        assert_eq!(validate_max_in(delta, amount0_max, amount1_max), Ok(()));
    }

    #[test]
    fn test_validate_max_in_exceeds_amount0() {
        let delta = BalanceDelta::new(I256::from(-2000), I256::from(-1000));
        let amount0_max = I256::from(1500_u32);
        let amount1_max = I256::from(2500_u32);
        assert_eq!(
            validate_max_in(delta, amount0_max, amount1_max),
            Err(BalanceDeltaValidationError::MaximumAmountExceeded {
                maximum: U256::from(1500_u32),
                received: U256::from(2000_u32),
            })
        );
    }

    #[test]
    fn test_validate_max_in_exceeds_amount1() {
        let delta = BalanceDelta::new(I256::from(-1000), I256::from(-3000));
        let amount0_max = I256::from(1500_u32);
        let amount1_max = I256::from(2500_u32);
        assert_eq!(
            validate_max_in(delta, amount0_max, amount1_max),
            Err(BalanceDeltaValidationError::MaximumAmountExceeded {
                maximum: U256::from(2500_u32),
                received: U256::from(3000_u32),
            })
        );
    }

    #[test]
    fn test_validate_max_in_positive_delta_ignored() {
        let delta = BalanceDelta::new(I256::from(1000), I256::from(-2000));
        let amount0_max = I256::from(500_u32); // Ignored for positive amount0
        let amount1_max = I256::from(2500_u32);
        assert_eq!(validate_max_in(delta, amount0_max, amount1_max), Ok(()));
    }

    #[test]
    fn test_validate_max_in_zero_delta() {
        let delta = BalanceDelta::new(I256::from(0), I256::from(0));
        let amount0_max = I256::from(500_u32);
        let amount1_max = I256::from(500_u32);
        assert_eq!(validate_max_in(delta, amount0_max, amount1_max), Ok(()));
    }
}
