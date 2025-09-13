use candid::Principal;
use ethnum::U256;
use ic_canister_log::log;
use icrc_ledger_types::icrc1::account::Account;

use crate::{
    balances::types::{UserBalance, UserBalanceKey},
    candid_types::DepositError,
    icrc_client::{memo::DepositMemo, LedgerClient, TransferIndex},
    libraries::safe_cast::u256_to_big_uint,
    logs::DEBUG,
    state::{get_user_balance, mutate_state},
};

// Internal function to deposit tokens and return ledger index
pub async fn _deposit(
    token: Principal,
    from: Account,
    amount: U256,
    memo: &mut DepositMemo,
) -> Result<TransferIndex, DepositError> {
    // Sets deposit amount in memo for ledger tracking

    log!(
        DEBUG,
        "Depositing token {:?} with amount {:?} from user {:?}",
        token.to_text(),
        amount,
        from.owner.to_text(),
    );

    memo.set_amount(amount);
    let ledger_index = LedgerClient::new(token)
        .deposit(from, u256_to_big_uint(amount), memo.clone())
        .await?;

    // Updates user balance, caps at U256::MAX to prevent overflow
    let latest_user_balance = get_user_balance(from.owner, token);
    mutate_state(|s| {
        s.update_user_balance(
            UserBalanceKey {
                user: from.owner,
                token,
            },
            UserBalance(latest_user_balance.checked_add(amount).unwrap_or(U256::MAX)),
        );
    });

    Ok(ledger_index)
}

// Deposits tokens if current balance is insufficient, returns updated balance
pub async fn _deposit_if_needed(
    token: Principal,
    from: Account,
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
            from.owner.to_text(),
        );

        memo.set_amount(deposit_amount);
        LedgerClient::new(token)
            .deposit(from, u256_to_big_uint(deposit_amount), memo.clone())
            .await?;

        // Updates user balance to desired amount
        mutate_state(|s| {
            s.update_user_balance(
                UserBalanceKey {
                    user: from.owner,
                    token,
                },
                UserBalance(desired_user_balance),
            );
        });
        return Ok(desired_user_balance);
    }
    Ok(user_current_balance)
}
