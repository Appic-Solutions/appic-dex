use candid::{CandidType, Deserialize, Int, Nat, Principal};
use icrc_ledger_types::icrc1::account::Subaccount;
use serde::Serialize;

use crate::{
    icrc_client::LedgerTransferError,
    pool::types::{PoolFee, PoolId},
};

#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub struct CreatePoolArgs {
    pub token_a: Principal,
    pub token_b: Principal,
    pub fee: Nat,
    pub sqrt_price_x96: Nat,
}

#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub enum CreatePoolError {
    InvalidFeeAmount,
    InvalidSqrtPriceX96,
    InvalidToken(Principal),
    PoolAlreadyExists,
}

#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub struct CandidPoolId {
    pub token0: Principal,
    pub token1: Principal,
    pub fee: Nat,
}

impl TryFrom<CandidPoolId> for PoolId {
    type Error = MintPositionError;

    fn try_from(value: CandidPoolId) -> Result<Self, Self::Error> {
        let fee: u16 = value
            .fee
            .0
            .try_into()
            .map_err(|_e| MintPositionError::InvalidPoolFee)?;

        Ok(PoolId {
            token0: value.token0,
            token1: value.token1,
            fee: PoolFee(fee),
        })
    }
}

#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub struct MintPositionArgs {
    pub pool: CandidPoolId,
    pub tick_lower: Int,
    pub tick_higher: Int,
    pub amount0_max: Nat,
    pub amount1_max: Nat,
    pub from_subaccount: Option<Subaccount>,
}

#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub enum MintPositionError {
    InvalidPoolFee,
    PoolNotInitialized,
    InvalidTick,
    InvalidAmount,
    TickNotAlignedWithTickSpacing,
    DepositError(DepositError),
    LiquidityOverflow,
    FeeOverflow,
    AmountOverflow,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum DepositError {
    AmountTooLow { min_withdrawal_amount: Nat },
    InsufficientFunds { balance: Nat },
    InsufficientAllowance { allowance: Nat },
    TemporarilyUnavailable(String),
    InvalidDestination(String),
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
            LedgerTransferError::AmountTooLow {
                minimum_burn_amount,
                failed_burn_amount,
                ledger,
            } => {
                panic!("BUG: withdrawal amount {failed_burn_amount} on the Native ledger {ledger:?} should always be higher than the ledger transaction fee {minimum_burn_amount}")
            }
        }
    }
}
