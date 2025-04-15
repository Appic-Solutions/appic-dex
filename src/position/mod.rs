use ethnum::U256;
use types::{PositionInfo, PositionKey};

use crate::{
    libraries::{
        constants::Q128,
        full_math::{mul_div, FullMathError},
        liquidity_math::{self, AddDeltaError},
    },
    state::read_state,
};

pub mod types;

#[derive(Debug, Clone, PartialEq)]
pub enum UpdatePsotionError {
    ZeropLiquidity,
    AddDeltaError(AddDeltaError),
    MathError(FullMathError),
}

#[derive(Debug, Clone, PartialEq)]
pub struct UpdatePsotionSuccess {
    pub fee0_owed: U256,
    pub fee1_owed: U256,
    pub updated_position_info: PositionInfo,
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
) -> Result<UpdatePsotionSuccess, UpdatePsotionError> {
    let mut position_info = read_state(|s| s.get_position(position_key));

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

    // Storing updated position info will not happen here since this function will be called in other
    // operations as well and in those operations there are other function calls that can fail, so tick
    // updating happens in those operations if nothing fails
    // This way we guarantee we dont need a state reverting mechanism

    Ok(UpdatePsotionSuccess {
        fee0_owed,
        fee1_owed,
        updated_position_info: position_info,
    })
}
