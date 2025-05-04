use super::*;

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

impl From<PoolId> for CandidPoolId {
    fn from(value: PoolId) -> CandidPoolId {
        let fee: Nat = value.fee.0.into();

        CandidPoolId {
            token0: value.token0,
            token1: value.token1,
            fee,
        }
    }
}
