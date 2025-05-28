pub mod storage;

use candid::Principal;
use ethnum::U256;
use minicbor::{Decode, Encode};

use crate::{position::types::PositionKey, validation::swap_args::ValidatedSwapArgs};

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
        amount0_paid: U256,
        #[cbor(n(3), with = "crate::cbor::u256")]
        amount1_paid: U256,
        #[cbor(n(4), with = "crate::cbor::principal")]
        principal: Principal,
    },
    #[n(1)]
    IncreasedLiquidity {
        #[n(0)]
        modified_position: PositionKey,
        #[cbor(n(1), with = "crate::cbor::u128")]
        liquidity_delta: u128,
        #[cbor(n(2), with = "crate::cbor::u256")]
        amount0_paid: U256,
        #[cbor(n(3), with = "crate::cbor::u256")]
        amount1_paid: U256,
        #[cbor(n(4), with = "crate::cbor::principal")]
        principal: Principal,
    },
    #[n(2)]
    BurntPosition {
        #[n(0)]
        burnt_position: PositionKey,
        #[cbor(n(1), with = "crate::cbor::u128")]
        liquidity: u128,
        #[cbor(n(2), with = "crate::cbor::u256")]
        amount0_received: U256,
        #[cbor(n(3), with = "crate::cbor::u256")]
        amount1_received: U256,
        #[cbor(n(4), with = "crate::cbor::principal")]
        principal: Principal,
    },
    #[n(3)]
    DecreasedLiquidity {
        #[n(0)]
        modified_position: PositionKey,
        #[cbor(n(1), with = "crate::cbor::u128")]
        liquidity_delta: u128,
        #[cbor(n(2), with = "crate::cbor::u256")]
        amount0_received: U256,
        #[cbor(n(3), with = "crate::cbor::u256")]
        amount1_received: U256,
        #[cbor(n(4), with = "crate::cbor::principal")]
        principal: Principal,
    },
    #[n(4)]
    CollectedFees {
        #[n(0)]
        position: PositionKey,
        #[cbor(n(1), with = "crate::cbor::u256")]
        amount0_collected: U256,
        #[cbor(n(2), with = "crate::cbor::u256")]
        amount1_collected: U256,
        #[cbor(n(3), with = "crate::cbor::principal")]
        principal: Principal,
    },
    #[n(5)]
    Swap {
        #[cbor(n(0), with = "crate::cbor::u256")]
        final_amount_in: U256,
        #[cbor(n(1), with = "crate::cbor::u256")]
        final_amount_out: U256,
        #[n(2)]
        swap_args: ValidatedSwapArgs,
        #[cbor(n(3), with = "crate::cbor::principal")]
        principal: Principal,
    },
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
