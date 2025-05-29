use candid::{CandidType, Nat, Principal};
use serde::Deserialize;

use crate::{events::Event, libraries::safe_cast::u256_to_nat, validation};

use super::{pool::CandidPoolId, position::CandidPositionKey};

#[derive(CandidType, Deserialize, Debug, Clone)]
pub struct GetEventsArg {
    pub start: u64,
    pub length: u64,
}

#[derive(CandidType, Deserialize, Debug, Clone)]
pub struct GetEventsResult {
    pub events: Vec<CandidEvent>,
    pub total_event_count: u64,
}

/// The event describing the  minter state transition.
#[derive(CandidType, Deserialize, Debug, Clone)]
pub enum CandidEventType {
    CreatedPool {
        token0: Principal,
        token1: Principal,
        pool_fee: Nat,
    },
    MintedPosition {
        created_position: CandidPositionKey,
        liquidity: Nat,
        amount0_paid: Nat,
        amount1_paid: Nat,
        principal: Principal,
    },
    IncreasedLiquidity {
        modified_position: CandidPositionKey,
        liquidity_delta: Nat,
        amount0_paid: Nat,
        amount1_paid: Nat,
        principal: Principal,
    },
    BurntPosition {
        burnt_position: CandidPositionKey,
        liquidity: Nat,
        amount0_received: Nat,
        amount1_received: Nat,
        principal: Principal,
    },
    DecreasedLiquidity {
        modified_position: CandidPositionKey,
        liquidity_delta: Nat,
        amount0_received: Nat,
        amount1_received: Nat,
        principal: Principal,
    },
    CollectedFees {
        position: CandidPositionKey,
        amount0_collected: Nat,
        amount1_collected: Nat,
        principal: Principal,
    },
    Swap {
        final_amount_in: Nat,
        final_amount_out: Nat,
        token_in: Principal,
        token_out: Principal,
        swap_type: SwapType,
        principal: Principal,
    },
}

#[derive(CandidType, Deserialize, Debug, Clone)]
pub enum SwapType {
    ExactInputSingle(CandidPoolId),
    ExactInput(Vec<CandidPoolId>),
    ExactOutputSingle(CandidPoolId),
    ExactOutput(Vec<CandidPoolId>),
}

#[derive(CandidType, Deserialize, Debug, Clone)]
pub struct CandidEvent {
    /// The canister time at which the minter generated this event.
    pub timestamp: u64,
    /// The event type.
    pub payload: CandidEventType,
}

impl From<Event> for CandidEvent {
    fn from(value: Event) -> Self {
        let payload = match value.payload {
            crate::events::EventType::CreatedPool {
                token0,
                token1,
                pool_fee,
            } => CandidEventType::CreatedPool {
                token0,
                token1,
                pool_fee: pool_fee.into(),
            },
            crate::events::EventType::MintedPosition {
                created_position,
                liquidity,
                amount0_paid,
                amount1_paid,
                principal,
            } => CandidEventType::MintedPosition {
                created_position: created_position.into(),
                liquidity: liquidity.into(),
                amount0_paid: u256_to_nat(amount0_paid),
                amount1_paid: u256_to_nat(amount1_paid),
                principal,
            },
            crate::events::EventType::IncreasedLiquidity {
                modified_position,
                liquidity_delta,
                amount0_paid,
                amount1_paid,
                principal,
            } => CandidEventType::IncreasedLiquidity {
                modified_position: modified_position.into(),
                liquidity_delta: liquidity_delta.into(),
                amount0_paid: u256_to_nat(amount0_paid),
                amount1_paid: u256_to_nat(amount1_paid),
                principal,
            },

            crate::events::EventType::BurntPosition {
                burnt_position,
                liquidity,
                amount0_received,
                amount1_received,
                principal,
            } => CandidEventType::BurntPosition {
                burnt_position: burnt_position.into(),
                liquidity: liquidity.into(),
                amount0_received: u256_to_nat(amount0_received),
                amount1_received: u256_to_nat(amount1_received),
                principal,
            },
            crate::events::EventType::DecreasedLiquidity {
                modified_position,
                liquidity_delta,
                amount0_received,
                amount1_received,
                principal,
            } => CandidEventType::DecreasedLiquidity {
                modified_position: modified_position.into(),
                liquidity_delta: liquidity_delta.into(),
                amount0_received: u256_to_nat(amount0_received),
                amount1_received: u256_to_nat(amount1_received),
                principal,
            },
            crate::events::EventType::CollectedFees {
                position,
                amount0_collected,
                amount1_collected,
                principal,
            } => CandidEventType::CollectedFees {
                position: position.into(),
                amount0_collected: u256_to_nat(amount0_collected),
                amount1_collected: u256_to_nat(amount1_collected),
                principal,
            },
            crate::events::EventType::Swap {
                final_amount_in,
                final_amount_out,
                swap_args,
                principal,
            } => {
                let (swap_type, token_in, token_out) = match swap_args {
                    validation::swap_args::ValidatedSwapArgs::ExactInputSingle {
                        pool_id,
                        zero_for_one: _,
                        amount_in: _,
                        amount_out_minimum: _,
                        from_subaccount: _,
                        token_in,
                        token_out,
                    } => (
                        SwapType::ExactInputSingle(pool_id.into()),
                        token_in,
                        token_out,
                    ),
                    validation::swap_args::ValidatedSwapArgs::ExactInput {
                        path,
                        amount_in: _,
                        amount_out_minimum: _,
                        from_subaccount: _,
                        token_in,
                        token_out,
                    } => (
                        SwapType::ExactInput(
                            path.into_iter()
                                .map(|swap| CandidPoolId::from(swap.pool_id))
                                .collect(),
                        ),
                        token_in,
                        token_out,
                    ),
                    validation::swap_args::ValidatedSwapArgs::ExactOutputSingle {
                        pool_id,
                        zero_for_one: _,
                        amount_out: _,
                        amount_in_maximum: _,
                        from_subaccount: _,
                        token_in,
                        token_out,
                    } => (
                        SwapType::ExactOutputSingle(pool_id.into()),
                        token_in,
                        token_out,
                    ),
                    validation::swap_args::ValidatedSwapArgs::ExactOutput {
                        path,
                        amount_out: _,
                        amount_in_maximum: _,
                        from_subaccount: _,
                        token_in,
                        token_out,
                    } => (
                        SwapType::ExactOutput(
                            path.into_iter()
                                .map(|swap| CandidPoolId::from(swap.pool_id))
                                .collect(),
                        ),
                        token_in,
                        token_out,
                    ),
                };

                CandidEventType::Swap {
                    final_amount_in: u256_to_nat(final_amount_in),
                    final_amount_out: u256_to_nat(final_amount_out),
                    token_in,
                    token_out,
                    swap_type,
                    principal,
                }
            }
        };
        Self {
            timestamp: value.timestamp,
            payload,
        }
    }
}
