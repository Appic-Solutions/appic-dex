use crate::{
    candid_types::{
        pool::CandidPoolState,
        swap::{CandidPathKey, CandidSwapSuccess, SwapArgs, SwapError, SwapFailedReason},
    },
    cbor::u256,
    libraries::sqrt_price_math::tests::ONE_ETHER,
};

use super::*;

///////////////////////////////////////////////////////////////
// EXACT INPUT
///////////////////////////////////////////////////////////////
#[test]
fn test_exact_input_single_amount_fails_for_amount_out() {
    let pic = set_up();

    five_ticks(&pic);
    five_ticks(&pic);

    let amount_in = *ONE_ETHER;
    let expected_amount_out = U256::from(992054607780215625_u128);

    let swap_args = SwapArgs::ExactInputSingle(crate::candid_types::swap::ExactInputSingleParams {
        pool_id: CandidPoolId {
            token0: token0_principal(),
            token1: token1_principal(),
            fee: Nat::from(3000_u32),
        },
        zero_for_one: true,
        amount_in: u256_to_nat(amount_in),
        amount_out_minimum: u256_to_nat(expected_amount_out + 1),
        from_subaccount: None,
    });

    let swap_result = update_call::<SwapArgs, Result<CandidSwapSuccess, SwapError>>(
        &pic,
        appic_dex_canister_id(),
        "swap",
        swap_args,
        Some(sender_principal()),
    );

    five_ticks(&pic);
    five_ticks(&pic);

    println!("{:?}", swap_result);
    assert_eq!(
        swap_result,
        Err(SwapError::SwapFailedRefunded {
            failed_reason: SwapFailedReason::TooLittleReceived,
            refund_error: None,
            refund_amount: Some(Nat::from(999990000000000000_u128)),
        })
    );
}

#[test]
fn test_exact_input_single_amount() {
    let pic = set_up();

    five_ticks(&pic);
    five_ticks(&pic);

    // balances brefore swap
    let pool_balance_before_token0 = get_balance(&pic, token0_principal(), appic_dex_canister_id());
    let pool_balance_before_token1 = get_balance(&pic, token1_principal(), appic_dex_canister_id());

    let user_balance_before_token0 = get_balance(&pic, token0_principal(), sender_principal());
    let user_balance_before_token1 = get_balance(&pic, token1_principal(), sender_principal());

    //
    //println!(
    //    "before swap balances, pool_token0 {:?}, pool_token1 {:?}, user_token0 {:?}, user_token1 {:?}",
    //    pool_balance_before_token0,
    //    pool_balance_before_token1,
    //    user_balance_before_token0,
    //    user_balance_before_token1
    //);

    let amount_in = *ONE_ETHER;
    let expected_amount_out = U256::from(992054607780215625_u128);

    let swap_args = SwapArgs::ExactInputSingle(crate::candid_types::swap::ExactInputSingleParams {
        pool_id: CandidPoolId {
            token0: token0_principal(),
            token1: token1_principal(),
            fee: Nat::from(3000_u32),
        },
        zero_for_one: true,
        amount_in: u256_to_nat(amount_in),
        amount_out_minimum: u256_to_nat(expected_amount_out),
        from_subaccount: None,
    });

    // pool state after swap
    let pool_state_before = query_call::<CandidPoolId, Option<CandidPoolState>>(
        &pic,
        appic_dex_canister_id(),
        "get_pool",
        CandidPoolId {
            token0: token0_principal(),
            token1: token1_principal(),
            fee: Nat::from(3000_u32),
        },
    )
    .unwrap();

    let swap_result = update_call::<SwapArgs, Result<CandidSwapSuccess, SwapError>>(
        &pic,
        appic_dex_canister_id(),
        "swap",
        swap_args,
        Some(sender_principal()),
    )
    .unwrap();

    five_ticks(&pic);
    five_ticks(&pic);

    // pool state after swap
    let pool_state_after = query_call::<CandidPoolId, Option<CandidPoolState>>(
        &pic,
        appic_dex_canister_id(),
        "get_pool",
        CandidPoolId {
            token0: token0_principal(),
            token1: token1_principal(),
            fee: Nat::from(3000_u32),
        },
    )
    .unwrap();

    //println!(
    //    "state before {:?} state after {:?}",
    //    pool_state_before, pool_state_after
    //);

    // balances after swap
    let pool_balance_after_token0 = get_balance(&pic, token0_principal(), appic_dex_canister_id());
    let pool_balance_after_token1 = get_balance(&pic, token1_principal(), appic_dex_canister_id());

    let user_balance_after_token0 = get_balance(&pic, token0_principal(), sender_principal());
    let user_balance_after_token1 = get_balance(&pic, token1_principal(), sender_principal());

    //println!(
    //    "after swap balances, pool_token0 {:?}, pool_token1 {:?}, user_token0 {:?}, user_token1 {:?}",
    //    pool_balance_after_token0,
    //    pool_balance_after_token1,
    //    user_balance_after_token0,
    //    user_balance_after_token1
    //);

    assert_eq!(
        pool_state_before.pool_reserves0 + u256_to_nat(amount_in),
        pool_state_after.pool_reserves0
    );
    assert_eq!(
        pool_state_before.pool_reserves1 - u256_to_nat(expected_amount_out),
        pool_state_after.pool_reserves1
    );

    assert_eq!(
        pool_state_after.swap_volume0_all_time,
        u256_to_nat(amount_in)
    );

    assert_eq!(
        pool_balance_before_token0 + u256_to_nat(amount_in.clone()),
        pool_balance_after_token0
    );

    assert_eq!(
        pool_balance_before_token1 - swap_result.amount_out.clone(),
        pool_balance_after_token1
    );

    assert_eq!(
        user_balance_before_token0 - u256_to_nat(amount_in.clone()) - Nat::from(TOKEN_TRANSFER_FEE),
        user_balance_after_token0
    );

    assert_eq!(
        user_balance_before_token1 + swap_result.amount_out.clone() - Nat::from(TOKEN_TRANSFER_FEE),
        user_balance_after_token1
    );
}

#[test]
fn test_exact_input_1_hop_one_for_zero() {
    let pic = set_up();

    five_ticks(&pic);
    five_ticks(&pic);

    let amount_in = *ONE_ETHER;
    let expected_amount_out = U256::from(992054607780215625_u128);

    let swap_args = SwapArgs::ExactInput(crate::candid_types::swap::ExactInputParams {
        amount_in: u256_to_nat(amount_in),
        amount_out_minimum: u256_to_nat(expected_amount_out),
        from_subaccount: None,
        token_in: token0_principal(),
        path: vec![CandidPathKey {
            intermediary_token: token1_principal(),
            fee: Nat::from(3000_u32),
        }],
    });

    let swap_result = update_call::<SwapArgs, Result<CandidSwapSuccess, SwapError>>(
        &pic,
        appic_dex_canister_id(),
        "swap",
        swap_args,
        Some(sender_principal()),
    );

    five_ticks(&pic);
    five_ticks(&pic);

    println!("{:?}", swap_result);
    assert_eq!(
        swap_result,
        Ok(CandidSwapSuccess {
            amount_in: u256_to_nat(amount_in),
            amount_out: u256_to_nat(expected_amount_out)
        })
    );
}

#[test]
fn test_exact_input_2_hops() {
    let pic = set_up();

    five_ticks(&pic);
    five_ticks(&pic);

    let amount_in = *ONE_ETHER;
    let expected_amount_out = U256::from(984211133872795298_u128);

    let swap_args = SwapArgs::ExactInput(crate::candid_types::swap::ExactInputParams {
        amount_in: u256_to_nat(amount_in),
        amount_out_minimum: u256_to_nat(expected_amount_out),
        from_subaccount: None,
        token_in: token0_principal(),
        path: vec![
            CandidPathKey {
                intermediary_token: token1_principal(),
                fee: Nat::from(3000_u32),
            },
            CandidPathKey {
                intermediary_token: token2_principal(),
                fee: Nat::from(3000_u32),
            },
        ],
    });

    let swap_result = update_call::<SwapArgs, Result<CandidSwapSuccess, SwapError>>(
        &pic,
        appic_dex_canister_id(),
        "swap",
        swap_args,
        Some(sender_principal()),
    );

    five_ticks(&pic);
    five_ticks(&pic);

    println!("{:?}", swap_result);
    assert_eq!(
        swap_result,
        Ok(CandidSwapSuccess {
            amount_in: u256_to_nat(amount_in),
            amount_out: u256_to_nat(expected_amount_out)
        })
    );
}

#[test]
fn test_exact_input_3_hops() {
    let pic = set_up();

    five_ticks(&pic);
    five_ticks(&pic);

    let amount_in = *ONE_ETHER;
    let expected_amount_out = U256::from(976467664490096191_u128);

    let swap_args = SwapArgs::ExactInput(crate::candid_types::swap::ExactInputParams {
        amount_in: u256_to_nat(amount_in),
        amount_out_minimum: u256_to_nat(expected_amount_out),
        from_subaccount: None,
        token_in: token0_principal(),
        path: vec![
            CandidPathKey {
                intermediary_token: token1_principal(),
                fee: Nat::from(3000_u32),
            },
            CandidPathKey {
                intermediary_token: token2_principal(),
                fee: Nat::from(3000_u32),
            },
            CandidPathKey {
                intermediary_token: token3_principal(),
                fee: Nat::from(3000_u32),
            },
        ],
    });

    let swap_result = update_call::<SwapArgs, Result<CandidSwapSuccess, SwapError>>(
        &pic,
        appic_dex_canister_id(),
        "swap",
        swap_args,
        Some(sender_principal()),
    );

    five_ticks(&pic);
    five_ticks(&pic);

    println!("{:?}", swap_result);
    assert_eq!(
        swap_result,
        Ok(CandidSwapSuccess {
            amount_in: u256_to_nat(amount_in),
            amount_out: u256_to_nat(expected_amount_out)
        })
    );
}

///////////////////////////////////////////////////////////////
// EXACT OUTPUT
///////////////////////////////////////////////////////////////
#[test]
fn test_exact_output_single_amount_fails_for_amount_in() {
    let pic = set_up();

    five_ticks(&pic);
    five_ticks(&pic);

    let amount_out = *ONE_ETHER;
    let expected_amount_in = U256::from(1008049273448486163_u128);

    let swap_args =
        SwapArgs::ExactOutputSingle(crate::candid_types::swap::ExactOutputSingleParams {
            pool_id: CandidPoolId {
                token0: token0_principal(),
                token1: token1_principal(),
                fee: Nat::from(3000_u32),
            },
            zero_for_one: true,
            amount_out: u256_to_nat(amount_out),
            amount_in_maximum: u256_to_nat(expected_amount_in - 1),
            from_subaccount: None,
        });

    let swap_result = update_call::<SwapArgs, Result<CandidSwapSuccess, SwapError>>(
        &pic,
        appic_dex_canister_id(),
        "swap",
        swap_args,
        Some(sender_principal()),
    );

    five_ticks(&pic);
    five_ticks(&pic);

    println!("{:?}", swap_result);
    assert_eq!(
        swap_result,
        Err(SwapError::SwapFailedRefunded {
            failed_reason: SwapFailedReason::TooMuchRequested,
            refund_error: None,
            refund_amount: Some(Nat::from(1008039273448486162_u128)),
        })
    );
}

#[test]
fn test_exact_output_single_amount() {
    let pic = set_up();

    five_ticks(&pic);
    five_ticks(&pic);

    let amount_out = *ONE_ETHER;
    let expected_amount_in = U256::from(1008049273448486163_u128);

    let swap_args =
        SwapArgs::ExactOutputSingle(crate::candid_types::swap::ExactOutputSingleParams {
            pool_id: CandidPoolId {
                token0: token0_principal(),
                token1: token1_principal(),
                fee: Nat::from(3000_u32),
            },
            zero_for_one: true,
            amount_out: u256_to_nat(amount_out),
            amount_in_maximum: u256_to_nat(expected_amount_in),
            from_subaccount: None,
        });

    let swap_result = update_call::<SwapArgs, Result<CandidSwapSuccess, SwapError>>(
        &pic,
        appic_dex_canister_id(),
        "swap",
        swap_args,
        Some(sender_principal()),
    );

    five_ticks(&pic);
    five_ticks(&pic);

    println!("{:?}", swap_result);
    assert_eq!(
        swap_result,
        Ok(CandidSwapSuccess {
            amount_in: u256_to_nat(expected_amount_in),
            amount_out: u256_to_nat(amount_out)
        })
    );
}

#[test]
fn test_exact_output_single_balances() {
    let pic = set_up();

    five_ticks(&pic);
    five_ticks(&pic);

    // balances brefore swap
    let pool_balance_before_token0 = get_balance(&pic, token0_principal(), appic_dex_canister_id());
    let pool_balance_before_token1 = get_balance(&pic, token1_principal(), appic_dex_canister_id());

    let user_balance_before_token0 = get_balance(&pic, token0_principal(), sender_principal());
    let user_balance_before_token1 = get_balance(&pic, token1_principal(), sender_principal());

    //println!(
    //    "before swap balances, pool_token0 {:?}, pool_token1 {:?}, user_token0 {:?}, user_token1 {:?}",
    //    pool_balance_before_token0,
    //    pool_balance_before_token1,
    //    user_balance_before_token0,
    //    user_balance_before_token1
    //);

    // pool state after swap
    let pool_state_before = query_call::<CandidPoolId, Option<CandidPoolState>>(
        &pic,
        appic_dex_canister_id(),
        "get_pool",
        CandidPoolId {
            token0: token0_principal(),
            token1: token1_principal(),
            fee: Nat::from(3000_u32),
        },
    )
    .unwrap();

    let amount_out = *ONE_ETHER;
    let expected_amount_in = U256::from(1008049273448486163_u128);

    let swap_args =
        SwapArgs::ExactOutputSingle(crate::candid_types::swap::ExactOutputSingleParams {
            pool_id: CandidPoolId {
                token0: token0_principal(),
                token1: token1_principal(),
                fee: Nat::from(3000_u32),
            },
            zero_for_one: true,
            from_subaccount: None,
            amount_out: u256_to_nat(amount_out),
            amount_in_maximum: u256_to_nat(expected_amount_in),
        });

    let swap_result = update_call::<SwapArgs, Result<CandidSwapSuccess, SwapError>>(
        &pic,
        appic_dex_canister_id(),
        "swap",
        swap_args,
        Some(sender_principal()),
    )
    .unwrap();

    five_ticks(&pic);
    five_ticks(&pic);

    // pool state after swap
    let pool_state_after = query_call::<CandidPoolId, Option<CandidPoolState>>(
        &pic,
        appic_dex_canister_id(),
        "get_pool",
        CandidPoolId {
            token0: token0_principal(),
            token1: token1_principal(),
            fee: Nat::from(3000_u32),
        },
    )
    .unwrap();

    assert_eq!(
        pool_state_before.pool_reserves0 + u256_to_nat(expected_amount_in),
        pool_state_after.pool_reserves0
    );
    assert_eq!(
        pool_state_before.pool_reserves1 - u256_to_nat(amount_out),
        pool_state_after.pool_reserves1
    );

    assert_eq!(
        pool_state_after.swap_volume0_all_time,
        u256_to_nat(expected_amount_in)
    );

    // balances after swap
    let pool_balance_after_token0 = get_balance(&pic, token0_principal(), appic_dex_canister_id());
    let pool_balance_after_token1 = get_balance(&pic, token1_principal(), appic_dex_canister_id());

    let user_balance_after_token0 = get_balance(&pic, token0_principal(), sender_principal());
    let user_balance_after_token1 = get_balance(&pic, token1_principal(), sender_principal());

    println!(
        "after swap balances, pool_token0 {:?}, pool_token1 {:?}, user_token0 {:?}, user_token1 {:?}",
        pool_balance_after_token0,
        pool_balance_after_token1,
        user_balance_after_token0,
        user_balance_after_token1
    );

    assert_eq!(
        pool_balance_before_token0 + u256_to_nat(expected_amount_in.clone()),
        pool_balance_after_token0
    );

    assert_eq!(
        pool_balance_before_token1 - swap_result.amount_out.clone(),
        pool_balance_after_token1
    );

    assert_eq!(
        user_balance_before_token0
            - u256_to_nat(expected_amount_in.clone())
            - Nat::from(TOKEN_TRANSFER_FEE),
        user_balance_after_token0
    );

    assert_eq!(
        user_balance_before_token1 + swap_result.amount_out.clone() - Nat::from(TOKEN_TRANSFER_FEE),
        user_balance_after_token1
    );
}

#[test]
fn test_exact_output_1_hop_one_for_zero() {
    let pic = set_up();

    five_ticks(&pic);
    five_ticks(&pic);

    let amount_out = *ONE_ETHER;
    let expected_amount_in = U256::from(1008049273448486163_u128);

    let swap_args = SwapArgs::ExactOutput(crate::candid_types::swap::ExactOutputParams {
        amount_out: u256_to_nat(amount_out),
        amount_in_maximum: u256_to_nat(expected_amount_in),
        from_subaccount: None,
        token_out: token1_principal(),
        path: vec![CandidPathKey {
            intermediary_token: token0_principal(),
            fee: Nat::from(3000_u32),
        }],
    });

    let swap_result = update_call::<SwapArgs, Result<CandidSwapSuccess, SwapError>>(
        &pic,
        appic_dex_canister_id(),
        "swap",
        swap_args,
        Some(sender_principal()),
    );

    five_ticks(&pic);
    five_ticks(&pic);

    println!("{:?}", swap_result);
    assert_eq!(
        swap_result,
        Ok(CandidSwapSuccess {
            amount_in: u256_to_nat(expected_amount_in),
            amount_out: u256_to_nat(amount_out)
        })
    );
}

#[test]
fn test_exact_output_2_hops() {
    let pic = set_up();

    five_ticks(&pic);
    five_ticks(&pic);

    let amount_out = *ONE_ETHER;
    let expected_amount_in = U256::from(1016204441757464409_u128);

    let swap_args = SwapArgs::ExactOutput(crate::candid_types::swap::ExactOutputParams {
        amount_out: u256_to_nat(amount_out),
        amount_in_maximum: u256_to_nat(expected_amount_in),
        from_subaccount: None,
        token_out: token2_principal(),
        path: vec![
            CandidPathKey {
                intermediary_token: token0_principal(),
                fee: Nat::from(3000_u32),
            },
            CandidPathKey {
                intermediary_token: token1_principal(),
                fee: Nat::from(3000_u32),
            },
        ],
    });

    let swap_result = update_call::<SwapArgs, Result<CandidSwapSuccess, SwapError>>(
        &pic,
        appic_dex_canister_id(),
        "swap",
        swap_args,
        Some(sender_principal()),
    );

    five_ticks(&pic);
    five_ticks(&pic);

    println!("{:?}", swap_result);
    assert_eq!(
        swap_result,
        Ok(CandidSwapSuccess {
            amount_in: u256_to_nat(expected_amount_in),
            amount_out: u256_to_nat(amount_out)
        })
    );
}

#[test]
fn test_exact_output_3_hops() {
    let pic = set_up();

    five_ticks(&pic);
    five_ticks(&pic);

    let amount_out = *ONE_ETHER;
    let expected_amount_in = U256::from(1024467570922834110_u128);

    let swap_args = SwapArgs::ExactOutput(crate::candid_types::swap::ExactOutputParams {
        amount_out: u256_to_nat(amount_out),
        amount_in_maximum: u256_to_nat(expected_amount_in),
        from_subaccount: None,
        token_out: token3_principal(),
        path: vec![
            CandidPathKey {
                intermediary_token: token0_principal(),
                fee: Nat::from(3000_u32),
            },
            CandidPathKey {
                intermediary_token: token1_principal(),
                fee: Nat::from(3000_u32),
            },
            CandidPathKey {
                intermediary_token: token2_principal(),
                fee: Nat::from(3000_u32),
            },
        ],
    });

    let swap_result = update_call::<SwapArgs, Result<CandidSwapSuccess, SwapError>>(
        &pic,
        appic_dex_canister_id(),
        "swap",
        swap_args,
        Some(sender_principal()),
    );

    five_ticks(&pic);
    five_ticks(&pic);

    println!("{:?}", swap_result);
    assert_eq!(
        swap_result,
        Ok(CandidSwapSuccess {
            amount_in: u256_to_nat(expected_amount_in),
            amount_out: u256_to_nat(amount_out)
        })
    );
}
