use candid::Principal;
use std::str::FromStr;

use crate::pool::types::{PoolFee, PoolId};

use super::{types::TickInfo, *};

fn generate_tick_for_test_pool(tick: i32) -> TickKey {
    TickKey {
        tick,
        pool_id: test_pool_id(),
    }
}
fn test_pool_id() -> PoolId {
    PoolId {
        fee: PoolFee(500),
        token0: Principal::from_str("ss2fx-dyaaa-aaaar-qacoq-cai").unwrap(),
        token1: Principal::from_str("pe5t5-diaaa-aaaar-qahwa-cai").unwrap(),
    }
}

fn set_tick_for_test_pool_id(
    tick: i32,
    liquidity_gross: u128,
    liquidity_net: i128,
    fee_growth_outside_0_x128: U256,
    fee_growth_outside_1_x128: U256,
) {
    mutate_state(|s| {
        s.update_tick(
            generate_tick_for_test_pool(tick),
            TickInfo {
                liquidity_gross,
                liquidity_net,
                fee_growth_outside_0_x128,
                fee_growth_outside_1_x128,
            },
        )
    })
}

fn get_tick_from_state(tick: i32) -> TickInfo {
    read_state(|s| s.get_tick(&generate_tick_for_test_pool(tick)))
}

mod tick_spacing {
    use super::*;
    use std::str::FromStr;

    // Fee amount tick spacings (from Uniswap constants)
    const TICK_SPACING_LOW: i32 = 10; // FeeAmount.LOW
    const TICK_SPACING_MEDIUM: i32 = 60; // FeeAmount.MEDIUM
    const TICK_SPACING_HIGH: i32 = 200; // FeeAmount.HIGH
    #[test]
    fn tick_spacing_to_max_liquidity_per_tick_low_fee() {
        let result = tick_spacing_to_max_liquidity_per_tick(TICK_SPACING_LOW);
        let expected = u128::from_str("1917569901783203986719870431555990").unwrap();
        assert_eq!(result, expected, "Low fee tick spacing mismatch");
    }

    #[test]
    fn tick_spacing_to_max_liquidity_per_tick_medium_fee() {
        let result = tick_spacing_to_max_liquidity_per_tick(TICK_SPACING_MEDIUM);
        let expected = u128::from_str("11505743598341114571880798222544994").unwrap();
        assert_eq!(result, expected, "Medium fee tick spacing mismatch");
    }

    #[test]
    fn tick_spacing_to_max_liquidity_per_tick_high_fee() {
        let result = tick_spacing_to_max_liquidity_per_tick(TICK_SPACING_HIGH);
        let expected = u128::from_str("38350317471085141830651933667504588").unwrap();
        assert_eq!(result, expected, "High fee tick spacing mismatch");
    }

    #[test]
    fn tick_spacing_to_max_liquidity_per_tick_entire_range() {
        let result = tick_spacing_to_max_liquidity_per_tick(887272);
        let expected = u128::MAX / 3; // MaxUint128.div(3)
        assert_eq!(result, expected, "Entire range tick spacing mismatch");
    }

    #[test]
    fn tick_spacing_to_max_liquidity_per_tick_2302() {
        let result = tick_spacing_to_max_liquidity_per_tick(2302);
        let expected = u128::from_str("441351967472034323558203122479595605").unwrap();
        assert_eq!(result, expected, "Tick spacing 2302 mismatch");
    }
}

mod fee_growth_inside {

    use super::*;

    #[test]
    fn fee_growth_inside_uninitialized_ticks_if_current_tick_inside() {
        let result = get_fee_growth_inside(
            &generate_tick_for_test_pool(-2),
            &generate_tick_for_test_pool(2),
            &generate_tick_for_test_pool(0),
            U256::from(15_u8),
            U256::from(15_u8),
        );

        assert_eq!(result, (U256::from(15_u8), U256::from(15_u8)))
    }

    #[test]
    fn fee_growth_inside_uninitialized_ticks_if_current_tick_above() {
        let result = get_fee_growth_inside(
            &generate_tick_for_test_pool(-2),
            &generate_tick_for_test_pool(2),
            &generate_tick_for_test_pool(4),
            U256::from(15_u8),
            U256::from(15_u8),
        );

        assert_eq!(result, (U256::from(0_u8), U256::from(0_u8)))
    }

    #[test]
    fn fee_growth_inside_uninitialized_ticks_if_current_tick_below() {
        let result = get_fee_growth_inside(
            &generate_tick_for_test_pool(-2),
            &generate_tick_for_test_pool(2),
            &generate_tick_for_test_pool(-5),
            U256::from(15_u8),
            U256::from(15_u8),
        );

        assert_eq!(result, (U256::from(0_u8), U256::from(0_u8)))
    }

    #[test]
    fn subtract_upper_tick_if_below() {
        set_tick_for_test_pool_id(2, 0, 0, U256::from(2_u8), U256::from(3_u8));

        let result = get_fee_growth_inside(
            &generate_tick_for_test_pool(-2),
            &generate_tick_for_test_pool(2),
            &generate_tick_for_test_pool(0),
            U256::from(15_u8),
            U256::from(15_u8),
        );

        assert_eq!(result, (U256::from(13_u8), U256::from(12_u8)));
    }

    #[test]
    fn subtract_lower_tick_if_above() {
        set_tick_for_test_pool_id(-2, 0, 0, U256::from(2_u8), U256::from(3_u8));

        let result = get_fee_growth_inside(
            &generate_tick_for_test_pool(-2),
            &generate_tick_for_test_pool(2),
            &generate_tick_for_test_pool(0),
            U256::from(15_u8),
            U256::from(15_u8),
        );

        assert_eq!(result, (U256::from(13_u8), U256::from(12_u8)));
    }

    #[test]
    fn subtract_lower_upper_tick_if_inside() {
        set_tick_for_test_pool_id(-2, 0, 0, U256::from(2_u8), U256::from(3_u8));
        set_tick_for_test_pool_id(2, 0, 0, U256::from(4_u8), U256::from(1_u8));

        let result = get_fee_growth_inside(
            &generate_tick_for_test_pool(-2),
            &generate_tick_for_test_pool(2),
            &generate_tick_for_test_pool(0),
            U256::from(15_u8),
            U256::from(15_u8),
        );

        assert_eq!(result, (U256::from(9_u8), U256::from(11_u8)));
    }

    #[test]
    fn should_not_panic_on_overflow_inside_tick() {
        set_tick_for_test_pool_id(
            -2,
            0,
            0,
            U256::MAX.wrapping_sub(U256::from(3_u8)),
            U256::MAX.wrapping_sub(U256::from(2_u8)),
        );
        set_tick_for_test_pool_id(2, 0, 0, U256::from(3_u8), U256::from(5_u8));

        let result = get_fee_growth_inside(
            &generate_tick_for_test_pool(-2),
            &generate_tick_for_test_pool(2),
            &generate_tick_for_test_pool(0),
            U256::from(15_u8),
            U256::from(15_u8),
        );

        assert_eq!(result, (U256::from(16_u8), U256::from(13_u8)));
    }
}

mod update_tick {

    use proptest::num::u128;

    use super::*;

    #[test]
    fn should_flip_when_zero_to_non_zero() {
        let result = update_tick(
            &generate_tick_for_test_pool(0),
            &generate_tick_for_test_pool(0),
            1,
            U256::ZERO,
            U256::ZERO,
            false,
        );

        assert_eq!(Ok((true, 1)), result)
    }

    #[test]
    fn should_not_flip_when_non_zero_to_greater_non_zero() {
        let _ = update_tick(
            &generate_tick_for_test_pool(0),
            &generate_tick_for_test_pool(0),
            1,
            U256::ZERO,
            U256::ZERO,
            false,
        );

        let result = update_tick(
            &generate_tick_for_test_pool(0),
            &generate_tick_for_test_pool(0),
            1,
            U256::ZERO,
            U256::ZERO,
            false,
        );

        assert_eq!(Ok((false, 2)), result)
    }

    #[test]
    fn should_not_flip_when_non_zero_to_lesser_non_zero() {
        let _ = update_tick(
            &generate_tick_for_test_pool(0),
            &generate_tick_for_test_pool(0),
            2,
            U256::ZERO,
            U256::ZERO,
            false,
        );

        let result = update_tick(
            &generate_tick_for_test_pool(0),
            &generate_tick_for_test_pool(0),
            -1,
            U256::ZERO,
            U256::ZERO,
            false,
        );

        assert_eq!(Ok((false, 1)), result)
    }

    #[test]
    fn update_nets_liquidity_based_on_upper_flag() {
        let _ = update_tick(
            &generate_tick_for_test_pool(0),
            &generate_tick_for_test_pool(0),
            2,
            U256::ZERO,
            U256::ZERO,
            false,
        );

        let _ = update_tick(
            &generate_tick_for_test_pool(0),
            &generate_tick_for_test_pool(0),
            1,
            U256::ZERO,
            U256::ZERO,
            true,
        );

        let _ = update_tick(
            &generate_tick_for_test_pool(0),
            &generate_tick_for_test_pool(0),
            3,
            U256::ZERO,
            U256::ZERO,
            true,
        );

        let _ = update_tick(
            &generate_tick_for_test_pool(0),
            &generate_tick_for_test_pool(0),
            1,
            U256::ZERO,
            U256::ZERO,
            false,
        );

        let tick_after_updates = get_tick_from_state(0);

        assert_eq!(tick_after_updates.liquidity_gross, 2 + 1 + 3 + 1);
        assert_eq!(tick_after_updates.liquidity_net, 2 - 1 - 3 + 1)
    }

    #[test]
    fn should_err_on_liquidity_net_overflow() {
        let _ = update_tick(
            &generate_tick_for_test_pool(0),
            &generate_tick_for_test_pool(0),
            (u128::MAX / 2 - 1) as i128,
            U256::ZERO,
            U256::ZERO,
            false,
        );

        let result = update_tick(
            &generate_tick_for_test_pool(0),
            &generate_tick_for_test_pool(0),
            (u128::MAX / 2 - 1) as i128,
            U256::ZERO,
            U256::ZERO,
            false,
        );

        assert_eq!(result, Err(UpdateTickError::LiquidityNetOverflow))
    }

    #[test]
    fn update_tick_assumes_all_growth_happens_below_ticks_lte_current_tick() {
        let _result = update_tick(
            &generate_tick_for_test_pool(1),
            &generate_tick_for_test_pool(1),
            1,
            U256::ONE,
            U256::from(2_u8),
            false,
        );

        let updated_tick = get_tick_from_state(1);

        assert_eq!(updated_tick.fee_growth_outside_0_x128, U256::ONE);
        assert_eq!(updated_tick.fee_growth_outside_1_x128, U256::from(2_u8))
    }

    #[test]
    fn update_tick_does_not_set_any_growth_fields_if_tick_already_initialized() {
        let _result = update_tick(
            &generate_tick_for_test_pool(1),
            &generate_tick_for_test_pool(1),
            1,
            U256::ONE,
            U256::from(2_u8),
            false,
        );

        let _result = update_tick(
            &generate_tick_for_test_pool(1),
            &generate_tick_for_test_pool(1),
            1,
            U256::from(6_u8),
            U256::from(7_u8),
            false,
        );

        let updated_tick = get_tick_from_state(1);

        assert_eq!(updated_tick.fee_growth_outside_0_x128, U256::ONE);
        assert_eq!(updated_tick.fee_growth_outside_1_x128, U256::from(2_u8))
    }

    #[test]
    fn update_tick_does_not_set_any_growth_fields_for_ticks_gt_current_tick() {
        let _result = update_tick(
            &generate_tick_for_test_pool(2),
            &generate_tick_for_test_pool(1),
            1,
            U256::ONE,
            U256::from(2_u8),
            false,
        );

        let updated_tick = get_tick_from_state(1);

        assert_eq!(updated_tick.fee_growth_outside_0_x128, U256::ZERO);
        assert_eq!(updated_tick.fee_growth_outside_1_x128, U256::ZERO)
    }

    #[test]
    fn update_tick_liquidity_parsing_parses_max_uint128_stored_liquidity_gross_before_update() {
        set_tick_for_test_pool_id(2, u128::MAX, 0, U256::ZERO, U256::ZERO);

        let _result = update_tick(
            &generate_tick_for_test_pool(2),
            &generate_tick_for_test_pool(1),
            -1,
            U256::ONE,
            U256::from(2_u8),
            false,
        );

        let tick_result = get_tick_from_state(2);

        assert_eq!(tick_result.liquidity_gross, u128::MAX - 1);
        assert_eq!(tick_result.liquidity_net, -1)
    }

    #[test]
    fn update_tick_liquidity_parsing_parses_max_uint128_stored_liquidity_gross_after_update() {
        set_tick_for_test_pool_id(2, (u128::MAX / 2) + 1, 0, U256::ZERO, U256::ZERO);

        let _result = update_tick(
            &generate_tick_for_test_pool(2),
            &generate_tick_for_test_pool(1),
            (u128::MAX / 2) as i128,
            U256::ONE,
            U256::from(2_u8),
            false,
        );

        let tick_result = get_tick_from_state(2);

        assert_eq!(tick_result.liquidity_gross, u128::MAX);
        assert_eq!(tick_result.liquidity_net, (u128::MAX / 2) as i128)
    }

    #[test]
    fn update_tick_liquidity_parsing_parses_max_int128_stored_liquidity_net_before_update() {
        set_tick_for_test_pool_id(2, 0, (u128::MAX / 2 - 1) as i128, U256::ZERO, U256::ZERO);

        let _result = update_tick(
            &generate_tick_for_test_pool(2),
            &generate_tick_for_test_pool(1),
            1,
            U256::ONE,
            U256::from(2_u8),
            false,
        );

        let tick_result = get_tick_from_state(2);

        assert_eq!(tick_result.liquidity_gross, 1);
        assert_eq!(tick_result.liquidity_net, (u128::MAX / 2) as i128)
    }
}

mod clear {
    use super::*;

    #[test]
    fn test_clear_tick_from_state() {
        set_tick_for_test_pool_id(2, 15, (u128::MAX / 2 - 1) as i128, U256::ZERO, U256::ZERO);

        mutate_state(|s| s.clear_tick(&generate_tick_for_test_pool(2)));

        let tick_result = get_tick_from_state(2);

        assert_eq!(tick_result.liquidity_gross, 0);
        assert_eq!(tick_result.liquidity_net, 0)
    }
}

mod cross {
    use super::*;

    #[test]
    fn cross_flips_the_growth_variables() {
        set_tick_for_test_pool_id(2, 3, 4, U256::ONE, U256::from(2_u8));

        cross_tick(
            &generate_tick_for_test_pool(2),
            U256::from(7_u8),
            U256::from(9_u8),
        );
        let tick_result = get_tick_from_state(2);

        assert_eq!(tick_result.fee_growth_outside_0_x128, U256::from(6_u8));
        assert_eq!(tick_result.fee_growth_outside_1_x128, U256::from(7_u8))
    }

    #[test]
    fn cross_two_flips_the_growth_variables() {
        set_tick_for_test_pool_id(2, 3, 4, U256::ONE, U256::from(2_u8));

        cross_tick(
            &generate_tick_for_test_pool(2),
            U256::from(7_u8),
            U256::from(9_u8),
        );

        cross_tick(
            &generate_tick_for_test_pool(2),
            U256::from(7_u8),
            U256::from(9_u8),
        );

        let tick_result = get_tick_from_state(2);

        assert_eq!(tick_result.fee_growth_outside_0_x128, U256::from(1_u8));
        assert_eq!(tick_result.fee_growth_outside_1_x128, U256::from(2_u8))
    }
}
