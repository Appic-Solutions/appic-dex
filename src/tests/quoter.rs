pub mod quoter {
    use candid::{Nat, Principal};
    use ethnum::U256;

    // Min tick for full range with tick spacing of 60
    const MIN_TICK: i32 = -887220;
    // Max tick for full range with tick spacing of 60
    const MAX_TICK: i32 = -MIN_TICK;

    use crate::{
        candid_types::{
            pool::CreatePoolArgs,
            quote::{QuoteExactParams, QuoteExactSingleParams},
            swap::CandidPathKey,
        },
        libraries::{
            liquidity_amounts::get_liquidity_for_amounts, safe_cast::u256_to_big_uint,
            sqrt_price_math::tests::SQRT_PRICE_1_1, tick_math,
        },
        pool::{
            create_pool::create_pool_inner,
            modify_liquidity::{modify_liquidity, ModifyLiquidityParams, ModifyLiquidityState},
            types::{PoolId, PoolTickSpacing},
        },
        quote::{
            process_multi_hop_exact_input, process_multi_hop_exact_output,
            process_single_hop_exact_input, process_single_hop_exact_output,
        },
        state::mutate_state,
    };

    #[test]
    fn test_quoter_zeroforone_exact_input_single_params_multiple_positions() {
        let (_pool1, _pool12, pool2) = set_up();

        let amount_in = U256::from(10_000_u32);
        let expected_amount_out = U256::from(9871_u32);

        let params = QuoteExactSingleParams {
            pool_id: pool2.into(),
            zero_for_one: true,
            exact_amount: Nat::from(u256_to_big_uint(amount_in)),
        };

        let amount_out = process_single_hop_exact_input(params).unwrap();
        assert_eq!(amount_out, expected_amount_out);
    }

    #[test]
    fn test_quoter_oneforzero_exact_input_single_params_multiple_positions() {
        let (_pool1, _pool12, pool2) = set_up();

        let amount_in = U256::from(10_000_u32);
        let expected_amount_out = U256::from(9871_u32);

        let params = QuoteExactSingleParams {
            pool_id: pool2.into(),
            zero_for_one: false,
            exact_amount: Nat::from(u256_to_big_uint(amount_in)),
        };

        let amount_out = process_single_hop_exact_input(params).unwrap();

        assert_eq!(amount_out, expected_amount_out);
    }

    #[test]
    fn test_quoter_exact_input_0to2_2ticks_loaded() {
        let (_pool1, _pool12, _pool2) = set_up();

        let path = vec![CandidPathKey {
            intermediary_token: generate_token_address(2),
            fee: Nat::from(3000_u32),
        }];

        let amount_in = U256::from(10_000_u32);
        let expected_amount_out = U256::from(9871_u32);

        let params = QuoteExactParams {
            exact_amount: Nat::from(u256_to_big_uint(amount_in)),
            exact_token: generate_token_address(0),
            path,
        };

        let amount_out = process_multi_hop_exact_input(params).unwrap();

        assert_eq!(amount_out, expected_amount_out);
    }

    #[test]
    fn test_quoter_exact_input_0to2_2ticks_loaded_initialized_after() {
        let (_pool1, _pool12, _pool2) = set_up();

        let path = vec![CandidPathKey {
            intermediary_token: generate_token_address(2),
            fee: Nat::from(3000_u32),
        }];

        let amount_in = U256::from(6200_u32);
        let expected_amount_out = U256::from(6143_u32);

        let params = QuoteExactParams {
            exact_amount: Nat::from(u256_to_big_uint(amount_in)),
            exact_token: generate_token_address(0),
            path,
        };

        let amount_out = process_multi_hop_exact_input(params).unwrap();

        assert_eq!(amount_out, expected_amount_out);
    }

    #[test]
    fn test_quoter_exact_input_0to2_1tick_loaded() {
        let (_pool1, _pool12, _pool2) = set_up();

        let path = vec![CandidPathKey {
            intermediary_token: generate_token_address(2),
            fee: Nat::from(3000_u32),
        }];

        let amount_in = U256::from(4000_u32);
        let expected_amount_out = U256::from(3971_u32);

        let params = QuoteExactParams {
            exact_amount: Nat::from(u256_to_big_uint(amount_in)),
            exact_token: generate_token_address(0),
            path,
        };

        let amount_out = process_multi_hop_exact_input(params).unwrap();

        assert_eq!(amount_out, expected_amount_out);
    }

    #[test]
    fn test_quoter_exact_input_0to2_0tick_loaded() {
        let (_pool1, _pool12, _pool2) = set_up();

        let path = vec![CandidPathKey {
            intermediary_token: generate_token_address(2),
            fee: Nat::from(3000_u32),
        }];

        let amount_in = U256::from(10_u32);
        let expected_amount_out = U256::from(8_u32);

        let params = QuoteExactParams {
            exact_amount: Nat::from(u256_to_big_uint(amount_in)),
            exact_token: generate_token_address(0),
            path,
        };

        let amount_out = process_multi_hop_exact_input(params).unwrap();

        assert_eq!(amount_out, expected_amount_out);
    }

    #[test]
    fn test_quoter_exact_input_0to2_0tick_loaded_starting_initialized() {
        // set 60 as tick spacing for 3000 bips fee
        mutate_state(|s| {
            s.set_tick_spacing(
                crate::pool::types::PoolFee(3000),
                crate::pool::types::PoolTickSpacing(60),
            )
        });

        let pool_id = create_pool(CreatePoolArgs {
            token_a: generate_token_address(0),
            token_b: generate_token_address(2),
            fee: Nat::from(3000_u32),
            sqrt_price_x96: Nat::from(u256_to_big_uint(*SQRT_PRICE_1_1)),
        });
        set_up_pool_with_0_ticks_initialized(pool_id);

        let path = vec![CandidPathKey {
            intermediary_token: generate_token_address(2),
            fee: Nat::from(3000_u32),
        }];

        let amount_in = U256::from(10_u32);
        let expected_amount_out = U256::from(8_u32);

        let params = QuoteExactParams {
            exact_amount: Nat::from(u256_to_big_uint(amount_in)),
            exact_token: generate_token_address(0),
            path,
        };

        let amount_out = process_multi_hop_exact_input(params).unwrap();

        assert_eq!(amount_out, expected_amount_out);
    }

    #[test]
    fn test_quoter_exact_input_2to0() {
        let (_pool1, _pool12, _pool2) = set_up();

        let path = vec![CandidPathKey {
            intermediary_token: generate_token_address(0),
            fee: Nat::from(3000_u32),
        }];

        let amount_in = U256::from(10_000_u32);
        let expected_amount_out = U256::from(9871_u32);

        let params = QuoteExactParams {
            exact_amount: Nat::from(u256_to_big_uint(amount_in)),
            exact_token: generate_token_address(2),
            path,
        };

        let amount_out = process_multi_hop_exact_input(params).unwrap();

        assert_eq!(amount_out, expected_amount_out);
    }

    #[test]
    fn test_quoter_exact_input_2to0_2ticks_loaded() {
        let (_pool1, _pool12, _pool2) = set_up();

        let path = vec![CandidPathKey {
            intermediary_token: generate_token_address(0),
            fee: Nat::from(3000_u32),
        }];

        let amount_in = U256::from(6250_u32);
        let expected_amount_out = U256::from(6190_u32);

        let params = QuoteExactParams {
            exact_amount: Nat::from(u256_to_big_uint(amount_in)),
            exact_token: generate_token_address(2),
            path,
        };

        let amount_out = process_multi_hop_exact_input(params).unwrap();

        assert_eq!(amount_out, expected_amount_out);
    }

    #[test]
    fn test_quoter_exact_input_2to0_0tick_loaded_starting_initialized() {
        // set 60 as tick spacing for 3000 bips fee
        mutate_state(|s| {
            s.set_tick_spacing(
                crate::pool::types::PoolFee(3000),
                crate::pool::types::PoolTickSpacing(60),
            )
        });

        let pool_id = create_pool(CreatePoolArgs {
            token_a: generate_token_address(0),
            token_b: generate_token_address(2),
            fee: Nat::from(3000_u32),
            sqrt_price_x96: Nat::from(u256_to_big_uint(*SQRT_PRICE_1_1)),
        });
        set_up_pool_with_0_ticks_initialized(pool_id);

        let path = vec![CandidPathKey {
            intermediary_token: generate_token_address(0),
            fee: Nat::from(3000_u32),
        }];

        let amount_in = U256::from(200_u32);
        let expected_amount_out = U256::from(198_u32);

        let params = QuoteExactParams {
            exact_amount: Nat::from(u256_to_big_uint(amount_in)),
            exact_token: generate_token_address(2),
            path,
        };

        let amount_out = process_multi_hop_exact_input(params).unwrap();

        assert_eq!(amount_out, expected_amount_out);
    }

    #[test]
    fn test_quoter_exact_input_2to0_0ticks_loaded_starting_not_initialized() {
        let (_pool1, _pool12, _pool2) = set_up();

        let path = vec![CandidPathKey {
            intermediary_token: generate_token_address(0),
            fee: Nat::from(3000_u32),
        }];

        let amount_in = U256::from(103_u32);
        let expected_amount_out = U256::from(101_u32);

        let params = QuoteExactParams {
            exact_amount: Nat::from(u256_to_big_uint(amount_in)),
            exact_token: generate_token_address(2),
            path,
        };

        let amount_out = process_multi_hop_exact_input(params).unwrap();

        assert_eq!(amount_out, expected_amount_out);
    }

    #[test]
    fn test_quoter_exact_input_2to1() {
        let (_pool1, _pool12, _pool2) = set_up();

        let path = vec![CandidPathKey {
            intermediary_token: generate_token_address(1),
            fee: Nat::from(3000_u32),
        }];

        let amount_in = U256::from(10000_u32);
        let expected_amount_out = U256::from(9871_u32);

        let params = QuoteExactParams {
            exact_amount: Nat::from(u256_to_big_uint(amount_in)),
            exact_token: generate_token_address(2),
            path,
        };

        let amount_out = process_multi_hop_exact_input(params).unwrap();

        assert_eq!(amount_out, expected_amount_out);
    }

    #[test]
    fn test_quoter_exact_input_0to2to1() {
        let (_pool1, _pool12, _pool2) = set_up();

        let path = vec![
            CandidPathKey {
                intermediary_token: generate_token_address(2),
                fee: Nat::from(3000_u32),
            },
            CandidPathKey {
                intermediary_token: generate_token_address(1),
                fee: Nat::from(3000_u32),
            },
        ];

        let amount_in = U256::from(10000_u32);
        let expected_amount_out = U256::from(9745_u32);

        let params = QuoteExactParams {
            exact_amount: Nat::from(u256_to_big_uint(amount_in)),
            exact_token: generate_token_address(0),
            path,
        };

        let amount_out = process_multi_hop_exact_input(params).unwrap();

        assert_eq!(amount_out, expected_amount_out);
    }

    #[test]
    fn test_quoter_zeroforone_exact_output_single_params() {
        let (pool1, _pool12, _pool2) = set_up();

        let amount_out = U256::from(10_000_u32);
        let expected_amount_in = U256::from(10133_u32);

        let params = QuoteExactSingleParams {
            pool_id: pool1.into(),
            zero_for_one: true,
            exact_amount: Nat::from(u256_to_big_uint(amount_out)),
        };

        let amount_in = process_single_hop_exact_output(params).unwrap();

        assert_eq!(amount_in, expected_amount_in);
    }

    #[test]
    fn test_quoter_oneforzero_exact_output_single_params() {
        let (pool1, _pool12, _pool2) = set_up();

        let amount_out = U256::from(10_000_u32);
        let expected_amount_in = U256::from(10133_u32);

        let params = QuoteExactSingleParams {
            pool_id: pool1.into(),
            zero_for_one: false,
            exact_amount: Nat::from(u256_to_big_uint(amount_out)),
        };

        let amount_in = process_single_hop_exact_output(params).unwrap();

        assert_eq!(amount_in, expected_amount_in);
    }

    #[test]
    fn test_quoter_zeroforone_exact_output_single_params_pool2() {
        let (_pool1, _pool12, pool2) = set_up();

        let amount_out = U256::from(15_000_u32);
        let expected_amount_in = U256::from(15273_u32);

        let params = QuoteExactSingleParams {
            pool_id: pool2.into(),
            zero_for_one: true,
            exact_amount: Nat::from(u256_to_big_uint(amount_out)),
        };

        let amount_in = process_single_hop_exact_output(params).unwrap();

        assert_eq!(amount_in, expected_amount_in);
    }

    #[test]
    fn test_quoter_exact_output_0to2_2ticks_loaded() {
        let (_pool1, _pool12, _pool2) = set_up();

        let path = vec![CandidPathKey {
            intermediary_token: generate_token_address(0),
            fee: Nat::from(3000_u32),
        }];

        let amount_out = U256::from(15_000u32);
        let expected_amount_in = U256::from(15273_u32);

        let params = QuoteExactParams {
            exact_amount: Nat::from(u256_to_big_uint(amount_out)),
            exact_token: generate_token_address(2),
            path,
        };

        let amount_in = process_multi_hop_exact_output(params).unwrap();

        assert_eq!(amount_in, expected_amount_in);
    }

    #[test]
    fn test_quoter_exact_output_0to2_1ticks_loaded_initialized_after() {
        let (_pool1, _pool12, _pool2) = set_up();

        let path = vec![CandidPathKey {
            intermediary_token: generate_token_address(0),
            fee: Nat::from(3000_u32),
        }];

        let amount_out = U256::from(6143u32);
        let expected_amount_in = U256::from(6200_u32);

        let params = QuoteExactParams {
            exact_amount: Nat::from(u256_to_big_uint(amount_out)),
            exact_token: generate_token_address(2),
            path,
        };

        let amount_in = process_multi_hop_exact_output(params).unwrap();

        assert_eq!(amount_in, expected_amount_in);
    }

    #[test]
    fn test_quoter_exact_output_0to2_1ticks_loaded() {
        let (_pool1, _pool12, _pool2) = set_up();

        let path = vec![CandidPathKey {
            intermediary_token: generate_token_address(0),
            fee: Nat::from(3000_u32),
        }];

        let amount_out = U256::from(4000_u32);
        let expected_amount_in = U256::from(4029_u32);

        let params = QuoteExactParams {
            exact_amount: Nat::from(u256_to_big_uint(amount_out)),
            exact_token: generate_token_address(2),
            path,
        };

        let amount_in = process_multi_hop_exact_output(params).unwrap();

        assert_eq!(amount_in, expected_amount_in);
    }

    #[test]
    fn test_quoter_exact_output_0to2_0tick_loaded_starting_initialized() {
        // set 60 as tick spacing for 3000 bips fee
        mutate_state(|s| {
            s.set_tick_spacing(
                crate::pool::types::PoolFee(3000),
                crate::pool::types::PoolTickSpacing(60),
            )
        });

        let pool_id = create_pool(CreatePoolArgs {
            token_a: generate_token_address(0),
            token_b: generate_token_address(2),
            fee: Nat::from(3000_u32),
            sqrt_price_x96: Nat::from(u256_to_big_uint(*SQRT_PRICE_1_1)),
        });
        set_up_pool_with_0_ticks_initialized(pool_id);

        let path = vec![CandidPathKey {
            intermediary_token: generate_token_address(0),
            fee: Nat::from(3000_u32),
        }];

        let amount_out = U256::from(100_u32);
        let expected_amount_in = U256::from(102_u32);

        let params = QuoteExactParams {
            exact_amount: Nat::from(u256_to_big_uint(amount_out)),
            exact_token: generate_token_address(2),
            path,
        };

        let amount_in = process_multi_hop_exact_output(params).unwrap();

        assert_eq!(amount_in, expected_amount_in);
    }

    #[test]
    fn test_quoter_exact_output_0to2_0tick_loaded_starting_not_initialized() {
        let (_pool1, _pool12, _pool2) = set_up();

        let path = vec![CandidPathKey {
            intermediary_token: generate_token_address(0),
            fee: Nat::from(3000_u32),
        }];

        let amount_out = U256::from(10_u32);
        let expected_amount_in = U256::from(12_u32);

        let params = QuoteExactParams {
            exact_amount: Nat::from(u256_to_big_uint(amount_out)),
            exact_token: generate_token_address(2),
            path,
        };

        let amount_in = process_multi_hop_exact_output(params).unwrap();

        assert_eq!(amount_in, expected_amount_in);
    }

    #[test]
    fn test_quoter_exact_output_2to0_2ticks_loaded() {
        let (_pool1, _pool12, _pool2) = set_up();

        let path = vec![CandidPathKey {
            intermediary_token: generate_token_address(2),
            fee: Nat::from(3000_u32),
        }];

        let amount_out = U256::from(15_000u32);
        let expected_amount_in = U256::from(15273_u32);

        let params = QuoteExactParams {
            exact_amount: Nat::from(u256_to_big_uint(amount_out)),
            exact_token: generate_token_address(0),
            path,
        };

        let amount_in = process_multi_hop_exact_output(params).unwrap();

        assert_eq!(amount_in, expected_amount_in);
    }

    #[test]
    fn test_quoter_exact_output_2to0_1tick_loaded() {
        let (_pool1, _pool12, _pool2) = set_up();

        let path = vec![CandidPathKey {
            intermediary_token: generate_token_address(2),
            fee: Nat::from(3000_u32),
        }];

        let amount_out = U256::from(6000u32);
        let expected_amount_in = U256::from(6055_u32);

        let params = QuoteExactParams {
            exact_amount: Nat::from(u256_to_big_uint(amount_out)),
            exact_token: generate_token_address(0),
            path,
        };

        let amount_in = process_multi_hop_exact_output(params).unwrap();

        assert_eq!(amount_in, expected_amount_in);
    }

    #[test]
    fn test_quoter_exact_output_2to1() {
        let (_pool1, _pool12, _pool2) = set_up();

        let path = vec![CandidPathKey {
            intermediary_token: generate_token_address(2),
            fee: Nat::from(3000_u32),
        }];

        let amount_out = U256::from(9871_u32);
        let expected_amount_in = U256::from(10000_u32);

        let params = QuoteExactParams {
            exact_amount: Nat::from(u256_to_big_uint(amount_out)),
            exact_token: generate_token_address(1),
            path,
        };

        let amount_in = process_multi_hop_exact_output(params).unwrap();

        assert_eq!(amount_in, expected_amount_in);
    }

    #[test]
    fn test_quoter_exact_output_0to2to1() {
        let (_pool1, _pool12, _pool2) = set_up();

        let path = vec![
            CandidPathKey {
                intermediary_token: generate_token_address(0),
                fee: Nat::from(3000_u32),
            },
            CandidPathKey {
                intermediary_token: generate_token_address(2),
                fee: Nat::from(3000_u32),
            },
        ];

        let amount_out = U256::from(9745_u32);
        let expected_amount_in = U256::from(10000_u32);

        let params = QuoteExactParams {
            exact_amount: Nat::from(u256_to_big_uint(amount_out)),
            exact_token: generate_token_address(1),
            path,
        };

        let amount_in = process_multi_hop_exact_output(params).unwrap();

        assert_eq!(amount_in, expected_amount_in);
    }

    pub fn generate_token_address(token_id: u8) -> Principal {
        Principal::from_slice(&[token_id])
    }

    pub fn set_up() -> (PoolId, PoolId, PoolId) {
        // set 60 as tick spacing for 3000 bips fee
        mutate_state(|s| {
            s.set_tick_spacing(
                crate::pool::types::PoolFee(3000),
                crate::pool::types::PoolTickSpacing(60),
            )
        });

        let token_0 = Principal::from_slice(&[0]);
        let token_1 = Principal::from_slice(&[1]);
        let token_2 = Principal::from_slice(&[2]);

        // pool 1
        let pool1 = create_pool(CreatePoolArgs {
            token_a: token_0,
            token_b: token_1,
            fee: Nat::from(3000_u32),
            sqrt_price_x96: Nat::from(u256_to_big_uint(*SQRT_PRICE_1_1)),
        });

        // pool 2
        let pool2 = create_pool(CreatePoolArgs {
            token_a: token_0,
            token_b: token_2,
            fee: Nat::from(3000_u32),
            sqrt_price_x96: Nat::from(u256_to_big_uint(*SQRT_PRICE_1_1)),
        });

        // pool 12
        let pool12 = create_pool(CreatePoolArgs {
            token_a: token_1,
            token_b: token_2,
            fee: Nat::from(3000_u32),
            sqrt_price_x96: Nat::from(u256_to_big_uint(*SQRT_PRICE_1_1)),
        });

        set_up_pool(pool1.clone());
        set_up_pool(pool12.clone());
        set_up_pool_multiple_positions(pool2.clone());

        (pool1, pool12, pool2)
    }

    pub fn create_pool(args: CreatePoolArgs) -> PoolId {
        create_pool_inner(args).expect("Pool creation failed")
    }

    pub fn set_up_pool(pool_id: PoolId) {
        let modify_liquidity_params = ModifyLiquidityParams {
            owner: Principal::management_canister(),
            pool_id,
            tick_lower: MIN_TICK,
            tick_upper: MAX_TICK,
            liquidity_delta: get_liquidity_for_amounts(
                *SQRT_PRICE_1_1,
                tick_math::TickMath::get_sqrt_ratio_at_tick(MIN_TICK),
                tick_math::TickMath::get_sqrt_ratio_at_tick(MAX_TICK),
                U256::from(1_000_000_u128),
                U256::from(1_000_000_u128),
            )
            .expect("Failed to get liquidity") as i128,
            tick_spacing: PoolTickSpacing(60),
        };
        let modifiy_liquidity_result =
            modify_liquidity(modify_liquidity_params).expect("Failed to modify liquidity");

        mutate_state(|s| {
            s.apply_modify_liquidity_buffer_state(modifiy_liquidity_result.buffer_state)
        });
    }

    pub fn set_up_pool_multiple_positions(pool_id: PoolId) {
        let modify_liquidity_params = ModifyLiquidityParams {
            owner: Principal::management_canister(),
            pool_id: pool_id.clone(),
            tick_lower: MIN_TICK,
            tick_upper: MAX_TICK,
            liquidity_delta: get_liquidity_for_amounts(
                *SQRT_PRICE_1_1,
                tick_math::TickMath::get_sqrt_ratio_at_tick(MIN_TICK),
                tick_math::TickMath::get_sqrt_ratio_at_tick(MAX_TICK),
                U256::from(1_000_000_u128),
                U256::from(1_000_000_u128),
            )
            .expect("Failed to get liquidity") as i128,
            tick_spacing: PoolTickSpacing(60),
        };

        let modifiy_liquidity_result =
            modify_liquidity(modify_liquidity_params).expect("Failed to modify liquidity");

        mutate_state(|s| {
            s.apply_modify_liquidity_buffer_state(modifiy_liquidity_result.buffer_state)
        });

        let modify_liquidity_params_60 = ModifyLiquidityParams {
            owner: Principal::management_canister(),
            pool_id: pool_id.clone(),
            tick_lower: -60,
            tick_upper: 60,
            liquidity_delta: get_liquidity_for_amounts(
                *SQRT_PRICE_1_1,
                tick_math::TickMath::get_sqrt_ratio_at_tick(-60),
                tick_math::TickMath::get_sqrt_ratio_at_tick(60),
                U256::from(100_u128),
                U256::from(100_u128),
            )
            .expect("Failed to get liquidity") as i128,
            tick_spacing: PoolTickSpacing(60),
        };

        let modifiy_liquidity_result =
            modify_liquidity(modify_liquidity_params_60).expect("Failed to modify liquidity");

        mutate_state(|s| {
            s.apply_modify_liquidity_buffer_state(modifiy_liquidity_result.buffer_state)
        });

        let modify_liquidity_params_120 = ModifyLiquidityParams {
            owner: Principal::management_canister(),
            pool_id: pool_id.clone(),
            tick_lower: -120,
            tick_upper: 120,
            liquidity_delta: get_liquidity_for_amounts(
                *SQRT_PRICE_1_1,
                tick_math::TickMath::get_sqrt_ratio_at_tick(-120),
                tick_math::TickMath::get_sqrt_ratio_at_tick(120),
                U256::from(100_u128),
                U256::from(100_u128),
            )
            .expect("Failed to get liquidity") as i128,
            tick_spacing: PoolTickSpacing(60),
        };

        let modifiy_liquidity_result =
            modify_liquidity(modify_liquidity_params_120).expect("Failed to modify liquidity");

        mutate_state(|s| {
            s.apply_modify_liquidity_buffer_state(modifiy_liquidity_result.buffer_state)
        });
    }

    pub fn set_up_pool_with_0_ticks_initialized(pool_id: PoolId) {
        let modify_liquidity_params = ModifyLiquidityParams {
            owner: Principal::management_canister(),
            pool_id: pool_id.clone(),
            tick_lower: MIN_TICK,
            tick_upper: MAX_TICK,
            liquidity_delta: get_liquidity_for_amounts(
                *SQRT_PRICE_1_1,
                tick_math::TickMath::get_sqrt_ratio_at_tick(MIN_TICK),
                tick_math::TickMath::get_sqrt_ratio_at_tick(MAX_TICK),
                U256::from(1_000_000_u128),
                U256::from(1_000_000_u128),
            )
            .expect("Failed to get liquidity") as i128,
            tick_spacing: PoolTickSpacing(60),
        };

        let modifiy_liquidity_result =
            modify_liquidity(modify_liquidity_params).expect("Failed to modify liquidity");

        mutate_state(|s| {
            s.apply_modify_liquidity_buffer_state(modifiy_liquidity_result.buffer_state)
        });

        let modify_liquidity_params_60 = ModifyLiquidityParams {
            owner: Principal::management_canister(),
            pool_id: pool_id.clone(),
            tick_lower: -60,
            tick_upper: 60,
            liquidity_delta: get_liquidity_for_amounts(
                *SQRT_PRICE_1_1,
                tick_math::TickMath::get_sqrt_ratio_at_tick(0),
                tick_math::TickMath::get_sqrt_ratio_at_tick(60),
                U256::from(100_u128),
                U256::from(100_u128),
            )
            .expect("Failed to get liquidity") as i128,
            tick_spacing: PoolTickSpacing(60),
        };

        let modifiy_liquidity_result =
            modify_liquidity(modify_liquidity_params_60).expect("Failed to modify liquidity");

        mutate_state(|s| {
            s.apply_modify_liquidity_buffer_state(modifiy_liquidity_result.buffer_state)
        });

        let modify_liquidity_params_120 = ModifyLiquidityParams {
            owner: Principal::management_canister(),
            pool_id: pool_id.clone(),
            tick_lower: -120,
            tick_upper: 120,
            liquidity_delta: get_liquidity_for_amounts(
                *SQRT_PRICE_1_1,
                tick_math::TickMath::get_sqrt_ratio_at_tick(-120),
                tick_math::TickMath::get_sqrt_ratio_at_tick(0),
                U256::from(100_u128),
                U256::from(100_u128),
            )
            .expect("Failed to get liquidity") as i128,
            tick_spacing: PoolTickSpacing(60),
        };

        let modifiy_liquidity_result =
            modify_liquidity(modify_liquidity_params_120).expect("Failed to modify liquidity");

        mutate_state(|s| {
            s.apply_modify_liquidity_buffer_state(modifiy_liquidity_result.buffer_state)
        });
    }
}
