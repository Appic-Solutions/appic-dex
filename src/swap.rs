use candid::Principal;
use ethnum::{I256, U256};

use crate::{
    balances::types::{UserBalance, UserBalanceKey},
    candid_types::swap::SwapFailedReason,
    events::{Event, EventType},
    pool::{
        swap::{swap_inner, SwapParams, SwapSuccess},
        types::PoolId,
    },
    quote::{get_sqrt_price_limit, select_amount},
    state::{mutate_state, read_state},
    validation::swap_args::ValidatedSwapArgs,
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SwapSuccessfulResult {
    amount_in: I256,                     // always <= 0
    amount_out: I256,                    // always >= 0
    token_out_transfer_fee: U256,        // used to know transfer fee at the time of withdrawal
    swap_success_list: Vec<SwapSuccess>, // contains buffer state for each hop
}

/// Executes a swap based on validated arguments, updating user balances and pool states.
/// Returns the positive input amount and output amount as
/// (amount_in,amount_out,token_out_transfer_fee).
/// Fails if balance, slippage, or swap conditions are not met.
pub fn execute_swap(
    validated_swap_args: &ValidatedSwapArgs,
    token_in: Principal,
    token_out: Principal,
    caller: Principal,
    timestamp: u64,
) -> Result<(I256, I256, U256), SwapFailedReason> {
    //  Initialize User Balance Keys
    let token_in_key = UserBalanceKey {
        token: token_in,
        user: caller,
    };
    let token_out_key = UserBalanceKey {
        token: token_out,
        user: caller,
    };

    //  Fetch Initial Balances
    let (token_in_balance_before, token_out_balance_before) =
        fetch_user_balances(&token_in_key, &token_out_key)?;

    // Execute Swap Based on Args
    let swap_result: SwapSuccessfulResult = match validated_swap_args {
        //  Single-Hop Exact Input
        ValidatedSwapArgs::ExactInputSingle {
            pool_id,
            zero_for_one,
            amount_in,
            amount_out_minimum,
            from_subaccount: _,
            token_in: _,
            token_out: _,
        } => {
            // Validate balance
            validate_balance(token_in_balance_before, *amount_in)?;

            // Build swap parameters
            let swap_params = build_swap_params(pool_id.clone(), -amount_in, *zero_for_one);

            // Execute swap
            let hop_result = swap_inner(swap_params).map_err(SwapFailedReason::from)?;

            // Calculate amounts
            let amount_out = select_amount(hop_result.swap_delta, *zero_for_one, false);
            let amount_in = *amount_in;

            // Check slippage
            check_exact_input_slippage(amount_out, *amount_out_minimum)?;

            SwapSuccessfulResult {
                amount_in,
                amount_out,
                token_out_transfer_fee: hop_result.token_out_transfer_fee,
                swap_success_list: vec![hop_result],
            }
        }
        //  Multi-Hop Exact Input
        ValidatedSwapArgs::ExactInput {
            path,
            amount_in,
            amount_out_minimum,
            from_subaccount: _,
            token_in: _,
            token_out: _,
        } => {
            // Validate balance
            validate_balance(token_in_balance_before, *amount_in)?;

            let mut current_amount = *amount_in;
            let mut swap_success_list = Vec::new();
            let mut token_out_transfer_fee = U256::ZERO;

            // Process each hop
            for swap in path {
                let swap_params =
                    build_swap_params(swap.pool_id.clone(), -current_amount, swap.zero_for_one);

                let hop_result = swap_inner(swap_params).map_err(SwapFailedReason::from)?;
                token_out_transfer_fee = hop_result.token_out_transfer_fee;
                current_amount = select_amount(hop_result.swap_delta, swap.zero_for_one, false);
                swap_success_list.push(hop_result);
            }

            // Final current_amount is the output amount
            let amount_out = current_amount;
            let amount_in = *amount_in;

            // Check slippage
            check_exact_input_slippage(amount_out, *amount_out_minimum)?;

            SwapSuccessfulResult {
                amount_in,
                amount_out,
                token_out_transfer_fee,
                swap_success_list,
            }
        }
        //  Single-Hop Exact Output
        ValidatedSwapArgs::ExactOutputSingle {
            pool_id,
            zero_for_one,
            amount_out,
            amount_in_maximum,
            from_subaccount: _,
            token_in: _,
            token_out: _,
        } => {
            // Validate balance
            validate_balance(token_in_balance_before, *amount_in_maximum)?;

            // Build swap parameters
            let swap_params = build_swap_params(pool_id.clone(), *amount_out, *zero_for_one);

            // Execute swap
            let hop_result = swap_inner(swap_params).map_err(SwapFailedReason::from)?;

            // Calculate amounts
            let amount_out = amount_out;
            let amount_in = -select_amount(hop_result.swap_delta, *zero_for_one, true);

            // Check slippage
            check_exact_output_slippage(amount_in, *amount_in_maximum)?;

            SwapSuccessfulResult {
                amount_in,
                amount_out: *amount_out,
                token_out_transfer_fee: hop_result.token_out_transfer_fee,
                swap_success_list: vec![hop_result],
            }
        }
        //  Multi-Hop Exact Output
        ValidatedSwapArgs::ExactOutput {
            path,
            amount_out,
            amount_in_maximum,
            from_subaccount: _,
            token_in: _,
            token_out: _,
        } => {
            // Validate balance
            validate_balance(token_in_balance_before, *amount_in_maximum)?;

            let mut current_amount = *amount_out;
            let mut swap_success_list = Vec::new();
            let mut token_out_transfer_fee = U256::ZERO;

            // Process each hop in reverse
            let mut i = 0;
            for swap in path.into_iter().rev() {
                let swap_direction = !swap.zero_for_one; // Reverse direction for exact output
                let swap_params =
                    build_swap_params(swap.pool_id.clone(), current_amount, swap_direction);

                ic_cdk::println!("swap params {:?}", swap_params);

                let hop_result = swap_inner(swap_params).map_err(SwapFailedReason::from)?;

                ic_cdk::println!("hop result {:?}", hop_result);

                if i == 0 {
                    token_out_transfer_fee = hop_result.token_out_transfer_fee;
                };
                current_amount = -select_amount(hop_result.swap_delta, swap_direction, true);

                ic_cdk::println!("current_amount {:?}", current_amount);
                swap_success_list.insert(0, hop_result);
                i += 1;
            }

            // Final current_amount is the input amount
            let amount_in = current_amount;
            let amount_out = amount_out;

            // Check slippage
            check_exact_output_slippage(amount_in, *amount_in_maximum)?;

            SwapSuccessfulResult {
                amount_in,
                token_out_transfer_fee,
                amount_out: *amount_out,
                swap_success_list,
            }
        }
    };

    let event = Event {
        timestamp,
        payload: EventType::Swap {
            final_amount_in: swap_result.amount_in.as_u256(),
            final_amount_out: swap_result.amount_out.as_u256(),
            swap_args: validated_swap_args.clone(),
            principal: caller,
        },
    };

    //  Update Balances and Pool States
    update_balances_and_states(
        token_in_key,
        token_out_key,
        token_in_balance_before,
        token_out_balance_before,
        &swap_result,
        event,
    )?;

    // Return positive input amount and output amount
    Ok((
        swap_result.amount_in,
        swap_result.amount_out,
        swap_result.token_out_transfer_fee,
    ))
}

//// Fetches user balances for input and output tokens.
fn fetch_user_balances(
    token_in_key: &UserBalanceKey,
    token_out_key: &UserBalanceKey,
) -> Result<(I256, I256), SwapFailedReason> {
    read_state(|s| {
        let in_balance = I256::try_from(s.get_user_balance(token_in_key).0)
            .map_err(|_| SwapFailedReason::BalanceOverflow)?;
        let out_balance = I256::try_from(s.get_user_balance(token_out_key).0)
            .map_err(|_| SwapFailedReason::BalanceOverflow)?;
        Ok((in_balance, out_balance))
    })
}

/// Validates that the user has sufficient balance for the swap.
fn validate_balance(balance: I256, required_amount: I256) -> Result<(), SwapFailedReason> {
    if balance < required_amount {
        Err(SwapFailedReason::InsufficientBalance)
    } else {
        Ok(())
    }
}

/// Builds swap parameters for a single hop.
fn build_swap_params(pool_id: PoolId, amount_specified: I256, swap_direction: bool) -> SwapParams {
    SwapParams {
        pool_id,
        amount_specified,
        zero_for_one: swap_direction,
        sqrt_price_limit_x96: get_sqrt_price_limit(swap_direction),
    }
}

/// Checks slippage for exact input swaps (amount_out >= minimum).
fn check_exact_input_slippage(
    amount_out: I256,
    amount_out_minimum: I256,
) -> Result<(), SwapFailedReason> {
    if amount_out < amount_out_minimum {
        Err(SwapFailedReason::TooLittleReceived)
    } else {
        Ok(())
    }
}

/// Checks slippage for exact output swaps (amount_in <= maximum).
fn check_exact_output_slippage(
    amount_in: I256,
    max_amount_in: I256,
) -> Result<(), SwapFailedReason> {
    if amount_in > max_amount_in {
        Err(SwapFailedReason::TooMuchRequested)
    } else {
        Ok(())
    }
}

/// Updates user balances and applies pool state changes.
fn update_balances_and_states(
    token_in_key: UserBalanceKey,
    token_out_key: UserBalanceKey,
    token_in_balance_before: I256,
    token_out_balance_before: I256,
    swap_result: &SwapSuccessfulResult,
    event: Event,
) -> Result<(), SwapFailedReason> {
    let token_in_balance_after = UserBalance(
        token_in_balance_before
            .checked_sub(swap_result.amount_in)
            .ok_or(SwapFailedReason::BalanceOverflow)?
            .as_u256(),
    );
    let token_out_balance_after = UserBalance(
        token_out_balance_before
            .checked_add(swap_result.amount_out)
            .ok_or(SwapFailedReason::BalanceOverflow)?
            .as_u256(),
    );

    mutate_state(|s| {
        s.update_user_balance(token_in_key, token_in_balance_after);
        s.update_user_balance(token_out_key, token_out_balance_after);
        for swap_success in &swap_result.swap_success_list {
            s.apply_swap_buffer_state(swap_success.buffer_state.clone());

            // add to accumulated protocol fee
            let fee_accumulated = s
                .get_protocol_fee_for_token(&swap_success.fee_token)
                .0
                .checked_add(swap_success.amount_to_protocol)
                .unwrap_or(U256::MAX);
            s.update_protocol_fee_for_token(swap_success.fee_token, UserBalance(fee_accumulated));
        }

        s.record_event(event);
    });

    Ok(())
}

/// Selects the appropriate token for (token_in, token_out) based on direction.
pub fn get_token_in_out(pool_id: &PoolId, zero_for_one: bool) -> (Principal, Principal) {
    if zero_for_one {
        (pool_id.token0, pool_id.token1)
    } else {
        (pool_id.token1, pool_id.token0)
    }
}
