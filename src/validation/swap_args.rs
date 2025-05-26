use std::collections::HashSet;

use candid::Principal;
use ethnum::I256;
use icrc_ledger_types::icrc1::account::Subaccount;

use crate::{
    candid_types::swap::{SwapArgs, SwapError},
    libraries::{
        path_key::{PathKey, Swap},
        safe_cast::big_uint_to_i256,
    },
    pool::types::PoolId,
    state::read_state,
    swap::get_token_in_out,
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ValidatedSwapArgs {
    ExactInputSingle {
        pool_id: PoolId,
        zero_for_one: bool,
        amount_in: I256,
        amount_out_minimum: I256,
        from_subaccount: Option<Subaccount>,
        token_in: Principal,
        token_out: Principal,
    },
    ExactInput {
        // order should be preserved
        path: Vec<Swap>,
        amount_in: I256,
        amount_out_minimum: I256,
        from_subaccount: Option<Subaccount>,
        token_in: Principal,
        token_out: Principal,
    },
    ExactOutputSingle {
        pool_id: PoolId,
        zero_for_one: bool,
        amount_out: I256,
        amount_in_maximum: I256,
        from_subaccount: Option<Subaccount>,
        token_in: Principal,
        token_out: Principal,
    },
    ExactOutput {
        // order should be preserved
        path: Vec<Swap>,
        amount_out: I256,
        amount_in_maximum: I256,
        from_subaccount: Option<Subaccount>,
        token_in: Principal,
        token_out: Principal,
    },
}

impl ValidatedSwapArgs {
    pub fn deposit_amount(&self) -> I256 {
        match self {
            ValidatedSwapArgs::ExactInputSingle { amount_in, .. } => *amount_in,
            ValidatedSwapArgs::ExactInput { amount_in, .. } => *amount_in,
            ValidatedSwapArgs::ExactOutputSingle {
                amount_in_maximum, ..
            } => *amount_in_maximum,
            ValidatedSwapArgs::ExactOutput {
                amount_in_maximum, ..
            } => *amount_in_maximum,
        }
    }
    pub fn token_in(&self) -> Principal {
        match self {
            ValidatedSwapArgs::ExactInputSingle { token_in, .. } => *token_in,
            ValidatedSwapArgs::ExactInput { token_in, .. } => *token_in,
            ValidatedSwapArgs::ExactOutputSingle { token_in, .. } => *token_in,
            ValidatedSwapArgs::ExactOutput { token_in, .. } => *token_in,
        }
    }

    pub fn token_out(&self) -> Principal {
        match self {
            ValidatedSwapArgs::ExactInputSingle { token_out, .. } => *token_out,
            ValidatedSwapArgs::ExactInput { token_out, .. } => *token_out,
            ValidatedSwapArgs::ExactOutputSingle { token_out, .. } => *token_out,
            ValidatedSwapArgs::ExactOutput { token_out, .. } => *token_out,
        }
    }

    pub fn from_subaccount(&self) -> Option<Subaccount> {
        match self {
            ValidatedSwapArgs::ExactInputSingle {
                from_subaccount, ..
            } => *from_subaccount,
            ValidatedSwapArgs::ExactInput {
                from_subaccount, ..
            } => *from_subaccount,
            ValidatedSwapArgs::ExactOutputSingle {
                from_subaccount, ..
            } => *from_subaccount,
            ValidatedSwapArgs::ExactOutput {
                from_subaccount, ..
            } => *from_subaccount,
        }
    }
}

// in multi hop swaps the maximum number of hops(swaps) should be <= MAX_PATH_LENGTH
pub const MAX_PATH_LENGTH: u8 = 4;

// in multi hop swaps the minimum number of hops(swaps) should be >= MIN_PATH_LENGTH
// if a swap has less than 1 hops, the swap is invalid
pub const MIN_PATH_LENGTH: u8 = 1;

pub fn validate_swap_args(args: SwapArgs) -> Result<ValidatedSwapArgs, SwapError> {
    match args {
        SwapArgs::ExactInputSingle(exact_input_single_params) => {
            let pool_id: PoolId = exact_input_single_params
                .pool_id
                .try_into()
                .map_err(|_| SwapError::InvalidPoolFee)?;

            let pool = read_state(|s| s.get_pool(&pool_id)).ok_or(SwapError::PoolNotInitialized)?;
            // In case in range liquidity is 0
            if pool.liquidity == 0 {
                return Err(SwapError::NoInRangeLiquidity);
            }

            let amount_in: I256 = big_uint_to_i256(exact_input_single_params.amount_in.0)
                .map_err(|_| SwapError::InvalidAmountIn)?;
            let amount_out_minimum =
                big_uint_to_i256(exact_input_single_params.amount_out_minimum.0)
                    .map_err(|_| SwapError::InvalidAmountOutMinimum)?;

            let (token_in, token_out) =
                get_token_in_out(&pool_id, exact_input_single_params.zero_for_one);

            Ok(ValidatedSwapArgs::ExactInputSingle {
                pool_id,
                zero_for_one: exact_input_single_params.zero_for_one,
                amount_in,
                amount_out_minimum,
                from_subaccount: exact_input_single_params.from_subaccount,
                token_in,
                token_out,
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
            for candid_path in exact_input_params.path.into_iter() {
                let path_key = PathKey::try_from(candid_path)?;
                swap_path.push(path_key.get_pool_and_swap_direction(token_in));
                token_in = path_key.intermediary_token;
            }
            // after the iteration token_in will be token_out
            let token_out = token_in;

            // there should not be a duplication in swap path, meaning users can not swap using the
            // same pool twice or more in a single swap transaction.
            if !all_unique(&swap_path) {
                return Err(SwapError::PathDuplicated);
            }

            // check pools
            for swap in swap_path.iter() {
                let pool = read_state(|s| s.get_pool(&swap.pool_id))
                    .ok_or(SwapError::PoolNotInitialized)?;
                // In case in range liquidity is 0
                if pool.liquidity == 0 {
                    return Err(SwapError::NoInRangeLiquidity);
                }
            }

            let amount_in = big_uint_to_i256(exact_input_params.amount_in.0)
                .map_err(|_| SwapError::InvalidAmountIn)?;
            let amount_out_minimum = big_uint_to_i256(exact_input_params.amount_out_minimum.0)
                .map_err(|_| SwapError::InvalidAmountOutMinimum)?;

            Ok(ValidatedSwapArgs::ExactInput {
                path: swap_path,
                amount_in,
                amount_out_minimum,
                from_subaccount: exact_input_params.from_subaccount,
                token_in: exact_input_params.token_in,
                token_out,
            })
        }
        SwapArgs::ExactOutputSingle(exact_output_single_params) => {
            let pool_id: PoolId = exact_output_single_params
                .pool_id
                .try_into()
                .map_err(|_| SwapError::InvalidPoolFee)?;

            let pool = read_state(|s| s.get_pool(&pool_id)).ok_or(SwapError::PoolNotInitialized)?;
            // In case in range liquidity is 0
            if pool.liquidity == 0 {
                return Err(SwapError::NoInRangeLiquidity);
            }

            let amount_out = big_uint_to_i256(exact_output_single_params.amount_out.0)
                .map_err(|_| SwapError::InvalidAmountIn)?;
            let amount_in_maximum =
                big_uint_to_i256(exact_output_single_params.amount_in_maximum.0)
                    .map_err(|_| SwapError::InvalidAmountInMaximum)?;

            let (token_in, token_out) =
                get_token_in_out(&pool_id, exact_output_single_params.zero_for_one);

            Ok(ValidatedSwapArgs::ExactOutputSingle {
                pool_id,
                zero_for_one: exact_output_single_params.zero_for_one,
                amount_out,
                amount_in_maximum,
                from_subaccount: exact_output_single_params.from_subaccount,
                token_in,
                token_out,
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
            for candid_path in exact_output_params.path.into_iter().rev() {
                let path_key = PathKey::try_from(candid_path)?;
                swap_path.push(path_key.get_pool_and_swap_direction(token_out));
                token_out = path_key.intermediary_token;
            }
            // after the last iteration token in will be token_out
            let token_in = token_out;

            // reverse the swap path
            // since we generated swap path using a reversed direction
            swap_path.reverse();

            if !all_unique(&swap_path) {
                return Err(SwapError::PathDuplicated);
            }

            // check pools
            for swap in swap_path.iter() {
                let pool = read_state(|s| s.get_pool(&swap.pool_id))
                    .ok_or(SwapError::PoolNotInitialized)?;
                // In case in range liquidity is 0
                if pool.liquidity == 0 {
                    return Err(SwapError::NoInRangeLiquidity);
                }
            }

            let amount_out = big_uint_to_i256(exact_output_params.amount_out.0)
                .map_err(|_| SwapError::InvalidAmountOut)?;
            let amount_in_maximum = big_uint_to_i256(exact_output_params.amount_in_maximum.0)
                .map_err(|_| SwapError::InvalidAmountInMaximum)?;

            Ok(ValidatedSwapArgs::ExactOutput {
                path: swap_path,
                amount_out,
                amount_in_maximum,
                from_subaccount: exact_output_params.from_subaccount,
                token_out,
                token_in,
            })
        }
    }
}

fn all_unique<T: Eq + std::hash::Hash>(vec: &[T]) -> bool {
    let set: HashSet<_> = vec.iter().collect();
    set.len() == vec.len()
}
