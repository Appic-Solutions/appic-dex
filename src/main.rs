use appic_dex::{
    balances::types::{UserBalance, UserBalanceKey},
    burn::execute_burn_position,
    candid_types::{
        pool::{CandidPoolId, CreatePoolArgs, CreatePoolError},
        position::{
            BurnPositionArgs, BurnPositionError, IncreaseLiquidtyArgs, IncreaseLiquidtyError,
            MintPositionArgs, MintPositionError,
        },
        quote::{QuoteArgs, QuoteError},
        swap::{CandidSwapSuccess, SwapArgs, SwapError},
        DepositArgs, DepositError, UserBalanceArgs, WithdrawalError,
    },
    guard::PrincipalGuard,
    icrc_client::{
        memo::{DepositMemo, WithdrawalMemo},
        LedgerClient, LedgerTransferError,
    },
    increase_liquidity::execute_increase_liquidty,
    libraries::{
        balance_delta::BalanceDelta,
        safe_cast::{big_uint_to_u256, u256_to_big_uint, u256_to_nat},
    },
    mint::execute_mint_position,
    pool::{
        create_pool::create_pool_inner,
        types::{PoolFee, PoolTickSpacing},
    },
    quote::{
        process_multi_hop_exact_input, process_multi_hop_exact_output,
        process_single_hop_exact_input, process_single_hop_exact_output,
    },
    state::{mutate_state, read_state},
    swap::execute_swap,
    validation::{
        burn_args::validate_burn_position_args, increase_args::validate_increase_liquidty_args,
        mint_args::validate_mint_position_args, swap_args::validate_swap_args,
    },
};

use candid::{Nat, Principal};
use ethnum::{I256, U256};
use ic_cdk::{init, query, update};
use icrc_ledger_types::icrc1::account::Account;

fn validate_caller_not_anonymous() -> candid::Principal {
    let principal = ic_cdk::caller();
    if principal == candid::Principal::anonymous() {
        panic!("anonymous principal is not allowed");
    }
    principal
}

#[init]
async fn init() {
    let fee_to_tick_spacnig = vec![
        (100_u32, 1_i32),
        (500_u32, 10i32),
        (3000_u32, 60_i32),
        (10_000_u32, 200_i32),
    ];

    for (fee, tick_spacing) in fee_to_tick_spacnig {
        mutate_state(|s| s.set_tick_spacing(PoolFee(fee), PoolTickSpacing(tick_spacing)));
    }
}

#[update]
async fn create_pool(args: CreatePoolArgs) -> Result<CandidPoolId, CreatePoolError> {
    // get the transfer fee for both tokens, meanwhile by getting the fee we also partially
    // validate token's standard
    let token_a_fee = big_uint_to_u256(
        LedgerClient::new(args.token_a)
            .icrc_fee()
            .await
            .expect("A problem was found in the token canister")
            .0,
    )
    .map_err(|_| CreatePoolError::InvalidToken(args.token_a))?;

    let token_b_fee = big_uint_to_u256(
        LedgerClient::new(args.token_a)
            .icrc_fee()
            .await
            .expect("A problem was found in the token canister")
            .0,
    )
    .map_err(|_| CreatePoolError::InvalidToken(args.token_b))?;

    let pool_id = create_pool_inner(args, token_a_fee, token_b_fee)?;

    Ok(pool_id.into())
}

#[update]
async fn mint_position(args: MintPositionArgs) -> Result<(), MintPositionError> {
    // Validate inputs and caller
    let caller = validate_caller_not_anonymous();

    // Principal Lock to prevent double processing(double spending, over paying, and under
    // paying)
    let _principal_guard = match PrincipalGuard::new_general_guard(caller) {
        Ok(gurad) => gurad,
        Err(_) => return Err(MintPositionError::LockedPrinciapl),
    };

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

async fn increase_liquidity(args: IncreaseLiquidtyArgs) -> Result<(), IncreaseLiquidtyError> {
    // Validate inputs and caller
    let caller = validate_caller_not_anonymous();

    // Principal Lock to prevent double processing(double spending, over paying, and under
    // paying)
    let _principal_guard = match PrincipalGuard::new_general_guard(caller) {
        Ok(gurad) => gurad,
        Err(_) => return Err(IncreaseLiquidtyError::LockedPrinciapl),
    };

    let validated_args = validate_increase_liquidty_args(args.clone(), caller)?;

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
    .map_err(|e| IncreaseLiquidtyError::DepositError(e.into()))?;

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
    .map_err(|e| IncreaseLiquidtyError::DepositError(e.into()))?;

    // Execute minting
    execute_increase_liquidty(caller, pool_id, token0, token1, validated_args)
}

#[update]
async fn burn(args: BurnPositionArgs) -> Result<(), BurnPositionError> {
    // Validate inputs and caller
    let caller = validate_caller_not_anonymous();

    // Principal Lock to prevent double processing(double spending, over paying, and under
    // paying)
    let _principal_guard = match PrincipalGuard::new_general_guard(caller) {
        Ok(gurad) => gurad,
        Err(_) => return Err(BurnPositionError::LockedPrinciapl),
    };

    let validated_args = validate_burn_position_args(args.clone(), caller)?;

    let pool_id = validated_args.pool_id.clone();
    let token0 = args.pool.token0;
    let token1 = args.pool.token1;

    let user_balance_after_burn =
        execute_burn_position(caller, pool_id.clone(), token0, token1, validated_args)?;

    let (token0_transfer_fee, token1_transfer_fee) = read_state(|s| {
        let pool_state = s.get_pool(&pool_id).unwrap();
        (
            pool_state.token0_transfer_fee,
            pool_state.token1_transfer_fee,
        )
    });

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
        token0_transfer_fee,
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
        token1_transfer_fee,
    )
    .await
    .map_err(|e| BurnPositionError::BurntPositionWithdrawalFailed(e.into()));

    Ok(())
}

#[update]
pub fn decrease_liquidity() -> Result<(), ()> {
    todo!()
}

#[update]
/// Executes a swap by depositing input tokens, swapping, and withdrawing output tokens.
/// Refunds the deposited amount on failure. Returns the input and output amounts on success.
async fn swap(args: SwapArgs) -> Result<CandidSwapSuccess, SwapError> {
    // Validate Inputs
    let validated_swap_args = validate_swap_args(args)?;

    ic_cdk::println!("{:?}", validated_swap_args);
    let caller = validate_caller_not_anonymous();

    // swap is desinged in a way that the same canister or user can send multiple swap request at a
    // time, but if there is any liquidity modification in proccess, the swapping should not be
    // allowed
    let _guard = match PrincipalGuard::new_swap_guard(caller) {
        Ok(guard) => guard,
        Err(_err) => return Err(SwapError::LockedPrincipal),
    };

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
    .map_err(|e| SwapError::DepositError(e))?;

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
                swap_delta.2,
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
/// return refunded amount(initial refund amount - transfer fee)
async fn _refund(
    caller: Principal,
    token: Principal,
    amount: U256,
    to: &Account,
) -> Result<U256, WithdrawalError> {
    let transfer_fee = big_uint_to_u256(
        LedgerClient::new(token)
            .icrc_fee()
            .await
            .map_err(|_| WithdrawalError::FeeUnknown)?
            .0,
    )
    .map_err(|_| WithdrawalError::FeeUnknown)?;

    _withdraw(
        caller,
        token,
        amount,
        to,
        &mut WithdrawalMemo::Refund {
            receiver: to.owner,
            amount: U256::ZERO,
        },
        transfer_fee,
    )
    .await
}

/// withdraws tokens if there is sufficient user balance, and update the user state balance
/// returns withrdrew amount(initial withdraw amount - transfer fee)
async fn _withdraw(
    caller: Principal,
    token: Principal,
    amount: U256,
    to: &Account,
    memo: &mut WithdrawalMemo,
    transfer_fee: U256,
) -> Result<U256, WithdrawalError> {
    let user_balance = get_user_balance(caller, token);

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
        Ok(_) => return Ok(withdrawal_amount),
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

            match err {
                LedgerTransferError::BadFee { expected_fee } => {
                    let new_transfer_fee = big_uint_to_u256(expected_fee.0)
                        .map_err(|_| WithdrawalError::FeeUnknown)?;

                    // update token transfer fee across all pools
                    mutate_state(|s| {
                        s.update_token_trnasfer_fee_across_all_pools(token, new_transfer_fee)
                    });
                    return Err(WithdrawalError::FeeUnknown);
                }
                _ => {
                    return Err(err.into());
                }
            }
        }
    };
}

#[update]
async fn deposit(deposit_args: DepositArgs) -> Result<(), DepositError> {
    let caller = validate_caller_not_anonymous();

    let mut from = Account::from(caller);
    if let Some(subaccount) = deposit_args.from_subaccount {
        from.subaccount = Some(subaccount);
    }

    let amount =
        big_uint_to_u256(deposit_args.amount.0).map_err(|_| DepositError::AmountOverflow)?;
    _deposit(
        caller,
        deposit_args.token,
        &from,
        amount,
        &mut DepositMemo::Deposit {
            sender: from.owner,
            amount: U256::ZERO,
        },
    )
    .await
}

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

/// Deposits tokens, then update user balance on success.
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

fn main() {}

// Enable Candid export
ic_cdk::export_candid!();
