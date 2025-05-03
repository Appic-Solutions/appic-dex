use core::panic;
use std::{fmt::format, process::Output};

use appic_dex::{
    balances::types::{UserBalance, UserBalanceKey},
    burn::execute_burn_position,
    candid_types::{
        position::{BurnPositionArgs, BurnPositionError, MintPositionArgs, MintPositionError},
        quote::{QuoteArgs, QuoteError},
        swap::{SwapArgs, SwapError},
        WithdrawalError,
    },
    icrc_client::{
        memo::{DepositMemo, WithdrawalMemo},
        LedgerClient, LedgerTransferError,
    },
    libraries::{
        balance_delta::{self, BalanceDelta},
        constants::{MAX_SQRT_RATIO, MIN_SQRT_RATIO},
        liquidity_amounts,
        path_key::PathKey,
        safe_cast::{big_uint_to_u256, u256_to_big_uint},
        slippage_check::BalanceDeltaValidationError,
        tick_math::{self, TickMath},
    },
    mint::execute_mint_position,
    pool::{
        modify_liquidity::{self, modify_liquidity, ModifyLiquidityError, ModifyLiquidityParams},
        swap::{swap_inner, SwapParams},
        types::{PoolId, PoolTickSpacing},
    },
    quote::{
        process_multi_hop_exact_input, process_multi_hop_exact_output,
        process_single_hop_exact_input, process_single_hop_exact_output,
    },
    state::{mutate_state, read_state},
    validate_candid_args::{
        self, validate_burn_position_args, validate_mint_position_args, validate_swap_args,
        ValidatedMintPositionArgs, MAX_PATH_LENGTH, MIN_PATH_LENGTH,
    },
};

use candid::{Nat, Principal};
use ethnum::{AsI256, I256, U256};
use ic_cdk::{query, update};
use icrc_ledger_types::icrc1::account::Account;
use num_traits::Zero;

fn validate_caller_not_anonymous() -> candid::Principal {
    let principal = ic_cdk::caller();
    if principal == candid::Principal::anonymous() {
        panic!("anonymous principal is not allowed");
    }
    principal
}

//#[update]
//pub fn create_pool(
//    CreatePoolArgs {
//        token_a,
//        token_b,
//        fee,
//        sqrt_price_x96,
//    }: CreatePoolArgs,
//) -> Result<(), CreatePoolError> {
//    // sort token_a and b, token 0 is always the smaller token
//    let (token0, token1) = if token_a < token_b {
//        (token_a, token_b)
//    } else {
//        (token_b, token_a)
//    };
//
//    let fee = PoolFee::try_from(fee).map_err(|_e| CreatePoolError::InvalidFeeAmount)?;
//
//    let pool_id = PoolId {
//        token0,
//        token1,
//        fee: fee.clone(),
//    };
//
//    if read_state(|s| s.get_pool(&pool_id)).is_some() {
//        return Err(CreatePoolError::PoolAlreadyExists);
//    }
//    let tick_spacing =
//        read_state(|s| s.get_tick_spacing(&fee)).ok_or(CreatePoolError::InvalidFeeAmount)?;
//
//    let sqrt_price_x96 = U256::from_str_radix(&sqrt_price_x96.0.to_str_radix(10), 10)
//        .map_err(|_e| CreatePoolError::InvalidSqrtPriceX96)?;
//
//    let tick = TickMath::get_tick_at_sqrt_ratio(sqrt_price_x96)
//        .map_err(|_e| CreatePoolError::InvalidSqrtPriceX96)?;
//
//    let max_liquidity_per_tick = tick::tick_spacing_to_max_liquidity_per_tick(tick_spacing.0);
//    let pool_state = PoolState {
//        sqrt_price_x96,
//        tick,
//        fee_growth_global_0_x128: U256::ZERO,
//        fee_growth_global_1_x128: U256::ZERO,
//        liquidity: 0,
//        tick_spacing,
//        max_liquidity_per_tick,
//        fee_protocol: todo!(),
//    };
//    mutate_state(|s| s.set_pool(pool_id, pool_state));
//
//    return Ok(());
//}
//

#[update]
async fn mint_position(args: MintPositionArgs) -> Result<(), MintPositionError> {
    // TODO: Principal Lock to be implemented

    // Validate inputs and caller
    let caller = validate_caller_not_anonymous();
    let validated_args = validate_mint_position_args(args.clone(), caller)?;

    let pool_id = validated_args.pool_id.clone();
    let token0 = args.pool.token0;
    let token1 = args.pool.token1;
    let max_deposit = BalanceDelta::new(validated_args.amount0_max, validated_args.amount1_max);

    let user_balance = read_state(|s| {
        BalanceDelta::new(
            s.get_user_balance(&UserBalanceKey {
                user: caller,
                token: token0,
            })
            .0
            .try_into()
            .unwrap_or(I256::MAX), // Safe due to balance constraints
            s.get_user_balance(&UserBalanceKey {
                user: caller,
                token: token1,
            })
            .0
            .try_into()
            .unwrap_or(I256::MAX),
        )
    });

    // Check if additional deposits are needed
    let deposit_needed = max_deposit
        .sub(user_balance)
        .map_err(|_| MintPositionError::AmountOverflow)?;

    // Prepare account for deposits
    let mut from: Account = caller.into();
    if let Some(subaccount) = args.from_subaccount {
        from.subaccount = Some(subaccount);
    }

    // Perform deposits if needed
    deposit_if_needed(
        caller,
        token0,
        deposit_needed.amount0,
        &from,
        max_deposit.amount0,
        &mut DepositMemo::MintPotion {
            sender: caller,
            amount: U256::ZERO,
        },
    )
    .await?;
    deposit_if_needed(
        caller,
        token1,
        deposit_needed.amount1,
        &from,
        max_deposit.amount1,
        &mut DepositMemo::MintPotion {
            sender: caller,
            amount: U256::ZERO,
        },
    )
    .await?;

    // Execute minting
    execute_mint_position(caller, pool_id, token0, token1, validated_args)
}

/// Deposits tokens if the required amount is positive, updating user balance on success.
async fn deposit_if_needed(
    caller: Principal,
    token: Principal,
    amount: I256,
    from: &Account,
    max_amount: I256,
    memo: &mut DepositMemo,
) -> Result<(), MintPositionError> {
    if amount > I256::ZERO {
        let deposit_amount = u256_to_big_uint(amount.as_u256());
        memo.set_amount(amount.as_u256());
        LedgerClient::new(token)
            .deposit(*from, deposit_amount, memo.clone())
            .await
            .map_err(|e| MintPositionError::DepositError(e.into()))?;

        mutate_state(|s| {
            s.update_user_balance(
                UserBalanceKey {
                    user: caller,
                    token,
                },
                UserBalance(max_amount.as_u256()),
            );
        });
    }
    Ok(())
}

#[update]
async fn burn(args: BurnPositionArgs) -> Result<(), BurnPositionError> {
    // TODO: Principal Lock to be implemented

    // Validate inputs and caller
    let caller = validate_caller_not_anonymous();
    let validated_args = validate_burn_position_args(args.clone(), caller)?;

    let pool_id = validated_args.pool_id.clone();
    let token0 = args.pool.token0;
    let token1 = args.pool.token1;

    let user_balance_after_burn =
        execute_burn_position(caller, pool_id, token0, token1, validated_args)?;

    let token0_fee = LedgerClient::new(token0).icrc_fee().await.map_err(|_e| {
        BurnPositionError::BurntPositionWithdrawalFailed(WithdrawalError::FeeUnknown)
    })?;

    let token1_fee = LedgerClient::new(token1).icrc_fee().await.map_err(|_e| {
        BurnPositionError::BurntPositionWithdrawalFailed(WithdrawalError::FeeUnknown)
    })?;

    let to_account = Account::from(caller);

    let _ = _withdraw(
        caller,
        token0,
        user_balance_after_burn.amount0(),
        user_balance_after_burn.amount0,
        token0_fee,
        &to_account,
        &mut WithdrawalMemo::BurnPotions {
            receiver: caller,
            amount: U256::ZERO,
        },
    )
    .await
    .map_err(|e| BurnPositionError::BurntPositionWithdrawalFailed(e.into()));

    let _ = _withdraw(
        caller,
        token1,
        user_balance_after_burn.amount1(),
        user_balance_after_burn.amount1,
        token1_fee,
        &to_account,
        &mut WithdrawalMemo::BurnPotions {
            receiver: caller,
            amount: U256::ZERO,
        },
    )
    .await
    .map_err(|e| BurnPositionError::BurntPositionWithdrawalFailed(e.into()));

    Ok(())
}

#[update]
async fn swap(args: SwapArgs) -> Result<(), SwapError> {
    let validated_swap_args = validate_swap_args(args);
    let caller = validate_caller_not_anonymous();

    todo!();
}

/// Quotes the output amount for an exact input or input amount for an exact output swap.
/// Returns the quoted amount as a `Nat`. Uses `swap_inner` to simulate swaps without modifying state.
/// Executes as a query, ensuring no state changes on the Internet Computer.
#[query]
pub fn quote(args: QuoteArgs) -> Result<Nat, QuoteError> {
    let result_amount = match args {
        // --- Single-Hop Exact Input ---
        QuoteArgs::QuoteExactInputSingleParams(params) => process_single_hop_exact_input(params)?,
        // --- Multi-Hop Exact Input ---
        QuoteArgs::QuoteExactInputParams(params) => process_multi_hop_exact_input(params)?,
        // --- Single-Hop Exact Output ---
        QuoteArgs::QuoteExactOutputSingleParams(params) => process_single_hop_exact_output(params)?,
        // --- Multi-Hop Exact Output ---
        QuoteArgs::QuoteExactOutput(params) => process_multi_hop_exact_output(params)?,
    };

    // Convert result to Nat for Candid output
    Ok(Nat::from(u256_to_big_uint(result_amount)))
}

//
//#[query]
//fn quote_exact(args: QuoteArgs) -> Result<Nat, QuoteError> {
//    let quote_amount: Result<U256, QuoteError> = match args {
//        QuoteArgs::QuoteExactInputSingleParams(quote_exact_single_params) => {
//            let pool_id: PoolId = quote_exact_single_params
//                .pool_id
//                .try_into()
//                .map_err(|_e| QuoteError::PoolNotInitialized)?;
//
//            let sqrt_price_limit_x96 = if quote_exact_single_params.zero_for_one {
//                *MIN_SQRT_RATIO + 1
//            } else {
//                *MAX_SQRT_RATIO - 1
//            };
//
//            let amount_specified = big_uint_to_u256(quote_exact_single_params.exact_amount.0)
//                .map_err(|_| QuoteError::InvalidAmount)?
//                .as_i256();
//
//            let swap_params = SwapParams {
//                pool_id,
//                amount_specified: -amount_specified,
//                zero_for_one: quote_exact_single_params.zero_for_one,
//                sqrt_price_limit_x96,
//            };
//
//            let result = swap_inner(swap_params)?;
//
//            let amount_out = if quote_exact_single_params.zero_for_one {
//                result.swap_delta.amount1()
//            } else {
//                result.swap_delta.amount0()
//            };
//
//            Ok(amount_out.as_u256())
//        }
//        QuoteArgs::QuoteExactInputParams(quote_exact_params) => {
//            let path_length = quote_exact_params.path.len();
//            if (path_length as u8) < MIN_PATH_LENGTH || (path_length as u8) > MAX_PATH_LENGTH {
//                return Err(QuoteError::InvalidPathLength);
//            };
//
//            let mut input_token = quote_exact_params.exact_token;
//
//            let mut amount_in = big_uint_to_u256(quote_exact_params.exact_amount.0)
//                .map_err(|_| QuoteError::InvalidAmount)?
//                .as_i256();
//
//            for candid_path in quote_exact_params.path {
//                let path_key =
//                    PathKey::try_from(candid_path).map_err(|_| QuoteError::InvalidFee)?;
//                let swap = path_key.get_pool_and_swap_direction(input_token);
//
//                let sqrt_price_limit_x96 = if swap.zero_for_one {
//                    *MIN_SQRT_RATIO + 1
//                } else {
//                    *MAX_SQRT_RATIO - 1
//                };
//
//                let swap_params = SwapParams {
//                    pool_id: swap.pool_id,
//                    amount_specified: -amount_in,
//                    zero_for_one: swap.zero_for_one,
//                    sqrt_price_limit_x96,
//                };
//
//                let result = swap_inner(swap_params)?;
//
//                amount_in = if swap.zero_for_one {
//                    result.swap_delta.amount1()
//                } else {
//                    result.swap_delta.amount0()
//                };
//                input_token = path_key.intermediary_token;
//            }
//
//            // amountIn after the loop actually holds the amountOut of the trade
//            Ok(amount_in.as_u256())
//        }
//        QuoteArgs::QuoteExactOutputSingleParams(quote_exact_single_params) => {
//            let pool_id: PoolId = quote_exact_single_params
//                .pool_id
//                .try_into()
//                .map_err(|_e| QuoteError::PoolNotInitialized)?;
//
//            let sqrt_price_limit_x96 = if quote_exact_single_params.zero_for_one {
//                *MIN_SQRT_RATIO + 1
//            } else {
//                *MAX_SQRT_RATIO - 1
//            };
//
//            let amount_specified = big_uint_to_u256(quote_exact_single_params.exact_amount.0)
//                .map_err(|_| QuoteError::InvalidAmount)?
//                .as_i256();
//
//            let swap_params = SwapParams {
//                pool_id,
//                amount_specified,
//                zero_for_one: quote_exact_single_params.zero_for_one,
//                sqrt_price_limit_x96,
//            };
//
//            let result = swap_inner(swap_params)?;
//
//            let amount_in = if quote_exact_single_params.zero_for_one {
//                result.swap_delta.amount0()
//            } else {
//                result.swap_delta.amount1()
//            };
//
//            Ok((-amount_in).as_u256())
//        }
//        QuoteArgs::QuoteExactOutput(quote_exact_params) => {
//            let path_length = quote_exact_params.path.len();
//            if (path_length as u8) < MIN_PATH_LENGTH || (path_length as u8) > MAX_PATH_LENGTH {
//                return Err(QuoteError::InvalidPathLength);
//            };
//
//            let mut output_token = quote_exact_params.exact_token;
//
//            let mut amount_out = big_uint_to_u256(quote_exact_params.exact_amount.0)
//                .map_err(|_| QuoteError::InvalidAmount)?
//                .as_i256();
//
//            for candid_path in quote_exact_params.path.into_iter().rev() {
//                let path_key =
//                    PathKey::try_from(candid_path).map_err(|_| QuoteError::InvalidFee)?;
//                let swap = path_key.get_pool_and_swap_direction(output_token);
//
//                let one_for_zero = swap.zero_for_one;
//
//                let sqrt_price_limit_x96 = if swap.zero_for_one {
//                    *MIN_SQRT_RATIO + 1
//                } else {
//                    *MAX_SQRT_RATIO - 1
//                };
//
//                let swap_params = SwapParams {
//                    pool_id: swap.pool_id,
//                    amount_specified: amount_out,
//                    zero_for_one: !one_for_zero,
//                    sqrt_price_limit_x96,
//                };
//
//                let result = swap_inner(swap_params)?;
//
//                amount_out = if one_for_zero {
//                    -result.swap_delta.amount1()
//                } else {
//                    -result.swap_delta.amount0()
//                };
//
//                output_token = path_key.intermediary_token;
//            }
//
//            // amountOut after the loop exits actually holds the amountIn of the trade
//            Ok(amount_out.as_u256())
//        }
//    };
//
//    Ok(Nat::from(u256_to_big_uint(quote_amount?)))
//}

/// withdraws tokens if the required amount is positive, updating user balance on success.
async fn _withdraw(
    caller: Principal,
    token: Principal,
    user_balance: I256,
    amount: I256,
    icrc_fee: Nat,
    to: &Account,
    memo: &mut WithdrawalMemo,
) -> Result<(), LedgerTransferError> {
    let fee: U256 = big_uint_to_u256(icrc_fee.0).expect("expect fee to be positive and above 0");
    if amount - fee.as_i256() > I256::ZERO {
        let withdrawal_amount = u256_to_big_uint(amount.as_u256() - fee);
        memo.set_amount(amount.as_u256());
        LedgerClient::new(token)
            .withdraw(*to, withdrawal_amount, memo.clone())
            .await?;

        mutate_state(|s| {
            s.update_user_balance(
                UserBalanceKey {
                    user: caller,
                    token,
                },
                UserBalance((user_balance - amount).as_u256()),
            );
        });
    }
    Ok(())
}

fn main() {
    println!("Hello, world!");
}
