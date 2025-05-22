use crate::{
    candid_types::swap::{CandidPathKey, CandidSwapSuccess, SwapArgs, SwapError, SwapFailedReason},
    libraries::{safe_cast::big_uint_to_u256, sqrt_price_math::tests::ONE_ETHER},
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

///////////////////////////////////////////////////////////////
// EXACT INPUT
///////////////////////////////////////////////////////////////
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

    println!("before swap balances, pool_token0 {:?}, pool_token1 {:?}, user_token0 {:?}, user_token1 {:?}",pool_balance_before_token0,pool_balance_before_token1,user_balance_before_token0,user_balance_before_token1);

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

    // balances after swap
    let pool_balance_after_token0 = get_balance(&pic, token0_principal(), appic_dex_canister_id());
    let pool_balance_after_token1 = get_balance(&pic, token1_principal(), appic_dex_canister_id());

    let user_balance_after_token0 = get_balance(&pic, token0_principal(), sender_principal());
    let user_balance_after_token1 = get_balance(&pic, token1_principal(), sender_principal());

    println!("after swap balances, pool_token0 {:?}, pool_token1 {:?}, user_token0 {:?}, user_token1 {:?}",pool_balance_after_token0,pool_balance_after_token1,user_balance_after_token0,user_balance_after_token1);

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
