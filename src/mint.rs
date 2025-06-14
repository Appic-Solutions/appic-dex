use candid::Principal;
use ethnum::{I256, U256};
use num_traits::ToPrimitive;

use crate::{
    balances::types::{UserBalance, UserBalanceKey},
    candid_types::position::MintPositionError,
    events::{Event, EventType},
    libraries::{
        balance_delta::BalanceDelta, liquidity_amounts, slippage_check::validate_max_in,
        tick_math::TickMath,
    },
    pool::{
        modify_liquidity::{modify_liquidity, ModifyLiquidityError, ModifyLiquidityParams},
        types::PoolId,
    },
    state::{mutate_state, read_state},
    validation::mint_args::ValidatedMintPositionArgs,
};

/// Executes the minting logic by computing liquidity and updating pool state.
pub fn execute_mint_position(
    caller: Principal,
    pool_id: PoolId,
    token0: Principal,
    token1: Principal,
    validated_args: ValidatedMintPositionArgs,
    timestamp: u64,
) -> Result<u128, MintPositionError> {
    // Fetch pool state
    let pool = read_state(|s| s.get_pool(&pool_id)).ok_or(MintPositionError::PoolNotInitialized)?;

    // Compute liquidity for the position
    let liquidity_delta = calculate_liquidity(
        pool.sqrt_price_x96,
        validated_args.lower_tick,
        validated_args.upper_tick,
        validated_args.amount0_max,
        validated_args.amount1_max,
    )?;

    // Prepare and execute liquidity modification
    let modify_params = ModifyLiquidityParams {
        owner: caller,
        pool_id,
        tick_lower: validated_args.lower_tick,
        tick_upper: validated_args.upper_tick,
        liquidity_delta,
        tick_spacing: pool.tick_spacing,
    };

    let success_result = modify_liquidity(modify_params).map_err(map_modify_liquidity_error)?;

    println!("{:?}", success_result);

    // Update user balances
    let user_balance = read_state(|s| {
        BalanceDelta::new(
            s.get_user_balance(&UserBalanceKey {
                user: caller,
                token: token0,
            })
            .0
            .try_into()
            .unwrap_or(I256::MAX),
            s.get_user_balance(&UserBalanceKey {
                user: caller,
                token: token1,
            })
            .0
            .try_into()
            .unwrap_or(I256::MAX),
        )
    });

    // fee delta can be ignored as this is a new position
    validate_max_in(
        success_result.balance_delta,
        validated_args.amount0_max,
        validated_args.amount1_max,
    )
    .map_err(|_| MintPositionError::SlippageFailed)?;

    let final_balance = user_balance
        .add(success_result.balance_delta)
        .map_err(|_| MintPositionError::AmountOverflow)?;

    // safe operation no overflow can happen since the balance_delta is always negative
    let amount0_paid = success_result.balance_delta.amount0().abs().as_u256();
    let amount1_paid = success_result.balance_delta.amount1().abs().as_u256();

    let event = Event {
        timestamp,
        payload: EventType::MintedPosition {
            created_position: success_result.buffer_state.position.clone().unwrap().0,
            liquidity: liquidity_delta as u128,
            amount0_paid,
            amount1_paid,
            principal: caller,
        },
    };

    //Batch state updates
    mutate_state(|s| {
        s.update_user_balance(
            UserBalanceKey {
                user: caller,
                token: token0,
            },
            UserBalance(final_balance.amount0().as_u256()),
        );
        s.update_user_balance(
            UserBalanceKey {
                user: caller,
                token: token1,
            },
            UserBalance(final_balance.amount1().as_u256()),
        );

        s.apply_modify_liquidity_buffer_state(success_result.buffer_state);

        s.record_event(event);
    });

    Ok(liquidity_delta as u128)
}

/// Computes liquidity for the given amounts and tick range.
pub fn calculate_liquidity(
    sqrt_price_x96: U256,
    lower_tick: i32,
    upper_tick: i32,
    amount0_max: I256,
    amount1_max: I256,
) -> Result<i128, MintPositionError> {
    let sqrt_price_a_x96 = TickMath::get_sqrt_ratio_at_tick(lower_tick);
    let sqrt_price_b_x96 = TickMath::get_sqrt_ratio_at_tick(upper_tick);

    liquidity_amounts::get_liquidity_for_amounts(
        sqrt_price_x96,
        sqrt_price_a_x96,
        sqrt_price_b_x96,
        amount0_max.as_u256(),
        amount1_max.as_u256(),
    )
    .map_err(|_| MintPositionError::LiquidityOverflow)?
    .to_i128()
    .ok_or(MintPositionError::LiquidityOverflow)
}

/// Maps ModifyLiquidityError to MintPositionError.
fn map_modify_liquidity_error(error: ModifyLiquidityError) -> MintPositionError {
    match error {
        ModifyLiquidityError::InvalidTick => MintPositionError::InvalidTick,
        ModifyLiquidityError::TickNotAlignedWithTickSpacing => {
            MintPositionError::TickNotAlignedWithTickSpacing
        }
        ModifyLiquidityError::PoolNotInitialized => MintPositionError::PoolNotInitialized,
        ModifyLiquidityError::LiquidityOverflow
        | ModifyLiquidityError::TickLiquidityOverflow
        | ModifyLiquidityError::PositionOverflow => MintPositionError::LiquidityOverflow,
        ModifyLiquidityError::FeeOwedOverflow => MintPositionError::FeeOverflow,
        ModifyLiquidityError::AmountDeltaOverflow => MintPositionError::AmountOverflow,
        ModifyLiquidityError::InvalidTickSpacing | ModifyLiquidityError::ZeroLiquidityPosition => {
            ic_cdk::trap("Bug: Invalid tick spacing or zero liquidity in mint");
        }
    }
}
