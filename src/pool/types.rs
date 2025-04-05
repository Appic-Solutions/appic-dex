use candid::Principal;
use ethnum::U256;
use minicbor::{Decode, Encode};

#[derive(Encode, Decode, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PoolId {
    #[cbor(n(0), with = "crate::cbor::principal")]
    pub token0: Principal, // Token0 identifier
    #[cbor(n(1), with = "crate::cbor::principal")]
    pub token1: Principal, // Token1 identifier
    #[n(2)]
    pub fee: u16, // Fee tier (e.g., 500 for 0.05%)
}

#[derive(Encode, Decode, Clone)]
pub struct ProtocolFees {
    #[cbor(n(0), with = "crate::cbor::u128")]
    pub token0: u128, // Fees in token0 units
    #[cbor(n(1), with = "crate::cbor::u128")]
    pub token1: u128, // Fees in token1 units
}

#[derive(Encode, Decode, Clone)]
pub struct PoolState {
    #[cbor(n(0), with = "crate::cbor::u256")]
    pub sqrt_price_x96: U256, // Current price in Q64.96 format
    #[n(1)]
    pub tick: i32, // Current tick index
    #[cbor(n(2), with = "crate::cbor::u256")]
    pub fee_growth_global_0_x128: U256, // Cumulative fees for token0
    #[cbor(n(3), with = "crate::cbor::u256")]
    pub fee_growth_global_1_x128: U256, // Cumulative fees for token1
    #[n(4)]
    pub protocol_fees: ProtocolFees, // Accumulated protocol fees
    #[cbor(n(5), with = "crate::cbor::u128")]
    pub liquidity: u128, // Total active liquidity
    #[n(6)]
    pub tick_spacing: i32, // Spacing between ticks
    #[cbor(n(7), with = "crate::cbor::u128")]
    pub max_liquidity_per_tick: u128, // Max liquidity per tick
    #[n(8)]
    pub fee_protocol: u8, // Protocol fee denominator (1/x%)
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TokenId {
    #[n(0)]
    pub pool_id: PoolId,
    #[n(1)]
    pub token_index: u8, // 0 for token0, 1 for token1
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TokenBalance(#[cbor(n(0), with = "crate::cbor::u256")] U256);
