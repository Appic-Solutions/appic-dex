use core::panic;

use candid::{CandidType, Deserialize, Int, Nat, Principal};
use icrc_ledger_types::icrc1::account::Subaccount;
use serde::Serialize;

use crate::{
    icrc_client::LedgerTransferError,
    libraries::path_key::PathKey,
    pool::types::{PoolFee, PoolId},
};

pub mod events;
pub mod pool;
pub mod pool_history;
pub mod position;
pub mod quote;
pub mod swap;
pub mod tick;

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub struct Balance {
    pub token: Principal,
    pub amount: Nat,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub struct UserBalanceArgs {
    pub token: Principal,
    pub user: Principal,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub struct DepositArgs {
    pub token: Principal,
    pub amount: Nat,
    pub from_subaccount: Option<Subaccount>,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub struct WithdrawArgs {
    pub token: Principal,
    pub amount: Nat,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub enum DepositError {
    LockedPrincipal,
    AmountTooLow { min_withdrawal_amount: Nat },
    InsufficientFunds { balance: Nat }, // not enough balance in user wallet
    InsufficientAllowance { allowance: Nat },
    TemporarilyUnavailable(String),
    InvalidDestination(String),
    AmountOverflow,
}

#[derive(Debug, Clone, CandidType, Deserialize, Serialize, PartialEq, Eq)]
pub enum WithdrawError {
    LockedPrincipal,
    AmountTooLow { min_withdrawal_amount: Nat },
    InsufficientBalance { balance: Nat }, // user has insufficient balance
    InsufficientAllowance { allowance: Nat },
    TemporarilyUnavailable(String),
    InvalidDestination(String),
    FeeUnknown,
    AmountOverflow,
}

impl From<LedgerTransferError> for WithdrawError {
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
            LedgerTransferError::BadFee { .. } => WithdrawError::FeeUnknown,
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
            LedgerTransferError::BadFee { .. } => {
                panic!("Bug: Fee is not required for deposit")
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
