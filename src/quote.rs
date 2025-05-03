use candid::Nat;
use ethnum::{I256, U256};

use crate::{
    candid_types::{
        pool::CandidPoolId,
        quote::{QuoteError, QuoteExactParams, QuoteExactSingleParams},
    },
    libraries::{
        balance_delta::BalanceDelta,
        constants::{MAX_SQRT_RATIO, MIN_SQRT_RATIO},
        path_key::PathKey,
        safe_cast::big_uint_to_u256,
    },
    pool::{
        swap::{swap_inner, SwapParams},
        types::PoolId,
    },
    validate_candid_args::{MAX_PATH_LENGTH, MIN_PATH_LENGTH},
};

/// Processes a single-hop exact input quote, calculating the output amount.
pub fn process_single_hop_exact_input(params: QuoteExactSingleParams) -> Result<U256, QuoteError> {
    let pool_id = validate_pool_id(params.pool_id)?;

    let exact_amount = convert_amount_to_i256(params.exact_amount)?;

    let swap_direction = params.zero_for_one;
    let sqrt_price_limit_x96 = get_sqrt_price_limit(swap_direction);
    let swap_params = SwapParams {
        pool_id,
        amount_specified: -exact_amount,
        zero_for_one: swap_direction,
        sqrt_price_limit_x96,
    };

    let swap_result = swap_inner(swap_params)?;

    let amount_out = select_amount(swap_result.swap_delta, swap_direction, false);
    Ok(amount_out.as_u256())
}

/// Processes a multi-hop exact input quote, iterating through the path.
pub fn process_multi_hop_exact_input(params: QuoteExactParams) -> Result<U256, QuoteError> {
    let path_length = params.path.len() as u8;
    if path_length < MIN_PATH_LENGTH || path_length > MAX_PATH_LENGTH {
        return Err(QuoteError::InvalidPathLength);
    }

    let mut input_token = params.exact_token;
    let mut input_amount = convert_amount_to_i256(params.exact_amount)?;

    for candid_path in params.path {
        let path_key = PathKey::try_from(candid_path).map_err(|_| QuoteError::InvalidFee)?;
        let swap = path_key.get_pool_and_swap_direction(input_token);

        let sqrt_price_limit_x96 = get_sqrt_price_limit(swap.zero_for_one);
        let swap_params = SwapParams {
            pool_id: swap.pool_id,
            amount_specified: -input_amount,
            zero_for_one: swap.zero_for_one,
            sqrt_price_limit_x96,
        };

        // Execute swap simulation
        let swap_result = swap_inner(swap_params)?;

        // Update amount and token for next hop
        input_amount = select_amount(swap_result.swap_delta, swap.zero_for_one, false);
        input_token = path_key.intermediary_token;
    }

    // Final input_amount is the output amount
    Ok(input_amount.as_u256())
}

/// Processes a single-hop exact output quote, calculating the input amount.
pub fn process_single_hop_exact_output(params: QuoteExactSingleParams) -> Result<U256, QuoteError> {
    let pool_id = validate_pool_id(params.pool_id)?;

    let exact_amount = convert_amount_to_i256(params.exact_amount)?;

    let swap_direction = params.zero_for_one;
    let sqrt_price_limit_x96 = get_sqrt_price_limit(swap_direction);
    let swap_params = SwapParams {
        pool_id,
        amount_specified: exact_amount,
        zero_for_one: swap_direction,
        sqrt_price_limit_x96,
    };

    let swap_result = swap_inner(swap_params)?;

    let amount_in = select_amount(swap_result.swap_delta, swap_direction, true);
    Ok((-amount_in).as_u256())
}

/// Processes a multi-hop exact output quote, iterating through the path in reverse.
pub fn process_multi_hop_exact_output(params: QuoteExactParams) -> Result<U256, QuoteError> {
    let path_length = params.path.len() as u8;
    if path_length < MIN_PATH_LENGTH || path_length > MAX_PATH_LENGTH {
        return Err(QuoteError::InvalidPathLength);
    }

    let mut output_token = params.exact_token;
    let mut output_amount = convert_amount_to_i256(params.exact_amount)?;

    for candid_path in params.path.into_iter().rev() {
        let path_key = PathKey::try_from(candid_path).map_err(|_| QuoteError::InvalidFee)?;
        let swap = path_key.get_pool_and_swap_direction(output_token);

        // Set swap parameters
        let one_for_zero = swap.zero_for_one;
        let sqrt_price_limit_x96 = get_sqrt_price_limit(swap.zero_for_one);
        let swap_params = SwapParams {
            pool_id: swap.pool_id,
            amount_specified: output_amount,
            zero_for_one: !one_for_zero,
            sqrt_price_limit_x96,
        };

        // Execute swap simulation
        let swap_result = swap_inner(swap_params)?;

        // Update amount and token for next hop
        output_amount = select_amount(swap_result.swap_delta, one_for_zero, true);
        output_amount = -output_amount;
        output_token = path_key.intermediary_token;
    }

    // Final current_amount is the input amount
    Ok(output_amount.as_u256())
}

fn validate_pool_id(pool_id: CandidPoolId) -> Result<PoolId, QuoteError> {
    pool_id
        .try_into()
        .map_err(|_| QuoteError::PoolNotInitialized)
}

/// Converts a `Nat` amount to `I256` safely.
fn convert_amount_to_i256(amount: Nat) -> Result<I256, QuoteError> {
    let u256_amount = big_uint_to_u256(amount.0).map_err(|_| QuoteError::InvalidAmount)?;
    u256_amount
        .try_into()
        .map_err(|_| QuoteError::InvalidAmount)
}

/// Determines the sqrt price limit based on swap direction.
fn get_sqrt_price_limit(zero_for_one: bool) -> U256 {
    if zero_for_one {
        *MIN_SQRT_RATIO + 1
    } else {
        *MAX_SQRT_RATIO - 1
    }
}

/// Selects the appropriate amount from swap delta based on direction and input/output.
fn select_amount(swap_delta: BalanceDelta, zero_for_one: bool, is_input: bool) -> I256 {
    match (zero_for_one, is_input) {
        (true, true) => swap_delta.amount0(),
        (true, false) => swap_delta.amount1(),
        (false, true) => swap_delta.amount1(),
        (false, false) => swap_delta.amount0(),
    }
}
