use crate::libraries::constants::{MAX_TICK, MIN_TICK};
use crate::libraries::liquidity_math::{self, AddDeltaError};
use crate::state::{mutate_state, read_state};
use ethnum::U256;
use types::{TickInfo, TickKey};

pub mod types;

#[cfg(test)]
pub mod tests;

/// Derives max liquidity per tick from tick spacing.
/// Used in pool initialization.
pub fn tick_spacing_to_max_liquidity_per_tick(tick_spacing: i32) -> u128 {
    let min_tick = (MIN_TICK / tick_spacing) * tick_spacing;
    let max_tick = (MAX_TICK / tick_spacing) * tick_spacing;
    let num_ticks = ((max_tick - min_tick) / tick_spacing) as u32 + 1;
    u128::MAX / num_ticks as u128
}

pub fn get_fee_growth_inside(
    tick_lower: &TickKey,
    tick_upper: &TickKey,
    lower_info: &TickInfo,
    upper_info: &TickInfo,
    tick_current: &TickKey,
    fee_growth_global_0_x128: U256,
    fee_growth_global_1_x128: U256,
) -> (U256, U256) {
    // Calculate fee growth inside the tick range based on the current tick position
    let (fee_growth_below_0_x128, fee_growth_below_1_x128) = if tick_current.tick >= tick_lower.tick
    {
        (
            lower_info.fee_growth_outside_0_x128,
            lower_info.fee_growth_outside_1_x128,
        )
    } else {
        (
            fee_growth_global_0_x128.wrapping_sub(lower_info.fee_growth_outside_0_x128),
            fee_growth_global_1_x128.wrapping_sub(lower_info.fee_growth_outside_1_x128),
        )
    };

    let (fee_growth_above_0_x128, fee_growth_above_1_x128) = if tick_current.tick < tick_upper.tick
    {
        (
            upper_info.fee_growth_outside_0_x128,
            upper_info.fee_growth_outside_1_x128,
        )
    } else {
        (
            fee_growth_global_0_x128.wrapping_sub(upper_info.fee_growth_outside_0_x128),
            fee_growth_global_1_x128.wrapping_sub(upper_info.fee_growth_outside_1_x128),
        )
    };

    let (fee_growth_inside_0_x128, fee_growth_inside_1_x128) = (
        fee_growth_global_0_x128
            .wrapping_sub(fee_growth_below_0_x128)
            .wrapping_sub(fee_growth_above_0_x128),
        fee_growth_global_1_x128
            .wrapping_sub(fee_growth_below_1_x128)
            .wrapping_sub(fee_growth_above_1_x128),
    );

    (fee_growth_inside_0_x128, fee_growth_inside_1_x128)
}

#[derive(Debug, Clone, PartialEq)]
pub enum UpdateTickError {
    AddDeltaError(AddDeltaError),
    LiquidityNetOverflow,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UpdateTickSuccess {
    pub flipped: bool,
    pub liquidity_gross_after: u128,
    pub updated_tick_info: TickInfo,
}

/// Updates a tick and returns true if the tick was flipped from initialized to uninitialized, or vice versa
/// returns liquidityGrossAfter The total amount of liquidity for all positions that references the tick after the update
pub fn update_tick(
    tick: &TickKey,
    tick_current: &TickKey,
    liquidity_delta: i128,
    fee_growth_global_0_x128: U256,
    fee_growth_global_1_x128: U256,
    upper: bool,
) -> Result<UpdateTickSuccess, UpdateTickError> {
    // Get mutable tick info, defaulting to zeroed if not found
    let mut tick_info = read_state(|s| s.get_tick(tick));

    let liquidity_gross_before = tick_info.liquidity_gross;
    let liquidity_gross_after = liquidity_math::add_delta(liquidity_gross_before, liquidity_delta)
        .map_err(|e| UpdateTickError::AddDeltaError(e))?;

    let flipped = (liquidity_gross_after == 0) != (liquidity_gross_before == 0);

    if liquidity_gross_before == 0 {
        // by convention, we assume that all growth before a tick was initialized happened _below_ the tick
        if tick.tick <= tick_current.tick {
            tick_info.fee_growth_outside_0_x128 = fee_growth_global_0_x128;
            tick_info.fee_growth_outside_1_x128 = fee_growth_global_1_x128;
        }
    }

    tick_info.liquidity_gross = liquidity_gross_after;

    // when the lower (upper) tick is crossed left to right, liquidity must be added (removed)
    // when the lower (upper) tick is crossed right to left, liquidity must be removed (added)
    tick_info.liquidity_net = if upper {
        tick_info
            .liquidity_net
            .checked_sub(liquidity_delta)
            .ok_or(UpdateTickError::LiquidityNetOverflow)?
    } else {
        tick_info
            .liquidity_net
            .checked_add(liquidity_delta)
            .ok_or(UpdateTickError::LiquidityNetOverflow)?
    };

    // Storing updated tick info will not happen here since this function will be called in other
    // operations and in those operations there are other function calls that can fail, so tick
    // updating happens in those operations if nothing fails
    // This way we guarantee we don't need a state reverting mechanism
    Ok(UpdateTickSuccess {
        flipped,
        liquidity_gross_after,
        updated_tick_info: tick_info,
    })
}

#[derive(Debug, Clone, PartialEq)]
pub struct CrossTickSuccess {
    pub liquidity_net: i128,
    pub updated_tick_info: TickInfo,
}

/// Transitions to next tick as needed by price movement
/// returns liquidityNet The amount of liquidity added (subtracted) when tick is crossed from left to right (right to left)
pub fn cross_tick(
    tick_info: &mut TickInfo,
    fee_growth_global_0_x128: U256,
    fee_growth_global_1_x128: U256,
) -> i128 {
    // Get mutable tick info, defaulting to zeroed if not found

    tick_info.fee_growth_outside_0_x128 =
        fee_growth_global_0_x128 - tick_info.fee_growth_outside_0_x128;
    tick_info.fee_growth_outside_1_x128 =
        fee_growth_global_1_x128 - tick_info.fee_growth_outside_1_x128;

    tick_info.liquidity_net
}
