use crate::{libraries::safe_cast::u256_to_nat, pool::types::PoolState};

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

#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub struct CandidPoolState {
    pub sqrt_price_x96: Nat,           // Current price in Q64.96 format
    pub tick: Int,                     // Current tick index
    pub fee_growth_global_0_x128: Nat, // Cumulative fees for token0
    pub fee_growth_global_1_x128: Nat, // Cumulative fees for token1
    pub liquidity: Nat,                // Total active liquidity
    pub tick_spacing: Int,             // Spacing between ticks
    pub max_liquidity_per_tick: Nat,   // Max liquidity per tick
    pub fee_protocol: Nat,             // Max protocol fee is 0.1% (1000 pips)
    pub token0_transfer_fee: Nat,
    pub token1_transfer_fee: Nat,
    pub swap_volume0_all_time: Nat,
    pub swap_volume1_all_time: Nat,
    pub pool_reserves0: Nat,
    pub pool_reserves1: Nat,
    pub generated_swap_fee0: Nat,
    pub generated_swap_fee1: Nat,
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

impl From<PoolState> for CandidPoolState {
    fn from(value: PoolState) -> Self {
        CandidPoolState {
            sqrt_price_x96: u256_to_nat(value.sqrt_price_x96),
            tick: value.tick.into(),
            fee_growth_global_0_x128: u256_to_nat(value.fee_growth_global_0_x128),
            fee_growth_global_1_x128: u256_to_nat(value.fee_growth_global_1_x128),
            liquidity: value.liquidity.into(),
            tick_spacing: value.tick_spacing.0.into(),
            max_liquidity_per_tick: value.max_liquidity_per_tick.into(),
            fee_protocol: value.fee_protocol.into(),
            token0_transfer_fee: u256_to_nat(value.token0_transfer_fee),
            token1_transfer_fee: u256_to_nat(value.token1_transfer_fee),
            swap_volume0_all_time: u256_to_nat(value.swap_volume0_all_time),
            swap_volume1_all_time: u256_to_nat(value.swap_volume1_all_time),
            pool_reserves0: u256_to_nat(value.pool_reserve0),
            pool_reserves1: u256_to_nat(value.pool_reserve1),
            generated_swap_fee0: u256_to_nat(value.generated_swap_fee0),
            generated_swap_fee1: u256_to_nat(value.generated_swap_fee1),
        }
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
