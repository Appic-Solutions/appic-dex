pub mod memo;

use candid::{Nat, Principal};
use ic_canister_log::log;
// use ic_canister_log::log;
use icrc_ledger_client_cdk::{CdkRuntime, ICRC1Client};
use icrc_ledger_types::{
    icrc1::{
        account::Account,
        transfer::{Memo, TransferArg, TransferError},
    },
    icrc2::transfer_from::{TransferFromArgs, TransferFromError},
};
use memo::{DepositMemo, WithdrawalMemo};

use crate::logs::DEBUG;

pub struct LedgerClient {
    client: ICRC1Client<CdkRuntime>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LedgerTransferError {
    TemporarilyUnavailable {
        message: String,
        ledger: Principal,
    },
    AmountTooLow {
        minimum_burn_amount: Nat,
        failed_burn_amount: Nat,
        ledger: Principal,
    },
    InsufficientFunds {
        balance: Nat,
        failed_burn_amount: Nat,
        ledger: Principal,
    },
    InsufficientAllowance {
        allowance: Nat,
        failed_burn_amount: Nat,
        ledger: Principal,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransferIndex(Nat);

impl LedgerClient {
    pub fn new(token: Principal) -> Self {
        Self {
            client: ICRC1Client {
                runtime: CdkRuntime,
                ledger_canister_id: token,
            },
        }
    }

    pub async fn deposit<A: Into<Nat>>(
        &self,
        from: Account,
        amount: A,
        memo: DepositMemo,
    ) -> Result<TransferIndex, LedgerTransferError> {
        let amount = amount.into();
        match self
            .client
            .transfer_from(TransferFromArgs {
                spender_subaccount: None,
                from,
                to: ic_cdk::id().into(),
                amount: amount.clone(),
                fee: None,
                memo: Some(Memo::from(memo)),
                created_at_time: None, // We don't set this field to disable transaction deduplication
                                       // which is unnecessary in canister-to-canister calls.
            })
            .await
        {
            Ok(Ok(block_index)) => Ok(TransferIndex(block_index)),
            Ok(Err(transfer_from_error)) => {
                log!(
                    DEBUG,
                    "[transfer_from]: failed to transfer_from from the {:?} ledger with error: {transfer_from_error:?}",
                    self.client.ledger_canister_id.to_text()
                );
                let burn_error = match transfer_from_error {
                    TransferFromError::BadFee { expected_fee } => {
                        panic!("BUG: bad fee, expected fee: {expected_fee}")
                    }
                    TransferFromError::BadBurn { min_burn_amount } => {
                        LedgerTransferError::AmountTooLow {
                            minimum_burn_amount: min_burn_amount,
                            failed_burn_amount: amount.clone(),
                            ledger: self.client.ledger_canister_id,
                        }
                    }
                    TransferFromError::InsufficientFunds { balance } => {
                        LedgerTransferError::InsufficientFunds {
                            balance,
                            failed_burn_amount: amount.clone(),
                            ledger: self.client.ledger_canister_id,
                        }
                    }
                    TransferFromError::InsufficientAllowance { allowance } => {
                        LedgerTransferError::InsufficientAllowance {
                            allowance,
                            failed_burn_amount: amount,
                            ledger: self.client.ledger_canister_id,
                        }
                    }
                    TransferFromError::TooOld => panic!("BUG: transfer too old"),
                    TransferFromError::CreatedInFuture { ledger_time } => {
                        panic!("BUG: created in future, ledger time: {ledger_time}")
                    }
                    TransferFromError::Duplicate { duplicate_of } => {
                        panic!("BUG: duplicate transfer of: {duplicate_of}")
                    }
                    TransferFromError::TemporarilyUnavailable => {
                        LedgerTransferError::TemporarilyUnavailable {
                            message: format!(
                                "{} ledger temporarily unavailable, try again",
                                self.client.ledger_canister_id.to_text()
                            ),
                            ledger: self.client.ledger_canister_id,
                        }
                    }
                    TransferFromError::GenericError {
                        error_code,
                        message,
                    } => LedgerTransferError::TemporarilyUnavailable {
                        message: format!(
                        "{} ledger unreachable, error code: {error_code}, with message: {message}",
                        self.client.ledger_canister_id.to_text()
                    ),
                        ledger: self.client.ledger_canister_id,
                    },
                };
                Err(burn_error)
            }
            Err((error_code, message)) => {
                let err_msg = format!(
                    "failed to call {} ledger with error_code: {error_code} and message: {message}",
                    self.client.ledger_canister_id.to_text()
                );
                log!(DEBUG, "[burn]: {err_msg}",);
                Err(LedgerTransferError::TemporarilyUnavailable {
                    message: err_msg,
                    ledger: self.client.ledger_canister_id,
                })
            }
        }
    }

    pub async fn withdraw<A: Into<Nat>>(
        &self,
        to: Account,
        amount: A,
        memo: WithdrawalMemo,
    ) -> Result<TransferIndex, LedgerTransferError> {
        let amount = amount.into();
        match self
            .client
            .transfer(TransferArg {
                from_subaccount: None,
                to,
                amount: amount.clone(),
                fee: None,
                memo: Some(Memo::from(memo)),
                created_at_time: None,
            })
            .await
        {
            Ok(Ok(block_index)) => Ok(TransferIndex(block_index)),
            Ok(Err(transfer_error)) => {
                log!(
                    DEBUG,
                    "[withdraw]: failed to transfer with error: {transfer_error:?}",
                );
                let transfer_err = match transfer_error {
                    TransferError::BadFee { expected_fee } => {
                        panic!("BUG: bad fee, expected fee: {expected_fee}")
                    }
                    TransferError::BadBurn { min_burn_amount: _ } => {
                        panic!("BUG: expected transfer")
                    }
                    TransferError::InsufficientFunds { balance: _ } => {
                        panic!("BUG: there should always be enough funds in the pool")
                    }

                    TransferError::TooOld => panic!("BUG: transfer too old"),
                    TransferError::CreatedInFuture { ledger_time } => {
                        panic!("BUG: created in future, ledger time: {ledger_time}")
                    }
                    TransferError::Duplicate { duplicate_of } => {
                        panic!("BUG: duplicate transfer of: {duplicate_of}")
                    }
                    TransferError::TemporarilyUnavailable => {
                        LedgerTransferError::TemporarilyUnavailable {
                            message: format!(
                                "{} ledger temporarily unavailable, try again",
                                self.client.ledger_canister_id.to_text()
                            ),
                            ledger: self.client.ledger_canister_id,
                        }
                    }
                    TransferError::GenericError {
                        error_code,
                        message,
                    } => LedgerTransferError::TemporarilyUnavailable {
                        message: format!(
                        "{} ledger unreachable, error code: {error_code}, with message: {message}",
                        self.client.ledger_canister_id.to_text()
                    ),
                        ledger: self.client.ledger_canister_id,
                    },
                };
                Err(transfer_err)
            }
            Err((error_code, message)) => {
                let err_msg = format!(
                    "failed to call {} ledger with error_code: {error_code} and message: {message}",
                    self.client.ledger_canister_id.to_text()
                );
                log!(DEBUG, "[withdraw]: {err_msg}",);
                Err(LedgerTransferError::TemporarilyUnavailable {
                    message: err_msg,
                    ledger: self.client.ledger_canister_id,
                })
            }
        }
    }
}
