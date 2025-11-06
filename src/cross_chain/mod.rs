use crate::{
    balances::types::{UserBalance, UserBalanceKey},
    cross_chain::types::{CrosschainSwapOrder, Recipient, RetryFailedDexOrder},
    events::{Event, EventType},
    icrc_client::{memo::WithdrawMemo, LedgerClient, LedgerTransferError},
    libraries::safe_cast::{u256_to_big_uint, u256_to_nat},
    logs::DEBUG,
    minter_client::{minter_types::DexOrderArgs, MinterClient},
    state::{get_user_balance, mutate_state},
    swap::execute_swap,
    withdraw::{_refund, _withdraw},
};
use candid::{Nat, Principal};
use ethnum::U256;
use ic_canister_log::log;
pub mod parser;
pub mod rlp_decoder;
pub mod swap_order;
pub mod types;

#[cfg(test)]
pub mod tests;

pub const REFUND_FAILED_SWAP_GAS_LIMIT: u64 = 100_000_u64;
// The deadline is valid for 20 years and is used for failed swaps that will be
// converted to USDC transfers.
pub const UNLIMITED_DEADLINE: u64 = 2388441600_u64;

pub async fn execute_crosschain_swap(args: CrosschainSwapOrder) {
    let timestamp = ic_cdk::api::time();
    match &args {
        CrosschainSwapOrder::EvmToEvm {
            tx_id,
            from_address,
            recipient,
            icp_swap_request,
            evm_swap_step,
            from_minter,
            to_minter,
        } => {
            log!(
                DEBUG,
                "[execute_crosschain_swap]: Starting EVM-to-EVM swap for tx_id: {:?}",
                tx_id
            );
            // Execute the ICP side of the swap.
            match execute_swap(
                icp_swap_request,
                icp_swap_request.token_in(),
                icp_swap_request.token_out(),
                from_minter.id,
                timestamp,
                to_minter.id,
                Some(tx_id.clone()),
            ) {
                // On success, transfer the output amount to the target minter and notify it with the DEX order.
                Ok((amount_in, amount_out, _token_out_transfer_fee)) => {
                    log!(
                        DEBUG,
                        "[execute_crosschain_swap]: ICP swap succeeded for tx_id: {:?}. Proceeding to transfer to target minter.",
                        tx_id
                    );
                    let qswap_data = evm_swap_step
                        .qswap_data
                        .clone()
                        .expect("BUG: qswap_data should exist for EVM swap");
                    let recipient = match recipient {
                        Recipient::EvmAddress(address) => address.to_string(),
                        Recipient::IcPrincipal(_principal) => {
                            panic!("BUG: In EVM-to-EVM swaps, recipient should be an EVM address")
                        }
                    };
                    let gas_limit = evm_swap_step
                        .gas_limit
                        .expect("BUG: Gas limit should exist for EVM swap");
                    let mut memo = WithdrawMemo::TransferToMinter {
                        amount: amount_out.as_u256(),
                        tx_id: tx_id.clone(),
                    };
                    let erc20_ledger_burn_index = match _transfer_to_minter(
                        icp_swap_request.token_out(),
                        amount_out.as_u256(),
                        to_minter.id,
                        &mut memo,
                    )
                    .await
                    {
                        Ok(erc20_ledger_burn_index) => erc20_ledger_burn_index,
                        Err(err) => {
                            log!(
                                DEBUG,
                                "[execute_crosschain_swap]: Transfer to target minter failed for tx_id: {:?} with error: {:?}. Recording for retry.",
                                tx_id,
                                err
                            );
                            mutate_state(|s| {
                                s.record_failed_dex_order_to_retry(RetryFailedDexOrder {
                                    tx_id: tx_id.clone(),
                                    token_in: icp_swap_request.token_out(),
                                    minter_id: to_minter.id,
                                    amount_in: amount_out.as_u256(),
                                    min_amount_out: evm_swap_step
                                        .min_amount_out
                                        .unwrap_or(U256::ZERO),
                                    commands: qswap_data.commands,
                                    commands_data: qswap_data.command_data,
                                    max_gas_fee_usd: evm_swap_step.gas_price_usd.clone(),
                                    gas_limit,
                                    deadline: qswap_data.deadline,
                                    recipient,
                                    erc20_ledger_burn_index: None,
                                    is_refund: false,
                                    signing_fee: evm_swap_step.canister_fee_usd.clone(),
                                })
                            });
                            return;
                        }
                    };
                    let dex_order = DexOrderArgs {
                        tx_id: tx_id.0.clone(),
                        amount_in: u256_to_nat(amount_out.as_u256()),
                        min_amount_out: u256_to_nat(
                            evm_swap_step.min_amount_out.unwrap_or(U256::ZERO),
                        ),
                        commands: qswap_data.commands.clone(),
                        commands_data: qswap_data.command_data.clone(),
                        max_gas_fee_usd: evm_swap_step.gas_price_usd.clone(),
                        gas_limit: u256_to_nat(gas_limit),
                        deadline: u256_to_nat(qswap_data.deadline),
                        recipient: recipient.clone(),
                        erc20_ledger_burn_index: erc20_ledger_burn_index.clone(),
                        is_refund: false,
                        signing_fee: evm_swap_step.canister_fee_usd.clone(),
                    };
                    let minter_client = MinterClient::new(to_minter.id);
                    match minter_client.dex_order(&dex_order).await {
                        Ok(_) => {
                            log!(
                                DEBUG,
                                "[execute_crosschain_swap]: Successfully notified target minter for tx_id: {:?}.",
                                tx_id
                            );
                            // record successful event
                            let event = Event {
                                timestamp,
                                payload: EventType::CrosschainSwap {
                                    swap_order: args.clone(),
                                    is_refunded: false,
                                    icp_amount_out: Some(amount_out.as_u256()),
                                    icp_token_in: Some(icp_swap_request.token_in()),
                                    icp_token_out: Some(icp_swap_request.token_out()),
                                    icp_amount_in: Some(amount_in.as_u256()),
                                },
                            };

                            mutate_state(|s| s.record_event(event));
                        }
                        Err(err) => {
                            log!(
                                DEBUG,
                                "[execute_crosschain_swap]: Failed to notify target minter for tx_id: {:?} with error: {:?}. Recording for retry.",
                                tx_id,
                                err
                            );
                            mutate_state(|s| {
                                s.record_failed_dex_order_to_retry(RetryFailedDexOrder {
                                    tx_id: tx_id.clone(),
                                    token_in: icp_swap_request.token_out(),
                                    minter_id: to_minter.id,
                                    amount_in: amount_out.as_u256(),
                                    min_amount_out: evm_swap_step
                                        .min_amount_out
                                        .unwrap_or(U256::ZERO),
                                    commands: qswap_data.commands,
                                    commands_data: qswap_data.command_data,
                                    max_gas_fee_usd: evm_swap_step.gas_price_usd.clone(),
                                    gas_limit,
                                    deadline: qswap_data.deadline,
                                    recipient,
                                    erc20_ledger_burn_index: Some(erc20_ledger_burn_index),
                                    is_refund: false,
                                    signing_fee: evm_swap_step.canister_fee_usd.clone(),
                                })
                            });
                        }
                    }
                }
                // On failure, refund the deposit amount to the origin minter via a refund DEX order.
                Err(err) => {
                    log!(
                        DEBUG,
                        "[execute_crosschain_swap]: ICP swap failed for tx_id: {:?} with error: {:?}. Initiating refund to origin minter.",
                        tx_id,
                        err
                    );
                    let mut memo = WithdrawMemo::TransferToMinter {
                        amount: icp_swap_request.deposit_amount().as_u256(),
                        tx_id: tx_id.clone(),
                    };
                    let erc20_ledger_burn_index = match _transfer_to_minter(
                        icp_swap_request.token_in(),
                        icp_swap_request.deposit_amount().as_u256(),
                        from_minter.id,
                        &mut memo,
                    )
                    .await
                    {
                        Ok(erc20_ledger_burn_index) => erc20_ledger_burn_index,
                        Err(err) => {
                            log!(
                                DEBUG,
                                "[execute_crosschain_swap]: Refund transfer to origin minter failed for tx_id: {:?} with error: {:?}. Recording for retry.",
                                tx_id,
                                err
                            );
                            mutate_state(|s| {
                                s.record_failed_dex_order_to_retry(RetryFailedDexOrder {
                                    tx_id: tx_id.clone(),
                                    minter_id: from_minter.id,
                                    token_in: icp_swap_request.token_in(),
                                    amount_in: icp_swap_request.deposit_amount().as_u256(),
                                    min_amount_out: icp_swap_request.deposit_amount().as_u256(),
                                    deadline: UNLIMITED_DEADLINE.into(),
                                    recipient: from_address.to_string(),
                                    erc20_ledger_burn_index: None,
                                    commands: vec![],
                                    commands_data: vec![],
                                    max_gas_fee_usd: None,
                                    gas_limit: REFUND_FAILED_SWAP_GAS_LIMIT.into(),
                                    is_refund: true,
                                    signing_fee: None,
                                })
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
                        gas_limit: u256_to_nat(REFUND_FAILED_SWAP_GAS_LIMIT.into()),
                        deadline: u256_to_nat(UNLIMITED_DEADLINE.into()),
                        recipient: from_address.to_string(),
                        erc20_ledger_burn_index: erc20_ledger_burn_index.clone(),
                        is_refund: true,
                        signing_fee: None,
                    };
                    let minter_client = MinterClient::new(from_minter.id);
                    match minter_client.dex_order(&dex_order).await {
                        Ok(_) => {
                            log!(
                                DEBUG,
                                "[execute_crosschain_swap]: Successfully notified origin minter for refund on tx_id: {:?}.",
                                tx_id
                            );
                            // TODO: Record successful refund on the ICP side.
                            // record successful event
                            let event = Event {
                                timestamp,
                                payload: EventType::CrosschainSwap {
                                    swap_order: args.clone(),
                                    is_refunded: true,
                                    icp_amount_out: None,
                                    icp_token_in: Some(icp_swap_request.token_in()),
                                    icp_token_out: Some(icp_swap_request.token_out()),
                                    icp_amount_in: Some(
                                        icp_swap_request.deposit_amount().as_u256(),
                                    ),
                                },
                            };

                            mutate_state(|s| s.record_event(event));
                        }
                        Err(err) => {
                            log!(
                                DEBUG,
                                "[execute_crosschain_swap]: Failed to notify origin minter for refund on tx_id: {:?} with error: {:?}. Recording for retry.",
                                tx_id,
                                err
                            );
                            mutate_state(|s| {
                                s.record_failed_dex_order_to_retry(RetryFailedDexOrder {
                                    tx_id: tx_id.clone(),
                                    minter_id: from_minter.id,
                                    token_in: icp_swap_request.token_in(),
                                    amount_in: icp_swap_request.deposit_amount().as_u256(),
                                    min_amount_out: icp_swap_request.deposit_amount().as_u256(),
                                    deadline: UNLIMITED_DEADLINE.into(),
                                    recipient: from_address.to_string(),
                                    erc20_ledger_burn_index: Some(erc20_ledger_burn_index),
                                    commands: vec![],
                                    commands_data: vec![],
                                    max_gas_fee_usd: None,
                                    gas_limit: REFUND_FAILED_SWAP_GAS_LIMIT.into(),
                                    is_refund: true,
                                    signing_fee: None,
                                })
                            });
                        }
                    }
                }
            }
        }
        CrosschainSwapOrder::EvmToIcp {
            tx_id,
            from_address,
            recipient,
            icp_swap_request,
            from_minter,
        } => {
            log!(
                DEBUG,
                "[execute_crosschain_swap]: Starting EVM-to-ICP swap for tx_id: {:?}",
                tx_id
            );
            let recipient = match recipient {
                Recipient::EvmAddress(_address) => {
                    panic!("BUG: In EVM-to-ICP swaps, recipient should be an ICP principal")
                }
                Recipient::IcPrincipal(principal) => principal,
            };
            // Execute the ICP side of the swap.
            match execute_swap(
                icp_swap_request,
                icp_swap_request.token_in(),
                icp_swap_request.token_out(),
                from_minter.id,
                timestamp,
                *recipient,
                Some(tx_id.clone()),
            ) {
                // On success, withdraw the output amount to the recipient's ICP principal.
                Ok((amount_in, amount_out, token_out_transfer_fee)) => {
                    log!(
                        DEBUG,
                        "[execute_crosschain_swap]: ICP swap succeeded for tx_id: {:?}. Proceeding to withdraw to recipient.",
                        tx_id
                    );

                    let mut memo = WithdrawMemo::SwapOut {
                        amount: amount_out.as_u256(),
                    };
                    match _withdraw(
                        icp_swap_request.token_out(),
                        amount_out.as_u256(),
                        *recipient,
                        &mut memo,
                        token_out_transfer_fee,
                    )
                    .await
                    {
                        Ok(_amount_sent_to_user) => {
                            log!(
                                DEBUG,
                                "[execute_crosschain_swap]: Successfully withdrew to recipient for tx_id: {:?}.",
                                tx_id
                            );
                        }
                        Err(err) => {
                            log!(
                                DEBUG,
                                "[execute_crosschain_swap]: Withdraw to recipient failed for tx_id: {:?} with error: {:?}. Tokens added to user balance for manual withdrawal.",
                                tx_id,
                                err
                            );
                            // No further action needed; tokens are in user balance for manual withdrawal.
                        }
                    };

                    let event = Event {
                        timestamp,
                        payload: EventType::CrosschainSwap {
                            swap_order: args.clone(),
                            is_refunded: false,
                            icp_amount_out: Some(amount_out.as_u256()),
                            icp_token_in: Some(icp_swap_request.token_in()),
                            icp_token_out: Some(icp_swap_request.token_out()),
                            icp_amount_in: Some(amount_in.as_u256()),
                        },
                    };

                    mutate_state(|s| s.record_event(event));
                }
                // On failure, refund the deposit amount to the origin minter via a refund DEX order.
                Err(err) => {
                    log!(
                        DEBUG,
                        "[execute_crosschain_swap]: ICP swap failed for tx_id: {:?} with error: {:?}. Initiating refund to origin minter.",
                        tx_id,
                        err
                    );
                    let mut memo = WithdrawMemo::TransferToMinter {
                        amount: icp_swap_request.deposit_amount().as_u256(),
                        tx_id: tx_id.clone(),
                    };
                    let erc20_ledger_burn_index = match _transfer_to_minter(
                        icp_swap_request.token_in(),
                        icp_swap_request.deposit_amount().as_u256(),
                        from_minter.id,
                        &mut memo,
                    )
                    .await
                    {
                        Ok(erc20_ledger_burn_index) => erc20_ledger_burn_index,
                        Err(err) => {
                            log!(
                                DEBUG,
                                "[execute_crosschain_swap]: Refund transfer to origin minter failed for tx_id: {:?} with error: {:?}. Recording for retry.",
                                tx_id,
                                err
                            );
                            mutate_state(|s| {
                                s.record_failed_dex_order_to_retry(RetryFailedDexOrder {
                                    tx_id: tx_id.clone(),
                                    minter_id: from_minter.id,
                                    token_in: icp_swap_request.token_in(),
                                    amount_in: icp_swap_request.deposit_amount().as_u256(),
                                    min_amount_out: icp_swap_request.deposit_amount().as_u256(),
                                    deadline: UNLIMITED_DEADLINE.into(),
                                    recipient: from_address.to_string(),
                                    erc20_ledger_burn_index: None,
                                    commands: vec![],
                                    commands_data: vec![],
                                    max_gas_fee_usd: None,
                                    gas_limit: REFUND_FAILED_SWAP_GAS_LIMIT.into(),
                                    is_refund: true,
                                    signing_fee: None,
                                })
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
                        gas_limit: u256_to_nat(REFUND_FAILED_SWAP_GAS_LIMIT.into()),
                        deadline: u256_to_nat(UNLIMITED_DEADLINE.into()),
                        recipient: from_address.to_string(),
                        erc20_ledger_burn_index: erc20_ledger_burn_index.clone(),
                        is_refund: true,
                        signing_fee: None,
                    };
                    let minter_client = MinterClient::new(from_minter.id);
                    match minter_client.dex_order(&dex_order).await {
                        Ok(_) => {
                            log!(
                                DEBUG,
                                "[execute_crosschain_swap]: Successfully notified origin minter for refund on tx_id: {:?}.",
                                tx_id
                            );
                            // TODO: Record successful refund on the ICP side.

                            let event = Event {
                                timestamp,
                                payload: EventType::CrosschainSwap {
                                    swap_order: args.clone(),
                                    is_refunded: true,
                                    icp_amount_out: None,
                                    icp_token_in: Some(icp_swap_request.token_in()),
                                    icp_token_out: Some(icp_swap_request.token_out()),
                                    icp_amount_in: Some(
                                        icp_swap_request.deposit_amount().as_u256(),
                                    ),
                                },
                            };

                            mutate_state(|s| s.record_event(event));
                        }
                        Err(err) => {
                            log!(
                                DEBUG,
                                "[execute_crosschain_swap]: Failed to notify origin minter for refund on tx_id: {:?} with error: {:?}. Recording for retry.",
                                tx_id,
                                err
                            );
                            mutate_state(|s| {
                                s.record_failed_dex_order_to_retry(RetryFailedDexOrder {
                                    tx_id: tx_id.clone(),
                                    minter_id: from_minter.id,
                                    token_in: icp_swap_request.token_in(),
                                    amount_in: icp_swap_request.deposit_amount().as_u256(),
                                    min_amount_out: icp_swap_request.deposit_amount().as_u256(),
                                    deadline: UNLIMITED_DEADLINE.into(),
                                    recipient: from_address.to_string(),
                                    erc20_ledger_burn_index: Some(erc20_ledger_burn_index),
                                    commands: vec![],
                                    commands_data: vec![],
                                    max_gas_fee_usd: None,
                                    gas_limit: REFUND_FAILED_SWAP_GAS_LIMIT.into(),
                                    is_refund: true,
                                    signing_fee: None,
                                })
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
            to_minter,
        } => {
            log!(
                DEBUG,
                "[execute_crosschain_swap]: Starting ICP-to-EVM swap for tx_id: {:?}",
                tx_id
            );
            // Execute the ICP side of the swap.
            match execute_swap(
                icp_swap_request,
                icp_swap_request.token_in(),
                icp_swap_request.token_out(),
                *from,
                timestamp,
                to_minter.id,
                Some(tx_id.clone()),
            ) {
                // On success, transfer the output amount to the target minter and notify it with the DEX order.
                Ok((amount_in, amount_out, _token_out_transfer_fee)) => {
                    log!(
                        DEBUG,
                        "[execute_crosschain_swap]: ICP swap succeeded for tx_id: {:?}. Proceeding to transfer to target minter.",
                        tx_id
                    );
                    let qswap_data = evm_swap_step
                        .qswap_data
                        .clone()
                        .expect("BUG: qswap_data should exist for EVM swap");
                    let recipient = match recipient {
                        Recipient::EvmAddress(address) => address.to_string(),
                        Recipient::IcPrincipal(_principal) => {
                            panic!("BUG: In ICP-to-EVM swaps, recipient should be an EVM address")
                        }
                    };
                    let gas_limit = evm_swap_step
                        .gas_limit
                        .expect("BUG: Gas limit should exist for EVM swap");
                    let mut memo = WithdrawMemo::TransferToMinter {
                        amount: amount_out.as_u256(),
                        tx_id: tx_id.clone(),
                    };
                    let erc20_ledger_burn_index = match _transfer_to_minter(
                        icp_swap_request.token_out(),
                        amount_out.as_u256(),
                        to_minter.id,
                        &mut memo,
                    )
                    .await
                    {
                        Ok(erc20_ledger_burn_index) => erc20_ledger_burn_index,
                        Err(err) => {
                            log!(
                                DEBUG,
                                "[execute_crosschain_swap]: Transfer to target minter failed for tx_id: {:?} with error: {:?}. Recording for retry.",
                                tx_id,
                                err
                            );
                            mutate_state(|s| {
                                s.record_failed_dex_order_to_retry(RetryFailedDexOrder {
                                    tx_id: tx_id.clone(),
                                    token_in: icp_swap_request.token_out(),
                                    minter_id: to_minter.id,
                                    amount_in: amount_out.as_u256(),
                                    min_amount_out: evm_swap_step
                                        .min_amount_out
                                        .unwrap_or(U256::ZERO),
                                    commands: qswap_data.commands,
                                    commands_data: qswap_data.command_data,
                                    max_gas_fee_usd: evm_swap_step.gas_price_usd.clone(),
                                    gas_limit,
                                    deadline: qswap_data.deadline,
                                    recipient,
                                    erc20_ledger_burn_index: None,
                                    is_refund: false,
                                    signing_fee: evm_swap_step.canister_fee_usd.clone(),
                                })
                            });
                            return;
                        }
                    };
                    let dex_order = DexOrderArgs {
                        tx_id: tx_id.0.clone(),
                        amount_in: u256_to_nat(amount_out.as_u256()),
                        min_amount_out: u256_to_nat(
                            evm_swap_step.min_amount_out.unwrap_or(U256::ZERO),
                        ),
                        commands: qswap_data.commands.clone(),
                        commands_data: qswap_data.command_data.clone(),
                        max_gas_fee_usd: evm_swap_step.gas_price_usd.clone(),
                        gas_limit: u256_to_nat(gas_limit),
                        deadline: u256_to_nat(qswap_data.deadline),
                        recipient: recipient.clone(),
                        erc20_ledger_burn_index: erc20_ledger_burn_index.clone(),
                        is_refund: false,
                        signing_fee: evm_swap_step.canister_fee_usd.clone(),
                    };
                    let minter_client = MinterClient::new(to_minter.id);
                    match minter_client.dex_order(&dex_order).await {
                        Ok(_) => {
                            log!(
                                DEBUG,
                                "[execute_crosschain_swap]: Successfully notified target minter for tx_id: {:?}.",
                                tx_id
                            );
                            //  Record successful swap on the ICP side.
                            let event = Event {
                                timestamp,
                                payload: EventType::CrosschainSwap {
                                    swap_order: args.clone(),
                                    is_refunded: false,
                                    icp_amount_out: Some(amount_out.as_u256()),
                                    icp_token_in: Some(icp_swap_request.token_in()),
                                    icp_token_out: Some(icp_swap_request.token_out()),
                                    icp_amount_in: Some(amount_in.as_u256()),
                                },
                            };

                            mutate_state(|s| s.record_event(event));
                        }
                        Err(err) => {
                            log!(
                                DEBUG,
                                "[execute_crosschain_swap]: Failed to notify target minter for tx_id: {:?} with error: {:?}. Recording for retry.",
                                tx_id,
                                err
                            );
                            mutate_state(|s| {
                                s.record_failed_dex_order_to_retry(RetryFailedDexOrder {
                                    tx_id: tx_id.clone(),
                                    token_in: icp_swap_request.token_out(),
                                    minter_id: to_minter.id,
                                    amount_in: amount_out.as_u256(),
                                    min_amount_out: evm_swap_step
                                        .min_amount_out
                                        .unwrap_or(U256::ZERO),
                                    commands: qswap_data.commands,
                                    commands_data: qswap_data.command_data,
                                    max_gas_fee_usd: evm_swap_step.gas_price_usd.clone(),
                                    gas_limit,
                                    deadline: qswap_data.deadline,
                                    recipient,
                                    erc20_ledger_burn_index: Some(erc20_ledger_burn_index),
                                    is_refund: false,
                                    signing_fee: evm_swap_step.canister_fee_usd.clone(),
                                })
                            });
                        }
                    }
                }
                // On failure, refund the deposit amount to the origin ICP principal.
                Err(err) => {
                    log!(
                        DEBUG,
                        "[execute_crosschain_swap]: ICP swap failed for tx_id: {:?} with error: {:?}. Initiating refund to origin principal.",
                        tx_id,
                        err
                    );
                    match _refund(
                        icp_swap_request.token_in(),
                        icp_swap_request.deposit_amount().as_u256(),
                        *from,
                    )
                    .await
                    {
                        Ok(_) => {
                            log!(
                                DEBUG,
                                "[execute_crosschain_swap]: Successfully refunded to origin principal for tx_id: {:?}.",
                                tx_id
                            );
                        }
                        Err(err) => {
                            log!(
                                DEBUG,
                                "[execute_crosschain_swap]: Refund to origin principal failed for tx_id: {:?} with error: {:?}. Tokens remain in user balance for manual withdrawal.",
                                tx_id,
                                err
                            );
                            // No further action needed; users can withdraw from their balance manually.
                        }
                    };

                    let event = Event {
                        timestamp,
                        payload: EventType::CrosschainSwap {
                            swap_order: args.clone(),
                            is_refunded: true,
                            icp_amount_out: None,
                            icp_token_in: Some(icp_swap_request.token_in()),
                            icp_token_out: Some(icp_swap_request.token_out()),
                            icp_amount_in: Some(icp_swap_request.deposit_amount().as_u256()),
                        },
                    };

                    mutate_state(|s| s.record_event(event));
                }
            };
        }
    }
}

// Transfers tokens to a minter, updates balance, and handles transfer errors with rollback.
pub async fn _transfer_to_minter(
    token: Principal,
    amount: U256,
    to: Principal,
    memo: &mut WithdrawMemo,
) -> Result<Nat, LedgerTransferError> {
    let user_balance = get_user_balance(to, token);
    log!(
        DEBUG,
        "[_transfer_to_minter]: Preparing to transfer token {:?} amount {:?} to minter {:?}. Current balance: {:?}",
        token.to_text(),
        amount,
        to.to_text(),
        user_balance,
    );
    // Check for sufficient balance.
    if amount > user_balance {
        panic!("BUG: There should always be enough balance for minter transfers in swaps");
    }
    // Deduct balance before transfer to ensure atomicity.
    mutate_state(|s| {
        s.update_user_balance(
            UserBalanceKey { user: to, token },
            UserBalance(user_balance - amount),
        );
    });
    match LedgerClient::new(token)
        .transfer_to_minter(to.into(), u256_to_big_uint(amount), memo.clone())
        .await
    {
        Ok(ledger_burn_index) => {
            log!(
                DEBUG,
                "[_transfer_to_minter]: Successfully transferred to minter {:?} with burn index: {:?}",
                to.to_text(),
                ledger_burn_index
            );
            Ok(ledger_burn_index.0)
        }
        Err(err) => {
            // Restore balance on transfer failure.
            let latest_user_balance = get_user_balance(to, token);
            mutate_state(|s| {
                s.update_user_balance(
                    UserBalanceKey { user: to, token },
                    UserBalance(latest_user_balance.checked_add(amount).unwrap_or(U256::MAX)),
                );
            });
            log!(
                DEBUG,
                "[_transfer_to_minter]: Transfer to minter {:?} failed with error: {:?}. Balance restored.",
                to.to_text(),
                err
            );
            Err(err)
        }
    }
}

pub async fn refund_failed_crosschain_swap_to_the_minter(args: RetryFailedDexOrder) {
    log!(
        DEBUG,
        "[refund_failed_crosschain_swap_to_the_minter]: Retrying failed DEX order for tx_id: {:?}",
        args.tx_id
    );
    let minter_client = MinterClient::new(args.minter_id);
    match args.erc20_ledger_burn_index {
        Some(erc20_ledger_burn_index) => {
            let dex_order = DexOrderArgs {
                tx_id: args.tx_id.0.clone(),
                amount_in: u256_to_nat(args.amount_in),
                min_amount_out: u256_to_nat(args.min_amount_out),
                commands: args.commands.clone(),
                commands_data: args.commands_data.clone(),
                max_gas_fee_usd: args.max_gas_fee_usd.clone(),
                gas_limit: u256_to_nat(args.gas_limit),
                deadline: u256_to_nat(args.deadline),
                recipient: args.recipient.clone(),
                erc20_ledger_burn_index,
                is_refund: args.is_refund,
                signing_fee: args.signing_fee,
            };
            match minter_client.dex_order(&dex_order).await {
                Ok(_) => {
                    log!(
                        DEBUG,
                        "[refund_failed_crosschain_swap_to_the_minter]: Successfully notified minter for tx_id: {:?}. Removing from retry queue.",
                        args.tx_id
                    );
                    // TODO: Record successful swap on the ICP side.
                    mutate_state(|s| s.remove_failed_dex_order_to_retry(&args.tx_id));
                }
                Err(err) => {
                    log!(
                        DEBUG,
                        "[refund_failed_crosschain_swap_to_the_minter]: Failed to notify minter for tx_id: {:?} with error: {:?}. Will retry later.",
                        args.tx_id,
                        err
                    );
                }
            }
        }
        None => {
            let mut memo = WithdrawMemo::TransferToMinter {
                amount: args.amount_in,
                tx_id: args.tx_id.clone(),
            };
            let erc20_ledger_burn_index = match _transfer_to_minter(
                args.token_in,
                args.amount_in,
                args.minter_id,
                &mut memo,
            )
            .await
            {
                Ok(erc20_ledger_burn_index) => erc20_ledger_burn_index,
                Err(err) => {
                    log!(
                        DEBUG,
                        "[refund_failed_crosschain_swap_to_the_minter]: Transfer to minter failed for tx_id: {:?} with error: {:?}. Will retry later.",
                        args.tx_id,
                        err
                    );
                    // To be retried later.
                    return;
                }
            };
            let dex_order = DexOrderArgs {
                tx_id: args.tx_id.0.clone(),
                amount_in: u256_to_nat(args.amount_in),
                min_amount_out: u256_to_nat(args.min_amount_out),
                commands: args.commands.clone(),
                commands_data: args.commands_data.clone(),
                max_gas_fee_usd: args.max_gas_fee_usd.clone(),
                gas_limit: u256_to_nat(args.gas_limit),
                deadline: u256_to_nat(args.deadline),
                recipient: args.recipient.clone(),
                erc20_ledger_burn_index: erc20_ledger_burn_index.clone(),
                is_refund: args.is_refund,
                signing_fee: args.signing_fee.clone(),
            };
            match minter_client.dex_order(&dex_order).await {
                Ok(_) => {
                    log!(
                        DEBUG,
                        "[refund_failed_crosschain_swap_to_the_minter]: Successfully notified minter for tx_id: {:?}. Removing from retry queue.",
                        args.tx_id
                    );
                    // TODO: Record successful swap on the ICP side.
                    mutate_state(|s| s.remove_failed_dex_order_to_retry(&args.tx_id));
                }
                Err(err) => {
                    log!(
                        DEBUG,
                        "[refund_failed_crosschain_swap_to_the_minter]: Failed to notify minter for tx_id: {:?} with error: {:?}. Recording updated retry with burn index.",
                        args.tx_id,
                        err
                    );
                    mutate_state(|s| {
                        s.record_failed_dex_order_to_retry(RetryFailedDexOrder {
                            erc20_ledger_burn_index: Some(erc20_ledger_burn_index),
                            ..args
                        })
                    })
                }
            }
        }
    }
}
