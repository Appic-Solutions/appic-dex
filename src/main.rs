use core::panic;
use std::{fmt::format, process::Output};

use appic_dex::{
    balances::types::{UserBalance, UserBalanceKey},
    burn::execute_burn_position,
    candid_types::{
        pool::{self, CreatePoolArgs, CreatePoolError},
        position::{BurnPositionArgs, BurnPositionError, MintPositionArgs, MintPositionError},
        quote::{QuoteArgs, QuoteError},
        swap::{self, CandidSwapSuccess, SwapArgs, SwapError, SwapFailedReason},
        DepositError, WithdrawalError,
    },
    guard::{PrincipalGuard, PrincipalGuardError},
    icrc_client::{
        memo::{DepositMemo, WithdrawalMemo},
        LedgerClient, LedgerTransferError,
    },
    libraries::{
        amount_delta,
        balance_delta::{self, BalanceDelta},
        constants::{DEFAULT_PROTOCOL_FEE, MAX_SQRT_RATIO, MIN_SQRT_RATIO},
        liquidity_amounts,
        path_key::{PathKey, Swap},
        safe_cast::{big_uint_to_u256, u256_to_big_uint, u256_to_nat},
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
    swap::{execute_swap, get_token_in_out},
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
    _deposit_if_needed(
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

    _deposit_if_needed(
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

    let to_account = Account::from(caller);

    let _ = _withdraw(
        caller,
        token0,
        user_balance_after_burn.amount0().as_u256(),
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
/// Executes a swap by depositing input tokens, swapping, and withdrawing output tokens.
/// Refunds the deposited amount on failure. Returns the input and output amounts on success.
async fn swap(args: SwapArgs) -> Result<CandidSwapSuccess, SwapError> {
    // Validate Inputs
    let validated_swap_args = validate_swap_args(args)?;
    let caller = validate_caller_not_anonymous();

    // Prepare User Account
    let mut user_address: Account = caller.into();
    if let Some(subaccount) = validated_swap_args.from_subaccount() {
        user_address.subaccount = Some(subaccount);
    }

    // Perform Deposit
    let deposit_amount = validated_swap_args.deposit_amount();

    let token_in = validated_swap_args.token_in();
    let token_out = validated_swap_args.token_out();

    let _ = _deposit(
        caller,
        token_in,
        &user_address,
        deposit_amount.as_u256(),
        &mut DepositMemo::SwapIn {
            sender: caller,
            amount: U256::ZERO,
        },
    )
    .await
    .map_err(|e| SwapError::DepositError(e));

    // Execute Swap
    let swap_result = execute_swap(&validated_swap_args, token_in, token_out, caller);

    // Handle Swap Result
    match swap_result {
        Ok(swap_delta) => {
            // Withdraw output tokens
            _withdraw(
                caller,
                token_out,
                swap_delta.1.as_u256(),
                &user_address,
                &mut WithdrawalMemo::SwapOut {
                    receiver: caller,
                    amount: U256::ZERO,
                },
            )
            .await
            .map_err(|e| SwapError::FailedToWithdraw {
                reason: e,
                amount_in: u256_to_nat(swap_delta.0.as_u256()),
                amount_out: u256_to_nat(swap_delta.1.as_u256()),
            })?;

            return Ok(CandidSwapSuccess {
                amount_in: u256_to_nat(swap_delta.0.as_u256()),
                amount_out: u256_to_nat(swap_delta.1.as_u256()),
            });
        }
        Err(err) => {
            // Refund deposited tokens
            _refund(caller, token_in, deposit_amount.as_u256(), &user_address)
                .await
                .map_err(|e| SwapError::SwapFailedRefunded {
                    failed_reason: err.clone(),
                    refund_error: Some(e),
                })?;

            return Err(SwapError::SwapFailedRefunded {
                failed_reason: err,
                refund_error: None,
            });
        }
    }
}

/// Quotes the output amount for an exact input or input amount for an exact output swap.
/// Returns the quoted amount as a `Nat`. Uses `swap_inner` to simulate swaps without modifying state.
/// Executes as a query, ensuring no state changes on the Internet Computer.
#[query]
pub fn quote(args: QuoteArgs) -> Result<Nat, QuoteError> {
    let quote_amount = match args {
        //  Single-Hop Exact Input
        QuoteArgs::QuoteExactInputSingleParams(params) => process_single_hop_exact_input(params)?,
        //  Multi-Hop Exact Input
        QuoteArgs::QuoteExactInputParams(params) => process_multi_hop_exact_input(params)?,
        //  Single-Hop Exact Output
        QuoteArgs::QuoteExactOutputSingleParams(params) => process_single_hop_exact_output(params)?,
        //  Multi-Hop Exact Output
        QuoteArgs::QuoteExactOutput(params) => process_multi_hop_exact_output(params)?,
    };

    // Convert result to Nat for Candid output
    Ok(Nat::from(u256_to_big_uint(quote_amount)))
}

/// refund, a wrapper around _withdraw for better readability
async fn _refund(
    caller: Principal,
    token: Principal,
    amount: U256,
    to: &Account,
) -> Result<(), WithdrawalError> {
    _withdraw(
        caller,
        token,
        amount,
        to,
        &mut WithdrawalMemo::Refund {
            receiver: to.owner,
            amount: U256::ZERO,
        },
    )
    .await
}

/// withdraws tokens if there is sufficient user balance, and update the user state balance
async fn _withdraw(
    caller: Principal,
    token: Principal,
    amount: U256,
    to: &Account,
    memo: &mut WithdrawalMemo,
) -> Result<(), WithdrawalError> {
    let ledger_client = LedgerClient::new(token);
    let icrc_fee = ledger_client
        .icrc_fee()
        .await
        .map_err(|_| WithdrawalError::FeeUnknown)?;

    let user_balance = get_user_balance(caller, token);

    let transfer_fee: U256 =
        big_uint_to_u256(icrc_fee.0).map_err(|_| WithdrawalError::FeeUnknown)?;

    if amount.checked_sub(transfer_fee).is_none() {
        return Err(WithdrawalError::AmountTooLow {
            min_withdrawal_amount: Nat::from(u256_to_big_uint(transfer_fee)),
        });
    }

    if amount > user_balance {
        return Err(WithdrawalError::InsufficientBalance {
            balance: Nat::from(u256_to_big_uint(user_balance)),
        });
    }

    // we first deduct the use balance
    // in case of transfer failure, we increase the user balance
    // this is done to prevent double spending
    mutate_state(|s| {
        s.update_user_balance(
            UserBalanceKey {
                user: caller,
                token,
            },
            UserBalance(user_balance - amount),
        );
    });

    let withdrawal_amount = u256_to_big_uint(amount - transfer_fee);
    memo.set_amount(amount);
    match LedgerClient::new(token)
        .withdraw(*to, withdrawal_amount, memo.clone())
        .await
    {
        Ok(_) => return Ok(()),
        Err(err) => {
            // transfer failed we need to add the remove balance to user
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
            return Err(err.into());
        }
    };
}

/// Deposits tokens, thenupdate user balance on success.
async fn _deposit(
    caller: Principal,
    token: Principal,
    from: &Account,
    amount: U256,
    memo: &mut DepositMemo,
) -> Result<(), DepositError> {
    memo.set_amount(amount);
    LedgerClient::new(token)
        .deposit(*from, u256_to_big_uint(amount), memo.clone())
        .await?;

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

/// Deposits tokens if the required amount is positive, updating user balance on success.
/// returns user balance after deposit
async fn _deposit_if_needed(
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

pub fn get_user_balance(user: Principal, token: Principal) -> U256 {
    read_state(|s| s.get_user_balance(&UserBalanceKey { user, token }).0)
}

fn main() {
    println!("Hello, world!");
}
