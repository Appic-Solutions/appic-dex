use ethnum::I256;
use std::ops::{Add, Sub};

#[derive(Debug, PartialEq)]
pub enum BalanceDeltaError {
    Overflow,
    Underflow,
}

// Define the BalanceDelta struct with two I256 fields
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BalanceDelta {
    amount0: I256, // Represents the delta for token0
    amount1: I256, // Represents the delta for token1
}

impl BalanceDelta {
    pub const ZERO_DELTA: BalanceDelta = BalanceDelta {
        amount0: I256::ZERO,
        amount1: I256::ZERO,
    };
    pub fn new(amount0: I256, amount1: I256) -> BalanceDelta {
        BalanceDelta { amount0, amount1 }
    }

    pub fn amount0(&self) -> I256 {
        self.amount0
    }

    pub fn amount1(&self) -> I256 {
        self.amount1
    }

    pub fn set_amount0(&mut self, amount: I256) {
        self.amount0 = amount;
    }

    pub fn set_amount1(&mut self, amount: I256) {
        self.amount1 = amount;
    }

    pub fn add_to_self(&mut self, other: BalanceDelta) -> Result<(), BalanceDeltaError> {
        let amount0 = self
            .amount0
            .checked_add(other.amount0)
            .ok_or(BalanceDeltaError::Overflow)?;
        let amount1 = self
            .amount1
            .checked_add(other.amount1)
            .ok_or(BalanceDeltaError::Overflow)?;
        self.amount0 = amount0;
        self.amount1 = amount1;

        Ok(())
    }

    pub fn sub_from_self(&mut self, other: BalanceDelta) -> Result<(), BalanceDeltaError> {
        let amount0 = self
            .amount0
            .checked_sub(other.amount0)
            .ok_or(BalanceDeltaError::Underflow)?;
        let amount1 = self
            .amount1
            .checked_sub(other.amount1)
            .ok_or(BalanceDeltaError::Underflow)?;
        self.amount0 = amount0;
        self.amount1 = amount1;

        Ok(())
    }

    pub fn add(self, other: BalanceDelta) -> Result<BalanceDelta, BalanceDeltaError> {
        let amount0 = self
            .amount0
            .checked_add(other.amount0)
            .ok_or(BalanceDeltaError::Overflow)?;
        let amount1 = self
            .amount1
            .checked_add(other.amount1)
            .ok_or(BalanceDeltaError::Overflow)?;
        Ok(BalanceDelta { amount0, amount1 })
    }

    pub fn sub(self, other: BalanceDelta) -> Result<BalanceDelta, BalanceDeltaError> {
        let amount0 = self
            .amount0
            .checked_sub(other.amount0)
            .ok_or(BalanceDeltaError::Underflow)?;
        let amount1 = self
            .amount1
            .checked_sub(other.amount1)
            .ok_or(BalanceDeltaError::Underflow)?;
        Ok(BalanceDelta { amount0, amount1 })
    }
}

// Operator overloading for convenience
impl Add for BalanceDelta {
    type Output = Result<BalanceDelta, BalanceDeltaError>;

    fn add(self, other: BalanceDelta) -> Self::Output {
        self.add(other)
    }
}

impl Sub for BalanceDelta {
    type Output = Result<BalanceDelta, BalanceDeltaError>;

    fn sub(self, other: BalanceDelta) -> Self::Output {
        self.sub(other)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethnum::I256;

    const I256_MAX: I256 = I256::MAX; // 2^255 - 1
    const I256_MIN: I256 = I256::MIN; // -2^255

    #[test]
    fn test_to_balance_delta() {
        let delta = BalanceDelta::new(I256::from(0), I256::from(0));
        assert_eq!(delta.amount0(), I256::from(0));
        assert_eq!(delta.amount1(), I256::from(0));

        let delta = BalanceDelta::new(I256::from(0), I256::from(1));
        assert_eq!(delta.amount0(), I256::from(0));
        assert_eq!(delta.amount1(), I256::from(1));

        let delta = BalanceDelta::new(I256::from(1), I256::from(0));
        assert_eq!(delta.amount0(), I256::from(1));
        assert_eq!(delta.amount1(), I256::from(0));

        let delta = BalanceDelta::new(I256_MAX, I256_MAX);
        assert_eq!(delta.amount0(), I256_MAX);
        assert_eq!(delta.amount1(), I256_MAX);

        let delta = BalanceDelta::new(I256_MIN, I256_MIN);
        assert_eq!(delta.amount0(), I256_MIN);
        assert_eq!(delta.amount1(), I256_MIN);
    }

    // Skipping test_fuzz_toBalanceDelta as it tests Solidity's bit-packing, not applicable here

    #[test]
    fn test_amount0_amount1() {
        let delta = BalanceDelta::new(I256::from(42), I256::from(-73));
        assert_eq!(delta.amount0(), I256::from(42));
        assert_eq!(delta.amount1(), I256::from(-73));

        let delta = BalanceDelta::new(I256_MAX, I256_MIN);
        assert_eq!(delta.amount0(), I256_MAX);
        assert_eq!(delta.amount1(), I256_MIN);
    }

    #[test]
    fn test_add() {
        let delta = (BalanceDelta::new(I256::from(0), I256::from(0))
            + BalanceDelta::new(I256::from(0), I256::from(0)))
        .unwrap();
        assert_eq!(delta.amount0(), I256::from(0));
        assert_eq!(delta.amount1(), I256::from(0));

        let delta = (BalanceDelta::new(I256::from(-1000), I256::from(1000))
            + BalanceDelta::new(I256::from(1000), I256::from(-1000)))
        .unwrap();
        assert_eq!(delta.amount0(), I256::from(0));
        assert_eq!(delta.amount1(), I256::from(0));

        let delta = (BalanceDelta::new(I256_MIN, I256_MAX) + BalanceDelta::new(I256_MAX, I256_MIN))
            .unwrap();
        assert_eq!(delta.amount0(), I256::from(-1));
        assert_eq!(delta.amount1(), I256::from(-1));

        let half_max = I256_MAX / I256::from(2);
        let delta = (BalanceDelta::new(half_max + I256::from(1), half_max + I256::from(1))
            + BalanceDelta::new(half_max, half_max))
        .unwrap();
        assert_eq!(delta.amount0(), I256_MAX);
        assert_eq!(delta.amount1(), I256_MAX);
    }

    #[test]
    fn test_add_reverts_on_overflow() {
        let result = BalanceDelta::new(I256_MAX, I256::from(0))
            + BalanceDelta::new(I256::from(1), I256::from(0));
        assert!(matches!(result, Err(BalanceDeltaError::Overflow)));

        let result = BalanceDelta::new(I256::from(0), I256_MAX)
            + BalanceDelta::new(I256::from(0), I256::from(1));
        assert!(matches!(result, Err(BalanceDeltaError::Overflow)));
    }

    #[test]
    fn test_fuzz_add() {
        let cases = [
            (0, 0, 0, 0),
            (-1000, 1000, 1000, -1000),
            (I256_MAX.0[0] / 2, I256_MIN.0[0] / 2, 1, -1), // Using lower 64 bits for simplicity
        ];

        for (a, b, c, d) in cases {
            let a = I256::from(a);
            let b = I256::from(b);
            let c = I256::from(c);
            let d = I256::from(d);
            let result = BalanceDelta::new(a, b) + BalanceDelta::new(c, d);

            match (a.checked_add(c), b.checked_add(d)) {
                (Some(ac), Some(bd)) => {
                    let delta = result.unwrap();
                    assert_eq!(delta.amount0(), ac);
                    assert_eq!(delta.amount1(), bd);
                }
                _ => assert!(matches!(result, Err(BalanceDeltaError::Overflow))),
            }
        }
    }

    #[test]
    fn test_sub() {
        let delta = (BalanceDelta::new(I256::from(0), I256::from(0))
            - BalanceDelta::new(I256::from(0), I256::from(0)))
        .unwrap();
        assert_eq!(delta.amount0(), I256::from(0));
        assert_eq!(delta.amount1(), I256::from(0));

        let delta = (BalanceDelta::new(I256::from(-1000), I256::from(1000))
            - BalanceDelta::new(I256::from(1000), I256::from(-1000)))
        .unwrap();
        assert_eq!(delta.amount0(), I256::from(-2000));
        assert_eq!(delta.amount1(), I256::from(2000));

        let delta = (BalanceDelta::new(I256::from(-1000), I256::from(-1000))
            - BalanceDelta::new(I256::from(1000), I256::from(1000)))
        .unwrap();
        assert_eq!(delta.amount0(), I256::from(-2000));
        assert_eq!(delta.amount1(), I256::from(-2000));

        let half_min = I256_MIN / I256::from(2);
        let delta = (BalanceDelta::new(half_min, half_min)
            - BalanceDelta::new(-half_min, -half_min))
        .unwrap();
        assert_eq!(delta.amount0(), I256_MIN);
        assert_eq!(delta.amount1(), I256_MIN);
    }

    #[test]
    fn test_sub_reverts_on_underflow() {
        let result = BalanceDelta::new(I256_MIN, I256::from(0))
            - BalanceDelta::new(I256::from(1), I256::from(0));
        assert!(matches!(result, Err(BalanceDeltaError::Underflow)));

        let result = BalanceDelta::new(I256::from(0), I256_MIN)
            - BalanceDelta::new(I256::from(0), I256::from(1));
        assert!(matches!(result, Err(BalanceDeltaError::Underflow)));
    }

    #[test]
    fn test_fuzz_sub() {
        let cases = [
            (0, 0, 0, 0),
            (-1000, 1000, 1000, -1000),
            (I256_MIN.0[0] / 2, I256_MAX.0[0] / 2, -1, 1), // Using lower 64 bits for simplicity
        ];

        for (a, b, c, d) in cases {
            let a = I256::from(a);
            let b = I256::from(b);
            let c = I256::from(c);
            let d = I256::from(d);
            let result = BalanceDelta::new(a, b) - BalanceDelta::new(c, d);

            match (a.checked_sub(c), b.checked_sub(d)) {
                (Some(ac), Some(bd)) => {
                    let delta = result.unwrap();
                    assert_eq!(delta.amount0(), ac);
                    assert_eq!(delta.amount1(), bd);
                }
                _ => assert!(matches!(result, Err(BalanceDeltaError::Underflow))),
            }
        }
    }

    #[test]
    fn test_fuzz_eq() {
        let cases = [
            (0, 0, 0, 0),
            (100, 200, 100, 200),
            (100, 200, 100, 300),
            (I256_MAX.0[0], I256_MIN.0[0], I256_MAX.0[0], I256_MIN.0[0]), // Using lower 64 bits
        ];

        for (a, b, c, d) in cases {
            let delta1 = BalanceDelta::new(I256::from(a), I256::from(b));
            let delta2 = BalanceDelta::new(I256::from(c), I256::from(d));
            if a == c && b == d {
                assert_eq!(delta1, delta2);
            } else {
                assert_ne!(delta1, delta2);
            }
        }
    }

    #[test]
    fn test_fuzz_neq() {
        let cases = [
            (0, 0, 0, 0),
            (100, 200, 100, 200),
            (100, 200, 100, 300),
            (I256_MAX.0[0], I256_MIN.0[0], I256_MIN.0[0], I256_MAX.0[0]), // Using lower 64 bits
        ];

        for (a, b, c, d) in cases {
            let delta1 = BalanceDelta::new(I256::from(a), I256::from(b));
            let delta2 = BalanceDelta::new(I256::from(c), I256::from(d));
            if a != c || b != d {
                assert_ne!(delta1, delta2);
            } else {
                assert_eq!(delta1, delta2);
            }
        }
    }
}
