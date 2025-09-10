use candid::{Nat, Principal};
use ethnum::U256;
use ic_canister_log::log;
use icrc_ledger_types::icrc1::account::Account;

use crate::{
    balances::types::{UserBalance, UserBalanceKey},
    candid_types::WithdrawError,
    icrc_client::{memo::WithdrawMemo, LedgerClient, LedgerTransferError},
    libraries::safe_cast::{big_uint_to_u256, u256_to_big_uint, u256_to_nat},
    logs::DEBUG,
    state::{get_user_balance, mutate_state},
};

// Withdraws tokens, updates balance, handles transfer errors with rollback
pub async fn _withdraw(
    token: Principal,
    amount: U256,
    to: Principal,
    memo: &mut WithdrawMemo,
    transfer_fee: U256,
) -> Result<U256, WithdrawError> {
    let user_balance = get_user_balance(to, token);

    log!(
        DEBUG,
        "Withdrawing token {:?} with amount {:?} with transfer fee {:?} to user {:?} with balance {:?}",
        token.to_text(),
        amount,
        transfer_fee,
        to.to_text(),
        user_balance
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
            UserBalanceKey { user: to, token },
            UserBalance(user_balance - amount),
        );
    });

    let withdrawal_amount = amount - transfer_fee;
    let icrc_fee = u256_to_big_uint(transfer_fee);
    memo.set_amount(amount);
    match LedgerClient::new(token)
        .withdraw(
            to.into(),
            u256_to_big_uint(withdrawal_amount),
            memo.clone(),
            icrc_fee,
        )
        .await
    {
        Ok(_) => Ok(withdrawal_amount),
        Err(err) => {
            // Restores balance on transfer failure
            let latest_user_balance = get_user_balance(to, token);
            mutate_state(|s| {
                s.update_user_balance(
                    UserBalanceKey { user: to, token },
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

// Refunds tokens to user, returns refunded amount after fees
pub async fn _refund(token: Principal, amount: U256, to: Principal) -> Result<U256, WithdrawError> {
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
        token,
        amount,
        to,
        &mut WithdrawMemo::Refund { amount: U256::ZERO },
        transfer_fee,
    )
    .await
}
