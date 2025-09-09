use candid::Principal;
use ethnum::U256;
use ic_canister_log::log;
use icrc_ledger_types::icrc1::account::Account;

use crate::{
    balances::types::{UserBalance, UserBalanceKey},
    candid_types::WithdrawError,
    cross_chain::types::CrosschainSwapOrder,
    icrc_client::{memo::WithdrawMemo, LedgerClient, LedgerTransferError},
    libraries::safe_cast::u256_to_big_uint,
    logs::DEBUG,
    state::{get_user_balance, mutate_state},
    swap::execute_swap,
};

pub mod parser;
pub mod rlp_decoder;
pub mod types;

pub async fn execute_crosshcain_swap(args: CrosschainSwapOrder) {
    match args {
        CrosschainSwapOrder::EvmToEvm {
            swap_tx_id,
            from_address,
            recipient,
            icp_swap_request,
            evm_swap_step,
            from_minter,
            to_minter,
        } => {
            let timestamp = ic_cdk::api::time();

            //let dex_order=if evm_swap_step

            // part one execute the icp side of the swap
            let icp_swap_result = match execute_swap(
                &icp_swap_request,
                icp_swap_request.token_in(),
                icp_swap_request.token_out(),
                from_minter.id,
                timestamp,
            ) {
                // in case of successful icp swap the amount should be transferred to the minter
                // and the dex order should be then sent to the minter to be executed
                Ok((amount_in, amount_out, _token_out_transfer_fee)) => {} // transfer to
                // minter and then notify minter with dex order,

                // in case of error we need to refund the user on the first chain
                Err(err) => {}
            };
        }
        CrosschainSwapOrder::EvmToIcp {
            swap_tx_id,
            from_address,
            recipient,
            amount_in,
            icp_swap_step,
            from_minter,
        } => todo!(),
        CrosschainSwapOrder::IcpToEvm {
            swap_tx_id,
            from,
            recipient,
            amount_in,
            evm_swap_step,
            minter,
            to_minter,
        } => todo!(),
    }
}

// Withdraws tokens, updates balance, handles transfer errors with rollback
pub async fn _transfer_to_minter(
    caller: Principal,
    token: Principal,
    amount: U256,
    to: &Account,
    memo: &mut WithdrawMemo,
) -> Result<U256, LedgerTransferError> {
    let user_balance = get_user_balance(caller, token);

    log!(
        DEBUG,
        "Transferring token {:?} with amount {:?} with to minter {:?} with balance {:?}",
        token.to_text(),
        amount,
        caller.to_text(),
        user_balance,
    );

    // Checks for sufficient balance
    if amount > user_balance {
        panic!("BUG: THERE should alway be enough balance for the minters to swap");
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

    memo.set_amount(amount);
    match LedgerClient::new(token)
        .transfer_to_minter(*to, u256_to_big_uint(amount), memo.clone())
        .await
    {
        Ok(_) => Ok(amount),
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
                _ => Err(err.into()),
            }
        }
    }
}
