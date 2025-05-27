pub mod storage;

use candid::Principal;
use ethnum::U256;
use minicbor::{Decode, Encode};

use crate::position::types::PositionKey;

/// The event describing the  minter state transition.
#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq)]
pub enum EventType {
    #[n(0)]
    MintedPosition {
        #[n(0)]
        created_position: PositionKey,
        #[cbor(n(1), with = "crate::cbor::u128")]
        liquidity: u128,
        #[cbor(n(2), with = "crate::cbor::u256")]
        amount0: U256,
        #[cbor(n(3), with = "crate::cbor::u256")]
        amount1: U256,
    },
    #[n(1)]
    IncreasedLiquidity {
        #[n(0)]
        modified_position: PositionKey,
        #[cbor(n(1), with = "crate::cbor::u128")]
        liquidity_delta: u128,
        #[cbor(n(2), with = "crate::cbor::u256")]
        amount0: U256,
        #[cbor(n(3), with = "crate::cbor::u256")]
        amount1: U256,
    },
    #[n(2)]
    BurntPosition {
        #[n(0)]
        burnt_position: PositionKey,
        #[cbor(n(1), with = "crate::cbor::u128")]
        liqudity: u128,
        #[cbor(n(2), with = "crate::cbor::u256")]
        amount0: U256,
        #[cbor(n(3), with = "crate::cbor::u256")]
        amount1: U256,
    },
    #[n(3)]
    DecreasedLiqudity {
        #[n(0)]
        modified_position: PositionKey,
        #[cbor(n(1), with = "crate::cbor::u128")]
        liquidity_delta: u128,
        #[cbor(n(2), with = "crate::cbor::u256")]
        amount0: U256,
        #[cbor(n(3), with = "crate::cbor::u256")]
        amount1: U256,
    },
    #[n(4)]
    CollectedFees {
        #[n(0)]
        position: PositionKey,
        #[cbor(n(1), with = "crate::cbor::u256")]
        amount0: U256,
        #[cbor(n(2), with = "crate::cbor::u256")]
        amount1: U256,
    },
    #[n(5)]
    Swap {},
}

#[derive(Encode, Decode, Debug, PartialEq, Eq)]
pub struct Event {
    /// The canister time at which the minter generated this event.
    #[n(0)]
    pub timestamp: u64,
    /// The event type.
    #[n(1)]
    pub payload: EventType,
}
