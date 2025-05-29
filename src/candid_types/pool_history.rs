use crate::{
    historical::types::{HistoryBucket, PoolHistory},
    libraries::safe_cast::u256_to_nat,
};

use super::*;

/// Historical data bucket for a specific timeframe.
#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub struct CandidHistoryBucket {
    pub start_timestamp: u64,
    pub end_timestamp: u64,
    pub swap_volume_token0_start: Nat,
    pub swap_volume_token0_during_bucket: Nat,
    pub swap_volume_token1_start: Nat,
    pub swap_volume_token1_during_bucket: Nat,
    pub fee_generated_token0_start: Nat,
    pub fee_generated_token0_during_bucket: Nat,
    pub fee_generated_token1_start: Nat,
    pub fee_generated_token1_during_bucket: Nat,
    pub token0_reserves: Nat,
    pub token1_reserves: Nat,
    pub last_sqrtx96_price: Nat,
    pub inrange_liquidity: Nat,
    pub active_tick: Int,
}

/// Stores historical data for a pool across multiple timeframes.
#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub struct CandidPoolHistory {
    pub hourly_frame: Vec<CandidHistoryBucket>,
    pub daily_frame: Vec<CandidHistoryBucket>,
    pub monthly_frame: Vec<CandidHistoryBucket>,
    pub yearly_frame: Vec<CandidHistoryBucket>,
}

impl From<PoolHistory> for CandidPoolHistory {
    fn from(value: PoolHistory) -> Self {
        Self {
            hourly_frame: value
                .hourly_frame
                .into_iter()
                .map(|history| CandidHistoryBucket::from(history))
                .collect(),
            daily_frame: value
                .daily_frame
                .into_iter()
                .map(|history| CandidHistoryBucket::from(history))
                .collect(),
            monthly_frame: value
                .monthly_frame
                .into_iter()
                .map(|history| CandidHistoryBucket::from(history))
                .collect(),
            yearly_frame: value
                .yearly_frame
                .into_iter()
                .map(|history| CandidHistoryBucket::from(history))
                .collect(),
        }
    }
}

impl From<HistoryBucket> for CandidHistoryBucket {
    fn from(value: HistoryBucket) -> Self {
        Self {
            start_timestamp: value.start_timestamp,
            end_timestamp: value.end_timestamp,
            swap_volume_token0_start: u256_to_nat(value.swap_volume_token0_start),
            swap_volume_token0_during_bucket: u256_to_nat(value.swap_volume_token0_during_bucket),
            swap_volume_token1_start: u256_to_nat(value.swap_volume_token1_start),
            swap_volume_token1_during_bucket: u256_to_nat(value.swap_volume_token1_during_bucket),
            fee_generated_token0_start: u256_to_nat(value.fee_generated_token0_start),
            fee_generated_token0_during_bucket: u256_to_nat(
                value.fee_generated_token0_during_bucket,
            ),
            fee_generated_token1_start: u256_to_nat(value.fee_generated_token1_start),
            fee_generated_token1_during_bucket: u256_to_nat(
                value.fee_generated_token1_during_bucket,
            ),
            token0_reserves: u256_to_nat(value.token0_reserves),
            token1_reserves: u256_to_nat(value.token1_reserves),
            last_sqrtx96_price: u256_to_nat(value.last_sqrtx96_price),
            inrange_liquidity: value.inrange_liquidity.into(),
            active_tick: value.active_tick.into(),
        }
    }
}
