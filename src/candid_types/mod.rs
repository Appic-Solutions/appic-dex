use candid::{CandidType, Deserialize, Int, Nat, Principal};
use icrc_ledger_types::icrc1::account::Subaccount;
use serde::Serialize;

use crate::{
    icrc_client::LedgerTransferError,
    libraries::path_key::PathKey,
    pool::types::{PoolFee, PoolId},
};

pub mod pool;
pub mod position;
pub mod quote;
pub mod swap;

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub enum DepositError {
    AmountTooLow { min_withdrawal_amount: Nat },
    InsufficientFunds { balance: Nat }, // not enough balance in user wallet
    InsufficientAllowance { allowance: Nat },
    TemporarilyUnavailable(String),
    InvalidDestination(String),
}

#[derive(Debug, Clone, CandidType, Deserialize, Serialize, PartialEq, Eq)]
pub enum WithdrawalError {
    AmountTooLow { min_withdrawal_amount: Nat },
    InsufficientBalance { balance: Nat }, // user has insufficient balance
    InsufficientAllowance { allowance: Nat },
    TemporarilyUnavailable(String),
    InvalidDestination(String),
    FeeUnknown,
}

impl From<LedgerTransferError> for WithdrawalError {
    fn from(error: LedgerTransferError) -> Self {
        match error {
            LedgerTransferError::TemporarilyUnavailable { message, .. } => {
                Self::TemporarilyUnavailable(message)
            }
            LedgerTransferError::InsufficientFunds { .. } => {
                panic!("Bug: Canister should always hold enough for withdrawal")
            }
            LedgerTransferError::InsufficientAllowance { allowance, .. } => {
                Self::InsufficientAllowance { allowance }
            }
            LedgerTransferError::FeeUnknown => Self::FeeUnknown,
            LedgerTransferError::AmountTooLow {
                minimum_amount,
                failed_amount,
                ledger,
            } => {
                panic!(
                    "BUG: withdrawal amount {failed_amount} on the {ledger:?} should always be higher than the ledger transaction fee {minimum_amount}"
                )
            }
        }
    }
}

impl From<LedgerTransferError> for DepositError {
    fn from(error: LedgerTransferError) -> Self {
        match error {
            LedgerTransferError::TemporarilyUnavailable { message, .. } => {
                Self::TemporarilyUnavailable(message)
            }
            LedgerTransferError::InsufficientFunds { balance, .. } => {
                Self::InsufficientFunds { balance }
            }
            LedgerTransferError::InsufficientAllowance { allowance, .. } => {
                Self::InsufficientAllowance { allowance }
            }
            LedgerTransferError::FeeUnknown => panic!("Bug: Fee is not required for deposit"),
            LedgerTransferError::AmountTooLow {
                minimum_amount,
                failed_amount,
                ledger,
            } => {
                panic!(
                    "BUG: deposit amount {failed_amount} on the {ledger:?} should always be higher than the ledger transaction fee {minimum_amount}"
                )
            }
        }
    }
}
