use candid::Principal;
use ethnum::{I256, U256};

use crate::{
    endpoints::{
        BurnPositionArgs, BurnPositionError, MintPositionArgs, MintPositionError, SwapArgs,
        SwapError,
    },
    libraries::{
        constants::{MAX_TICK, MIN_TICK},
        path_key::{PathKey, Swap},
        safe_cast::big_uint_to_u256,
    },
    pool::types::{PoolId, PoolTickSpacing},
    position::types::{PositionInfo, PositionKey},
    state::read_state,
};

pub struct ValidatedMintPositionArgs {
    pub tick_spacing: PoolTickSpacing,
    pub lower_tick: i32,
    pub upper_tick: i32,
    pub pool_id: PoolId,
    pub amount0_max: I256,
    pub amount1_max: I256,
}
pub fn validate_mint_position_args(
    args: MintPositionArgs,
    caller: Principal,
) -> Result<ValidatedMintPositionArgs, MintPositionError> {
    // check pool
    let pool_id: PoolId = args
        .pool
        .try_into()
        .map_err(|_e| MintPositionError::InvalidPoolFee)?;

    let pool = read_state(|s| s.get_pool(&pool_id)).ok_or(MintPositionError::PoolNotInitialized)?;
    let tick_spacing = pool.tick_spacing;
    // check ticks
    let lower_tick: i32 = args
        .tick_lower
        .0
        .try_into()
        .map_err(|_e| MintPositionError::InvalidTick)?;
    let upper_tick: i32 = args
        .tick_upper
        .0
        .try_into()
        .map_err(|_e| MintPositionError::InvalidTick)?;
    if lower_tick < MIN_TICK || upper_tick > MAX_TICK || lower_tick >= upper_tick {
        return Err(MintPositionError::InvalidTick);
    };

    // position should not exist
    let position_key = PositionKey {
        owner: caller,
        pool_id: pool_id.clone(),
        tick_lower: lower_tick,
        tick_upper: upper_tick,
    };
    if read_state(|s| s.get_position(&position_key)).liquidity != 0 {
        return Err(MintPositionError::PositionAlreadyExists);
    }

    // check alignment with tick spacing
    if upper_tick % tick_spacing.0 != 0 || lower_tick % tick_spacing.0 != 0 {
        return Err(MintPositionError::TickNotAlignedWithTickSpacing);
    };

    let amount0_max: U256 =
        big_uint_to_u256(args.amount0_max.0).map_err(|_e| MintPositionError::InvalidAmount)?;
    let amount1_max: U256 =
        big_uint_to_u256(args.amount1_max.0).map_err(|_e| MintPositionError::InvalidAmount)?;

    // MAX amount should be I256 to prevent overflow
    let amount0_max: I256 = amount0_max
        .try_into()
        .map_err(|_e| MintPositionError::InvalidAmount)?;
    let amount1_max: I256 = amount1_max
        .try_into()
        .map_err(|_e| MintPositionError::InvalidAmount)?;

    Ok(ValidatedMintPositionArgs {
        tick_spacing,
        lower_tick,
        upper_tick,
        pool_id,
        amount0_max,
        amount1_max,
    })
}

pub struct ValidatedBurnPositionArgs {
    pub tick_spacing: PoolTickSpacing,
    pub lower_tick: i32,
    pub upper_tick: i32,
    pub position_key: PositionKey,
    pub position_info: PositionInfo,
    pub pool_id: PoolId,
    pub amount0_min: I256,
    pub amount1_min: I256,
    pub liquidity_delta: i128,
}

pub fn validate_burn_position_args(
    args: BurnPositionArgs,
    caller: Principal,
) -> Result<ValidatedBurnPositionArgs, BurnPositionError> {
    // check pool
    let pool_id: PoolId = args
        .pool
        .try_into()
        .map_err(|_e| BurnPositionError::InvalidPoolFee)?;
    let pool = read_state(|s| s.get_pool(&pool_id)).ok_or(BurnPositionError::PoolNotInitialized)?;
    let tick_spacing = pool.tick_spacing;
    // check ticks
    let lower_tick: i32 = args
        .tick_lower
        .0
        .try_into()
        .map_err(|_e| BurnPositionError::InvalidTick)?;
    let upper_tick: i32 = args
        .tick_upper
        .0
        .try_into()
        .map_err(|_e| BurnPositionError::InvalidTick)?;
    if lower_tick < MIN_TICK || upper_tick > MAX_TICK || lower_tick >= upper_tick {
        return Err(BurnPositionError::InvalidTick);
    };

    // position should not exist
    let position_key = PositionKey {
        owner: caller,
        pool_id: pool_id.clone(),
        tick_lower: lower_tick,
        tick_upper: upper_tick,
    };
    let position_info = read_state(|s| s.get_position(&position_key));
    if position_info.liquidity == 0 {
        return Err(BurnPositionError::PositionNotFound);
    }

    let amount0_min: U256 =
        big_uint_to_u256(args.amount0_min.0).map_err(|_e| BurnPositionError::InvalidAmount)?;
    let amount1_min: U256 =
        big_uint_to_u256(args.amount1_min.0).map_err(|_e| BurnPositionError::InvalidAmount)?;

    // MIN amount should be I256 to prevent overflow
    let amount0_min: I256 = amount0_min
        .try_into()
        .map_err(|_e| BurnPositionError::InvalidAmount)?;
    let amount1_min: I256 = amount1_min
        .try_into()
        .map_err(|_e| BurnPositionError::InvalidAmount)?;

    let liquidity_delta = i128::try_from(position_info.liquidity)
        .map_err(|_e| BurnPositionError::LiquidityOverflow)?
        .checked_mul(-1i128)
        .ok_or(BurnPositionError::LiquidityOverflow)?;

    Ok(ValidatedBurnPositionArgs {
        tick_spacing,
        lower_tick,
        upper_tick,
        position_key,
        position_info,
        pool_id,
        amount0_min,
        amount1_min,
        liquidity_delta,
    })
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ValidatedSwapArgs {
    ExactInputSingle {
        pool_id: PoolId,
        zero_for_one: bool,
        amount_in: U256,
        amount_out_minimum: U256,
    },
    ExactInput {
        // order should be preserved
        path: Vec<Swap>,
        amount_in: U256,
        amount_out_minimum: U256,
    },
    ExactOutputSingle {
        pool_id: PoolId,
        zero_for_one: bool,
        amount_out: U256,
        amount_in_maximum: U256,
    },
    ExactOutput {
        // order should be preserved
        path: Vec<Swap>,
        amount_out: U256,
        amount_in_maximum: U256,
    },
}

// in multi hop swaps the maximum number of hops(swaps) should be <= MAX_PATH_LENGTH
pub const MAX_PATH_LENGTH: u8 = 4;

// in multi hop swaps the minumm number of hops(swaps) should be >= MIN_PATH_LENGTH
// if a swap has less than 2 hops, the single hop swap type should be selected
pub const MIN_PATH_LENGTH: u8 = 2;

pub fn validate_swap_args(args: SwapArgs) -> Result<ValidatedSwapArgs, SwapError> {
    match args {
        SwapArgs::ExactInputSingle(exact_input_single_params) => {
            let pool_id: PoolId = exact_input_single_params
                .pool_id
                .try_into()
                .map_err(|_| SwapError::InvalidPoolFee)?;

            let pool = read_state(|s| s.get_pool(&pool_id)).ok_or(SwapError::PoolNotInitialized)?;
            // In case in range liquidty is 0
            if pool.liquidity == 0 {
                return Err(SwapError::NoInRangeLiquidity);
            }

            let amount_in: U256 = big_uint_to_u256(exact_input_single_params.amount_in.0)
                .map_err(|_| SwapError::InvalidAmountIn)?;
            let amount_out_minimum =
                big_uint_to_u256(exact_input_single_params.amount_out_minimum.0)
                    .map_err(|_| SwapError::InvalidAmountOutMinimum)?;
            Ok(ValidatedSwapArgs::ExactInputSingle {
                pool_id,
                zero_for_one: exact_input_single_params.zero_for_one,
                amount_in,
                amount_out_minimum,
            })
        }
        SwapArgs::ExactInput(exact_input_params) => {
            let path_len = exact_input_params.path.len() as u8;
            if path_len < MIN_PATH_LENGTH {
                return Err(SwapError::PathLengthTooSmall {
                    minimum: MIN_PATH_LENGTH,
                    received: path_len,
                });
            } else if path_len > MAX_PATH_LENGTH {
                return Err(SwapError::PathLengthTooBig {
                    maximum: MAX_PATH_LENGTH,
                    received: path_len,
                });
            };

            let mut token_in = exact_input_params.token_in;
            let mut swap_path = Vec::new();
            for canidid_path in exact_input_params.path.into_iter() {
                let path_key = PathKey::try_from(canidid_path)?;
                swap_path.push(path_key.get_pool_and_swap_direction(token_in));
                token_in = path_key.intermediary_token;
            }

            // check pools
            for swap in swap_path.iter() {
                let pool = read_state(|s| s.get_pool(&swap.pool_id))
                    .ok_or(SwapError::PoolNotInitialized)?;
                // In case in range liquidty is 0
                if pool.liquidity == 0 {
                    return Err(SwapError::NoInRangeLiquidity);
                }
            }

            let amount_in: U256 = big_uint_to_u256(exact_input_params.amount_in.0)
                .map_err(|_| SwapError::InvalidAmountIn)?;
            let amount_out_minimum = big_uint_to_u256(exact_input_params.amount_out_minimum.0)
                .map_err(|_| SwapError::InvalidAmountOutMinimum)?;
            Ok(ValidatedSwapArgs::ExactInput {
                path: swap_path,
                amount_in,
                amount_out_minimum,
            })
        }
        SwapArgs::ExactOutputSingle(exact_output_single_params) => {
            let pool_id: PoolId = exact_output_single_params
                .pool_id
                .try_into()
                .map_err(|_| SwapError::InvalidPoolFee)?;

            let pool = read_state(|s| s.get_pool(&pool_id)).ok_or(SwapError::PoolNotInitialized)?;
            // In case in range liquidty is 0
            if pool.liquidity == 0 {
                return Err(SwapError::NoInRangeLiquidity);
            }

            let amount_out: U256 = big_uint_to_u256(exact_output_single_params.amount_out.0)
                .map_err(|_| SwapError::InvalidAmountIn)?;
            let amount_in_maximum =
                big_uint_to_u256(exact_output_single_params.amount_in_maximum.0)
                    .map_err(|_| SwapError::InvalidAmountInMaximum)?;
            Ok(ValidatedSwapArgs::ExactOutputSingle {
                pool_id,
                zero_for_one: exact_output_single_params.zero_for_one,
                amount_out,
                amount_in_maximum,
            })
        }
        SwapArgs::ExactOutput(exact_output_params) => {
            let path_len = exact_output_params.path.len() as u8;
            if path_len < MIN_PATH_LENGTH {
                return Err(SwapError::PathLengthTooSmall {
                    minimum: MIN_PATH_LENGTH,
                    received: path_len,
                });
            } else if path_len > MAX_PATH_LENGTH {
                return Err(SwapError::PathLengthTooBig {
                    maximum: MAX_PATH_LENGTH,
                    received: path_len,
                });
            };

            let mut token_out = exact_output_params.token_out;
            // in multi hop exact output we go from the opposite direction
            let mut swap_path = Vec::new();
            for canidid_path in exact_output_params.path.into_iter().rev() {
                let path_key = PathKey::try_from(canidid_path)?;
                swap_path.push(path_key.get_pool_and_swap_direction(token_out));
                token_out = path_key.intermediary_token;
            }

            // reverse the swap path
            // since we generated swap path using a reversed direction
            swap_path.reverse();

            // check pools
            for swap in swap_path.iter() {
                let pool = read_state(|s| s.get_pool(&swap.pool_id))
                    .ok_or(SwapError::PoolNotInitialized)?;
                // In case in range liquidty is 0
                if pool.liquidity == 0 {
                    return Err(SwapError::NoInRangeLiquidity);
                }
            }

            let amount_out: U256 = big_uint_to_u256(exact_output_params.amount_out.0)
                .map_err(|_| SwapError::InvalidAmountOut)?;
            let amount_in_maximum = big_uint_to_u256(exact_output_params.amount_in_maximum.0)
                .map_err(|_| SwapError::InvalidAmountInMaximum)?;
            Ok(ValidatedSwapArgs::ExactOutput {
                path: swap_path,
                amount_out,
                amount_in_maximum,
            })
        }
    }
}
