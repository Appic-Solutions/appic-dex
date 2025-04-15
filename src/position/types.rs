use crate::pool::types::PoolId;
use candid::Principal;
use ethnum::U256;
use minicbor::{Decode, Encode};

#[derive(Encode, Decode, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PositionKey {
    #[cbor(n(0), with = "crate::cbor::principal")]
    pub owner: Principal,
    #[n(1)]
    pub pool_id: PoolId,
    #[n(2)]
    pub tick_lower: i32,
    #[n(3)]
    pub tick_upper: i32,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, Default)]
pub struct PositionInfo {
    #[cbor(n(0), with = "crate::cbor::u128")]
    pub liquidity: u128, // Position liquidity
    #[cbor(n(1), with = "crate::cbor::u256")]
    pub fee_growth_inside_0_last_x128: U256, // Fees for token0 at last update
    #[cbor(n(2), with = "crate::cbor::u256")]
    pub fee_growth_inside_1_last_x128: U256, // Fees for token1 at last update
}
