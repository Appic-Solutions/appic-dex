use candid::{Nat, Principal};
use ethnum::U256;
use ic_canister_log::log;
use icrc_ledger_types::icrc1::account::Account;
use serde::de::IntoDeserializer;

use crate::{
    balances::types::{UserBalance, UserBalanceKey},
    cross_chain::types::{CrosschainSwapOrder, Recipient},
    icrc_client::{memo::WithdrawMemo, LedgerClient, LedgerTransferError},
    libraries::safe_cast::{u256_to_big_uint, u256_to_nat},
    logs::DEBUG,
    minter_client::{minter_types::DexOrderArgs, MinterClient},
    state::{
        get_user_balance, memory_manager::received_swap_orders_memory_id, mutate_state, read_state,
    },
    swap::execute_swap,
    validation::swap_args,
    withdraw::{_refund, _withdraw},
};

pub mod parser;
pub mod rlp_decoder;
pub mod types;

pub const REFUND_FAILED_SWAP_GAS_LIMIT: u64 = 100_000_u64;
// the deadline is valid for 20 years and it is used for the the failed swaps that will be
// converted to usdc transfer
pub const UNLIMITED_DEADLINE: u64 = 2388441600_u64;

pub async fn execute_crosshcain_swap(args: CrosschainSwapOrder) {
    let timestamp = ic_cdk::api::time();

    match args {
        CrosschainSwapOrder::EvmToEvm {
            tx_id,
            from_address,
            recipient,
            icp_swap_request,
            evm_swap_step,
            from_minter,
            to_minter,
        } => {
            // part one execute the icp side of the swap
            match execute_swap(
                &icp_swap_request,
                icp_swap_request.token_in(),
                icp_swap_request.token_out(),
                from_minter.id,
                timestamp,
                to_minter.id,
            ) {
                // in case of successful icp swap the amount should be transferred to the minter
                // and the dex order should be then sent to the minter to be executed
                Ok((amount_in, amount_out, _token_out_transfer_fee)) => {
                    let qswap_data = evm_swap_step
                        .qswap_data
                        .expect("BUG: Gas limit should exist on evm swap");

                    let recipient = match recipient {
                        Recipient::EvmAddress(address) => address.to_string(),
                        Recipient::IcPrincipal(principal) => {
                            panic!("BUG: In EVM TO EVM swaps recipient should be and evm address")
                        }
                    };

                    let gas_limit = u256_to_nat(
                        evm_swap_step
                            .gas_limit
                            .expect("BUG: Gas limit should exist on evm swap"),
                    );

                    let erc20_ledger_burn_index = match _transfer_to_minter(
                        icp_swap_request.token_out(),
                        amount_out.as_u256(),
                        to_minter.id,
                        &mut WithdrawMemo::TransferToMinter {
                            amount: amount_out.as_u256(),
                            tx_id: tx_id.clone(),
                        },
                    )
                    .await
                    {
                        Ok(erc20_ledger_burn_index) => erc20_ledger_burn_index,
                        Err(err) => {
                            mutate_state(|s| {
                                s.record_failed_minter_transfer_notify(tx_id, todo!())
                            });
                            return;
                        }
                    };

                    let dex_order = DexOrderArgs {
                        tx_id: tx_id.0.clone(),
                        amount_in: u256_to_nat(amount_in.as_u256()),
                        min_amount_out: u256_to_nat(
                            evm_swap_step.min_amount_out.unwrap_or(U256::ZERO),
                        ),
                        commands: qswap_data.commands,
                        commands_data: qswap_data.command_data,
                        max_gas_fee_usd: evm_swap_step.gas_price_usd,
                        gas_limit,
                        deadline: u256_to_nat(qswap_data.deadline),
                        recipient,
                        erc20_ledger_burn_index,
                    };

                    let minter_client = MinterClient::new(to_minter.id);
                    match minter_client.dex_order(&dex_order).await {
                        Ok(_) => {
                            // todo!() record successful swap on the ICP side
                        }
                        Err(err) => {
                            log!(
                                DEBUG,
                                "[notify_minter]: failed to notify minter of dex order: {dex_order:?} due to error: {err:?} will retry again later",
                            );
                            mutate_state(|s| {
                                s.record_failed_minter_transfer_notify(tx_id, todo!())
                            });
                            return;
                        }
                    }
                }

                // in case of error we need to refund the user on the first chain
                Err(err) => {
                    // the icp swap step failed so the user should be refunded with the usdc of
                    // origin chain to the from address

                    let erc20_ledger_burn_index = match _transfer_to_minter(
                        icp_swap_request.token_out(),
                        icp_swap_request.deposit_amount().as_u256(),
                        from_minter.id,
                        &mut WithdrawMemo::TransferToMinter {
                            amount: icp_swap_request.deposit_amount().as_u256(),
                            tx_id: tx_id.clone(),
                        },
                    )
                    .await
                    {
                        Ok(erc20_ledger_burn_index) => erc20_ledger_burn_index,
                        Err(err) => {
                            mutate_state(|s| {
                                s.record_failed_minter_transfer_notify(tx_id, todo!())
                            });
                            return;
                        }
                    };

                    let dex_order = DexOrderArgs {
                        tx_id: tx_id.0.clone(),
                        amount_in: u256_to_nat(icp_swap_request.deposit_amount().as_u256()),
                        min_amount_out: u256_to_nat(icp_swap_request.deposit_amount().as_u256()),
                        commands: vec![],
                        commands_data: vec![],
                        max_gas_fee_usd: None,
                        gas_limit: REFUND_FAILED_SWAP_GAS_LIMIT.into(),
                        deadline: UNLIMITED_DEADLINE.into(),
                        recipient: from_address.to_string(),
                        erc20_ledger_burn_index,
                    };

                    let minter_client = MinterClient::new(from_minter.id);
                    match minter_client.dex_order(&dex_order).await {
                        Ok(_) => {
                            // todo!() record successful swap on the ICP side
                        }
                        Err(err) => {
                            log!(
                                DEBUG,
                                "[notify_minter]: failed to notify minter of dex order: {dex_order:?} due to error: {err:?} will retry again later",
                            );
                            mutate_state(|s| {
                                s.record_failed_minter_transfer_notify(tx_id, todo!())
                            });
                            return;
                        }
                    }
                }
            };
        }
        CrosschainSwapOrder::EvmToIcp {
            tx_id,
            from_address,
            recipient,
            icp_swap_request,
            from_minter,
        } => {
            let recipient = match recipient {
                Recipient::EvmAddress(address) => {
                    panic!("BUG: In EVM TO ICP swaps recipient should be and evm address")
                }
                Recipient::IcPrincipal(principal) => principal,
            };

            // part one execute the icp side of the swap
            match execute_swap(
                &icp_swap_request,
                icp_swap_request.token_in(),
                icp_swap_request.token_out(),
                from_minter.id,
                timestamp,
                recipient,
            ) {
                // in case of successful icp swap the amount should be transferred to the user
                // principal
                Ok((amount_in, amount_out, token_out_transfer_fee)) => {
                    match _withdraw(
                        icp_swap_request.token_out(),
                        amount_out.as_u256(),
                        recipient,
                        &mut &mut WithdrawMemo::SwapOut {
                            amount: amount_out.as_u256(),
                        },
                        token_out_transfer_fee,
                    )
                    .await
                    {
                        Ok(amount_sent_to_user) => todo!(),
                        Err(err) => {
                            mutate_state(|s| {
                                s.record_failed_minter_transfer_notify(tx_id, todo!())
                            });
                            return;
                        }
                    };
                }

                // in case of error we need to refund the user on the first chain
                Err(err) => {
                    // the icp swap step failed so the user should be refunded with the usdc of
                    // origin chain to the from address

                    let erc20_ledger_burn_index = match _transfer_to_minter(
                        icp_swap_request.token_in(),
                        icp_swap_request.deposit_amount().as_u256(),
                        from_minter.id,
                        &mut WithdrawMemo::TransferToMinter {
                            amount: icp_swap_request.deposit_amount().as_u256(),
                            tx_id: tx_id.clone(),
                        },
                    )
                    .await
                    {
                        Ok(erc20_ledger_burn_index) => erc20_ledger_burn_index,
                        Err(err) => {
                            mutate_state(|s| {
                                s.record_failed_minter_transfer_notify(tx_id, todo!())
                            });
                            return;
                        }
                    };

                    let dex_order = DexOrderArgs {
                        tx_id: tx_id.0.clone(),
                        amount_in: u256_to_nat(icp_swap_request.deposit_amount().as_u256()),
                        min_amount_out: u256_to_nat(icp_swap_request.deposit_amount().as_u256()),
                        commands: vec![],
                        commands_data: vec![],
                        max_gas_fee_usd: None,
                        gas_limit: REFUND_FAILED_SWAP_GAS_LIMIT.into(),
                        deadline: UNLIMITED_DEADLINE.into(),
                        recipient: from_address.to_string(),
                        erc20_ledger_burn_index,
                    };

                    let minter_client = MinterClient::new(from_minter.id);
                    match minter_client.dex_order(&dex_order).await {
                        Ok(_) => {
                            // todo!() record successful swap on the ICP side
                        }
                        Err(err) => {
                            log!(
                                DEBUG,
                                "[notify_minter]: failed to notify minter of dex order: {dex_order:?} due to error: {err:?} will retry again later",
                            );
                            mutate_state(|s| {
                                s.record_failed_minter_transfer_notify(tx_id, todo!())
                            });
                        }
                    }
                }
            };
        }
        CrosschainSwapOrder::IcpToEvm {
            tx_id,
            from,
            recipient,
            icp_swap_request,
            evm_swap_step,
            from_minter,
            to_minter,
        } => {
            // part one execute the icp side of the swap
            match execute_swap(
                &icp_swap_request,
                icp_swap_request.token_in(),
                icp_swap_request.token_out(),
                from_minter.id,
                timestamp,
                to_minter.id,
            ) {
                // in case of successful icp swap the amount should be transferred to the minter
                // and the dex order should be then sent to the minter to be executed
                Ok((amount_in, amount_out, _token_out_transfer_fee)) => {
                    let qswap_data = evm_swap_step
                        .qswap_data
                        .expect("BUG: Gas limit should exist on evm swap");

                    let recipient = match recipient {
                        Recipient::EvmAddress(address) => address.to_string(),
                        Recipient::IcPrincipal(principal) => {
                            panic!("BUG: In EVM TO EVM swaps recipient should be and evm address")
                        }
                    };

                    let gas_limit = u256_to_nat(
                        evm_swap_step
                            .gas_limit
                            .expect("BUG: Gas limit should exist on evm swap"),
                    );

                    let erc20_ledger_burn_index = match _transfer_to_minter(
                        icp_swap_request.token_out(),
                        amount_out.as_u256(),
                        to_minter.id,
                        &mut WithdrawMemo::TransferToMinter {
                            amount: amount_out.as_u256(),
                            tx_id: tx_id.clone(),
                        },
                    )
                    .await
                    {
                        Ok(erc20_ledger_burn_index) => erc20_ledger_burn_index,
                        Err(err) => {
                            mutate_state(|s| {
                                s.record_failed_minter_transfer_notify(tx_id, todo!())
                            });
                            return;
                        }
                    };

                    let dex_order = DexOrderArgs {
                        tx_id: tx_id.0.clone(),
                        amount_in: u256_to_nat(amount_in.as_u256()),
                        min_amount_out: u256_to_nat(
                            evm_swap_step.min_amount_out.unwrap_or(U256::ZERO),
                        ),
                        commands: qswap_data.commands,
                        commands_data: qswap_data.command_data,
                        max_gas_fee_usd: evm_swap_step.gas_price_usd,
                        gas_limit,
                        deadline: u256_to_nat(qswap_data.deadline),
                        recipient,
                        erc20_ledger_burn_index,
                    };

                    let minter_client = MinterClient::new(to_minter.id);
                    match minter_client.dex_order(&dex_order).await {
                        Ok(_) => {
                            // todo!() record successful swap on the ICP side
                        }
                        Err(err) => {
                            log!(
                                DEBUG,
                                "[notify_minter]: failed to notify minter of dex order: {dex_order:?} due to error: {err:?} will retry again later",
                            );
                            mutate_state(|s| {
                                s.record_failed_minter_transfer_notify(tx_id, todo!())
                            });
                            return;
                        }
                    }
                }

                // in case of error we need to refund the user on the first chain
                Err(err) => {
                    // the icp swap step failed so the user should be refunded with the usdc of
                    // origin chain to the from address

                    match _refund(
                        icp_swap_request.token_in(),
                        icp_swap_request.deposit_amount().as_u256(),
                        from,
                    )
                    .await
                    {
                        Ok(_) => {}
                        Err(err) => {
                            mutate_state(|s| {
                                s.record_failed_minter_transfer_notify(tx_id, todo!())
                            });
                            return;
                        }
                    };
                }
            };
        }
    }
}

// Withdraws tokens, updates balance, handles transfer errors with rollback
pub async fn _transfer_to_minter(
    token: Principal,
    amount: U256,
    to: Principal,
    memo: &mut WithdrawMemo,
) -> Result<Nat, LedgerTransferError> {
    let user_balance = get_user_balance(to, token);

    log!(
        DEBUG,
        "Transferring token {:?} with amount {:?} with to minter {:?} with balance {:?}",
        token.to_text(),
        amount,
        to.to_text(),
        user_balance,
    );

    // Checks for sufficient balance
    if amount > user_balance {
        panic!("BUG: THERE should alway be enough balance for the minters to swap");
    }

    // Deducts balance before transfer to prevent double-spending
    mutate_state(|s| {
        s.update_user_balance(
            UserBalanceKey { user: to, token },
            UserBalance(user_balance - amount),
        );
    });

    memo.set_amount(amount);
    match LedgerClient::new(token)
        .transfer_to_minter(to.into(), u256_to_big_uint(amount), memo.clone())
        .await
    {
        Ok(ledger_burn_index) => Ok(ledger_burn_index.0),
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
                _ => Err(err.into()),
            }
        }
    }
}
