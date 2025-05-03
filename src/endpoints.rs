use candid::{CandidType, Deserialize, Int, Nat, Principal};
use icrc_ledger_types::icrc1::account::Subaccount;
use serde::Serialize;

use crate::{
    icrc_client::LedgerTransferError,
    libraries::path_key::PathKey,
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

#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub struct CandidPositionId {
    pub owner: Principal,
    pub pool_id: CandidPoolId,
    pub tick_lower: Int,
    pub tick_upper: Int,
}

#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub struct BurnPositionArgs {
    pub tick_lower: Int,
    pub tick_upper: Int,
    pub pool: CandidPoolId,
    pub amount0_min: Nat,
    pub amount1_min: Nat,
}

#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub enum BurnPositionError {
    PositionNotFound,
    PoolNotInitialized,
    InvalidTick,
    InvalidPoolFee,
    InvalidAmount,
    LiquidityOverflow,
    FeeOverflow,
    AmountOverflow,
    InsufficientBalance,
    BurntPositionWithdrawalFailed(WithdrawalError),
}

impl TryFrom<CandidPoolId> for PoolId {
    type Error = String;

    fn try_from(value: CandidPoolId) -> Result<Self, Self::Error> {
        let fee: u32 = value
            .fee
            .0
            .try_into()
            .map_err(|_e| String::from("Invalid Pool Fee"))?;

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
    pub tick_upper: Int,
    pub amount0_max: Nat,
    pub amount1_max: Nat,
    pub from_subaccount: Option<Subaccount>,
}

#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub enum MintPositionError {
    InvalidPoolFee,
    PoolNotInitialized,
    PositionAlreadyExists,
    InvalidTick,
    InvalidAmount,
    TickNotAlignedWithTickSpacing,
    DepositError(DepositError),
    LiquidityOverflow,
    FeeOverflow,
    AmountOverflow,
    InsufficientBalance,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum DepositError {
    AmountTooLow { min_withdrawal_amount: Nat },
    InsufficientFunds { balance: Nat },
    InsufficientAllowance { allowance: Nat },
    TemporarilyUnavailable(String),
    InvalidDestination(String),
}

#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub enum WithdrawalError {
    AmountTooLow { min_withdrawal_amount: Nat },
    InsufficientFunds { balance: Nat },
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
            LedgerTransferError::InsufficientFunds { balance, .. } => {
                Self::InsufficientFunds { balance }
            }
            LedgerTransferError::InsufficientAllowance { allowance, .. } => {
                Self::InsufficientAllowance { allowance }
            }
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

// SWAP TYPES
#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub struct CandidPathKey {
    pub intermediary_token: Principal,
    pub fee: Nat,
}

impl TryFrom<CandidPathKey> for PathKey {
    type Error = SwapError;

    fn try_from(value: CandidPathKey) -> Result<Self, Self::Error> {
        Ok(Self {
            intermediary_token: value.intermediary_token,
            fee: PoolFee(
                value
                    .fee
                    .0
                    .try_into()
                    .map_err(|_| Self::Error::InvalidPoolFee)?,
            ),
        })
    }
}

/// Parameters for a single-hop exact-input swap
#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub struct ExactInputSingleParams {
    pub pool_id: CandidPoolId,
    pub zero_for_one: bool,
    pub amount_in: Nat,
    pub amount_out_minimum: Nat,
}

/// Parameters for a multi-hop exact-input swap
#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub struct ExactInputParams {
    pub token_in: Principal,
    pub path: Vec<CandidPathKey>,
    pub amount_in: Nat,
    pub amount_out_minimum: Nat,
}

/// Parameters for a single-hop exact-output swap
#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub struct ExactOutputSingleParams {
    pub pool_id: CandidPoolId,
    pub zero_for_one: bool,
    pub amount_out: Nat,
    pub amount_in_maximum: Nat,
}

/// notice Parameters for a multi-hop exact-output swap
#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub struct ExactOutputParams {
    pub token_out: Principal,
    pub path: Vec<CandidPathKey>,
    pub amount_out: Nat,
    pub amount_in_maximum: Nat,
}

#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub enum SwapArgs {
    ExactInputSingle(ExactInputSingleParams),
    ExactInput(ExactInputParams),
    ExactOutputSingle(ExactOutputSingleParams),
    ExactOutput(ExactOutputParams),
}

#[derive(Debug, Clone, CandidType, Deserialize, Serialize, PartialEq, Eq)]
pub enum SwapError {
    InvalidPoolFee,
    PoolNotInitialized,
    NoInRangeLiquidity,
    InvalidAmountIn,
    InvalidAmountOut,
    InvalidAmountOutMinimum,
    InvalidAmountInMaximum,
    PathLengthTooBig { maximum: u8, received: u8 },
    PathLengthTooSmall { minimum: u8, received: u8 },
}
