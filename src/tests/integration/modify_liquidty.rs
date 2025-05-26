use crate::candid_types::{
    pool::CandidPoolState,
    position::{
        CandidPositionInfo, CandidPositionKey, IncreaseLiquidtyArgs, IncreaseLiquidtyError,
    },
    swap::{CandidSwapSuccess, ExactInputParams, ExactInputSingleParams, SwapArgs, SwapError},
};

use super::*;

// This test contains the whole flow of adding liquidity, increasing liquidity, swapping,
// collecting fees, decreasing liquidty, and burning added liquidity.
#[test]
fn flow_test() {
    let pic = PocketIc::new();

    create_and_install_canisters(&pic);

    // adding liquidity
    let create_args = CreatePoolArgs {
        token_a: token0_principal(),
        token_b: token1_principal(),
        fee: Nat::from(3000_u32),
        sqrt_price_x96: u256_to_nat(*SQRT_PRICE_1_1),
    };

    let pool_id = update_call::<CreatePoolArgs, Result<CandidPoolId, CreatePoolError>>(
        &pic,
        appic_dex_canister_id(),
        "create_pool",
        create_args,
        Some(liquidity_provider_principal()),
    )
    .unwrap();

    println!("{:?}", pool_id);

    // Approval Section
    // Calling icrc2_approve and giving the permission to appic_dex for taking funds from users principal
    let _approve_result = update_call::<ApproveArgs, Result<Nat, ApproveError>>(
        &pic,
        token0_principal(),
        "icrc2_approve",
        ApproveArgs {
            from_subaccount: None,
            spender: LedgerAccount {
                owner: appic_dex_canister_id(),
                subaccount: None,
            },
            amount: Nat::from(
                TWO_HUNDRED_ETH + TWO_HUNDRED_ETH, // 400 ethers
            ),
            expected_allowance: None,
            expires_at: None,
            fee: None,
            memo: None,
            created_at_time: None,
        },
        Some(liquidity_provider_principal()),
    )
    .unwrap();

    // Approval Section
    // Calling icrc2_approve and giving the permission to appic_dex for taking funds from users principal
    let _approve_result = update_call::<ApproveArgs, Result<Nat, ApproveError>>(
        &pic,
        token1_principal(),
        "icrc2_approve",
        ApproveArgs {
            from_subaccount: None,
            spender: LedgerAccount {
                owner: appic_dex_canister_id(),
                subaccount: None,
            },
            amount: Nat::from(
                TWO_HUNDRED_ETH + TWO_HUNDRED_ETH, // 400 ethers
            ),
            expected_allowance: None,
            expires_at: None,
            fee: None,
            memo: None,
            created_at_time: None,
        },
        Some(liquidity_provider_principal()),
    )
    .unwrap();

    five_ticks(&pic);
    five_ticks(&pic);
    five_ticks(&pic);

    let mint_args = MintPositionArgs {
        pool: CandidPoolId {
            token0: token0_principal(),
            token1: token1_principal(),
            fee: Nat::from(3000_u32),
        },
        tick_lower: candid::Int::from(-887220),
        tick_upper: candid::Int::from(887220),
        amount0_max: u256_to_nat(U256::from(TWO_HUNDRED_ETH)),
        amount1_max: u256_to_nat(U256::from(TWO_HUNDRED_ETH)),
        from_subaccount: None,
    };

    println!("{:?}", mint_args);

    let mint_result = update_call::<MintPositionArgs, Result<Nat, MintPositionError>>(
        &pic,
        appic_dex_canister_id(),
        "mint_position",
        mint_args,
        Some(liquidity_provider_principal()),
    )
    .unwrap();

    assert_eq!(mint_result, Nat::from(TWO_HUNDRED_ETH + 10));

    five_ticks(&pic);
    five_ticks(&pic);

    let position = query_call::<CandidPositionKey, Option<CandidPositionInfo>>(
        &pic,
        appic_dex_canister_id(),
        "get_position",
        CandidPositionKey {
            owner: liquidity_provider_principal(),
            pool: pool_id.clone(),
            tick_lower: candid::Int::from(-887220),
            tick_upper: candid::Int::from(887220),
        },
    )
    .unwrap();

    assert_eq!(position.liquidity, Nat::from(TWO_HUNDRED_ETH + 10));

    five_ticks(&pic);
    five_ticks(&pic);

    // increase liquidty
    let liquidity_delta = update_call::<IncreaseLiquidtyArgs, Result<Nat, IncreaseLiquidtyError>>(
        &pic,
        appic_dex_canister_id(),
        "increase_liquidity",
        IncreaseLiquidtyArgs {
            pool: pool_id.clone(),
            tick_lower: candid::Int::from(-887220),
            tick_upper: candid::Int::from(887220),
            amount0_max: Nat::from(TWO_HUNDRED_ETH / 2),
            amount1_max: Nat::from(TWO_HUNDRED_ETH / 2),
            from_subaccount: None,
        },
        Some(liquidity_provider_principal()),
    )
    .unwrap();

    assert_eq!(liquidity_delta, TWO_HUNDRED_ETH / 2 + 5);

    let position = query_call::<CandidPositionKey, Option<CandidPositionInfo>>(
        &pic,
        appic_dex_canister_id(),
        "get_position",
        CandidPositionKey {
            owner: liquidity_provider_principal(),
            pool: pool_id.clone(),
            tick_lower: candid::Int::from(-887220),
            tick_upper: candid::Int::from(887220),
        },
    )
    .unwrap();

    assert_eq!(
        position.liquidity,
        Nat::from((TWO_HUNDRED_ETH + 10) + (TWO_HUNDRED_ETH / 2 + 5))
    );

    // swap and fee collection
    // we make couple of swaps per direction(zero for one)
    // and then collect the fees

    let _approve_result = update_call::<ApproveArgs, Result<Nat, ApproveError>>(
        &pic,
        token0_principal(),
        "icrc2_approve",
        ApproveArgs {
            from_subaccount: None,
            spender: LedgerAccount {
                owner: appic_dex_canister_id(),
                subaccount: None,
            },
            amount: Nat::from(
                TWO_HUNDRED_ETH + TWO_HUNDRED_ETH, // 400 ethers
            ),
            expected_allowance: None,
            expires_at: None,
            fee: None,
            memo: None,
            created_at_time: None,
        },
        Some(sender_principal()),
    )
    .unwrap();

    // Approval Section
    // Calling icrc2_approve and giving the permission to appic_dex for taking funds from users principal
    let _approve_result = update_call::<ApproveArgs, Result<Nat, ApproveError>>(
        &pic,
        token1_principal(),
        "icrc2_approve",
        ApproveArgs {
            from_subaccount: None,
            spender: LedgerAccount {
                owner: appic_dex_canister_id(),
                subaccount: None,
            },
            amount: Nat::from(
                TWO_HUNDRED_ETH + TWO_HUNDRED_ETH, // 400 ethers
            ),
            expected_allowance: None,
            expires_at: None,
            fee: None,
            memo: None,
            created_at_time: None,
        },
        Some(sender_principal()),
    )
    .unwrap();

    five_ticks(&pic);
    five_ticks(&pic);
    five_ticks(&pic);

    for i in 0..10 {
        five_ticks(&pic);
        five_ticks(&pic);
        five_ticks(&pic);

        let zero_for_one = i % 2 == 0;
        let swap_args = SwapArgs::ExactInputSingle(ExactInputSingleParams {
            amount_in: Nat::from(TWO_HUNDRED_ETH / 20),
            amount_out_minimum: Nat::from(0_u8),
            from_subaccount: None,
            pool_id: pool_id.clone(),
            zero_for_one,
        });

        let swap_result = update_call::<SwapArgs, Result<CandidSwapSuccess, SwapError>>(
            &pic,
            appic_dex_canister_id(),
            "swap",
            swap_args,
            Some(sender_principal()),
        )
        .unwrap();

        let pool_state = query_call::<CandidPoolId, Option<CandidPoolState>>(
            &pic,
            appic_dex_canister_id(),
            "get_pool",
            pool_id.clone(),
        )
        .unwrap();

        println!("swap_result {:?}, pool_state {:?}", swap_result, pool_state);
    }

    let position = query_call::<CandidPositionKey, Option<CandidPositionInfo>>(
        &pic,
        appic_dex_canister_id(),
        "get_position",
        CandidPositionKey {
            owner: liquidity_provider_principal(),
            pool: pool_id.clone(),
            tick_lower: candid::Int::from(-887220),
            tick_upper: candid::Int::from(887220),
        },
    )
    .unwrap();

    assert!(
        position.fees_token0_owed == Nat::from(150000000000000003_u128)
            || position.fees_token0_owed == Nat::from(150000000000000004_u128)
    ); // impersision due to rounding

    assert!(
        position.fees_token1_owed == Nat::from(150000000000000003_u128)
            || position.fees_token1_owed == Nat::from(150000000000000004_u128)
    ); // impersision due to rounding

    // collcting fees
}
