use lazy_static::lazy_static;
use num_bigint::{BigInt, BigUint};
use num_traits::{FromPrimitive, One, ToPrimitive, Zero};

/// Math library for computing sqrt prices from ticks and vice versa
/// notice Computes sqrt price for ticks of size 1.0001, i.e. sqrt(1.0001^tick) as fixed point Q64.96 numbers. Supports
/// prices between 2.pow-128 and 2.pow128
pub struct TickMath;

// Precomputed constants to avoid runtime parsing
lazy_static! {
    static ref MIN_SQRT_RATIO: BigUint = BigUint::parse_bytes(b"4295128739", 10).unwrap();
    static ref MAX_SQRT_RATIO: BigUint =
        BigUint::parse_bytes(b"1461446703485210103287273052203988822378723970342", 10).unwrap();
    static ref TWO_POW_32: BigUint = BigUint::one() << 32;
    static ref TWO_POW_128: BigUint = BigUint::one() << 128;
    static ref TWO_POW_256_MINUS_1: BigUint = BigUint::from_bytes_be(&[0xff; 32]);
    static ref CONSTANTS: [BigUint; 19] = [
        BigUint::parse_bytes(b"fff97272373d413259a46990580e213a", 16).unwrap(),
        BigUint::parse_bytes(b"fff2e50f5f656932ef12357cf3c7fdcc", 16).unwrap(),
        BigUint::parse_bytes(b"ffe5caca7e10e4e61c3624eaa0941cd0", 16).unwrap(),
        BigUint::parse_bytes(b"ffcb9843d60f6159c9db58835c926644", 16).unwrap(),
        BigUint::parse_bytes(b"ff973b41fa98c081472e6896dfb254c0", 16).unwrap(),
        BigUint::parse_bytes(b"ff2ea16466c96a3843ec78b326b52861", 16).unwrap(),
        BigUint::parse_bytes(b"fe5dee046a99a2a811c461f1969c3053", 16).unwrap(),
        BigUint::parse_bytes(b"fcbe86c7900a88aedcffc83b479aa3a4", 16).unwrap(),
        BigUint::parse_bytes(b"f987a7253ac413176f2b074cf7815e54", 16).unwrap(),
        BigUint::parse_bytes(b"f3392b0822b70005940c7a398e4b70f3", 16).unwrap(),
        BigUint::parse_bytes(b"e7159475a2c29b7443b29c7fa6e889d9", 16).unwrap(),
        BigUint::parse_bytes(b"d097f3bdfd2022b8845ad8f792aa5825", 16).unwrap(),
        BigUint::parse_bytes(b"a9f746462d870fdf8a65dc1f90e061e5", 16).unwrap(),
        BigUint::parse_bytes(b"70d869a156d2a1b890bb3df62baf32f7", 16).unwrap(),
        BigUint::parse_bytes(b"31be135f97d08fd981231505542fcfa6", 16).unwrap(),
        BigUint::parse_bytes(b"9aa508b5b7a84e1c677de54f3e99bc9", 16).unwrap(),
        BigUint::parse_bytes(b"5d6af8dedb81196699c329225ee604", 16).unwrap(),
        BigUint::parse_bytes(b"2216e584f5fa1ea926041bedfe98", 16).unwrap(),
        BigUint::parse_bytes(b"48a170391f7dc42444e8fa2", 16).unwrap(),
    ];
    static ref MSB_THRESHOLDS: [(BigUint, u32); 8] = [
        (BigUint::from_bytes_be(&[0xFF; 16]), 128),
        (BigUint::from_bytes_be(&[0xFF; 8]), 64),
        (BigUint::from_bytes_be(&[0xFF; 4]), 32),
        (BigUint::from_bytes_be(&[0xFF; 2]), 16),
        (BigUint::from_bytes_be(&[0xFF]), 8),
        (BigUint::from_u8(0xF).unwrap(), 4),
        (BigUint::from_u8(0x3).unwrap(), 2),
        (BigUint::from_u8(0x1).unwrap(), 1),
    ];
    static ref LOG_2_COEFF: BigInt = BigInt::parse_bytes(b"255738958999603826347141", 10).unwrap();
    static ref TICK_LOW_OFFSET: BigInt =
        BigInt::parse_bytes(b"3402992956809132418596140100660247210", 10).unwrap();
    static ref TICK_HI_OFFSET: BigInt =
        BigInt::parse_bytes(b"291339464771989622907027621153398088495", 10).unwrap();
}

// Custom error type for TickMath
#[derive(Debug, PartialEq)]
pub enum TickMathError {
    TickOutOfBounds,
    SqrtPriceOutOfBounds,
    ArithmeticOverflow,
}

impl TickMath {
    const MIN_SQRT_RATIO: &'static [u8] = b"4295128739"; // Decimal string
    const MAX_SQRT_RATIO: &'static [u8] = b"1461446703485210103287273052203988822378723970342"; // Decimal string

    const MIN_TICK: i32 = -887272;
    const MAX_TICK: i32 = 887272;

    /// Calculates sqrt(1.0001^tick) * 2^96.
    /// Returns a Q64.96 fixed-point number representing the sqrt of the price ratio.
    pub fn get_sqrt_ratio_at_tick(tick: i32) -> Result<BigUint, TickMathError> {
        if tick < Self::MIN_TICK || tick > Self::MAX_TICK {
            return Err(TickMathError::TickOutOfBounds);
        }

        let abs_tick = tick.unsigned_abs();
        let mut ratio = if abs_tick & 0x1 != 0 {
            BigUint::parse_bytes(b"fffcb933bd6fad37aa2d162d1a594001", 16).unwrap()
        } else {
            TWO_POW_128.clone()
        };

        // Use bitmask to apply constants efficiently
        for (i, constant) in CONSTANTS.iter().enumerate() {
            if abs_tick & (1 << (i + 1)) != 0 {
                ratio = (ratio * constant) >> 128;
            }
        }

        if tick > 0 {
            ratio = TWO_POW_256_MINUS_1.clone() / ratio;
        }

        // Convert to Q64.96 with rounding up
        let sqrt_price_x96 = (&ratio >> 32)
            + if ratio % (BigUint::one() << 32) == BigUint::zero() {
                BigUint::zero()
            } else {
                BigUint::one()
            };
        Ok(sqrt_price_x96)
    }

    /// Computes the tick corresponding to a given sqrtPriceX96.
    pub fn get_tick_at_sqrt_ratio(sqrt_price_x96: BigUint) -> Result<i32, TickMathError> {
        if sqrt_price_x96 < *MIN_SQRT_RATIO || sqrt_price_x96 >= *MAX_SQRT_RATIO {
            return Err(TickMathError::SqrtPriceOutOfBounds);
        }

        let ratio = &sqrt_price_x96 << 32;
        let msb = Self::compute_msb_fast(&ratio);
        let r = if msb >= 128 {
            ratio >> (msb - 127)
        } else {
            ratio << (127 - msb)
        };

        let log_2 = Self::compute_log_2(r, msb)?;
        let log_sqrt10001 = log_2 * &*LOG_2_COEFF;

        let tick_low = ((&log_sqrt10001 - &*TICK_LOW_OFFSET) >> 128_u8)
            .to_i32()
            .ok_or(TickMathError::ArithmeticOverflow)?;
        let tick_hi = ((&log_sqrt10001 + &*TICK_HI_OFFSET) >> 128_u8)
            .to_i32()
            .ok_or(TickMathError::ArithmeticOverflow)?;

        Ok(if tick_low == tick_hi {
            tick_low
        } else {
            let sqrt_ratio_at_tick_hi = Self::get_sqrt_ratio_at_tick(tick_hi)?;
            if sqrt_ratio_at_tick_hi <= sqrt_price_x96 {
                tick_hi
            } else {
                tick_low
            }
        })
    }

    /// Fast MSB computation using binary search over precomputed thresholds.
    fn compute_msb_fast(value: &BigUint) -> u32 {
        let mut msb = 0;
        let mut r = value.clone();

        for &(ref threshold, bit) in MSB_THRESHOLDS.iter() {
            if r > *threshold {
                msb |= bit;
                r >>= bit;
            }
        }

        msb
    }

    // Helper to compute log_2 with binary fraction
    fn compute_log_2(mut r: BigUint, msb: u32) -> Result<BigInt, TickMathError> {
        let mut log_2 = BigInt::from(msb as i32 - 128) << 64;

        for shift in (50..=63).rev() {
            r = (&r * &r) >> 127;
            let f: BigUint = &r >> 128;
            log_2 |= BigInt::from_biguint(num_bigint::Sign::Plus, f.clone()) << shift;
            r >>= f.to_u32().ok_or(TickMathError::ArithmeticOverflow)?;
        }

        Ok(log_2)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_large_ticks() {
        assert!(TickMath::get_sqrt_ratio_at_tick(10000).is_ok());
        assert!(TickMath::get_sqrt_ratio_at_tick(-10000).is_ok());
    }

    #[test]
    fn test_between_ticks() {
        let tick_1 = TickMath::get_sqrt_ratio_at_tick(1).unwrap();
        let tick_2 = TickMath::get_sqrt_ratio_at_tick(2).unwrap();
        let mid = (&tick_1 + &tick_2) / 2u32;
        let tick = TickMath::get_tick_at_sqrt_ratio(mid).unwrap();
        assert_eq!(tick, 1); // Should select greatest tick <= mid
    }

    #[test]
    fn test_near_max_tick() {
        let tick = TickMath::MAX_TICK - 10;
        let sqrt_price = TickMath::get_sqrt_ratio_at_tick(tick).unwrap();
        assert_eq!(TickMath::get_tick_at_sqrt_ratio(sqrt_price).unwrap(), tick);
    }

    #[test]
    fn test_get_sqrt_ratio_at_tick() {
        let two_pow_96 = BigUint::one() << 96;

        // Tick 0
        assert_eq!(
            TickMath::get_sqrt_ratio_at_tick(0).unwrap(),
            two_pow_96,
            "Tick 0 should be 2^96"
        );

        // Tick 1
        let expected_tick_1 = BigUint::parse_bytes(b"79232123823359799118286999568", 10).unwrap();
        let tick_1 = TickMath::get_sqrt_ratio_at_tick(1).unwrap();
        assert!(tick_1 > two_pow_96, "Tick 1 should be > 2^96");
        assert_eq!(tick_1, expected_tick_1);

        // Max Tick - 1
        let expected_max_tick_minus_one =
            BigUint::parse_bytes(b"1461373636630004318706518188784493106690254656249", 10).unwrap();
        let max_tick_minus_one = TickMath::get_sqrt_ratio_at_tick(TickMath::MAX_TICK - 1).unwrap();
        assert!(max_tick_minus_one > two_pow_96, "Tick -1 should be < 2^96");
        assert_eq!(max_tick_minus_one, expected_max_tick_minus_one);

        // Min Tick + 1
        let expected_min_tick_plus_one = BigUint::parse_bytes(b"4295343490", 10).unwrap();
        let min_tick_plus_one = TickMath::get_sqrt_ratio_at_tick(TickMath::MIN_TICK + 1).unwrap();
        assert!(min_tick_plus_one < two_pow_96);
        assert_eq!(min_tick_plus_one, expected_min_tick_plus_one);

        // MIN_TICK
        assert_eq!(
            TickMath::get_sqrt_ratio_at_tick(TickMath::MIN_TICK).unwrap(),
            BigUint::parse_bytes(TickMath::MIN_SQRT_RATIO, 10).unwrap()
        );

        // MAX_TICK
        assert_eq!(
            TickMath::get_sqrt_ratio_at_tick(TickMath::MAX_TICK).unwrap(),
            BigUint::parse_bytes(TickMath::MAX_SQRT_RATIO, 10).unwrap()
        );

        // Out of bounds
        assert!(TickMath::get_sqrt_ratio_at_tick(TickMath::MAX_TICK + 1).is_err());
        assert!(TickMath::get_sqrt_ratio_at_tick(TickMath::MIN_TICK - 1).is_err());
    }

    #[test]
    fn test_get_sqrt_ratio_at_tick_accuracy() {
        const ABS_TICKS: [u32; 14] = [
            50, 100, 250, 500, 1000, 2500, 3000, 4000, 5000, 50000, 150000, 250000, 500000, 738203,
        ];

        for &abs_tick in &ABS_TICKS {
            for &tick in &[abs_tick as i32, -(abs_tick as i32)] {
                let precise_sqrt_ratio = precise_sqrt_ratio_at_tick(tick);
                let calculated_sqrt_ratio =
                    TickMath::get_sqrt_ratio_at_tick(tick).expect("Failed to compute sqrt ratio");
                let abs_diff = (precise_sqrt_ratio
                    - calculated_sqrt_ratio.to_f64().expect("Failed to convert"))
                .abs();
                let rel_diff = abs_diff / precise_sqrt_ratio;
                assert!(
                    rel_diff < 0.000001,
                    "Tick {}: relative difference too large: {}",
                    tick,
                    rel_diff
                );
            }
        }
    }

    #[test]
    fn test_get_tick_at_sqrt_ratio() {
        let two_pow_96: BigUint = BigUint::one() << 96;

        // 2^96 -> Tick 0
        assert_eq!(
            TickMath::get_tick_at_sqrt_ratio(two_pow_96.clone()).unwrap(),
            0,
            "2^96 should map to tick 0"
        );

        // MIN_SQRT_RATIO -> MIN_TICK
        assert_eq!(
            TickMath::get_tick_at_sqrt_ratio(
                BigUint::parse_bytes(TickMath::MIN_SQRT_RATIO, 10).unwrap()
            )
            .unwrap(),
            TickMath::MIN_TICK
        );

        let min_tick_plus_one = BigUint::parse_bytes(b"4295343490", 10).unwrap();
        assert_eq!(
            TickMath::get_tick_at_sqrt_ratio(min_tick_plus_one).unwrap(),
            TickMath::MIN_TICK + 1
        );

        // closest to MAX_SQRT_RATIO -> MAX_TICK
        let max_minus_one =
            BigUint::parse_bytes(TickMath::MAX_SQRT_RATIO, 10).unwrap() - BigUint::one();
        assert_eq!(
            TickMath::get_tick_at_sqrt_ratio(max_minus_one).unwrap(),
            TickMath::MAX_TICK - 1
        );

        // Max tick - 1
        let max_tick_minus_one_sqrt_price =
            BigUint::parse_bytes(b"1461373636630004318706518188784493106690254656249", 10).unwrap();
        assert_eq!(
            TickMath::get_tick_at_sqrt_ratio(max_tick_minus_one_sqrt_price).unwrap(),
            TickMath::MAX_TICK - 1
        );

        // Out of bounds
        assert!(TickMath::get_tick_at_sqrt_ratio(
            BigUint::parse_bytes(TickMath::MIN_SQRT_RATIO, 10).unwrap() - BigUint::one()
        )
        .is_err());
        assert!(TickMath::get_tick_at_sqrt_ratio(
            BigUint::parse_bytes(TickMath::MAX_SQRT_RATIO, 10).unwrap()
        )
        .is_err());
    }

    #[test]
    fn test_round_trip() {
        let ticks = [
            0,
            1,
            -1,
            295,
            -295,
            TickMath::MIN_TICK,
            TickMath::MAX_TICK - 1,
        ];
        for &tick in ticks.iter() {
            let sqrt_price = TickMath::get_sqrt_ratio_at_tick(tick).unwrap();
            let computed_tick = TickMath::get_tick_at_sqrt_ratio(sqrt_price.clone()).unwrap();
            assert!(
                computed_tick == tick || computed_tick == tick - 1,
                "Round trip failed for tick {}: got {}",
                tick,
                computed_tick
            );
            assert!(
                TickMath::get_sqrt_ratio_at_tick(computed_tick).unwrap() <= sqrt_price,
                "Computed tick ratio exceeds original"
            );
        }
    }

    #[test]
    fn test_get_tick_at_sqrt_ratio_accuracy() {
        let ratios = vec![
            (BigUint::parse_bytes(TickMath::MIN_SQRT_RATIO, 10).unwrap()),
            (BigUint::parse_bytes(b"42951287390", 10).unwrap()), // Rough approximation
            (BigUint::parse_bytes(b"42950927989", 10).unwrap()),
            (BigUint::parse_bytes(b"792281625142643375935439", 10).unwrap()),
            (BigUint::parse_bytes(b"112045541949572279837463876301", 10).unwrap()),
            (BigUint::parse_bytes(b"214762854672513427421562245760", 10).unwrap()),
            (BigUint::parse_bytes(b"429525703431838627161847955968", 10).unwrap()),
            (BigUint::parse_bytes(TickMath::MAX_SQRT_RATIO, 10).unwrap() - BigUint::one()),
        ];

        for ratio in ratios {
            println!("{}", ratio);
            test_ratio(ratio);
        }
    }

    fn test_ratio(ratio: BigUint) {
        let tick = TickMath::get_tick_at_sqrt_ratio(ratio.clone()).unwrap();
        let ratio_of_tick = TickMath::get_sqrt_ratio_at_tick(tick).unwrap();
        let ratio_of_tick_plus_one = TickMath::get_sqrt_ratio_at_tick(tick + 1).unwrap();
        assert!(
            ratio >= ratio_of_tick,
            "Ratio {} < tick ratio {}",
            ratio,
            ratio_of_tick
        );
        assert!(
            ratio < ratio_of_tick_plus_one,
            "Ratio {} >= tick+1 ratio {}",
            ratio,
            ratio_of_tick_plus_one
        );
    }

    fn precise_sqrt_ratio_at_tick(tick: i32) -> f64 {
        let one_point_0001 = 1.0001_f64;
        let tick_float = tick;
        let price = one_point_0001.powi(tick_float);
        let sqrt_price = price.sqrt();
        let two_pow_96 = 2_u128.pow(96);
        sqrt_price * two_pow_96.to_f64().expect("Failed to convert")
    }
}
