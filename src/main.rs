use core::panic;
use std::{fmt::format, process::Output};

use appic_dex::{
    balances::types::{UserBalance, UserBalanceKey},
    burn::execute_burn_position,
    candid_types::{
        pool::{self, CreatePoolArgs, CreatePoolError},
        position::{BurnPositionArgs, BurnPositionError, MintPositionArgs, MintPositionError},
        quote::{QuoteArgs, QuoteError},
        swap::{SwapArgs, SwapError, SwapFailedReason},
        DepositError, WithdrawalError,
    },
    guard::{PrincipalGuard, PrincipalGuardError},
    icrc_client::{
        memo::{DepositMemo, WithdrawalMemo},
        LedgerClient, LedgerTransferError,
    },
    libraries::{
        balance_delta::{self, BalanceDelta},
        constants::{DEFAULT_PROTOCOL_FEE, MAX_SQRT_RATIO, MIN_SQRT_RATIO},
        liquidity_amounts,
        path_key::{PathKey, Swap},
        safe_cast::{big_uint_to_u256, u256_to_big_uint},
        slippage_check::BalanceDeltaValidationError,
        sqrt_price_math,
        tick_math::{self, TickMath},
    },
    mint::execute_mint_position,
    pool::{
        create_pool::create_pool_inner,
        modify_liquidity::{self, modify_liquidity, ModifyLiquidityError, ModifyLiquidityParams},
        swap::{swap_inner, SwapParams},
        types::{PoolFee, PoolId, PoolState, PoolTickSpacing},
    },
    quote::{
        get_sqrt_price_limit, process_multi_hop_exact_input, process_multi_hop_exact_output,
        process_single_hop_exact_input, process_single_hop_exact_output, select_amount,
    },
    state::{mutate_state, read_state},
    tick::tick_spacing_to_max_liquidity_per_tick,
    validate_candid_args::{
        self, validate_burn_position_args, validate_mint_position_args, validate_swap_args,
        ValidatedMintPositionArgs, ValidatedSwapArgs, MAX_PATH_LENGTH, MIN_PATH_LENGTH,
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

#[update]
async fn create_pool(args: CreatePoolArgs) -> Result<(), CreatePoolError> {
    // TODO: async Token Checks
    let _ = create_pool_inner(args)?;
    Ok(())
}

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

    // Prepare account for deposits
    let mut from: Account = caller.into();
    if let Some(subaccount) = args.from_subaccount {
        from.subaccount = Some(subaccount);
    }

    // Perform deposits if needed
    deposit_if_needed(
        caller,
        token0,
        &from,
        user_balance.amount0().as_u256(),
        max_deposit.amount0().as_u256(),
        &mut DepositMemo::MintPotion {
            sender: caller,
            amount: U256::ZERO,
        },
    )
    .await
    .map_err(|e| MintPositionError::DepositError(e.into()))?;

    deposit_if_needed(
        caller,
        token1,
        &from,
        user_balance.amount1().as_u256(),
        max_deposit.amount1().as_u256(),
        &mut DepositMemo::MintPotion {
            sender: caller,
            amount: U256::ZERO,
        },
    )
    .await
    .map_err(|e| MintPositionError::DepositError(e.into()))?;

    // Execute minting
    execute_mint_position(caller, pool_id, token0, token1, validated_args)
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
        user_balance_after_burn.amount0().as_u256(),
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
        user_balance_after_burn.amount1().as_u256(),
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
    let validated_swap_args = validate_swap_args(args)?;
    let caller = validate_caller_not_anonymous();

    //
    //let _ = match validated_swap_args {
    //    ValidatedSwapArgs::ExactInputSingle {
    //        pool_id,
    //        zero_for_one,
    //        amount_in,
    //        amount_out_minimum,
    //        from_subaccount,
    //    } => {
    //        let (token_in, token_out) = if zero_for_one {
    //            (pool_id.token0, pool_id.token1)
    //        } else {
    //            (pool_id.token1, pool_id.token0)
    //        };
    //        let user_balance = read_state(|s| {
    //            s.get_user_balance(&UserBalanceKey {
    //                user: caller,
    //                token: token_in,
    //            })
    //            .0
    //        });
    //
    //        // Prepare account for deposits
    //        let mut user_address: Account = caller.into();
    //        if let Some(subaccount) = from_subaccount {
    //            user_address.subaccount = Some(subaccount);
    //        }
    //
    //        let user_balance_after_deposit = deposit_if_needed(
    //            caller,
    //            token_in,
    //            &user_address,
    //            user_balance,
    //            amount_in.as_u256(),
    //            &mut DepositMemo::SwapIn {
    //                sender: caller,
    //                amount: U256::ZERO,
    //            },
    //        )
    //        .await
    //        .map_err(|e| SwapError::DepositError(e.into()))?;
    //
    //        let sqrt_price_limit_x96 = get_sqrt_price_limit(zero_for_one);
    //        let amount_specified = amount_in;
    //
    //        let swap_params = SwapParams {
    //            pool_id,
    //            amount_specified: -amount_specified,
    //            zero_for_one,
    //            sqrt_price_limit_x96,
    //        };
    //
    //        let swap_result = match swap_inner(swap_params) {
    //            Ok(result) => result,
    //            Err(err) => {
    //                match _refund(caller, token_in, user_balance_after_deposit, &user_address).await
    //                {
    //                    Ok(_) => {
    //                        return Err(SwapError::FailedRefunded {
    //                            failed_reason: err.into(),
    //                            refund_error: None,
    //                        })
    //                    }
    //                    Err(refund_err) => {
    //                        return Err(SwapError::FailedRefunded {
    //                            failed_reason: err.into(),
    //                            refund_error: Some(refund_err.into()),
    //                        })
    //                    }
    //                }
    //            }
    //        };
    //
    //        // slippage check
    //        let amount_out = select_amount(swap_result.swap_delta, zero_for_one, false);
    //        if amount_out < amount_out_minimum {
    //            match _refund(caller, token_in, user_balance_after_deposit, &user_address).await {
    //                Ok(_) => {
    //                    return Err(SwapError::FailedRefunded {
    //                        failed_reason: SwapFailedReason::TooLittleReceived,
    //                        refund_error: None,
    //                    })
    //                }
    //                Err(refund_err) => {
    //                    return Err(SwapError::FailedRefunded {
    //                        failed_reason: SwapFailedReason::TooLittleReceived,
    //                        refund_error: Some(refund_err.into()),
    //                    })
    //                }
    //            }
    //        }
    //
    //        let user_token_in_balance = user_balance_after_deposit - amount_in.as_u256();
    //        let user_token_out_balance = read_state(|s| {
    //            s.get_user_balance(&UserBalanceKey {
    //                user: caller,
    //                token: token_out,
    //            })
    //            .0
    //        })
    //        .checked_add(amount_out.as_u256())
    //        .unwrap_or(U256::MAX);
    //
    //        // update state
    //        mutate_state(|s| {
    //            s.apply_swap_buffer_state(swap_result.buffer_state);
    //
    //            s.update_user_balance(
    //                UserBalanceKey {
    //                    user: caller,
    //                    token: token_in,
    //                },
    //                UserBalance(user_token_in_balance),
    //            );
    //
    //            s.update_user_balance(
    //                UserBalanceKey {
    //                    user: caller,
    //                    token: token_out,
    //                },
    //                UserBalance(user_token_out_balance),
    //            );
    //        });
    //
    //        // Withdraw token out
    //        let token_out_transfer_fee = LedgerClient::new(token_out)
    //            .icrc_fee()
    //            .await
    //            .map_err(|_e| SwapError::WithdrawalError(WithdrawalError::FeeUnknown))?;
    //
    //        _withdraw(
    //            caller,
    //            token_out,
    //            user_token_out_balance,
    //            token_out_transfer_fee,
    //            &user_address,
    //            &mut WithdrawalMemo::SwapOut {
    //                receiver: caller,
    //                amount: U256::ZERO,
    //            },
    //        )
    //        .await
    //        .map_err(|e| SwapError::WithdrawalError(e.into()))?;
    //    }
    //    ValidatedSwapArgs::ExactInput {
    //        path,
    //        amount_in,
    //        amount_out_minimum,
    //        from_subaccount,
    //    } => todo!(),
    //    ValidatedSwapArgs::ExactOutputSingle {
    //        pool_id,
    //        zero_for_one,
    //        amount_out,
    //        amount_in_maximum,
    //        from_subaccount,
    //    } => todo!(),
    //    ValidatedSwapArgs::ExactOutput {
    //        path,
    //        amount_out,
    //        amount_in_maximum,
    //        from_subaccount,
    //    } => todo!(),
    //};
    //
    todo!();
}

/// Quotes the output amount for an exact input or input amount for an exact output swap.
/// Returns the quoted amount as a `Nat`. Uses `swap_inner` to simulate swaps without modifying state.
/// Executes as a query, ensuring no state changes on the Internet Computer.
#[query]
pub fn quote(args: QuoteArgs) -> Result<Nat, QuoteError> {
    let quote_amount = match args {
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
    Ok(Nat::from(u256_to_big_uint(quote_amount)))
}

/// refund, a wrapper around _withdraw for better readability
async fn _refund(
    caller: Principal,
    token: Principal,
    user_balance: U256,
    to: &Account,
) -> Result<(), LedgerTransferError> {
    let token_fee = LedgerClient::new(token)
        .icrc_fee()
        .await
        .map_err(|_e| LedgerTransferError::FeeUnknown)?;

    _withdraw(
        caller,
        token,
        user_balance,
        token_fee,
        to,
        &mut WithdrawalMemo::Refund {
            receiver: to.owner,
            amount: U256::ZERO,
        },
    )
    .await
}

/// withdraws tokens if the required amount is positive, updating user balance on success.
async fn _withdraw(
    caller: Principal,
    token: Principal,
    user_balance: U256,
    icrc_fee: Nat,
    to: &Account,
    memo: &mut WithdrawalMemo,
) -> Result<(), LedgerTransferError> {
    let fee: U256 = big_uint_to_u256(icrc_fee.0).expect("expect fee to be positive and above 0");
    if user_balance - fee > U256::ZERO {
        let withdrawal_amount = u256_to_big_uint(user_balance - fee);
        memo.set_amount(user_balance);
        LedgerClient::new(token)
            .withdraw(*to, withdrawal_amount, memo.clone())
            .await?;

        mutate_state(|s| {
            s.update_user_balance(
                UserBalanceKey {
                    user: caller,
                    token,
                },
                UserBalance(U256::ZERO),
            );
        });
    }
    Ok(())
}

/// Deposits tokens if the required amount is positive, updating user balance on success.
/// returns user balance after deposit
async fn deposit_if_needed(
    caller: Principal,
    token: Principal,
    from: &Account,
    user_current_balance: U256,
    desired_user_balance: U256,
    memo: &mut DepositMemo,
) -> Result<U256, DepositError> {
    if desired_user_balance > user_current_balance {
        let deposit_amount = desired_user_balance - user_current_balance;
        memo.set_amount(deposit_amount);
        LedgerClient::new(token)
            .deposit(*from, u256_to_big_uint(deposit_amount), memo.clone())
            .await?;

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

fn main() {
    println!("Hello, world!");
}
