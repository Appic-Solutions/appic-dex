use candid::Principal;
use ethnum::{I256, U256};

use crate::{
    libraries::{
        balance_delta::BalanceDelta,
        constants::{MAX_SQRT_RATIO, MAX_TICK, MIN_SQRT_RATIO, MIN_TICK, Q128},
        fee_math::{calculate_swap_fee, PIPS_DENOMINATOR},
        full_math::mul_div,
        liquidity_math,
        swap_math::{compute_swap_step, get_sqrt_price_target, MAX_SWAP_FEE},
        tick_bitmap::next_initialized_tick_within_one_word,
        tick_math::TickMath,
    },
    state::read_state,
    tick::{
        cross_tick,
        types::{TickInfo, TickKey},
    },
};

use super::types::{PoolId, PoolState};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct SwapParams {
    pub pool_id: PoolId,
    pub amount_specified: I256,
    pub zero_for_one: bool,
    pub sqrt_price_limit_x96: U256,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
pub struct StepComputations {
    // the price at the beginning of the step
    pub sqrt_price_start_x96: U256,
    // the next tick to swap to from the current tick in the swap direction
    pub tick_next: i32,
    // whether tickNext is initialized or not
    pub initialized: bool,
    // sqrt(price) for the next tick (1/0)
    pub sqrt_price_next_x96: U256,
    // how much is being swapped in in this step
    pub amount_in: U256,
    // how much is being swapped out
    pub amount_out: U256,
    // how much fee is being paid in
    pub fee_amount: U256,
    // the global fee growth of the input token. updated in storage at the end of swap
    pub fee_growth_global_x128: U256,
}

/// Keeps state changes, in case of success, state transition will be applied using this buffer
/// state, In case of failure no state transition will be triggered
/// Buffer for state changes to apply only on successful modification.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct SwapBufferState {
    pub pool: (PoolId, PoolState),
    pub shifted_ticks: Vec<(TickKey, TickInfo)>,
}

// Tracks the state of a pool throughout a swap, and returns these values at the end of the swap
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct SwapResult {
    // the current sqrt(price)
    pub sqrt_price_x96: U256,
    // the tick associated with the current price
    pub tick: i32,
    // the current liquidity in range
    pub liquidity: u128,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct SwapSuccess {
    pub swap_delta: BalanceDelta,
    pub token_out_transfer_fee: U256,
    pub amount_to_protocol: U256,
    pub fee_token: Principal,
    pub swap_fee: u32,
    pub total_swap_fee_amount: U256,
    pub buffer_state: SwapBufferState,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum InnerSwapError {
    PoolNotInitialized,
    IlliquidPool,
    InvalidFeeForExactOutput,
    PriceLimitAlreadyExceeded,
    PriceLimitOutOfBounds,
    CalculationOverflow,
}

/// Executes a swap against the pool, returning deltas and updated state.
/// Uses `SwapBufferState` to buffer changes, applied only on success to mimic Solidity's revert behavior on ICP.
pub fn swap_inner(params: SwapParams) -> Result<SwapSuccess, InnerSwapError> {
    let pool_state_initial =
        read_state(|s| s.get_pool(&params.pool_id)).ok_or(InnerSwapError::PoolNotInitialized)?;
    let tick_spacing = pool_state_initial.tick_spacing;

    let token_out_transfer_fee = if params.zero_for_one {
        pool_state_initial.token1_transfer_fee
    } else {
        pool_state_initial.token0_transfer_fee
    };

    let protocol_fee = pool_state_initial.fee_protocol;
    let swap_fee = calculate_swap_fee(protocol_fee, params.pool_id.fee.0);

    // A 100% fee (MAX_SWAP_FEE) consumes all input, making exact output swaps impossible.
    if swap_fee >= MAX_SWAP_FEE && params.amount_specified > 0 {
        return Err(InnerSwapError::InvalidFeeForExactOutput);
    }

    // protocol fee after the swap, initially set to 0
    let mut amount_to_protocol: U256 = U256::ZERO;

    let mut total_swap_fee_amount: U256 = U256::ZERO;

    let mut buffer_state = SwapBufferState {
        pool: (params.pool_id.clone(), pool_state_initial.clone()),
        shifted_ticks: vec![],
    };

    let fee_token = if params.zero_for_one {
        params.pool_id.token0
    } else {
        params.pool_id.token1
    };

    // swapFee is the pool's fee in pips (LP fee + protocol fee)
    // when the amount swapped is 0, there is no protocolFee applied and the fee amount paid to the protocol is set to 0
    if params.amount_specified == 0 {
        return Ok(SwapSuccess {
            swap_delta: BalanceDelta::ZERO_DELTA,
            token_out_transfer_fee,
            amount_to_protocol,
            swap_fee,
            buffer_state,
            fee_token,
            total_swap_fee_amount,
        });
    };

    validate_price_limits(
        params.zero_for_one,
        params.sqrt_price_limit_x96,
        pool_state_initial.sqrt_price_x96,
    )?;

    // Initialize Swap State

    // the amount remaining to be swapped in/out of the input/output asset. initially set to the amountSpecified
    let mut remaining_amount = params.amount_specified;
    // the amount swapped out/in of the output/input asset. initially set to 0
    let mut calculated_amount = I256::ZERO;
    let mut swap_result = SwapResult {
        sqrt_price_x96: pool_state_initial.sqrt_price_x96,
        tick: pool_state_initial.tick,
        liquidity: pool_state_initial.liquidity,
    };
    let mut step = StepComputations {
        fee_growth_global_x128: if params.zero_for_one {
            pool_state_initial.fee_growth_global_0_x128
        } else {
            pool_state_initial.fee_growth_global_1_x128
        },
        ..Default::default()
    };

    // continue swapping as long as we haven't used the entire input/output and haven't reached the price limit
    while !(remaining_amount == 0 || swap_result.sqrt_price_x96 == params.sqrt_price_limit_x96) {
        // Store the starting price for this step.
        step.sqrt_price_start_x96 = swap_result.sqrt_price_x96;

        // Find the next initialized tick in the swap direction.
        let (tick_next, initialized) = next_initialized_tick_within_one_word(
            &TickKey {
                pool_id: params.pool_id.clone(),
                tick: swap_result.tick,
            },
            tick_spacing.0,
            params.zero_for_one,
        );
        step.tick_next = clamp_tick(tick_next);
        step.initialized = initialized;

        // Get the sqrt price for the next tick.
        step.sqrt_price_next_x96 = TickMath::get_sqrt_ratio_at_tick(step.tick_next);

        // Compute swap step to reach the target tick, price limit, or exhaust the amount.
        let (next_sqrt_price, amount_in, amount_out, fee_amount) = compute_swap_step(
            swap_result.sqrt_price_x96,
            get_sqrt_price_target(
                params.zero_for_one,
                step.sqrt_price_next_x96,
                params.sqrt_price_limit_x96,
            )
            .expect("Prices should be between MIN_SQRT_RATIO and MAX_SQRT_RATIO"),
            swap_result.liquidity,
            remaining_amount,
            swap_fee,
        )
        .map_err(|_| InnerSwapError::CalculationOverflow)?;

        // Update step and swap result.
        swap_result.sqrt_price_x96 = next_sqrt_price;
        step.amount_in = amount_in;
        step.amount_out = amount_out;
        step.fee_amount = fee_amount;

        // Update remaining and calculated amounts based on swap direction.
        update_amounts(
            params.amount_specified > 0,
            &mut remaining_amount,
            &mut calculated_amount,
            &step,
        )?;

        // update total_swap_fee_amount
        total_swap_fee_amount += step.fee_amount;

        // Apply protocol fee if applicable.
        if protocol_fee > 0 {
            let delta = calculate_protocol_fee_delta(
                swap_fee,
                protocol_fee,
                step.amount_in,
                step.fee_amount,
            );
            step.fee_amount -= delta;
            amount_to_protocol += delta;
        }

        // Update global fee growth for non-zero liquidity.
        if swap_result.liquidity > 0 {
            let fee_growth_delta =
                mul_div(step.fee_amount, *Q128, U256::from(swap_result.liquidity))
                    .map_err(|_| InnerSwapError::CalculationOverflow)?;
            step.fee_growth_global_x128 = step
                .fee_growth_global_x128
                .checked_add(fee_growth_delta)
                .ok_or(InnerSwapError::CalculationOverflow)?;
        }

        // Shift tick if we reached the next price, and preemptively decrement for zeroForOne swaps to tickNext - 1.
        // If the swap doesn't continue (if amountRemaining == 0 or sqrtPriceLimit is met), slot0.tick will be 1 less
        // than getTickAtSqrtPrice(slot0.sqrtPrice). This doesn't affect swaps.
        if swap_result.sqrt_price_x96 == step.sqrt_price_next_x96 {
            // if the tick is initialized, run the tick transition
            if step.initialized {
                let (fee_growth_global_0_x128, fee_growth_global_1_x128) = if params.zero_for_one {
                    (
                        step.fee_growth_global_x128,
                        pool_state_initial.fee_growth_global_1_x128,
                    )
                } else {
                    (
                        pool_state_initial.fee_growth_global_0_x128,
                        step.fee_growth_global_x128,
                    )
                };

                let next_tick_key = TickKey {
                    pool_id: params.pool_id.clone(),
                    tick: step.tick_next,
                };

                // tick crossing
                let mut liquidity_net = {
                    match buffer_state
                        .shifted_ticks
                        .iter_mut()
                        .find(|(key, _info)| key == &next_tick_key)
                    {
                        Some((_key, info)) => {
                            cross_tick(info, fee_growth_global_0_x128, fee_growth_global_1_x128)
                        }
                        None => {
                            let mut next_tick_from_state =
                                read_state(|s| s.get_tick(&next_tick_key));
                            let liquidity_net = cross_tick(
                                &mut next_tick_from_state,
                                fee_growth_global_0_x128,
                                fee_growth_global_1_x128,
                            );
                            buffer_state
                                .shifted_ticks
                                .push((next_tick_key, next_tick_from_state.clone()));
                            liquidity_net
                        }
                    }
                };

                // if we're moving leftward, we interpret liquidityNet as the opposite sign
                // safe because liquidityNet cannot be i128::MIN
                if params.zero_for_one {
                    liquidity_net = -liquidity_net;
                }

                swap_result.liquidity =
                    liquidity_math::add_delta(swap_result.liquidity, liquidity_net)
                        .map_err(|_| InnerSwapError::CalculationOverflow)?;
            }
            swap_result.tick = if params.zero_for_one {
                step.tick_next - 1
            } else {
                step.tick_next
            };
        } else if swap_result.sqrt_price_x96 != step.sqrt_price_start_x96 {
            // recompute unless we're on a lower tick boundary (i.e. already transitioned ticks), and haven't moved
            swap_result.tick = TickMath::get_tick_at_sqrt_ratio(swap_result.sqrt_price_x96);
        }
    }

    // Compute Swap Delta
    let swap_delta = compute_swap_delta(
        params.zero_for_one,
        params.amount_specified,
        remaining_amount,
        calculated_amount,
    );

    // Update Buffered State
    update_buffer_state(
        &mut buffer_state,
        &swap_result,
        &pool_state_initial,
        &step,
        params.zero_for_one,
        &swap_delta,
        total_swap_fee_amount,
    );

    // check if pool is illiquid
    let actual_amount_specified = if params.zero_for_one == (params.amount_specified < 0) {
        swap_delta.amount0()
    } else {
        swap_delta.amount1()
    };

    if actual_amount_specified != params.amount_specified {
        return Err(InnerSwapError::IlliquidPool);
    }

    Ok(SwapSuccess {
        swap_delta,
        token_out_transfer_fee,
        amount_to_protocol,
        swap_fee,
        total_swap_fee_amount,
        buffer_state,
        fee_token,
    })
}

/// Validates price limits for the swap based on direction and pool state.
fn validate_price_limits(
    zero_for_one: bool,
    sqrt_price_limit_x96: U256,
    sqrt_price_current_x96: U256,
) -> Result<(), InnerSwapError> {
    if zero_for_one {
        if sqrt_price_limit_x96 >= sqrt_price_current_x96 {
            return Err(InnerSwapError::PriceLimitAlreadyExceeded);
        }
        if sqrt_price_limit_x96 <= *MIN_SQRT_RATIO {
            return Err(InnerSwapError::PriceLimitOutOfBounds);
        }
    } else {
        if sqrt_price_limit_x96 <= sqrt_price_current_x96 {
            return Err(InnerSwapError::PriceLimitAlreadyExceeded);
        }
        if sqrt_price_limit_x96 >= *MAX_SQRT_RATIO {
            return Err(InnerSwapError::PriceLimitOutOfBounds);
        }
    }
    Ok(())
}

/// Clamps the tick to ensure it stays within MIN_TICK and MAX_TICK bounds.
fn clamp_tick(tick: i32) -> i32 {
    if tick <= MIN_TICK {
        MIN_TICK
    } else if tick >= MAX_TICK {
        MAX_TICK
    } else {
        tick
    }
}

/// Updates remaining and calculated amounts based on whether the swap is exact output.
fn update_amounts(
    is_exact_output: bool,
    remaining_amount: &mut I256,
    calculated_amount: &mut I256,
    step: &StepComputations,
) -> Result<(), InnerSwapError> {
    if is_exact_output {
        *remaining_amount = remaining_amount.wrapping_sub(step.amount_out.as_i256());
        *calculated_amount = calculated_amount
            .checked_sub((step.amount_in + step.fee_amount).as_i256())
            .ok_or(InnerSwapError::CalculationOverflow)?;
    } else {
        *remaining_amount =
            remaining_amount.wrapping_add((step.amount_in + step.fee_amount).as_i256());
        *calculated_amount = calculated_amount
            .checked_add(step.amount_out.as_i256())
            .ok_or(InnerSwapError::CalculationOverflow)?;
    }
    Ok(())
}

/// step.amount_in does not include the swap fee, as it's already been taken from it,
/// so add it back to get the total amountIn and use that to calculate the amount of fees owed to the protocol
/// cannot overflow due to limits on the size of protocolFee and params.amountSpecified
/// this rounds down to favor LPs over the protocol
fn calculate_protocol_fee_delta(
    swap_fee: u32,
    protocol_fee: u16,
    amount_in: U256,
    fee_amount: U256,
) -> U256 {
    if swap_fee == protocol_fee as u32 {
        fee_amount // LP fee is 0, so entire fee goes to protocol.
    } else {
        (amount_in + fee_amount)
            .wrapping_mul(U256::from(protocol_fee))
            .wrapping_div(U256::from(PIPS_DENOMINATOR))
    }
}

/// Updates the buffer state with final swap results.
fn update_buffer_state(
    buffer_state: &mut SwapBufferState,
    swap_result: &SwapResult,
    pool_state_initial: &PoolState,
    step: &StepComputations,
    zero_for_one: bool,
    swap_delta: &BalanceDelta,
    total_swap_fee_amount: U256,
) {
    // update sqrt_price and tick
    buffer_state.pool.1.sqrt_price_x96 = swap_result.sqrt_price_x96;
    buffer_state.pool.1.tick = swap_result.tick;

    // update pool reserves
    let old_pool_reserves = BalanceDelta::new(
        buffer_state.pool.1.pool_reserve0.as_i256(),
        buffer_state.pool.1.pool_reserve1.as_i256(),
    );

    // amount in is negative and amount out is positive, so pool_reserves - swap_delta = new
    // pool reserves
    let pool_reserves_after = old_pool_reserves
        .sub(*swap_delta)
        .expect("Bug: this operation should be fail, since swap was successful");

    buffer_state.pool.1.pool_reserve0 = pool_reserves_after.amount0().as_u256();
    buffer_state.pool.1.pool_reserve1 = pool_reserves_after.amount1().as_u256();

    // update liqudity
    if pool_state_initial.liquidity != swap_result.liquidity {
        buffer_state.pool.1.liquidity = swap_result.liquidity;
    }

    // update fee_growth_global_x128, accumulated swap volume, and accumulated swap fee
    if zero_for_one {
        buffer_state.pool.1.fee_growth_global_0_x128 = step.fee_growth_global_x128;

        let generated_swap_fee0 = buffer_state
            .pool
            .1
            .generated_swap_fee0
            .checked_add(total_swap_fee_amount)
            .unwrap_or(U256::MAX);

        buffer_state.pool.1.generated_swap_fee0 = generated_swap_fee0;

        let swap_volume0_all_time = buffer_state
            .pool
            .1
            .swap_volume0_all_time
            .as_i256()
            .checked_sub(swap_delta.amount0())
            .unwrap_or(I256::MAX);
        buffer_state.pool.1.swap_volume0_all_time = swap_volume0_all_time.as_u256();
    } else {
        buffer_state.pool.1.fee_growth_global_1_x128 = step.fee_growth_global_x128;

        let generated_swap_fee1 = buffer_state
            .pool
            .1
            .generated_swap_fee1
            .checked_add(total_swap_fee_amount)
            .unwrap_or(U256::MAX);

        buffer_state.pool.1.generated_swap_fee1 = generated_swap_fee1;

        let swap_volume1_all_time = buffer_state
            .pool
            .1
            .swap_volume1_all_time
            .as_i256()
            .checked_sub(swap_delta.amount1())
            .unwrap_or(I256::MAX);
        buffer_state.pool.1.swap_volume1_all_time = swap_volume1_all_time.as_u256();
    }
}

/// Computes the final swap delta based on swap direction and amounts.
fn compute_swap_delta(
    zero_for_one: bool,
    amount_specified: I256,
    remaining_amount: I256,
    calculated_amount: I256,
) -> BalanceDelta {
    if zero_for_one != (amount_specified < 0) {
        BalanceDelta::new(
            calculated_amount,
            amount_specified.wrapping_sub(remaining_amount),
        )
    } else {
        BalanceDelta::new(
            amount_specified.wrapping_sub(remaining_amount),
            calculated_amount,
        )
    }
}
