use std::time::Duration;

use appic_dex::{
    balances::types::{UserBalance, UserBalanceKey},
    burn::execute_burn_position,
    candid_types::{
        events::{CandidEvent, GetEventsArg, GetEventsResult},
        pool::{CandidPoolId, CandidPoolState, CreatePoolArgs, CreatePoolError},
        pool_history::CandidPoolHistory,
        position::{
            BurnPositionArgs, BurnPositionError, CandidPositionInfo, CandidPositionKey,
            CollectFeesError, CollectFeesSuccess, DecreaseLiquidityArgs, DecreaseLiquidityError,
            IncreaseLiquidityArgs, IncreaseLiquidityError, MintPositionArgs, MintPositionError,
        },
        quote::{QuoteArgs, QuoteError},
        swap::{CandidSwapSuccess, SwapArgs, SwapError},
        Balance, DepositArgs, DepositError, UserBalanceArgs, WithdrawArgs, WithdrawError,
    },
    collect_fees::execute_collect_fees,
    decrease_liquidity::execute_decrease_liquidity,
    guard::PrincipalGuard,
    historical::capture_historical_data,
    icrc_client::{
        memo::{DepositMemo, WithdrawMemo},
        LedgerClient, LedgerTransferError,
    },
    increase_liquidity::execute_increase_liquidity,
    libraries::{
        balance_delta::BalanceDelta,
        safe_cast::{big_uint_to_u256, u256_to_big_uint, u256_to_nat},
    },
    logs::DEBUG,
    mint::execute_mint_position,
    pool::{
        create_pool::create_pool_inner,
        types::{PoolFee, PoolId, PoolTickSpacing},
    },
    position::types::PositionKey,
    quote::{
        process_multi_hop_exact_input, process_multi_hop_exact_output,
        process_single_hop_exact_input, process_single_hop_exact_output,
    },
    state::{mutate_state, read_state},
    swap::execute_swap,
    validation::{
        burn_args::validate_burn_position_args, decrease_args::validate_decrease_liquidity_args,
        increase_args::validate_increase_liquidity_args, mint_args::validate_mint_position_args,
        swap_args::validate_swap_args,
    },
};

use candid::{Nat, Principal};
use ethnum::{I256, U256};
use ic_canister_log::log;
use ic_cdk::{init, post_upgrade, query, update};
use icrc_ledger_types::icrc1::account::Account;

// Ensures caller is not anonymous, panics if anonymous to prevent unauthorized access
fn validate_caller_not_anonymous() -> candid::Principal {
    let principal = ic_cdk::caller();
    if principal == candid::Principal::anonymous() {
        panic!("anonymous principal is not allowed");
    }
    principal
}

// Schedules periodic capture of historical data every 10 minutes for analytics
fn set_up_timers() {
    ic_cdk_timers::set_timer_interval(Duration::from_secs(10 * 60), capture_historical_data);
}

// Initializes canister with fee-to-tick-spacing mappings for pool configurations and sets timers
#[init]
fn init() {
    let fee_to_tick_spacing = vec![
        (100_u32, 1_i32),      // 0.01% fee, 1 tick spacing
        (500_u32, 10i32),      // 0.05% fee, 10 tick spacing
        (1_000_u32, 20i32),    // 0.1% fee, 20 tick spacing
        (3_000_u32, 60_i32),   // 0.3% fee, 60 tick spacing
        (10_000_u32, 200_i32), // 1% fee, 200 tick spacing
    ];

    for (fee, tick_spacing) in fee_to_tick_spacing {
        // Maps fee levels to tick spacings for pool creation
        mutate_state(|s| s.set_tick_spacing(PoolFee(fee), PoolTickSpacing(tick_spacing)));
    }

    set_up_timers();
}

// Restarts timers after canister upgrade to maintain historical data collection
#[post_upgrade]
fn post_upgrade() {
    set_up_timers();
}

// Queries state of a specific pool by ID, converts to Candid format, returns None if not found
#[query]
fn get_pool(pool_id: CandidPoolId) -> Option<CandidPoolState> {
    let pool_id: PoolId = pool_id.try_into().ok()?;
    read_state(|s| {
        s.get_pool(&pool_id)
            .map(|pool_state| CandidPoolState::from(pool_state))
    })
}

// Retrieves all pools with their IDs and states, converted to Candid format
#[query]
fn get_pools() -> Vec<(CandidPoolId, CandidPoolState)> {
    read_state(|s| s.get_pools())
        .into_iter()
        .map(|(id, state)| (CandidPoolId::from(id), CandidPoolState::from(state)))
        .collect()
}

// Queries historical data for a pool, returns None if no data exists
#[query]
fn get_pool_history(pool_id: CandidPoolId) -> Option<CandidPoolHistory> {
    let pool_id: PoolId = pool_id.try_into().ok()?;
    let pool_history = read_state(|s| s.get_pool_history(&pool_id));
    if pool_history.hourly_frame.len() == 0 {
        None
    } else {
        Some(CandidPoolHistory::from(pool_history))
    }
}

// Queries position details including fees owed, returns None if position not found
#[query]
fn get_position(position_key: CandidPositionKey) -> Option<CandidPositionInfo> {
    let position_key = PositionKey::try_from(position_key).ok()?;
    let (position, token0_owed, token1_owed) =
        read_state(|s| s.get_position_with_fees_owed(&position_key))?;

    Some(CandidPositionInfo {
        liquidity: position.liquidity.into(),
        fee_growth_inside_0_last_x128: u256_to_nat(position.fee_growth_inside_0_last_x128),
        fee_growth_inside_1_last_x128: u256_to_nat(position.fee_growth_inside_1_last_x128),
        fees_token0_owed: u256_to_nat(token0_owed),
        fees_token1_owed: u256_to_nat(token1_owed),
    })
}

// Retrieves all positions for an owner with details and fees owed, converted to Candid format
#[query]
fn get_positions_by_owner(owner: Principal) -> Vec<(CandidPositionKey, CandidPositionInfo)> {
    read_state(|s| s.get_positions_by_owner(owner))
        .into_iter()
        .map(|(key, info, token0_owed, token1_owed)| {
            let candid_key = CandidPositionKey {
                owner,
                pool: key.pool_id.into(),
                tick_lower: key.tick_lower.into(),
                tick_upper: key.tick_upper.into(),
            };

            let candid_info = CandidPositionInfo {
                liquidity: info.liquidity.into(),
                fee_growth_inside_0_last_x128: u256_to_nat(info.fee_growth_inside_0_last_x128),
                fee_growth_inside_1_last_x128: u256_to_nat(info.fee_growth_inside_1_last_x128),
                fees_token0_owed: u256_to_nat(token0_owed),
                fees_token1_owed: u256_to_nat(token1_owed),
            };

            (candid_key, candid_info)
        })
        .collect()
}

// Quotes swap output/input amounts for single or multi-hop swaps without state changes
#[query]
pub fn quote(args: QuoteArgs) -> Result<Nat, QuoteError> {
    let quote_amount = match args {
        QuoteArgs::QuoteExactInputSingleParams(params) => process_single_hop_exact_input(params)?,
        QuoteArgs::QuoteExactInputParams(params) => process_multi_hop_exact_input(params)?,
        QuoteArgs::QuoteExactOutputSingleParams(params) => process_single_hop_exact_output(params)?,
        QuoteArgs::QuoteExactOutput(params) => process_multi_hop_exact_output(params)?,
    };

    // Converts U256 quote amount to Nat for Candid compatibility
    Ok(Nat::from(u256_to_big_uint(quote_amount)))
}

// Queries all token balances for a user, returned as a list of token-amount pairs
#[query]
pub fn user_balances(user: Principal) -> Vec<Balance> {
    read_state(|s| s.get_user_balances(user))
        .into_iter()
        .map(|(token, balance)| Balance {
            token,
            amount: u256_to_nat(balance),
        })
        .collect()
}

// Queries a specific token balance for a user, converted to Nat
#[query]
fn user_balance(args: UserBalanceArgs) -> Nat {
    u256_to_nat(
        read_state(|s| {
            s.get_user_balance(&UserBalanceKey {
                user: args.user,
                token: args.token,
            })
        })
        .0,
    )
}

// Retrieves paginated events, capped at 100 per response for performance
#[query]
fn get_events(args: GetEventsArg) -> GetEventsResult {
    const MAX_EVENTS_PER_RESPONSE: u64 = 100;

    let (total_event_count, events) = read_state(|s| {
        (
            s.total_event_count(),
            s.get_events(args.start, args.length.min(MAX_EVENTS_PER_RESPONSE))
                .into_iter()
                .map(|event| CandidEvent::from(event))
                .collect::<Vec<CandidEvent>>(),
        )
    });

    GetEventsResult {
        events,
        total_event_count,
    }
}

// Creates a new liquidity pool, validates tokens and fees, returns pool ID
#[update]
async fn create_pool(args: CreatePoolArgs) -> Result<CandidPoolId, CreatePoolError> {
    // Prevents pool creation with identical tokens
    if args.token_a == args.token_b {
        return Err(CreatePoolError::DuplicatedTokens);
    }

    // Fetches transfer fees to validate token standards and ensure compatibility
    let token_a_fee = big_uint_to_u256(
        LedgerClient::new(args.token_a)
            .icrc_fee()
            .await
            .expect("A problem was found in the token canister")
            .0,
    )
    .map_err(|_| CreatePoolError::InvalidToken(args.token_a))?;

    let token_b_fee = big_uint_to_u256(
        LedgerClient::new(args.token_b) // Should be args.token_b
            .icrc_fee()
            .await
            .expect("A problem was found in the token canister")
            .0,
    )
    .map_err(|_| CreatePoolError::InvalidToken(args.token_b))?;

    let timestamp = ic_cdk::api::time();
    let pool_id = create_pool_inner(args, token_a_fee, token_b_fee, timestamp)?;

    Ok(pool_id.into())
}

// Mints a new liquidity position, deposits tokens if needed, returns liquidity amount
#[update]
async fn mint_position(args: MintPositionArgs) -> Result<Nat, MintPositionError> {
    let caller = validate_caller_not_anonymous();

    // Locks principal to prevent concurrent modifications, avoiding double-spending
    let _principal_guard = match PrincipalGuard::new_general_guard(caller) {
        Ok(guard) => guard,
        Err(_) => return Err(MintPositionError::LockedPrincipal),
    };

    let validated_args = validate_mint_position_args(args.clone(), caller)?;

    let pool_id = validated_args.pool_id.clone();
    let token0 = args.pool.token0;
    let token1 = args.pool.token1;
    let max_deposit = BalanceDelta::new(validated_args.amount0_max, validated_args.amount1_max);

    // Fetches user balances, using I256::MAX as fallback for overflow safety
    let user_balance = read_state(|s| {
        BalanceDelta::new(
            s.get_user_balance(&UserBalanceKey {
                user: caller,
                token: token0,
            })
            .0
            .try_into()
            .unwrap_or(I256::MAX),
            s.get_user_balance(&UserBalanceKey {
                user: caller,
                token: token1,
            })
            .0
            .try_into()
            .unwrap_or(I256::MAX),
        )
    });

    // Configures account with optional subaccount for token deposits
    let mut from: Account = caller.into();
    if let Some(subaccount) = args.from_subaccount {
        from.subaccount = Some(subaccount);
    }

    // Deposits tokens if user balance is insufficient for max deposit amounts
    _deposit_if_needed(
        caller,
        token0,
        &from,
        user_balance.amount0().as_u256(),
        max_deposit.amount0().as_u256(),
        &mut DepositMemo::MintPosition {
            // Typo: Should be MintPosition
            amount: U256::ZERO,
        },
    )
    .await
    .map_err(|e| MintPositionError::DepositError(e.into()))?;

    _deposit_if_needed(
        caller,
        token1,
        &from,
        user_balance.amount1().as_u256(),
        max_deposit.amount1().as_u256(),
        &mut DepositMemo::MintPosition {
            // Typo: Should be MintPosition
            amount: U256::ZERO,
        },
    )
    .await
    .map_err(|e| MintPositionError::DepositError(e.into()))?;

    let timestamp = ic_cdk::api::time();

    // Executes minting and converts liquidity amount to Nat
    execute_mint_position(caller, pool_id, token0, token1, validated_args, timestamp)
        .map(|mint_result| Nat::from(mint_result))
}

// Increases liquidity in an existing position, deposits tokens if needed, returns liquidity delta
#[update]
async fn increase_liquidity(args: IncreaseLiquidityArgs) -> Result<Nat, IncreaseLiquidityError> {
    let caller = validate_caller_not_anonymous();

    // Locks principal to prevent concurrent modifications
    let _principal_guard = match PrincipalGuard::new_general_guard(caller) {
        Ok(guard) => guard,
        Err(_) => return Err(IncreaseLiquidityError::LockedPrincipal),
    };

    let validated_args = validate_increase_liquidity_args(args.clone(), caller)?;

    let pool_id = validated_args.pool_id.clone();
    let token0 = args.pool.token0;
    let token1 = args.pool.token1;
    let max_deposit = BalanceDelta::new(validated_args.amount0_max, validated_args.amount1_max);

    // Fetches user balances for both tokens
    let user_balance = read_state(|s| {
        BalanceDelta::new(
            s.get_user_balance(&UserBalanceKey {
                user: caller,
                token: token0,
            })
            .0
            .try_into()
            .unwrap_or(I256::MAX),
            s.get_user_balance(&UserBalanceKey {
                user: caller,
                token: token1,
            })
            .0
            .try_into()
            .unwrap_or(I256::MAX),
        )
    });

    let mut from: Account = caller.into();
    if let Some(subaccount) = args.from_subaccount {
        from.subaccount = Some(subaccount);
    }

    // Deposits tokens if needed for both tokens
    _deposit_if_needed(
        caller,
        token0,
        &from,
        user_balance.amount0().as_u256(),
        max_deposit.amount0().as_u256(),
        &mut DepositMemo::MintPosition {
            // Typo: Should be MintPosition
            amount: U256::ZERO,
        },
    )
    .await
    .map_err(|e| IncreaseLiquidityError::DepositError(e.into()))?;

    _deposit_if_needed(
        caller,
        token1,
        &from,
        user_balance.amount1().as_u256(),
        max_deposit.amount1().as_u256(),
        &mut DepositMemo::MintPosition {
            // Typo: Should be MintPosition
            amount: U256::ZERO,
        },
    )
    .await
    .map_err(|e| IncreaseLiquidityError::DepositError(e.into()))?;

    let timestamp = ic_cdk::api::time();

    // Increases liquidity and returns the delta
    execute_increase_liquidity(caller, pool_id, token0, token1, validated_args, timestamp)
        .map(|liquidity_delta| Nat::from(liquidity_delta))
}

// Burns a liquidity position, withdraws tokens, returns success or error
#[update]
async fn burn(args: BurnPositionArgs) -> Result<(), BurnPositionError> {
    let caller = validate_caller_not_anonymous();

    // Locks principal to prevent concurrent modifications
    let _principal_guard = match PrincipalGuard::new_general_guard(caller) {
        Ok(guard) => guard,
        Err(_) => return Err(BurnPositionError::LockedPrincipal),
    };

    let validated_args = validate_burn_position_args(args.clone(), caller)?;

    let pool_id = validated_args.pool_id.clone();
    let token0 = args.pool.token0;
    let token1 = args.pool.token1;

    let timestamp = ic_cdk::api::time();

    // Burns position and updates user balance with withdrawn amounts
    let user_balance_after_burn = execute_burn_position(
        caller,
        pool_id.clone(),
        token0,
        token1,
        validated_args,
        timestamp,
    )?;

    // Retrieves transfer fees for both tokens from pool state
    let (token0_transfer_fee, token1_transfer_fee) = read_state(|s| {
        let pool_state = s.get_pool(&pool_id).unwrap();
        (
            pool_state.token0_transfer_fee,
            pool_state.token1_transfer_fee,
        )
    });

    let to_account = Account::from(caller);

    // Withdraws burned tokens for token0
    let _ = _withdraw(
        caller,
        token0,
        user_balance_after_burn.amount0().as_u256(),
        &to_account,
        &mut WithdrawMemo::BurnPosition {
            // Typo: Should be BurnPosition
            amount: U256::ZERO,
        },
        token0_transfer_fee,
    )
    .await
    .map_err(|e| BurnPositionError::BurntPositionWithdrawalFailed(e.into()))?;

    // Withdraws burned tokens for token1
    let _ = _withdraw(
        caller,
        token1,
        user_balance_after_burn.amount1().as_u256(),
        &to_account,
        &mut WithdrawMemo::BurnPosition {
            // Typo: Should be BurnPosition
            amount: U256::ZERO,
        },
        token1_transfer_fee,
    )
    .await
    .map_err(|e| BurnPositionError::BurntPositionWithdrawalFailed(e.into()))?;

    Ok(())
}

// Decreases liquidity in a position, withdraws tokens, returns success or error
#[update]
async fn decrease_liquidity(args: DecreaseLiquidityArgs) -> Result<(), DecreaseLiquidityError> {
    let caller = validate_caller_not_anonymous();

    // Locks principal to prevent concurrent modifications
    let _principal_guard = match PrincipalGuard::new_general_guard(caller) {
        Ok(guard) => guard,
        Err(_) => return Err(DecreaseLiquidityError::LockedPrincipal),
    };

    let validated_args = validate_decrease_liquidity_args(args.clone(), caller)?;

    let pool_id = validated_args.pool_id.clone();
    let token0 = args.pool.token0;
    let token1 = args.pool.token1;

    let timestamp = ic_cdk::api::time();

    // Decreases liquidity and updates user balance
    let user_balance_after_burn = execute_decrease_liquidity(
        caller,
        pool_id.clone(),
        token0,
        token1,
        validated_args,
        timestamp,
    )?;

    // Retrieves transfer fees for both tokens
    let (token0_transfer_fee, token1_transfer_fee) = read_state(|s| {
        let pool_state = s.get_pool(&pool_id).unwrap();
        (
            pool_state.token0_transfer_fee,
            pool_state.token1_transfer_fee,
        )
    });

    let to_account = Account::from(caller);

    // Withdraws decreased liquidity for token0
    let _ = _withdraw(
        caller,
        token0,
        user_balance_after_burn.amount0().as_u256(),
        &to_account,
        &mut WithdrawMemo::BurnPosition {
            // Typo: Should be BurnPosition
            amount: U256::ZERO,
        },
        token0_transfer_fee,
    )
    .await
    .map_err(|e| DecreaseLiquidityError::DecreasedPositionWithdrawalFailed(e.into()))?;

    // Withdraws decreased liquidity for token1
    let _ = _withdraw(
        caller,
        token1,
        user_balance_after_burn.amount1().as_u256(),
        &to_account,
        &mut WithdrawMemo::BurnPosition {
            // Typo: Should be BurnPosition
            amount: U256::ZERO,
        },
        token1_transfer_fee,
    )
    .await
    .map_err(|e| DecreaseLiquidityError::DecreasedPositionWithdrawalFailed(e.into()))?;

    Ok(())
}

// Executes a token swap, deposits input, withdraws output, refunds on failure
#[update]
async fn swap(args: SwapArgs) -> Result<CandidSwapSuccess, SwapError> {
    let validated_swap_args = validate_swap_args(args)?;
    ic_cdk::println!("{:?}", validated_swap_args);
    let caller = validate_caller_not_anonymous();

    // Uses swap-specific lock to allow concurrent swaps but block liquidity changes
    let _guard = match PrincipalGuard::new_swap_guard(caller) {
        Ok(guard) => guard,
        Err(_err) => return Err(SwapError::LockedPrincipal),
    };

    // Configures user account with optional subaccount
    let mut user_address: Account = caller.into();
    if let Some(subaccount) = validated_swap_args.from_subaccount() {
        user_address.subaccount = Some(subaccount);
    }

    let deposit_amount = validated_swap_args.deposit_amount();
    let token_in = validated_swap_args.token_in();
    let token_out = validated_swap_args.token_out();

    // Deposits input tokens for the swap
    let _ = _deposit(
        caller,
        token_in,
        &user_address,
        deposit_amount.as_u256(),
        &mut DepositMemo::SwapIn { amount: U256::ZERO },
    )
    .await
    .map_err(|e| SwapError::DepositError(e))?;

    let timestamp = ic_cdk::api::time();

    let swap_result = execute_swap(&validated_swap_args, token_in, token_out, caller, timestamp);

    match swap_result {
        Ok(swap_delta) => {
            // Withdraws output tokens after successful swap
            _withdraw(
                caller,
                token_out,
                swap_delta.1.as_u256(),
                &user_address,
                &mut WithdrawMemo::SwapOut { amount: U256::ZERO },
                swap_delta.2,
            )
            .await
            .map_err(|e| SwapError::FailedToWithdraw {
                reason: e,
                amount_in: u256_to_nat(swap_delta.0.as_u256()),
                amount_out: u256_to_nat(swap_delta.1.as_u256()),
            })?;

            // Returns input and output amounts on success
            return Ok(CandidSwapSuccess {
                amount_in: u256_to_nat(swap_delta.0.as_u256()),
                amount_out: u256_to_nat(swap_delta.1.as_u256()),
            });
        }
        Err(err) => {
            // Refunds input tokens if swap fails
            let refunded_amount =
                _refund(caller, token_in, deposit_amount.as_u256(), &user_address)
                    .await
                    .map_err(|e| SwapError::SwapFailedRefunded {
                        refund_amount: None,
                        failed_reason: err.clone(),
                        refund_error: Some(e),
                    })?;

            return Err(SwapError::SwapFailedRefunded {
                failed_reason: err,
                refund_error: None,
                refund_amount: Some(u256_to_nat(refunded_amount)),
            });
        }
    }
}

// Collects fees from a position, withdraws them, returns collected amounts
#[update]
async fn collect_fees(position: CandidPositionKey) -> Result<CollectFeesSuccess, CollectFeesError> {
    let caller = validate_caller_not_anonymous();
    let _principal_guard = match PrincipalGuard::new_general_guard(caller) {
        Ok(guard) => guard,
        Err(_) => return Err(CollectFeesError::LockedPrincipal),
    };

    // Ensures position belongs to caller by setting owner
    let mut position_key: PositionKey = position
        .try_into()
        .map_err(|_| CollectFeesError::PositionNotFound)?;
    position_key.owner = caller;

    let (_position, token0_owed, token1_owed) =
        read_state(|s| s.get_position_with_fees_owed(&position_key))
            .ok_or(CollectFeesError::PositionNotFound)?;

    let pool = read_state(|s| s.get_pool(&position_key.pool_id))
        .ok_or(CollectFeesError::PositionNotFound)?;

    // Checks if there are fees to collect
    if token0_owed == U256::ZERO && token1_owed == U256::ZERO {
        return Err(CollectFeesError::NoFeeToCollect);
    }

    // Executes fee collection and updates position state
    let fee_delta = execute_collect_fees(caller, &position_key, pool.tick_spacing)?;

    if fee_delta != BalanceDelta::ZERO_DELTA {
        // Withdraws collected fees for token0
        let _ = _withdraw(
            caller,
            position_key.pool_id.token0,
            fee_delta.amount0().as_u256(),
            &caller.into(),
            &mut WithdrawMemo::CollectFees { amount: U256::ZERO },
            pool.token0_transfer_fee,
        )
        .await
        .map_err(|e| CollectFeesError::CollectedFeesWithdrawalFailed(e.into()))?;

        // Withdraws collected fees for token1, using token0_transfer_fee (likely a bug)
        let _ = _withdraw(
            caller,
            position_key.pool_id.token1,
            fee_delta.amount1().as_u256(),
            &caller.into(),
            &mut WithdrawMemo::CollectFees { amount: U256::ZERO },
            pool.token0_transfer_fee, // Should likely be token1_transfer_fee
        )
        .await
        .map_err(|e| CollectFeesError::CollectedFeesWithdrawalFailed(e.into()))?;

        Ok(CollectFeesSuccess {
            token0_collected: u256_to_nat(fee_delta.amount0().as_u256()),
            token1_collected: u256_to_nat(fee_delta.amount1().as_u256()),
        })
    } else {
        Err(CollectFeesError::NoFeeToCollect)
    }
}

// Deposits tokens into the canister, updates user balance
#[update]
async fn deposit(deposit_args: DepositArgs) -> Result<(), DepositError> {
    let caller = validate_caller_not_anonymous();
    let _principal_guard = match PrincipalGuard::new_general_guard(caller) {
        Ok(guard) => guard,
        Err(_) => return Err(DepositError::LockedPrincipal),
    };

    let mut from = Account::from(caller);
    if let Some(subaccount) = deposit_args.from_subaccount {
        from.subaccount = Some(subaccount);
    }

    // Converts deposit amount to U256, checks for overflow
    let amount =
        big_uint_to_u256(deposit_args.amount.0).map_err(|_| DepositError::AmountOverflow)?;

    _deposit(
        caller,
        deposit_args.token,
        &from,
        amount,
        &mut DepositMemo::Deposit { amount: U256::ZERO },
    )
    .await
}

// Withdraws tokens, updates user balance, returns withdrawn amount
#[update]
async fn withdraw(withdraw_args: WithdrawArgs) -> Result<Nat, WithdrawError> {
    let caller = validate_caller_not_anonymous();
    let _principal_guard = match PrincipalGuard::new_general_guard(caller) {
        Ok(guard) => guard,
        Err(_) => return Err(WithdrawError::LockedPrincipal),
    };

    // Fetches token transfer fee from ledger
    let transfer_fee = big_uint_to_u256(
        LedgerClient::new(withdraw_args.token)
            .icrc_fee()
            .await
            .map_err(|_| WithdrawError::FeeUnknown)?
            .0,
    )
    .map_err(|_| WithdrawError::FeeUnknown)?;

    let amount =
        big_uint_to_u256(withdraw_args.amount.0).map_err(|_| WithdrawError::AmountOverflow)?;

    _withdraw(
        caller,
        withdraw_args.token,
        amount,
        &caller.into(),
        &mut WithdrawMemo::Withdraw { amount: U256::ZERO },
        transfer_fee,
    )
    .await
    .map(|ledger_index| u256_to_nat(ledger_index))
}

// Internal function to deposit tokens and update user balance
async fn _deposit(
    caller: Principal,
    token: Principal,
    from: &Account,
    amount: U256,
    memo: &mut DepositMemo,
) -> Result<(), DepositError> {
    // Sets deposit amount in memo for ledger tracking

    log!(
        DEBUG,
        "Depositing token {:?} with amount {:?} from user {:?}",
        token.to_text(),
        amount,
        caller.to_text(),
    );

    memo.set_amount(amount);
    LedgerClient::new(token)
        .deposit(*from, u256_to_big_uint(amount), memo.clone())
        .await?;

    // Updates user balance, caps at U256::MAX to prevent overflow
    let latest_user_balance = get_user_balance(caller, token);
    mutate_state(|s| {
        s.update_user_balance(
            UserBalanceKey {
                user: caller,
                token,
            },
            UserBalance(latest_user_balance.checked_add(amount).unwrap_or(U256::MAX)),
        );
    });

    Ok(())
}

// Deposits tokens if current balance is insufficient, returns updated balance
async fn _deposit_if_needed(
    caller: Principal,
    token: Principal,
    from: &Account,
    user_current_balance: U256,
    desired_user_balance: U256,
    memo: &mut DepositMemo,
) -> Result<U256, DepositError> {
    if desired_user_balance > user_current_balance {
        // Calculates additional amount needed and deposits it
        let deposit_amount = desired_user_balance - user_current_balance;

        log!(
            DEBUG,
            "Depositing token {:?} with amount {:?} from user {:?}",
            token.to_text(),
            deposit_amount,
            caller.to_text(),
        );

        memo.set_amount(deposit_amount);
        LedgerClient::new(token)
            .deposit(*from, u256_to_big_uint(deposit_amount), memo.clone())
            .await?;

        // Updates user balance to desired amount
        mutate_state(|s| {
            s.update_user_balance(
                UserBalanceKey {
                    user: caller,
                    token,
                },
                UserBalance(desired_user_balance),
            );
        });
        return Ok(desired_user_balance);
    }
    Ok(user_current_balance)
}

// Refunds tokens to user, returns refunded amount after fees
async fn _refund(
    caller: Principal,
    token: Principal,
    amount: U256,
    to: &Account,
) -> Result<U256, WithdrawError> {
    // Fetches transfer fee for refund calculation
    let transfer_fee = big_uint_to_u256(
        LedgerClient::new(token)
            .icrc_fee()
            .await
            .map_err(|_| WithdrawError::FeeUnknown)?
            .0,
    )
    .map_err(|_| WithdrawError::FeeUnknown)?;

    _withdraw(
        caller,
        token,
        amount,
        to,
        &mut WithdrawMemo::Refund { amount: U256::ZERO },
        transfer_fee,
    )
    .await
}

// Withdraws tokens, updates balance, handles transfer errors with rollback
async fn _withdraw(
    caller: Principal,
    token: Principal,
    amount: U256,
    to: &Account,
    memo: &mut WithdrawMemo,
    transfer_fee: U256,
) -> Result<U256, WithdrawError> {
    let user_balance = get_user_balance(caller, token);

    log!(
        DEBUG,
        "Withdrawing token {:?} with amount {:?} with transfer fee {:?} to user {:?} with balance {:?}",
        token.to_text(), amount, transfer_fee, caller.to_text(),user_balance
    );

    // Ensures amount covers transfer fee
    if amount.checked_sub(transfer_fee).is_none() {
        return Err(WithdrawError::AmountTooLow {
            min_withdrawal_amount: Nat::from(u256_to_big_uint(transfer_fee)),
        });
    }

    // Checks for sufficient balance
    if amount > user_balance {
        return Err(WithdrawError::InsufficientBalance {
            balance: u256_to_nat(user_balance),
        });
    }

    // Deducts balance before transfer to prevent double-spending
    mutate_state(|s| {
        s.update_user_balance(
            UserBalanceKey {
                user: caller,
                token,
            },
            UserBalance(user_balance - amount),
        );
    });

    let withdrawal_amount = amount - transfer_fee;
    let icrc_fee = u256_to_big_uint(transfer_fee);
    memo.set_amount(amount);
    match LedgerClient::new(token)
        .withdraw(
            *to,
            u256_to_big_uint(withdrawal_amount),
            memo.clone(),
            icrc_fee,
        )
        .await
    {
        Ok(_) => Ok(withdrawal_amount),
        Err(err) => {
            // Restores balance on transfer failure
            let latest_user_balance = get_user_balance(caller, token);
            mutate_state(|s| {
                s.update_user_balance(
                    UserBalanceKey {
                        user: caller,
                        token,
                    },
                    UserBalance(latest_user_balance.checked_add(amount).unwrap_or(U256::MAX)),
                );
            });

            // Handles fee mismatch by updating pool fees
            match err {
                LedgerTransferError::BadFee { expected_fee } => {
                    let new_transfer_fee =
                        big_uint_to_u256(expected_fee.0).map_err(|_| WithdrawError::FeeUnknown)?;

                    // Updates transfer fee across all pools for consistency
                    mutate_state(|s| {
                        s.update_token_transfer_fee_across_all_pools(token, new_transfer_fee)
                    });
                    Err(WithdrawError::FeeUnknown)
                }
                _ => Err(err.into()),
            }
        }
    }
}

// Retrieves user's token balance from state
pub fn get_user_balance(user: Principal, token: Principal) -> U256 {
    read_state(|s| s.get_user_balance(&UserBalanceKey { user, token }).0)
}

fn main() {}

ic_cdk::export_candid!();
