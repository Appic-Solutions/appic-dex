use super::{pool::CandidPoolId, *};

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
    LockedPrinciapl,
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

#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub enum MintPositionError {
    LockedPrinciapl,
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
pub struct IncreaseLiquidtyArgs {
    pub pool: CandidPoolId,
    pub tick_lower: Int,
    pub tick_upper: Int,
    pub amount0_max: Nat,
    pub amount1_max: Nat,
    pub from_subaccount: Option<Subaccount>,
}

#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub enum IncreaseLiquidtyError {
    LockedPrinciapl,
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
