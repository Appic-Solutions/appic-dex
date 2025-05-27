use candid::{Nat, Principal};
use ethnum::U256;
use minicbor::{Decode, Encode};

#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct PoolFee(#[n(0)] pub u32);

impl TryFrom<Nat> for PoolFee {
    type Error = String;

    fn try_from(value: Nat) -> Result<Self, Self::Error> {
        u32::try_from(value.0)
            .map(|fee| PoolFee(fee))
            .map_err(|err| format!("{}", err))
    }
}

#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, PartialOrd, Debug, Ord)]
pub struct PoolTickSpacing(#[n(0)] pub i32);

#[derive(Encode, Decode, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct PoolId {
    #[cbor(n(0), with = "crate::cbor::principal")]
    pub token0: Principal, // Token0 identifier
    #[cbor(n(1), with = "crate::cbor::principal")]
    pub token1: Principal, // Token1 identifier
    #[n(2)]
    pub fee: PoolFee, // Fee tier (e.g., 500 for 0.05%)
}

#[derive(Encode, Decode, Clone, Debug, Eq, PartialEq)]
pub struct PoolState {
    #[cbor(n(0), with = "crate::cbor::u256")]
    pub sqrt_price_x96: U256, // Current price in Q64.96 format
    #[n(1)]
    pub tick: i32, // Current tick index
    #[cbor(n(2), with = "crate::cbor::u256")]
    pub fee_growth_global_0_x128: U256, // Cumulative fees for token0
    #[cbor(n(3), with = "crate::cbor::u256")]
    pub fee_growth_global_1_x128: U256, // Cumulative fees for token1
    #[cbor(n(4), with = "crate::cbor::u128")]
    pub liquidity: u128, // Total active liquidity
    #[n(5)]
    pub tick_spacing: PoolTickSpacing, // Spacing between ticks
    #[cbor(n(6), with = "crate::cbor::u128")]
    pub max_liquidity_per_tick: u128, // Max liquidity per tick
    #[n(7)]
    pub fee_protocol: u16, // Max protocol fee is 0.1% (1000 pips)

    #[cbor(n(8), with = "crate::cbor::u256")]
    pub token0_transfer_fee: U256, // transfe fee for token 0
    #[cbor(n(9), with = "crate::cbor::u256")]
    pub token1_transfer_fee: U256, // transfer fee for token 1
    #[cbor(n(10), with = "crate::cbor::u256")]
    pub swap_volume0_all_time: U256, // swap volume for token0 in the whole pool's life time
    #[cbor(n(11), with = "crate::cbor::u256")]
    pub swap_volume1_all_time: U256, // swap volume for token1 in the whole pool's life time
    #[cbor(n(12), with = "crate::cbor::u256")]
    pub pool_reserve0: U256, // balance of token 0 in pool
    #[cbor(n(13), with = "crate::cbor::u256")]
    pub pool_reserve1: U256, // balance of token 1 in the pool
}
