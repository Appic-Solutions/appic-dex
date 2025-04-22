use candid::{CandidType, Deserialize, Int, Nat, Principal};
use ic_stable_structures::cell::ValueError;
use serde::Serialize;

use crate::pool::types::{PoolFee, PoolId};

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
    token0: Principal,
    token1: Principal,
    fee: Nat,
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
}

#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub enum MintPositionError {
    InvalidPoolFee,
    PoolNotInitialized,
    InvalidTick,
    InvalidAmount,
    TickNotAlignedWithTickSpacing,
}
