#[cfg(test)]
mod swap_args_tests {
    use crate::{
        candid_types::{
            pool::CandidPoolId,
            swap::{
                CandidPathKey, ExactInputParams, ExactInputSingleParams, ExactOutputParams,
                ExactOutputSingleParams, SwapArgs, SwapError,
            },
        },
        libraries::{
            sqrt_price_math::tests::SQRT_PRICE_1_1, swap_math::tests::SQRT_PRICE_1_2, tick_math,
        },
        pool::types::{PoolFee, PoolId},
        state::mutate_state,
        validation::swap_args::{
            validate_swap_args, ValidatedSwapArgs, MAX_PATH_LENGTH, MIN_PATH_LENGTH,
        },
    };

    use candid::{Nat, Principal};
    use ethnum::{I256, U256};
    use std::convert::TryInto;

    // Helper to create a valid CandidPoolId
    fn add_pools_to_state() {
        let pool_id_1 = CandidPoolId {
            token0: Principal::from_slice(&[1]),
            token1: Principal::from_slice(&[2]),
            fee: Nat::from(3000u32),
        };

        let pool_id_2 = CandidPoolId {
            token0: Principal::from_slice(&[2]),
            token1: Principal::from_slice(&[3]),
            fee: Nat::from(3000u32),
        };

        mutate_state(|s| {
            s.set_pool(
                pool_id_1.clone().try_into().unwrap(),
                crate::pool::types::PoolState {
                    sqrt_price_x96: *SQRT_PRICE_1_1,
                    tick: tick_math::TickMath::get_tick_at_sqrt_ratio(*SQRT_PRICE_1_1),
                    fee_growth_global_0_x128: U256::ZERO,
                    fee_growth_global_1_x128: U256::ZERO,
                    liquidity: 100000u128,
                    tick_spacing: crate::pool::types::PoolTickSpacing(10),
                    max_liquidity_per_tick: u128::MAX / 10,
                    fee_protocol: 1000,
                    token0_transfer_fee: U256::ZERO,
                    token1_transfer_fee: U256::ZERO,
                },
            )
        });

        mutate_state(|s| {
            s.set_pool(
                pool_id_2.clone().try_into().unwrap(),
                crate::pool::types::PoolState {
                    sqrt_price_x96: *SQRT_PRICE_1_2,
                    tick: tick_math::TickMath::get_tick_at_sqrt_ratio(*SQRT_PRICE_1_2),
                    fee_growth_global_0_x128: U256::ZERO,
                    fee_growth_global_1_x128: U256::ZERO,
                    liquidity: 100000u128,
                    tick_spacing: crate::pool::types::PoolTickSpacing(10),
                    max_liquidity_per_tick: u128::MAX / 10,
                    fee_protocol: 1000,
                    token1_transfer_fee: U256::ZERO,
                    token0_transfer_fee: U256::ZERO,
                },
            )
        });
    }

    fn valid_pool_id() -> CandidPoolId {
        add_pools_to_state();
        let pool_id = CandidPoolId {
            token0: Principal::from_slice(&[1]),
            token1: Principal::from_slice(&[2]),
            fee: Nat::from(3000u32),
        };

        pool_id
    }

    // Helper to create a valid CandidPathKey
    fn valid_path_key(token: Principal, fee: u32) -> CandidPathKey {
        CandidPathKey {
            intermediary_token: token,
            fee: Nat::from(fee),
        }
    }

    #[test]
    fn test_exact_input_single_valid() {
        let args = SwapArgs::ExactInputSingle(ExactInputSingleParams {
            pool_id: valid_pool_id(),
            zero_for_one: true,
            amount_in: Nat::from(1000u64),
            amount_out_minimum: Nat::from(500u64),
            from_subaccount: None,
        });

        let result = validate_swap_args(args).unwrap();
        match result {
            ValidatedSwapArgs::ExactInputSingle {
                pool_id,
                zero_for_one,
                amount_in,
                amount_out_minimum,
                from_subaccount: _,
                token_out,
                token_in,
            } => {
                assert_eq!(pool_id, valid_pool_id().try_into().unwrap());
                assert!(zero_for_one);
                assert_eq!(amount_in, I256::from(1000u64));
                assert_eq!(amount_out_minimum, I256::from(500u64));
            }
            _ => panic!("Expected ExactInputSingle"),
        }
    }

    #[test]
    fn test_exact_input_single_invalid_pool() {
        let args = SwapArgs::ExactInputSingle(ExactInputSingleParams {
            pool_id: CandidPoolId {
                token0: Principal::from_slice(&[5]),
                token1: Principal::from_slice(&[6]),
                fee: Nat::from(3000u32),
            },
            zero_for_one: true,
            amount_in: Nat::from(1000u64),
            amount_out_minimum: Nat::from(500u64),
            from_subaccount: None,
        });

        assert_eq!(validate_swap_args(args), Err(SwapError::PoolNotInitialized));
    }

    #[test]
    fn test_exact_input_valid_path() {
        add_pools_to_state();
        let token_a = Principal::from_slice(&[1]);
        let token_b = Principal::from_slice(&[2]);
        let token_c = Principal::from_slice(&[3]);

        let args = SwapArgs::ExactInput(ExactInputParams {
            token_in: token_a,
            path: vec![valid_path_key(token_b, 3000), valid_path_key(token_c, 3000)],
            amount_in: Nat::from(1000u64),
            amount_out_minimum: Nat::from(500u64),
            from_subaccount: None,
        });

        let result = validate_swap_args(args).unwrap();
        match result {
            ValidatedSwapArgs::ExactInput {
                path,
                amount_in,
                amount_out_minimum,
                from_subaccount: _,
                token_in,
                token_out,
            } => {
                assert_eq!(path.len(), 2);
                assert_eq!(amount_in, I256::from(1000u64));
                assert_eq!(amount_out_minimum, I256::from(500u64));
                assert_eq!(path[0].pool_id, valid_pool_id().try_into().unwrap());
                assert!(path[0].zero_for_one);
                assert_eq!(
                    path[1].pool_id,
                    PoolId {
                        token0: token_b,
                        token1: token_c,
                        fee: PoolFee(3000),
                    }
                );
            }
            _ => panic!("Expected ExactInput"),
        }
    }

    #[test]
    fn test_exact_input_path_too_short() {
        add_pools_to_state();
        let args = SwapArgs::ExactInput(ExactInputParams {
            token_in: Principal::from_slice(&[1]),
            path: vec![],
            amount_in: Nat::from(1000u64),
            amount_out_minimum: Nat::from(500u64),
            from_subaccount: None,
        });

        assert_eq!(
            validate_swap_args(args),
            Err(SwapError::PathLengthTooSmall {
                minimum: MIN_PATH_LENGTH,
                received: 0,
            })
        );
    }

    #[test]
    fn test_exact_input_path_too_long() {
        let token = Principal::from_slice(&[2]);
        let args = SwapArgs::ExactInput(ExactInputParams {
            token_in: Principal::from_slice(&[1]),
            path: vec![
                valid_path_key(token, 3000),
                valid_path_key(token, 3000),
                valid_path_key(token, 3000),
                valid_path_key(token, 3000),
                valid_path_key(token, 3000),
            ],
            amount_in: Nat::from(1000u64),
            amount_out_minimum: Nat::from(500u64),
            from_subaccount: None,
        });

        assert_eq!(
            validate_swap_args(args),
            Err(SwapError::PathLengthTooBig {
                maximum: MAX_PATH_LENGTH,
                received: 5,
            })
        );
    }

    #[test]
    fn test_exact_output_single_valid() {
        add_pools_to_state();

        let args = SwapArgs::ExactOutputSingle(ExactOutputSingleParams {
            pool_id: valid_pool_id(),
            zero_for_one: false,
            amount_out: Nat::from(500u64),
            amount_in_maximum: Nat::from(1000u64),
            from_subaccount: None,
        });

        let result = validate_swap_args(args).unwrap();
        match result {
            ValidatedSwapArgs::ExactOutputSingle {
                pool_id,
                zero_for_one,
                amount_out,
                amount_in_maximum,
                from_subaccount: _,
                token_out,
                token_in,
            } => {
                assert_eq!(pool_id, valid_pool_id().try_into().unwrap());
                assert!(!zero_for_one);
                assert_eq!(amount_out, I256::from(500u64));
                assert_eq!(amount_in_maximum, I256::from(1000u64));
            }
            _ => panic!("Expected ExactOutputSingle"),
        }
    }

    #[test]
    fn test_exact_output_single_invalid_fee() {
        let args = SwapArgs::ExactOutputSingle(ExactOutputSingleParams {
            pool_id: CandidPoolId {
                token0: Principal::from_slice(&[0]),
                token1: Principal::from_slice(&[1]),
                fee: Nat::from(u64::MAX), // Invalid fee
            },
            zero_for_one: false,
            amount_out: Nat::from(500u64),
            from_subaccount: None,
            amount_in_maximum: Nat::from(1000u64),
        });

        assert_eq!(validate_swap_args(args), Err(SwapError::InvalidPoolFee));
    }

    #[test]
    fn test_exact_output_valid_path() {
        add_pools_to_state();
        let token_a = Principal::from_slice(&[1]);
        let token_b = Principal::from_slice(&[2]);
        let token_c = Principal::from_slice(&[3]);

        let args = SwapArgs::ExactOutput(ExactOutputParams {
            token_out: token_c,
            path: vec![valid_path_key(token_a, 3000), valid_path_key(token_b, 3000)],
            amount_out: Nat::from(500u64),
            amount_in_maximum: Nat::from(1000u64),
            from_subaccount: None,
        });

        let result = validate_swap_args(args).unwrap();
        match result {
            ValidatedSwapArgs::ExactOutput {
                path,
                amount_out,
                amount_in_maximum,
                from_subaccount: _,
                token_in,
                token_out,
            } => {
                assert_eq!(path.len(), 2);
                assert_eq!(amount_out, I256::from(500u64));
                assert_eq!(amount_in_maximum, I256::from(1000u64));
                assert_eq!(
                    path[1].pool_id,
                    PoolId {
                        token0: token_b,
                        token1: token_c,
                        fee: PoolFee(3000),
                    }
                );
                assert_eq!(path[1].zero_for_one, false);
                assert_eq!(path[0].pool_id, valid_pool_id().try_into().unwrap());
                assert_eq!(path[0].zero_for_one, false);
            }
            _ => panic!("Expected ExactOutput"),
        }
    }

    #[test]
    fn test_exact_output_invalid_path_key() {
        let args = SwapArgs::ExactOutput(ExactOutputParams {
            token_out: Principal::from_slice(&[3]),
            path: vec![
                CandidPathKey {
                    intermediary_token: Principal::from_slice(&[2]),
                    fee: Nat::from(u64::MAX), // Invalid fee
                },
                CandidPathKey {
                    intermediary_token: Principal::from_slice(&[1]),
                    fee: Nat::from(3000_u32),
                },
            ],
            amount_out: Nat::from(500u64),
            amount_in_maximum: Nat::from(1000u64),
            from_subaccount: None,
        });

        assert_eq!(validate_swap_args(args), Err(SwapError::InvalidPoolFee));
    }
}
