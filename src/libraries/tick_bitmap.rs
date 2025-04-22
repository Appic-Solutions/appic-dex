use ethnum::U256;

use crate::{
    state::read_state,
    tick::types::{BitmapWord, TickBitmapKey, TickKey},
};

use super::{
    bit_math,
    constants::{MAX_TICK, MIN_TICK},
};

/// dev round towards negative infinity by tick_spacing
pub fn compress(tick: i32, tick_spacing: i32) -> i32 {
    //No need to check for tick_spacing and tick range since the caller of this functions checks it
    // Compute quotient and remainder
    let quotient = tick / tick_spacing;
    let remainder = tick % tick_spacing;

    // Round down: subtract 1 if tick is negative and remainder exists
    let compressed = if tick < 0 && remainder != 0 {
        quotient - 1
    } else {
        quotient
    };

    compressed
}

/// Computes the position in the tick bitmap where the initialized bit for a tick lives.
///
/// # Arguments
/// * `tick_key` - The tick key containing the pool_id and tick for which to compute the position.
///
/// # Returns
/// * `word_pos` - The key in the mapping (word_pos) containing the word in which the bit is stored.
/// * `bit_pos` - The bit position in the word where the flag is stored.
pub fn position(tick: i32) -> (i16, u8) {
    if tick < MIN_TICK || tick > MAX_TICK {
        panic!("BUG: InvalidTick");
    }

    let word_pos = tick >> 8;

    let bit_pos = (tick & 0xff) as u8;

    if word_pos < i16::MIN as i32 || word_pos > i16::MAX as i32 {
        panic!("BUG: word_pos should fit in an i_16")
    }

    (word_pos as i16, bit_pos)
}

/// Flips the initialized state for a given tick from false to true, or vice versa.
///
/// # Arguments
/// * `tick_key` - The tick key containing the pool_id and tick to flip.
/// * `tick_spacing` - The spacing between usable ticks.
#[derive(Debug, PartialEq)]
pub enum TickBitmapError {
    TickMisaligned(i32, i32),
}

#[derive(Debug, PartialEq)]
pub struct TickBitmapFlipSuccess {
    pub bitmap_key: TickBitmapKey,
    pub flipped_bitmap_word: BitmapWord,
}

pub fn flip_tick(
    tick_key: &TickKey,
    tick_spacing: i32,
) -> Result<TickBitmapFlipSuccess, TickBitmapError> {
    // Ensure tick_spacing is positive and non-zero
    if tick_spacing <= 0 {
        panic!("Bug: TickSpacing can not be zero")
    }

    let tick = tick_key.tick;

    // Ensure tick is within int24 bounds
    if tick < MIN_TICK || tick > MAX_TICK {
        panic!("Bug: InvalidTick")
    }

    // Check if tick is aligned (tick % tick_spacing == 0)
    if tick % tick_spacing != 0 {
        return Err(TickBitmapError::TickMisaligned(tick, tick_spacing));
    }

    // Compute position for tick / tick_spacing
    let aligned_tick = tick / tick_spacing;

    let (word_pos, bit_pos) = position(aligned_tick);

    // Create the bitmap key
    let bitmap_key = TickBitmapKey {
        pool_id: tick_key.pool_id.clone(),
        word_pos,
    };

    let mut bitmap_word = read_state(|s| s.get_bitmap_word(&bitmap_key));

    bitmap_word.0 ^= U256::ONE << bit_pos;

    Ok(TickBitmapFlipSuccess {
        bitmap_key,
        flipped_bitmap_word: bitmap_word,
    })
}

/// Returns the next initialized tick contained in the same word (or adjacent word) as the tick that is
/// either to the left (less than or equal to) or right (greater than) of the given tick.
///
/// # Arguments
/// * `tick_key` - The tick key containing the pool_id and starting tick.
/// * `tick_spacing` - The spacing between usable ticks.
/// * `lte` - Whether to search for the next initialized tick to the left (true) or right (false).
///
/// # Returns
/// * `next` - The next initialized or uninitialized tick up to 256 ticks away from the current tick.
/// * `initialized` - Whether the next tick is initialized.
pub fn next_initialized_tick_within_one_word(
    tick_key: &TickKey,
    tick_spacing: i32,
    lte: bool,
) -> (i32, bool) {
    // Ensure tick_spacing is positive and non-zero
    if tick_spacing <= 0 {
        panic!("Bug: TickSpacing can not be zero")
    };

    let tick = tick_key.tick;

    // Ensure tick is within int24 bounds
    if tick < MIN_TICK || tick > MAX_TICK {
        panic!("Bug: InvalidTick")
    }

    // Compress the tick
    let compressed = compress(tick, tick_spacing);

    if lte {
        // Search left (less than or equal to)
        let (word_pos, bit_pos) = position(compressed);

        // Mask: all 1s at or to the right of bitPos (e.g., bitPos=2 -> 0b...111100)
        let mask = U256::MAX >> (255u32 - bit_pos as u32);
        let bitmap_key = TickBitmapKey {
            pool_id: tick_key.pool_id.clone(),
            word_pos,
        };

        // Get the bitmap word
        let masked = read_state(|state| state.get_bitmap_word(&bitmap_key).0 & mask);

        let initialized = masked != U256::ZERO;
        let next = if initialized {
            let msb = bit_math::get_msb_bit_position(&masked)
                .expect("Bug: U256::ZERO should never be passed as an argument");
            (compressed - (bit_pos as i32 - msb as i32)) * tick_spacing
        } else {
            (compressed - bit_pos as i32) * tick_spacing
        };

        (next, initialized)
    } else {
        // Search right (greater than)
        let compressed_plus_one = compressed + 1;
        let (word_pos, bit_pos) = position(compressed_plus_one);

        // Mask: all 1s at or to the left of bitPos (e.g., bitPos=2 -> 0b...11100)
        let mask = !((U256::from(1u64) << bit_pos) - 1);
        let bitmap_key = TickBitmapKey {
            pool_id: tick_key.pool_id.clone(),
            word_pos,
        };

        // Get the bitmap word
        let masked = read_state(|state| state.get_bitmap_word(&bitmap_key).0 & mask);

        let initialized = masked != U256::ZERO;
        let next = if initialized {
            let lsb = bit_math::get_lsb_bit_position(&masked)
                .expect("Bug: U256::ZERO should never be passed as an argument");

            (compressed_plus_one + (lsb as i32 - bit_pos as i32)) * tick_spacing
        } else {
            (compressed_plus_one + (255i32 - bit_pos as i32)) * tick_spacing
        };

        (next, initialized)
    }
}

#[cfg(test)]
pub mod tests {

    use crate::{state::mutate_state, tick::tests::test_pool_id};

    use super::*;
    use ethnum::U256;
    use proptest::prelude::*;

    const INITIALIZED_TICK: i32 = 70;
    const SOLO_INITIALIZED_TICK_IN_WORD: i32 = -10_000;

    fn create_tick_key(tick: i32) -> TickKey {
        TickKey {
            pool_id: test_pool_id(),
            tick,
        }
    }

    fn setup_state() {
        let ticks = [
            SOLO_INITIALIZED_TICK_IN_WORD,
            -200,
            -55,
            -4,
            INITIALIZED_TICK,
            78,
            84,
            139,
            240,
            535,
        ];
        for &tick in ticks.iter().take(ticks.len() - 1) {
            let tick_key = create_tick_key(tick);
            if let Ok(success) = flip_tick(&tick_key, 1) {
                mutate_state(|s| {
                    s.set_bitmap_word(success.bitmap_key, success.flipped_bitmap_word)
                });
            }
        }
    }

    pub fn is_initialized(tick_key: &TickKey, tick_spacing: i32) -> bool {
        if tick_key.tick % tick_spacing != 0 {
            return false;
        }
        let aligned_tick = tick_key.tick / tick_spacing;
        let (word_pos, bit_pos) = position(aligned_tick);
        let bitmap_key = TickBitmapKey {
            pool_id: test_pool_id(),
            word_pos,
        };
        read_state(|s| s.get_bitmap_word(&bitmap_key).0 & (U256::ONE << bit_pos) != U256::ZERO)
    }

    #[test]
    fn test_compress() {
        // Specific cases
        assert_eq!(compress(25, 10), 2); // 25 / 10 = 2.5, rounds to 2
        assert_eq!(compress(-25, 10), -3); // -25 / 10 = -2.5, rounds to -3
        assert_eq!(compress(0, 10), 0);
        assert_eq!(compress(8388607, 10), 838860); // Max tick
        assert_eq!(compress(-8388608, 10), -838861); // Min tick
    }

    proptest! {
        #[test]
        fn test_fuzz_compress(tick in MIN_TICK..=MAX_TICK, tick_spacing in 1i32..=MAX_TICK) {
            let compressed = compress(tick, tick_spacing);
            let expected = {
                let quotient = tick / tick_spacing;
                if tick < 0 && tick % tick_spacing != 0 {
                    quotient - 1
                } else {
                    quotient
                }
            };
            prop_assert_eq!(compressed, expected);
        }
    }

    #[test]
    fn test_position() {
        // Specific cases
        assert_eq!(position(256), (1, 0)); // 256 >> 8 = 1, 256 & 0xff = 0
        assert_eq!(position(511), (1, 255)); // 511 >> 8 = 1, 511 & 0xff = 255
        assert_eq!(position(-256), (-1, 0)); // -256 >> 8 = -1, -256 & 0xff = 0
        assert_eq!(position(-511), (-2, 1)); // -511 >> 8 = -2, -511 & 0xff = 1
        assert_eq!(position(0), (0, 0));
    }

    proptest! {
        #[test]
        fn test_fuzz_position(tick in MIN_TICK..=MAX_TICK) {
            let (word_pos, bit_pos) = position(tick);
            prop_assert_eq!(word_pos as i32, tick >> 8);
            prop_assert_eq!(bit_pos as i32, tick & 0xff);
        }
    }

    #[test]
    fn test_is_initialized_is_false_at_first() {
        setup_state();
        let tick_key = create_tick_key(1);
        assert_eq!(is_initialized(&tick_key, 1), false);
    }

    #[test]
    fn test_is_initialized_is_flipped_by_flip_tick() {
        setup_state();
        let tick_key = create_tick_key(1);
        let success = flip_tick(&tick_key, 1).unwrap();
        mutate_state(|s| {
            s.set_bitmap_word(success.bitmap_key, success.flipped_bitmap_word);
        });
        assert_eq!(is_initialized(&tick_key, 1), true);
    }

    #[test]
    fn test_is_initialized_is_flipped_back_by_flip_tick() {
        setup_state();
        let tick_key = create_tick_key(1);
        let success = flip_tick(&tick_key, 1).unwrap();
        mutate_state(|s| {
            s.set_bitmap_word(success.bitmap_key, success.flipped_bitmap_word);
        });
        let success = flip_tick(&tick_key, 1).unwrap();
        mutate_state(|s| {
            s.set_bitmap_word(success.bitmap_key, success.flipped_bitmap_word);
        });
        assert_eq!(is_initialized(&tick_key, 1), false);
    }

    #[test]
    fn test_is_initialized_not_changed_by_different_tick() {
        setup_state();
        let tick_key_2 = create_tick_key(2);
        let success = flip_tick(&tick_key_2, 1).unwrap();
        mutate_state(|s| {
            s.set_bitmap_word(success.bitmap_key, success.flipped_bitmap_word);
        });
        let tick_key_1 = create_tick_key(1);
        assert_eq!(is_initialized(&tick_key_1, 1), false);
    }

    #[test]
    fn test_is_initialized_not_changed_by_different_word() {
        setup_state();
        let tick_key_257 = create_tick_key(257);
        let success = flip_tick(&tick_key_257, 1).unwrap();
        mutate_state(|s| {
            s.set_bitmap_word(success.bitmap_key, success.flipped_bitmap_word);
        });
        assert_eq!(is_initialized(&tick_key_257, 1), true);
        let tick_key_1 = create_tick_key(1);
        assert_eq!(is_initialized(&tick_key_1, 1), false);
    }

    #[test]
    fn test_flip_tick_flips_only_specified_tick() {
        setup_state();
        let tick_key = create_tick_key(-230);
        let success = flip_tick(&tick_key, 1).unwrap();
        mutate_state(|s| {
            s.set_bitmap_word(success.bitmap_key, success.flipped_bitmap_word);
        });
        assert_eq!(is_initialized(&create_tick_key(-230), 1), true);
        assert_eq!(is_initialized(&create_tick_key(-231), 1), false);
        assert_eq!(is_initialized(&create_tick_key(-229), 1), false);
        assert_eq!(is_initialized(&create_tick_key(-230 + 256), 1), false);
        assert_eq!(is_initialized(&create_tick_key(-230 - 256), 1), false);

        let success = flip_tick(&tick_key, 1).unwrap();
        mutate_state(|s| {
            s.set_bitmap_word(success.bitmap_key, success.flipped_bitmap_word);
        });
        assert_eq!(is_initialized(&create_tick_key(-230), 1), false);
        assert_eq!(is_initialized(&create_tick_key(-231), 1), false);
        assert_eq!(is_initialized(&create_tick_key(-229), 1), false);
        assert_eq!(is_initialized(&create_tick_key(-230 + 256), 1), false);
        assert_eq!(is_initialized(&create_tick_key(-230 - 256), 1), false);
    }

    #[test]
    fn test_flip_tick_reverts_only_itself() {
        setup_state();
        let ticks = [-230, -259, -229, 500, -259, -229, -259];
        for &tick in ticks.iter() {
            let tick_key = create_tick_key(tick);
            let success = flip_tick(&tick_key, 1).unwrap();
            mutate_state(|s| {
                s.set_bitmap_word(success.bitmap_key, success.flipped_bitmap_word);
            });
        }
        assert_eq!(is_initialized(&create_tick_key(-259), 1), true);
        assert_eq!(is_initialized(&create_tick_key(-229), 1), false);
    }

    proptest! {
        #[test]
        fn test_fuzz_flip_tick(
            tick in MIN_TICK..=MAX_TICK,
            tick_spacing in 1i32..=MAX_TICK
        ) {
            setup_state();
            let tick_key = create_tick_key( tick);
            if tick % tick_spacing != 0 {
                prop_assert_eq!(
                    flip_tick(&tick_key, tick_spacing),
                    Err(TickBitmapError::TickMisaligned(tick, tick_spacing))
                );
            } else {
                let initialized_before = is_initialized(&tick_key, tick_spacing);
                let success = flip_tick(&tick_key, tick_spacing).unwrap();
                mutate_state(|s| {

                        s.set_bitmap_word(success.bitmap_key, success.flipped_bitmap_word);
                });
                prop_assert_eq!(is_initialized(&tick_key, tick_spacing), !initialized_before);
                let success = flip_tick(&tick_key, tick_spacing).unwrap();
                mutate_state(|s| {

                        s.set_bitmap_word(success.bitmap_key, success.flipped_bitmap_word);
                });
                prop_assert_eq!(is_initialized(&tick_key, tick_spacing), initialized_before);
            }
        }
    }

    #[test]
    fn test_next_initialized_tick_minus_one_lte_false() {
        let tick_key = create_tick_key(-1);
        let (_next, initialized) = next_initialized_tick_within_one_word(&tick_key, 1, false);
        assert_eq!(initialized, false);
    }

    #[test]
    fn test_next_initialized_tick_within_one_word_lte_false_right_initialized() {
        setup_state();
        let tick_key = create_tick_key(78);
        let (next, initialized) = next_initialized_tick_within_one_word(&tick_key, 1, false);
        assert_eq!(next, 84);
        assert_eq!(initialized, true);

        let tick_key = create_tick_key(-55);
        let (next, initialized) = next_initialized_tick_within_one_word(&tick_key, 1, false);
        assert_eq!(next, -4);
        assert_eq!(initialized, true);
    }

    #[test]
    fn test_next_initialized_tick_within_one_word_lte_false_direct_right() {
        setup_state();
        let tick_key = create_tick_key(77);
        let (next, initialized) = next_initialized_tick_within_one_word(&tick_key, 1, false);
        assert_eq!(next, 78);
        assert_eq!(initialized, true);

        let tick_key = create_tick_key(-56);
        let (next, initialized) = next_initialized_tick_within_one_word(&tick_key, 1, false);
        assert_eq!(next, -55);
        assert_eq!(initialized, true);
    }

    #[test]
    fn test_next_initialized_tick_within_one_word_lte_false_right_boundary() {
        setup_state();
        let tick_key = create_tick_key(255);
        let (next, initialized) = next_initialized_tick_within_one_word(&tick_key, 1, false);
        assert_eq!(next, 511);
        assert_eq!(initialized, false);

        let tick_key = create_tick_key(-257);
        let (next, initialized) = next_initialized_tick_within_one_word(&tick_key, 1, false);
        assert_eq!(next, -200);
        assert_eq!(initialized, true);
    }

    #[test]
    fn test_next_initialized_tick_within_one_word_lte_false_next_word() {
        setup_state();
        let tick_key_340 = create_tick_key(340);
        let success = flip_tick(&tick_key_340, 1).unwrap();
        mutate_state(|s| {
            s.set_bitmap_word(success.bitmap_key, success.flipped_bitmap_word);
        });
        let tick_key = create_tick_key(328);
        let (next, initialized) = next_initialized_tick_within_one_word(&tick_key, 1, false);
        assert_eq!(next, 340);
        assert_eq!(initialized, true);
    }

    #[test]
    fn test_next_initialized_tick_within_one_word_lte_false_no_exceed_boundary() {
        setup_state();
        let tick_key = create_tick_key(508);
        let (next, initialized) = next_initialized_tick_within_one_word(&tick_key, 1, false);
        assert_eq!(next, 511);
        assert_eq!(initialized, false);
    }

    #[test]
    fn test_next_initialized_tick_within_one_word_lte_false_skips_word() {
        setup_state();
        let tick_key = create_tick_key(255);
        let (next, initialized) = next_initialized_tick_within_one_word(&tick_key, 1, false);
        assert_eq!(next, 511);
        assert_eq!(initialized, false);

        let tick_key = create_tick_key(383);
        let (next, initialized) = next_initialized_tick_within_one_word(&tick_key, 1, false);
        assert_eq!(next, 511);
        assert_eq!(initialized, false);
    }

    #[test]
    fn test_next_initialized_tick_within_one_word_lte_true_same_tick() {
        setup_state();
        let tick_key = create_tick_key(78);
        let (next, initialized) = next_initialized_tick_within_one_word(&tick_key, 1, true);
        assert_eq!(next, 78);
        assert_eq!(initialized, true);
    }

    #[test]
    fn test_next_initialized_tick_within_one_word_lte_true_direct_left() {
        setup_state();
        let tick_key = create_tick_key(79);
        let (next, initialized) = next_initialized_tick_within_one_word(&tick_key, 1, true);
        assert_eq!(next, 78);
        assert_eq!(initialized, true);
    }

    #[test]
    fn test_next_initialized_tick_within_one_word_lte_true_no_exceed_boundary() {
        setup_state();
        let tick_key = create_tick_key(258);
        let (next, initialized) = next_initialized_tick_within_one_word(&tick_key, 1, true);
        assert_eq!(next, 256);
        assert_eq!(initialized, false);
    }

    #[test]
    fn test_next_initialized_tick_within_one_word_lte_true_at_boundary() {
        setup_state();
        let tick_key = create_tick_key(256);
        let (next, initialized) = next_initialized_tick_within_one_word(&tick_key, 1, true);
        assert_eq!(next, 256);
        assert_eq!(initialized, false);
    }

    #[test]
    fn test_next_initialized_tick_within_one_word_lte_true_prev_word() {
        setup_state();
        let tick_key = create_tick_key(72);
        let (next, initialized) = next_initialized_tick_within_one_word(&tick_key, 1, true);
        assert_eq!(next, 70);
        assert_eq!(initialized, true);
    }

    #[test]
    fn test_next_initialized_tick_within_one_word_lte_true_word_boundary() {
        setup_state();
        let tick_key = create_tick_key(-257);
        let (next, initialized) = next_initialized_tick_within_one_word(&tick_key, 1, true);
        assert_eq!(next, -512);
        assert_eq!(initialized, false);
    }

    #[test]
    fn test_next_initialized_tick_within_one_word_lte_true_empty_word() {
        setup_state();
        let tick_key = create_tick_key(1023);
        let (next, initialized) = next_initialized_tick_within_one_word(&tick_key, 1, true);
        assert_eq!(next, 768);
        assert_eq!(initialized, false);

        let tick_key = create_tick_key(900);
        let (next, initialized) = next_initialized_tick_within_one_word(&tick_key, 1, true);
        assert_eq!(next, 768);
        assert_eq!(initialized, false);
    }

    #[test]
    fn test_next_initialized_tick_within_one_word_lte_true_boundary_initialized() {
        setup_state();
        let tick_key_329 = create_tick_key(329);
        let success = flip_tick(&tick_key_329, 1).unwrap();
        mutate_state(|s| {
            s.set_bitmap_word(success.bitmap_key, success.flipped_bitmap_word);
        });
        let tick_key = create_tick_key(456);
        let (next, initialized) = next_initialized_tick_within_one_word(&tick_key, 1, true);
        assert_eq!(next, 329);
        assert_eq!(initialized, true);
    }

    proptest! {
        #[test]
        fn test_fuzz_next_initialized_tick_within_one_word(
            tick in MIN_TICK..MAX_TICK,
            lte in any::<bool>()
        ) {
            setup_state();
            let tick_key = create_tick_key( tick);
            let (next, initialized) = next_initialized_tick_within_one_word(&tick_key, 1, lte);
            if next > MIN_TICK && next < MAX_TICK{
                 if lte {
                prop_assert!(next <= tick);
                prop_assert!((tick - next) <= 256);
                for i in (next + 1)..=tick {
                    prop_assert!(!is_initialized(&create_tick_key( i), 1));
                }
                prop_assert_eq!(is_initialized(&create_tick_key( next), 1), initialized);
            } else {
                prop_assert!(next > tick);
                prop_assert!((next - tick) <= 256);
                for i in (tick + 1)..next {
                    prop_assert!(!is_initialized(&create_tick_key( i), 1));
                }
                prop_assert_eq!(is_initialized(&create_tick_key( next), 1), initialized);
            }

            }
        }
    }

    proptest! {
        #[test]
        fn test_fuzz_next_initialized_tick_within_one_word_on_empty_bitmap(
            tick in MIN_TICK..=MAX_TICK,
            tick_spacing in 1i32..=16384,
            lte in any::<bool>()
        ) {
           let tick_key = create_tick_key( tick);
           let (next, initialized) = next_initialized_tick_within_one_word(&tick_key, 1, lte);
                if next > MIN_TICK && next < MAX_TICK{

                if lte {
                    prop_assert!(next <= tick);
                    prop_assert!((tick - next) <= 256);
                    for i in (next + 1)..=tick {
                        prop_assert!(!is_initialized(&create_tick_key( i), tick_spacing));
                    }
                    prop_assert_eq!(is_initialized(&create_tick_key( next), tick_spacing), initialized);
                } else {
                    prop_assert!(next > tick);
                    prop_assert!((next - tick) <= 256);
                    for i in (tick + 1)..next {
                        prop_assert!(!is_initialized(&create_tick_key( i), tick_spacing));
                    }
                    prop_assert_eq!(is_initialized(&create_tick_key( next), tick_spacing), initialized);
                }
            }
        }
    }
}
