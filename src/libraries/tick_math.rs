use ethnum::{I256, U256};
use lazy_static::lazy_static;

use super::constants::{MAX_SQRT_RATIO, MAX_TICK, MIN_SQRT_RATIO, MIN_TICK};

pub struct TickMath;

// Precomputed constants using U256
lazy_static! {
    static ref TWO_POW_32: U256 = U256::from(1_u8) << 32;
    static ref TWO_POW_128: U256 = U256::from(1_u8) << 128;
    static ref TWO_POW_256_MINUS_1: U256 = U256::from_be_bytes([0xff; 32]);
    static ref CONSTANTS: [U256; 19] = [
        U256::from_str_radix("fff97272373d413259a46990580e213a", 16).unwrap(),
        U256::from_str_radix("fff2e50f5f656932ef12357cf3c7fdcc", 16).unwrap(),
        U256::from_str_radix("ffe5caca7e10e4e61c3624eaa0941cd0", 16).unwrap(),
        U256::from_str_radix("ffcb9843d60f6159c9db58835c926644", 16).unwrap(),
        U256::from_str_radix("ff973b41fa98c081472e6896dfb254c0", 16).unwrap(),
        U256::from_str_radix("ff2ea16466c96a3843ec78b326b52861", 16).unwrap(),
        U256::from_str_radix("fe5dee046a99a2a811c461f1969c3053", 16).unwrap(),
        U256::from_str_radix("fcbe86c7900a88aedcffc83b479aa3a4", 16).unwrap(),
        U256::from_str_radix("f987a7253ac413176f2b074cf7815e54", 16).unwrap(),
        U256::from_str_radix("f3392b0822b70005940c7a398e4b70f3", 16).unwrap(),
        U256::from_str_radix("e7159475a2c29b7443b29c7fa6e889d9", 16).unwrap(),
        U256::from_str_radix("d097f3bdfd2022b8845ad8f792aa5825", 16).unwrap(),
        U256::from_str_radix("a9f746462d870fdf8a65dc1f90e061e5", 16).unwrap(),
        U256::from_str_radix("70d869a156d2a1b890bb3df62baf32f7", 16).unwrap(),
        U256::from_str_radix("31be135f97d08fd981231505542fcfa6", 16).unwrap(),
        U256::from_str_radix("9aa508b5b7a84e1c677de54f3e99bc9", 16).unwrap(),
        U256::from_str_radix("5d6af8dedb81196699c329225ee604", 16).unwrap(),
        U256::from_str_radix("2216e584f5fa1ea926041bedfe98", 16).unwrap(),
        U256::from_str_radix("48a170391f7dc42444e8fa2", 16).unwrap(),
    ];
    static ref MSB_THRESHOLDS: [(U256, u32); 8] = [
        (U256::from_str_radix("ffffffffffffffffffffffffffffffff", 16).unwrap(), 128), // 2^128 - 1
        (U256::from_str_radix("ffffffffffffffff", 16).unwrap(), 64),         // 2^64 - 1
        (U256::from_str_radix("ffffffff", 16).unwrap(), 32),               // 2^32 - 1
        (U256::from_str_radix("ffff", 16).unwrap(), 16),                   // 2^16 - 1
        (U256::from_str_radix("ff", 16).unwrap(), 8),                      // 2^8 - 1
        (U256::from_str_radix("f", 16).unwrap(), 4),                       // 2^4 - 1
        (U256::from_str_radix("3", 16).unwrap(), 2),                       // 2^2 - 1
        (U256::from_str_radix("1", 16).unwrap(), 1),                       // 2^1 - 1
    ];
    static ref LOG_2_COEFF: I256 = I256::from_str_radix("255738958999603826347141", 10).unwrap();
    static ref TICK_LOW_OFFSET: I256 =
        I256::from_str_radix("3402992956809132418596140100660247210", 10).unwrap();
    static ref TICK_HI_OFFSET: I256 =
        I256::from_str_radix("291339464771989622907027621153398088495", 10).unwrap();
}

// Custom error type
#[derive(Debug, PartialEq)]
pub enum TickMathError {
    TickOutOfBounds,
    SqrtPriceOutOfBounds,
}

impl TickMath {
    /// Calculates sqrt(1.0001^tick) * 2^96 as a Q64.96 number (returns U256).
    pub fn get_sqrt_ratio_at_tick(tick: i32) -> U256 {
        if tick < MIN_TICK || tick > MAX_TICK {
            panic!("Bug: TickOutOfBounds")
        }

        let abs_tick = tick.unsigned_abs();
        let mut ratio = if abs_tick & 0x1 != 0 {
            U256::from_str_radix("fffcb933bd6fad37aa2d162d1a594001", 16).unwrap()
        } else {
            *TWO_POW_128
        };

        for (i, constant) in CONSTANTS.iter().enumerate() {
            if abs_tick & (1 << (i + 1)) != 0 {
                ratio = (ratio * constant) >> 128;
            }
        }

        if tick > 0 {
            ratio = *TWO_POW_256_MINUS_1 / ratio;
        }

        let sqrt_price_x96 = (ratio >> 32)
            + if ratio % *TWO_POW_32 == U256::ZERO {
                U256::ZERO
            } else {
                U256::ONE
            };
        sqrt_price_x96
    }

    /// Computes the tick corresponding to a given sqrtPriceX96 (U256).
    pub fn get_tick_at_sqrt_ratio(sqrt_price_x96: U256) -> i32 {
        if sqrt_price_x96 < *MIN_SQRT_RATIO || sqrt_price_x96 >= *MAX_SQRT_RATIO {
            panic!("Bug: SqrtPriceOutOfBounds");
        }

        let ratio = sqrt_price_x96 << 32;
        let msb = Self::compute_msb_fast(&ratio);
        let r = if msb >= 128 {
            ratio >> (msb - 127)
        } else {
            ratio << (127 - msb)
        };

        let log_2 = Self::compute_log_2(r, msb);
        let log_sqrt10001 = log_2 * *LOG_2_COEFF;

        let tick_low = ((log_sqrt10001 - *TICK_LOW_OFFSET) >> 128_u8).as_i32();
        let tick_hi = ((log_sqrt10001 + *TICK_HI_OFFSET) >> 128_u8).as_i32();

        if tick_low == tick_hi {
            tick_low
        } else {
            let sqrt_ratio_at_tick_hi = Self::get_sqrt_ratio_at_tick(tick_hi);
            if sqrt_ratio_at_tick_hi <= sqrt_price_x96 {
                tick_hi
            } else {
                tick_low
            }
        }
    }

    fn compute_msb_fast(value: &U256) -> u32 {
        let mut msb = 0;
        let mut r = *value;

        for &(threshold, bit) in MSB_THRESHOLDS.iter() {
            if r > threshold {
                msb |= bit;
                r >>= bit;
            }
        }
        msb
    }

    fn compute_log_2(mut r: U256, msb: u32) -> I256 {
        let mut log_2 = I256::from(msb as i32 - 128) << 64;

        for shift in (50..=63).rev() {
            r = (r * r) >> 127;
            let f: U256 = r >> 128;
            let f_u32 = f.as_u32();
            log_2 |= I256::from(f_u32) << shift;
            r >>= f_u32; // Use the u32 value here too for consistency
        }
        log_2
    }
}

#[cfg(test)]
mod tests {

    use num_traits::ToPrimitive;

    use super::*;

    #[test]
    fn test_large_ticks() {
        assert_eq!(
            TickMath::get_sqrt_ratio_at_tick(10000),
            U256::from_str_radix("130621891405341611593710811006", 10).unwrap()
        );
        assert_eq!(
            TickMath::get_sqrt_ratio_at_tick(-10000),
            U256::from_str_radix("48055510970269007215549348797", 10).unwrap()
        );
    }

    #[test]
    fn test_between_ticks() {
        let tick_1 = TickMath::get_sqrt_ratio_at_tick(1);
        let tick_2 = TickMath::get_sqrt_ratio_at_tick(2);
        let mid = (&tick_1 + &tick_2) / 2u128;
        let tick = TickMath::get_tick_at_sqrt_ratio(mid);
        assert_eq!(tick, 1); // Should select greatest tick <= mid
    }

    #[test]
    fn test_near_max_tick() {
        let tick = MAX_TICK - 10;
        let sqrt_price = TickMath::get_sqrt_ratio_at_tick(tick);
        assert_eq!(TickMath::get_tick_at_sqrt_ratio(sqrt_price), tick);
    }

    #[test]
    fn test_get_sqrt_ratio_at_tick() {
        let two_pow_96 = U256::ONE << 96;

        // Tick 0
        assert_eq!(
            TickMath::get_sqrt_ratio_at_tick(0),
            two_pow_96,
            "Tick 0 should be 2^96"
        );

        // Tick 1
        let expected_tick_1 = U256::from_str_radix("79232123823359799118286999568", 10).unwrap();
        let tick_1 = TickMath::get_sqrt_ratio_at_tick(1);
        assert!(tick_1 > two_pow_96, "Tick 1 should be > 2^96");
        assert_eq!(tick_1, expected_tick_1);

        // Max Tick - 1
        let expected_max_tick_minus_one =
            U256::from_str_radix("1461373636630004318706518188784493106690254656249", 10).unwrap();
        let max_tick_minus_one = TickMath::get_sqrt_ratio_at_tick(MAX_TICK - 1);
        assert!(max_tick_minus_one > two_pow_96, "Tick -1 should be < 2^96");
        assert_eq!(max_tick_minus_one, expected_max_tick_minus_one);

        // Min Tick + 1
        let expected_min_tick_plus_one = U256::from_str_radix("4295343490", 10).unwrap();
        let min_tick_plus_one = TickMath::get_sqrt_ratio_at_tick(MIN_TICK + 1);
        assert!(min_tick_plus_one < two_pow_96);
        assert_eq!(min_tick_plus_one, expected_min_tick_plus_one);

        // MIN_TICK
        assert_eq!(TickMath::get_sqrt_ratio_at_tick(MIN_TICK), *MIN_SQRT_RATIO);

        // MAX_TICK
        assert_eq!(TickMath::get_sqrt_ratio_at_tick(MAX_TICK), *MAX_SQRT_RATIO);
    }

    #[test]
    #[should_panic]
    fn get_sqrt_ratio_above_max_tick_should_panic() {
        TickMath::get_sqrt_ratio_at_tick(MAX_TICK + 1);
    }

    #[test]
    #[should_panic]
    fn get_sqrt_ratio_below_min_tick_should_panic() {
        TickMath::get_sqrt_ratio_at_tick(MIN_TICK - 1);
    }

    #[test]
    #[should_panic]
    fn get_tick_below_min_sqrt_should_panic() {
        TickMath::get_tick_at_sqrt_ratio(*MIN_SQRT_RATIO - U256::ONE);
    }

    #[test]
    #[should_panic]
    fn get_tick_above_max_sqrt_should_panic() {
        TickMath::get_tick_at_sqrt_ratio(*MAX_SQRT_RATIO);
    }

    #[test]
    fn test_get_sqrt_ratio_at_tick_accuracy() {
        const ABS_TICKS: [u32; 14] = [
            50, 100, 250, 500, 1000, 2500, 3000, 4000, 5000, 50000, 150000, 250000, 500000, 738203,
        ];

        for &abs_tick in &ABS_TICKS {
            for &tick in &[abs_tick as i32, -(abs_tick as i32)] {
                let precise_sqrt_ratio = precise_sqrt_ratio_at_tick(tick);
                let calculated_sqrt_ratio = TickMath::get_sqrt_ratio_at_tick(tick);
                let abs_diff = (precise_sqrt_ratio - calculated_sqrt_ratio.as_f64()).abs();
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
        let two_pow_96: U256 = U256::ONE << 96;

        // 2^96 -> Tick 0
        assert_eq!(
            TickMath::get_tick_at_sqrt_ratio(two_pow_96.clone()),
            0,
            "2^96 should map to tick 0"
        );

        // MIN_SQRT_RATIO -> MIN_TICK
        assert_eq!(TickMath::get_tick_at_sqrt_ratio(*MIN_SQRT_RATIO), MIN_TICK);

        let min_tick_plus_one = U256::from_str_radix("4295343490", 10).unwrap();
        assert_eq!(
            TickMath::get_tick_at_sqrt_ratio(min_tick_plus_one),
            MIN_TICK + 1
        );

        // closest to MAX_SQRT_RATIO -> MAX_TICK
        let max_minus_one = *MAX_SQRT_RATIO - U256::ONE;
        assert_eq!(
            TickMath::get_tick_at_sqrt_ratio(max_minus_one),
            MAX_TICK - 1
        );

        // Max tick - 1
        let max_tick_minus_one_sqrt_price =
            U256::from_str_radix("1461373636630004318706518188784493106690254656249", 10).unwrap();
        assert_eq!(
            TickMath::get_tick_at_sqrt_ratio(max_tick_minus_one_sqrt_price),
            MAX_TICK - 1
        );
    }

    #[test]
    fn test_round_trip() {
        let ticks = [0, 1, -1, 295, -295, MIN_TICK, MAX_TICK - 1];
        for &tick in ticks.iter() {
            let sqrt_price = TickMath::get_sqrt_ratio_at_tick(tick);
            let computed_tick = TickMath::get_tick_at_sqrt_ratio(sqrt_price.clone());
            assert!(
                computed_tick == tick || computed_tick == tick - 1,
                "Round trip failed for tick {}: got {}",
                tick,
                computed_tick
            );
            assert!(
                TickMath::get_sqrt_ratio_at_tick(computed_tick) <= sqrt_price,
                "Computed tick ratio exceeds original"
            );
        }
    }

    #[test]
    fn test_get_tick_at_sqrt_ratio_accuracy() {
        let ratios = vec![
            *MIN_SQRT_RATIO,
            (U256::from_str_radix("42951287390", 10).unwrap()), // Rough approximation
            (U256::from_str_radix("42950927989", 10).unwrap()),
            (U256::from_str_radix("792281625142643375935439", 10).unwrap()),
            (U256::from_str_radix("112045541949572279837463876301", 10).unwrap()),
            (U256::from_str_radix("214762854672513427421562245760", 10).unwrap()),
            (U256::from_str_radix("429525703431838627161847955968", 10).unwrap()),
            (*MAX_SQRT_RATIO - U256::ONE),
        ];

        for ratio in ratios {
            println!("{}", ratio);
            test_ratio(ratio);
        }
    }

    fn test_ratio(ratio: U256) {
        let tick = TickMath::get_tick_at_sqrt_ratio(ratio.clone());
        let ratio_of_tick = TickMath::get_sqrt_ratio_at_tick(tick);
        let ratio_of_tick_plus_one = TickMath::get_sqrt_ratio_at_tick(tick + 1);
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
        sqrt_price * two_pow_96.to_f64().unwrap()
    }
}
