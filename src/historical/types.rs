use ethnum::U256;
use minicbor::{Decode, Encode};

/// Historical data bucket for a specific timeframe.
#[derive(Encode, Decode, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Default)]
pub struct HistoryBucket {
    #[n(0)]
    pub start_timestamp: u64,
    #[n(1)]
    pub end_timestamp: u64,
    #[cbor(n(2), with = "crate::cbor::u256")]
    pub swap_volume_token0_start: U256,
    #[cbor(n(3), with = "crate::cbor::u256")]
    pub swap_volume_token0_during_bucket: U256,
    #[cbor(n(4), with = "crate::cbor::u256")]
    pub swap_volume_token1_start: U256,
    #[cbor(n(5), with = "crate::cbor::u256")]
    pub swap_volume_token1_during_bucket: U256,
    #[cbor(n(6), with = "crate::cbor::u256")]
    pub fee_generated_token0_start: U256,
    #[cbor(n(7), with = "crate::cbor::u256")]
    pub fee_generated_token0_during_bucket: U256,
    #[cbor(n(8), with = "crate::cbor::u256")]
    pub fee_generated_token1_start: U256,
    #[cbor(n(9), with = "crate::cbor::u256")]
    pub fee_generated_token1_during_bucket: U256,
    #[cbor(n(10), with = "crate::cbor::u256")]
    pub token0_reserves: U256,
    #[cbor(n(11), with = "crate::cbor::u256")]
    pub token1_reserves: U256,
    #[cbor(n(12), with = "crate::cbor::u256")]
    pub last_sqrtx96_price: U256,
    #[cbor(n(13), with = "crate::cbor::u128")]
    pub inrange_liquidity: u128,
    #[n(14)]
    pub active_tick: i32,
}

/// Stores historical data for a pool across multiple timeframes.
#[derive(Encode, Decode, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Default)]
pub struct PoolHistory {
    #[n(0)]
    pub hourly_frame: Vec<HistoryBucket>,
    #[n(1)]
    pub daily_frame: Vec<HistoryBucket>,
    #[n(2)]
    pub monthly_frame: Vec<HistoryBucket>,
    #[n(3)]
    pub yearly_frame: Vec<HistoryBucket>,
}
