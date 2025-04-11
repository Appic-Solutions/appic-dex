use ethnum::U256;
use types::PositionKey;

use crate::{
    libraries::{
        constants::Q128,
        full_math::{mul_div, FullMathError},
        liquidity_math::{self, AddDeltaError},
    },
    state::{mutate_state, read_state},
};

pub mod types;

#[derive(Debug, Clone, PartialEq)]
pub enum UpdatePsotionError {
    PositionNotFound,
    ZeropLiquidity,
    AddDeltaError(AddDeltaError),
    MathError(FullMathError),
}

/// Credits accumulated fees to a user's position
/// param position_key of the individual position to update
/// param liquidityDelta The change in pool liquidity as a result of the position update
/// param feeGrowthInside0X128 The all-time fee growth in currency0, per unit of liquidity, inside the position's tick boundaries
/// param feeGrowthInside1X128 The all-time fee growth in currency1, per unit of liquidity, inside the position's tick boundaries
/// return feesOwed0 The amount of currency0 owed to the position owner
/// return feesOwed1 The amount of currency1 owed to the position owner
pub fn update_position(
    position_key: &PositionKey,
    liquidity_delta: i128,
    fee_growth_inside_0_x128: U256,
    fee_growth_inside_1_x128: U256,
) -> Result<(U256, U256), UpdatePsotionError> {
    let mut position_info =
        read_state(|s| s.get_position(position_key)).ok_or(UpdatePsotionError::PositionNotFound)?;

    let liquidity = position_info.liquidity.clone();
    if liquidity_delta == 0 {
        // disallow pokes for 0 liquidity positions
        if liquidity == 0 {
            return Err(UpdatePsotionError::ZeropLiquidity);
        } else {
            position_info.liquidity = liquidity_math::add_delta(liquidity, liquidity_delta)
                .map_err(|e| UpdatePsotionError::AddDeltaError(e))?;
        }
    };

    // calculate accumulated fees. overflow in the subtraction of fee growth is expected
    let fee0_owed = mul_div(
        fee_growth_inside_0_x128 - position_info.fee_growth_inside_0_last_x128,
        liquidity.into(),
        Q128.clone(),
    )
    .map_err(|e| UpdatePsotionError::MathError(e))?;

    let fee1_owed = mul_div(
        fee_growth_inside_1_x128 - position_info.fee_growth_inside_1_last_x128,
        liquidity.into(),
        Q128.clone(),
    )
    .map_err(|e| UpdatePsotionError::MathError(e))?;

    position_info.fee_growth_inside_0_last_x128 = fee_growth_inside_0_x128;
    position_info.fee_growth_inside_1_last_x128 = fee_growth_inside_1_x128;

    mutate_state(|s| s.update_position(position_key.clone(), position_info));

    Ok((fee0_owed, fee1_owed))
}
