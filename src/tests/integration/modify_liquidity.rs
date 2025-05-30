use std::time::Duration;

use crate::candid_types::{
    events::{GetEventsArg, GetEventsResult},
    pool::CandidPoolState,
    pool_history::CandidPoolHistory,
    position::{
        BurnPositionArgs, BurnPositionError, CandidPositionInfo, CandidPositionKey,
        CollectFeesError, CollectFeesSuccess, DecreaseLiquidityArgs, DecreaseLiquidityError,
        IncreaseLiquidityArgs, IncreaseLiquidityError,
    },
    swap::{CandidSwapSuccess, ExactInputSingleParams, SwapArgs, SwapError},
};

use super::*;

// This test contains the whole flow of adding liquidity, increasing liquidity, swapping,
// collecting fees, decreasing liquidity, and burning added liquidity.
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

    // advancing time for more than 10 min to check historical data recording
    pic.advance_time(Duration::from_secs(700));

    five_ticks(&pic);
    five_ticks(&pic);
    five_ticks(&pic);

    let historical_data_before = query_call::<CandidPoolId, Option<CandidPoolHistory>>(
        &pic,
        appic_dex_canister_id(),
        "get_pool_history",
        CandidPoolId {
            token0: token0_principal(),
            token1: token1_principal(),
            fee: Nat::from(3000_u32),
        },
    );

    println!(" \n \n{:?}", historical_data_before);

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

    // increase liquidity
    let liquidity_delta =
        update_call::<IncreaseLiquidityArgs, Result<Nat, IncreaseLiquidityError>>(
            &pic,
            appic_dex_canister_id(),
            "increase_liquidity",
            IncreaseLiquidityArgs {
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

        let _swap_result = update_call::<SwapArgs, Result<CandidSwapSuccess, SwapError>>(
            &pic,
            appic_dex_canister_id(),
            "swap",
            swap_args,
            Some(sender_principal()),
        )
        .unwrap();

        let _pool_state = query_call::<CandidPoolId, Option<CandidPoolState>>(
            &pic,
            appic_dex_canister_id(),
            "get_pool",
            pool_id.clone(),
        )
        .unwrap();

        //println!("swap_result {:?}, pool_state {:?}", swap_result, pool_state);
    }

    // pool state after swap
    let pool_state_after_swap = query_call::<CandidPoolId, Option<CandidPoolState>>(
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
    ); // imprecision due to rounding

    assert!(
        position.fees_token1_owed == Nat::from(150000000000000003_u128)
            || position.fees_token1_owed == Nat::from(150000000000000004_u128)
    ); // imprecision due to rounding

    five_ticks(&pic);
    five_ticks(&pic);
    five_ticks(&pic);

    // collecting fees
    let fee_collection_result =
        update_call::<CandidPositionKey, Result<CollectFeesSuccess, CollectFeesError>>(
            &pic,
            appic_dex_canister_id(),
            "collect_fees",
            CandidPositionKey {
                owner: liquidity_provider_principal(),
                pool: pool_id.clone(),
                tick_lower: candid::Int::from(-887220),
                tick_upper: candid::Int::from(887220),
            },
            Some(liquidity_provider_principal()),
        )
        .unwrap();

    println!("{:?}", fee_collection_result);

    five_ticks(&pic);
    five_ticks(&pic);
    five_ticks(&pic);

    // pool state after fee collection
    let pool_state_after_fee_collection = query_call::<CandidPoolId, Option<CandidPoolState>>(
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

    five_ticks(&pic);
    five_ticks(&pic);
    five_ticks(&pic);

    println!(
        "after swap {:?} after fee collection {:?}",
        pool_state_after_swap, pool_state_after_fee_collection
    );

    assert!(false);

    assert_eq!(
        pool_state_after_fee_collection.pool_reserves0 + position.fees_token0_owed,
        pool_state_after_swap.pool_reserves0
    );

    assert_eq!(
        pool_state_after_fee_collection.pool_reserves1 + position.fees_token1_owed,
        pool_state_after_swap.pool_reserves1
    );

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

    assert!(position.fees_token0_owed == Nat::from(0_u8));
    assert!(position.fees_token1_owed == Nat::from(0_u8));

    five_ticks(&pic);
    five_ticks(&pic);
    five_ticks(&pic);

    // decreasing liquidity
    let _ = update_call::<DecreaseLiquidityArgs, Result<(), DecreaseLiquidityError>>(
        &pic,
        appic_dex_canister_id(),
        "decrease_liquidity",
        DecreaseLiquidityArgs {
            pool: pool_id.clone(),
            tick_lower: candid::Int::from(-887220),
            tick_upper: candid::Int::from(887220),
            liquidity: (TWO_HUNDRED_ETH / 4).into(),
            amount0_min: Nat::from(0_u8),
            amount1_min: Nat::from(0_u8),
        },
        Some(liquidity_provider_principal()),
    )
    .unwrap();

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
        Nat::from(TWO_HUNDRED_ETH + (TWO_HUNDRED_ETH / 2) - (TWO_HUNDRED_ETH / 4) + 15)
    );

    five_ticks(&pic);
    five_ticks(&pic);
    five_ticks(&pic);
    five_ticks(&pic);

    // burn position
    let _ = update_call::<BurnPositionArgs, Result<(), BurnPositionError>>(
        &pic,
        appic_dex_canister_id(),
        "burn",
        BurnPositionArgs {
            pool: pool_id.clone(),
            tick_lower: candid::Int::from(-887220),
            tick_upper: candid::Int::from(887220),
            amount0_min: Nat::from(0_u8),
            amount1_min: Nat::from(0_u8),
        },
        Some(liquidity_provider_principal()),
    )
    .unwrap();

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

    assert_eq!(position.liquidity, Nat::from(0_u8));

    let events = query_call::<GetEventsArg, GetEventsResult>(
        &pic,
        appic_dex_canister_id(),
        "get_events",
        GetEventsArg {
            start: 0_u64,
            length: 100u64,
        },
    );

    assert_eq!(16, events.total_event_count);

    // advancing time for more than 10 min to check historical data recording
    pic.advance_time(Duration::from_secs(700));

    five_ticks(&pic);
    five_ticks(&pic);

    let historical_data_after = query_call::<CandidPoolId, Option<CandidPoolHistory>>(
        &pic,
        appic_dex_canister_id(),
        "get_pool_history",
        CandidPoolId {
            token0: token0_principal(),
            token1: token1_principal(),
            fee: Nat::from(3000_u32),
        },
    )
    .unwrap();

    println!(" \n \n{:?}", historical_data_after);

    assert_eq!(
        historical_data_after.hourly_frame[0].swap_volume_token0_during_bucket,
        Nat::from(50000000000000000000_u128)
    );
    assert_eq!(
        historical_data_after.hourly_frame[0].swap_volume_token1_during_bucket,
        Nat::from(50000000000000000000_u128)
    );
}
