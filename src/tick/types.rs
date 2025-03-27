use crate::pool::types::PoolId;
use ethnum::U256;
use minicbor::{Decode, Encode};

#[derive(Encode, Decode, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TickKey {
    #[n(0)]
    pub pool_id: PoolId,
    #[n(1)]
    pub tick: i32,
}

#[derive(Encode, Decode, Clone)]
#[cbor(map)]
pub struct TickInfo {
    #[cbor(n(0), with = "crate::cbor::u128")]
    pub liquidity_gross: u128, // Total liquidity at this tick
    #[cbor(n(1), with = "crate::cbor::i128")]
    pub liquidity_net: i128, // Net liquidity change
    #[cbor(n(2), with = "crate::cbor::u256")]
    pub fee_growth_outside_0_x128: U256, // Fees outside for token0
    #[cbor(n(3), with = "crate::cbor::u256")]
    pub fee_growth_outside_1_x128: U256, // Fees outside for token1
    #[n(4)]
    pub tick_cumulative_outside: i64, // Cumulative tick value
    #[cbor(n(5), with = "crate::cbor::u256")]
    pub seconds_per_liquidity_outside_x128: U256, // Time-weighted liquidity
    #[n(6)]
    pub seconds_outside: u32, // Seconds spent outside
    #[n(7)]
    pub initialized: bool, // Whether the tick is initialized
}
