use crate::{
    libraries::safe_cast::u256_to_nat,
    tick::types::{TickInfo, TickKey},
};

use super::*;

#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub struct CandidTickInfo {
    pub tick: Int,                      // Tick number
    pub liquidity_gross: Nat,           // Total liquidity at this tick
    pub liquidity_net: Int,             // Net liquidity change
    pub fee_growth_outside_0_x128: Nat, // Fees outside for token0
    pub fee_growth_outside_1_x128: Nat, // Fees outside for token1
}

impl From<(TickKey, TickInfo)> for CandidTickInfo {
    fn from(value: (TickKey, TickInfo)) -> Self {
        Self {
            liquidity_gross: value.1.liquidity_gross.into(),
            liquidity_net: value.1.liquidity_net.into(),
            fee_growth_outside_0_x128: u256_to_nat(value.1.fee_growth_outside_0_x128),
            fee_growth_outside_1_x128: u256_to_nat(value.1.fee_growth_outside_1_x128),
            tick: value.0.tick.into(),
        }
    }
}
