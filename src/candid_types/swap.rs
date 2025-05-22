use crate::pool::swap::InnerSwapError;

use super::{pool::CandidPoolId, *};

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
    pub from_subaccount: Option<Subaccount>,
}

/// Parameters for a multi-hop exact-input swap
#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub struct ExactInputParams {
    pub token_in: Principal,
    pub path: Vec<CandidPathKey>,
    pub amount_in: Nat,
    pub amount_out_minimum: Nat,
    pub from_subaccount: Option<Subaccount>,
}

/// Parameters for a single-hop exact-output swap
#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub struct ExactOutputSingleParams {
    pub pool_id: CandidPoolId,
    pub zero_for_one: bool,
    pub amount_out: Nat,
    pub amount_in_maximum: Nat,
    pub from_subaccount: Option<Subaccount>,
}

/// notice Parameters for a multi-hop exact-output swap
#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub struct ExactOutputParams {
    pub token_out: Principal,
    pub path: Vec<CandidPathKey>,
    pub amount_out: Nat,
    pub amount_in_maximum: Nat,
    pub from_subaccount: Option<Subaccount>,
}

#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub enum SwapArgs {
    ExactInputSingle(ExactInputSingleParams),
    ExactInput(ExactInputParams),
    ExactOutputSingle(ExactOutputSingleParams),
    ExactOutput(ExactOutputParams),
}

#[derive(Debug, Clone, CandidType, Deserialize, Serialize, PartialEq, Eq)]
pub enum SwapFailedReason {
    PriceLimitAlreadyExceeded, // means there is a bug, should not happen
    PriceLimitOutOfBounds,     // means there is a bug, should not happen
    CalculationOverflow,
    InvalidFeeForExactOutput,
    PoolNotInitialized,
    NoInRangeLiquidity,
    InvalidAmount,
    TooLittleReceived,
    TooMuchRequeted,
    BalanceOverflow,
    InsufficientBalance,
}

#[derive(Debug, Clone, CandidType, Deserialize, Serialize, PartialEq, Eq)]
pub enum SwapError {
    LockedPrincipal,
    InvalidPoolFee,
    PoolNotInitialized,
    NoInRangeLiquidity,
    InvalidAmountIn,
    InvalidAmountOut,
    InvalidAmountOutMinimum,
    InvalidAmountInMaximum,
    PathDuplicated,
    PathLengthTooBig {
        maximum: u8,
        received: u8,
    },
    PathLengthTooSmall {
        minimum: u8,
        received: u8,
    },
    DepositError(DepositError),
    SwapFailedRefunded {
        failed_reason: SwapFailedReason,
        refund_amount: Option<Nat>,
        refund_error: Option<WithdrawalError>, // if refund fails, refund error
    }, // swap failed but refunded, if refund fails refund_error will be Some(WithdrawalError)

    FailedToWithdraw {
        reason: WithdrawalError,
        amount_in: Nat,
        amount_out: Nat,
    }, // swap was successful but amount_out withdrawal failed
}

#[derive(Debug, Clone, CandidType, Deserialize, Serialize, PartialEq, Eq)]
pub struct CandidSwapSuccess {
    pub amount_in: Nat,
    pub amount_out: Nat,
}

impl From<InnerSwapError> for SwapFailedReason {
    fn from(value: InnerSwapError) -> Self {
        match value {
            InnerSwapError::PoolNotInitialized => Self::PoolNotInitialized,
            InnerSwapError::IlliquidPool => Self::NoInRangeLiquidity,
            InnerSwapError::InvalidFeeForExactOutput => Self::InvalidFeeForExactOutput,
            InnerSwapError::PriceLimitAlreadyExceeded => Self::PriceLimitAlreadyExceeded,
            InnerSwapError::PriceLimitOutOfBounds => Self::PriceLimitOutOfBounds,
            InnerSwapError::CalculationOverflow => Self::CalculationOverflow,
        }
    }
}
