mod modify_liquidity_tests {
    use candid::Principal;
    use ethnum::U256;
    use proptest::{prop_assert, prop_assert_eq, proptest};

    use crate::{
        libraries::{
            constants::{MAX_TICK, MIN_TICK},
            tick_bitmap::tests::is_initialized,
            tick_math,
        },
        pool::{
            modify_liquidity::{modify_liquidity, ModifyLiquidityError, ModifyLiquidityParams},
            types::{PoolState, PoolTickSpacing},
        },
        position::types::PositionKey,
        state::{mutate_state, read_state},
        tick::{tests::test_pool_id, tick_spacing_to_max_liquidity_per_tick, types::TickKey},
    };

    pub fn test_modify_liquidity_params() -> ModifyLiquidityParams {
        ModifyLiquidityParams {
            owner: Principal::management_canister(),
            pool_id: test_pool_id(),
            tick_lower: -120,
            tick_upper: 120,
            liquidity_delta: 10_i128.pow(18),
            tick_spacing: PoolTickSpacing(10),
        }
    }

    pub fn test_modify_liquidity_params_2() -> ModifyLiquidityParams {
        ModifyLiquidityParams {
            owner: Principal::management_canister(),
            pool_id: test_pool_id(),
            tick_lower: -190,
            tick_upper: 190,
            liquidity_delta: 50_i128.pow(18),
            tick_spacing: PoolTickSpacing(10),
        }
    }

    pub fn test_position_key() -> PositionKey {
        PositionKey {
            owner: Principal::management_canister(),
            pool_id: test_pool_id(),
            tick_lower: -120,
            tick_upper: 120,
        }
    }

    pub fn test_position_key_2() -> PositionKey {
        PositionKey {
            owner: Principal::management_canister(),
            pool_id: test_pool_id(),
            tick_lower: -190,
            tick_upper: 190,
        }
    }

    pub fn sqrt_price_1_1() -> U256 {
        U256::from(79228162514264337593543950336_u128)
    }
    pub fn initialize_test_pool(tick_spacing: i32) {
        mutate_state(|s| {
            s.set_pool(
                test_pool_id(),
                PoolState {
                    sqrt_price_x96: sqrt_price_1_1(),
                    tick: tick_math::TickMath::get_tick_at_sqrt_ratio(sqrt_price_1_1()),
                    fee_growth_global_0_x128: U256::ZERO,
                    fee_growth_global_1_x128: U256::ZERO,
                    liquidity: 0_u128,
                    tick_spacing: PoolTickSpacing(tick_spacing),
                    max_liquidity_per_tick: tick_spacing_to_max_liquidity_per_tick(10),
                    fee_protocol: 500,
                    token1_transfer_fee: U256::ZERO,
                    token0_transfer_fee: U256::ZERO,
                },
            )
        });
    }

    #[test]
    fn modify_liquidity_should_fail_uninitialized_pool() {
        let position = read_state(|s| s.get_position(&test_position_key()));
        assert_eq!(position.liquidity, 0);

        let result = modify_liquidity(test_modify_liquidity_params());

        assert_eq!(result, Err(ModifyLiquidityError::PoolNotInitialized));
    }

    #[test]
    fn modify_liquidity_on_the_same_position() {
        initialize_test_pool(10);
        let position = read_state(|s| s.get_position(&test_position_key()));
        assert_eq!(position.liquidity, 0);

        let result = modify_liquidity(test_modify_liquidity_params()).unwrap();
        mutate_state(|s| s.apply_modify_liquidity_buffer_state(result.buffer_state.clone()));
        let position = read_state(|s| s.get_position(&test_position_key()));
        assert_eq!(
            position.liquidity,
            test_modify_liquidity_params().liquidity_delta as u128
        );

        let result = modify_liquidity(test_modify_liquidity_params()).unwrap();
        mutate_state(|s| s.apply_modify_liquidity_buffer_state(result.buffer_state.clone()));

        let position = read_state(|s| s.get_position(&test_position_key()));
        assert_eq!(
            position.liquidity,
            test_modify_liquidity_params().liquidity_delta as u128 * 2
        );

        // remove liquidity from the same position
        let mut remove_liquidity_params = test_modify_liquidity_params();
        remove_liquidity_params.liquidity_delta = -remove_liquidity_params.liquidity_delta;
        let result = modify_liquidity(remove_liquidity_params).unwrap();
        mutate_state(|s| s.apply_modify_liquidity_buffer_state(result.buffer_state.clone()));

        let position = read_state(|s| s.get_position(&test_position_key()));
        assert_eq!(
            position.liquidity,
            test_modify_liquidity_params().liquidity_delta as u128
        );
    }

    #[test]
    fn modify_liquidity_on_the_different_positions_should_not_modify_each_other() {
        initialize_test_pool(10);
        let position = read_state(|s| s.get_position(&test_position_key()));
        assert_eq!(position.liquidity, 0);

        let result = modify_liquidity(test_modify_liquidity_params()).unwrap();
        let result_2 = modify_liquidity(test_modify_liquidity_params_2()).unwrap();

        mutate_state(|s| s.apply_modify_liquidity_buffer_state(result.buffer_state.clone()));
        mutate_state(|s| s.apply_modify_liquidity_buffer_state(result_2.buffer_state.clone()));

        let position = read_state(|s| s.get_position(&test_position_key()));
        let position_2 = read_state(|s| s.get_position(&test_position_key_2()));

        assert_eq!(
            position.liquidity,
            test_modify_liquidity_params().liquidity_delta as u128
        );

        assert_eq!(
            position_2.liquidity,
            test_modify_liquidity_params_2().liquidity_delta as u128
        );

        let result = modify_liquidity(test_modify_liquidity_params()).unwrap();
        let result_2 = modify_liquidity(test_modify_liquidity_params_2()).unwrap();

        mutate_state(|s| s.apply_modify_liquidity_buffer_state(result.buffer_state.clone()));
        mutate_state(|s| s.apply_modify_liquidity_buffer_state(result_2.buffer_state.clone()));

        let position = read_state(|s| s.get_position(&test_position_key()));
        let position_2 = read_state(|s| s.get_position(&test_position_key_2()));

        assert_eq!(
            position.liquidity,
            test_modify_liquidity_params().liquidity_delta as u128 * 2
        );

        assert_eq!(
            position_2.liquidity,
            test_modify_liquidity_params_2().liquidity_delta as u128 * 2
        );

        assert_ne!(position.liquidity, position_2.liquidity)
    }

    #[test]
    fn modify_liquidity_should_update_and_flip_ticks() {
        initialize_test_pool(10);
        let position = read_state(|s| s.get_position(&test_position_key()));
        assert_eq!(position.liquidity, 0);

        let result = modify_liquidity(test_modify_liquidity_params()).unwrap();
        mutate_state(|s| s.apply_modify_liquidity_buffer_state(result.buffer_state.clone()));

        let is_lower_flipped = is_initialized(
            &TickKey {
                pool_id: test_pool_id(),
                tick: test_modify_liquidity_params().tick_lower,
            },
            test_modify_liquidity_params().tick_spacing.0,
        );

        assert!(is_lower_flipped);

        let is_upper_flipped = is_initialized(
            &TickKey {
                pool_id: test_pool_id(),
                tick: test_modify_liquidity_params().tick_upper,
            },
            test_modify_liquidity_params().tick_spacing.0,
        );

        assert!(is_upper_flipped);

        // check tick liquidity
        let tick_lower_info = read_state(|s| {
            s.get_tick(&TickKey {
                pool_id: test_pool_id(),
                tick: test_modify_liquidity_params().tick_lower,
            })
        });

        let tick_upper_info = read_state(|s| {
            s.get_tick(&TickKey {
                pool_id: test_pool_id(),
                tick: test_modify_liquidity_params().tick_upper,
            })
        });

        println!("{:?}{:?}", tick_lower_info, tick_upper_info);

        assert_eq!(
            tick_lower_info.liquidity_gross,
            test_modify_liquidity_params().liquidity_delta as u128
        );
        assert_eq!(
            tick_upper_info.liquidity_gross,
            test_modify_liquidity_params().liquidity_delta as u128
        );
    }

    proptest! {
    #[test]
    fn test_fuzz_modify_liquidity(
        tick_lower in MIN_TICK..-1i32,
        tick_upper in 1i32..MAX_TICK,
        liquidity_delta in -1_000_000_000i64..1_000_000_000i64,
        tick_spacing in 10i32..1000i32,
    ) {
        let pool_id = test_pool_id();
        initialize_test_pool(10);



        let tick_lower = std::cmp::max((tick_lower - tick_spacing / tick_spacing) * tick_spacing,(MAX_TICK / tick_spacing) * tick_spacing);
        let tick_upper = std::cmp::min((tick_upper + tick_spacing / tick_spacing) * tick_spacing,(MIN_TICK / tick_spacing) * tick_spacing);

            println!("{},{},{}",tick_lower,tick_upper,liquidity_delta);
        if tick_lower >= tick_upper { return Ok(()); }

        let params = ModifyLiquidityParams {
            owner: Principal::management_canister(),
            pool_id:pool_id.clone(),
            tick_lower,
            tick_upper,
            liquidity_delta: liquidity_delta as i128,
            tick_spacing: PoolTickSpacing(tick_spacing),
        };

        let position_key = PositionKey {
            owner: params.owner,
            pool_id:pool_id.clone(),
            tick_lower,
            tick_upper,
        };

        // Before liquidity modification
        let position_before = read_state(|s| s.get_position(&position_key));
            println!("{:?}",position_before);
        let pool_before = read_state(|s| s.get_pool(&pool_id));
        let tick_lower_info_before = read_state(|s| s.get_tick(&TickKey { pool_id:pool_id.clone(), tick: tick_lower }));
        let tick_upper_info_before = read_state(|s| s.get_tick(&TickKey { pool_id:pool_id.clone(), tick: tick_upper }));

        if pool_before.clone().unwrap().liquidity != 0 ||  liquidity_delta > 0{
               let result = modify_liquidity(params.clone()).unwrap();
           mutate_state(|s| s.apply_modify_liquidity_buffer_state(result.buffer_state.clone()));
        }

         // After modification
        let position_after = read_state(|s| s.get_position(&position_key));
                        println!("{:?}",position_after);
        let pool_after = read_state(|s| s.get_pool(&pool_id));
        let tick_lower_info_after = read_state(|s| s.get_tick(&TickKey { pool_id:pool_id.clone(), tick: tick_lower }));
        let tick_upper_info_after = read_state(|s| s.get_tick(&TickKey { pool_id:pool_id.clone(), tick: tick_upper }));

        let is_lower_initialized=is_initialized(&TickKey { pool_id: pool_id.clone(), tick: tick_lower }, tick_spacing);
        let is_upper_initialized=is_initialized(&TickKey { pool_id: pool_id.clone(), tick: tick_upper }, tick_spacing);

        prop_assert_eq!(pool_after.clone().unwrap().tick,0);

        prop_assert_eq!(position_after.liquidity - position_before.liquidity, liquidity_delta as u128);
        prop_assert_eq!(pool_after.unwrap().liquidity - pool_before.unwrap().liquidity, liquidity_delta as u128);

        if tick_lower_info_before.liquidity_gross == 0{
            prop_assert!(is_lower_initialized);

        }
                        prop_assert_eq!(tick_lower_info_after.liquidity_gross - tick_lower_info_before.liquidity_gross, liquidity_delta as u128);

        if tick_upper_info_before.liquidity_gross == 0{
            prop_assert!(is_upper_initialized);

        }
                        prop_assert_eq!(tick_upper_info_after.liquidity_gross - tick_upper_info_before.liquidity_gross, liquidity_delta as u128);


    }}
}
