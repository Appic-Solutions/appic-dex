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
