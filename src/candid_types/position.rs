use minicbor::{Decode, Encode};

use crate::position::types::PositionKey;

use super::{pool::CandidPoolId, *};

#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub struct CandidPositionKey {
    pub owner: Principal,
    pub pool: CandidPoolId,
    pub tick_lower: Int,
    pub tick_upper: Int,
}

impl TryFrom<CandidPositionKey> for PositionKey {
    type Error = String;

    fn try_from(value: CandidPositionKey) -> Result<Self, Self::Error> {
        let pool_id = PoolId::try_from(value.pool)?;

        let tick_lower: i32 = value
            .tick_lower
            .0
            .try_into()
            .map_err(|_| String::from("Invalid Tick"))?;
        let tick_upper: i32 = value
            .tick_upper
            .0
            .try_into()
            .map_err(|_| String::from("Invalid Tick"))?;

        Ok(PositionKey {
            owner: value.owner,
            pool_id,
            tick_lower,
            tick_upper,
        })
    }
}

#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub struct CandidPositionInfo {
    pub liquidity: Nat,                     // Position liquidity
    pub fee_growth_inside_0_last_x128: Nat, // Fees for token0 at last update
    pub fee_growth_inside_1_last_x128: Nat, // Fees for token1 at last update
    pub fees_token0_owed: Nat,
    pub fees_token1_owed: Nat,
}

#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub struct BurnPositionArgs {
    pub tick_lower: Int,
    pub tick_upper: Int,
    pub pool: CandidPoolId,
    pub amount0_min: Nat,
    pub amount1_min: Nat,
}

#[derive(Debug, Clone, CandidType, Deserialize, Serialize, PartialEq, Eq)]
pub enum BurnPositionError {
    LockedPrincipal,
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

#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub struct MintPositionArgs {
    pub pool: CandidPoolId,
    pub tick_lower: Int,
    pub tick_upper: Int,
    pub amount0_max: Nat,
    pub amount1_max: Nat,
    pub from_subaccount: Option<Subaccount>,
}

#[derive(Debug, Clone, CandidType, Deserialize, Serialize, Eq, PartialEq)]
pub enum MintPositionError {
    LockedPrincipal,
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

#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub struct IncreaseLiquidityArgs {
    pub pool: CandidPoolId,
    pub tick_lower: Int,
    pub tick_upper: Int,
    pub amount0_max: Nat,
    pub amount1_max: Nat,
    pub from_subaccount: Option<Subaccount>,
}

#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub enum IncreaseLiquidity {
    LockedPrincipal,
    InvalidPoolFee,
    PoolNotInitialized,
    InvalidTick,
    InvalidAmount,
    TickNotAlignedWithTickSpacing,
    PositionDoesNotExist,
    DepositError(DepositError),
    LiquidityOverflow,
    FeeOverflow,
    AmountOverflow,
    InsufficientBalance,
}

#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub struct DecreaseLiquidityArgs {
    pub pool: CandidPoolId,
    pub tick_lower: Int,
    pub tick_upper: Int,
    pub liquidity: Nat,
    pub amount0_min: Nat,
    pub amount1_min: Nat,
}

#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub enum DecreaseLiquidityError {
    LockedPrincipal,
    PositionNotFound,
    PoolNotInitialized,
    InvalidTick,
    InvalidPoolFee,
    InvalidLiquidity,
    LiquidityOverflow,
    FeeOverflow,
    AmountOverflow,
    InvalidAmount,
    InsufficientBalance,
    DecreasedPositionWithdrawalFailed(WithdrawalError),
}

#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub struct CollectFeesSuccess {
    pub token0_collected: Nat,
    pub token1_collected: Nat,
}

#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub enum CollectFeesError {
    LockedPrincipal,
    PositionNotFound,
    FeeOverflow,
    NoFeeToCollect,
    CollectedFeesWithdrawalFailed(WithdrawalError),
}
