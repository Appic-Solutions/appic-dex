use crate::pool::types::PoolId;
use ethnum::U256;
use minicbor::{Decode, Encode};

#[derive(Encode, Decode, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct TickKey {
    #[n(0)]
    pub pool_id: PoolId,
    #[n(1)]
    pub tick: i32,
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub struct TickInfo {
    #[cbor(n(0), with = "crate::cbor::u128")]
    pub liquidity_gross: u128, // Total liquidity at this tick
    #[cbor(n(1), with = "crate::cbor::i128")]
    pub liquidity_net: i128, // Net liquidity change
    #[cbor(n(2), with = "crate::cbor::u256")]
    pub fee_growth_outside_0_x128: U256, // Fees outside for token0
    #[cbor(n(3), with = "crate::cbor::u256")]
    pub fee_growth_outside_1_x128: U256, // Fees outside for token1
}

impl Default for TickInfo {
    fn default() -> Self {
        Self {
            liquidity_gross: 0,
            liquidity_net: 0,
            fee_growth_outside_0_x128: U256::ZERO,
            fee_growth_outside_1_x128: U256::ZERO,
        }
    }
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct TickBitmapKey {
    #[n(0)]
    pub pool_id: PoolId, // Pool identifier
    #[n(1)]
    pub word_pos: i16, // Bitmap word position (tick >> 8)
}

#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, Debug)]
pub struct BitmapWord(#[cbor(n(0), with = "crate::cbor::u256")] pub U256);
