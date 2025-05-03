use crate::pool::swap::InnerSwapError;

use super::{pool::CandidPoolId, swap::CandidPathKey, *};

#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
// parameters for a single hop quote
pub struct QuoteExactSingleParams {
    pub pool_id: CandidPoolId,
    pub zero_for_one: bool,
    pub exact_amount: Nat,
}

#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
// parameters for multi hop quotes
pub struct QuoteExactParams {
    pub exact_token: Principal,
    pub path: Vec<CandidPathKey>,
    pub exact_amount: Nat,
}

#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub enum QuoteArgs {
    QuoteExactInputSingleParams(QuoteExactSingleParams),
    QuoteExactInputParams(QuoteExactParams),
    QuoteExactOutputSingleParams(QuoteExactSingleParams),
    QuoteExactOutput(QuoteExactParams),
}

#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub enum QuoteError {
    InvalidAmount,
    IlliquidPool,
    InvalidFee,
    PoolNotInitialized,
    InvalidFeeForExactOutput,
    PriceLimitAlreadyExceeded,
    PriceLimitOutOfBounds,
    CalculationOverflow,
    InvalidPathLength,
}

impl From<InnerSwapError> for QuoteError {
    fn from(value: InnerSwapError) -> Self {
        match value {
            InnerSwapError::PoolNotInitialized => Self::PoolNotInitialized,
            InnerSwapError::InvalidFeeForExactOutput => Self::InvalidFeeForExactOutput,
            InnerSwapError::PriceLimitAlreadyExceeded => Self::PriceLimitAlreadyExceeded,
            InnerSwapError::PriceLimitOutOfBounds => Self::PriceLimitOutOfBounds,
            InnerSwapError::CalculationOverflow => Self::CalculationOverflow,
            InnerSwapError::IlliquidPool => Self::IlliquidPool,
        }
    }
}
