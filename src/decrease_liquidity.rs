use candid::Principal;
use ethnum::I256;

use crate::{
    balances::types::{UserBalance, UserBalanceKey},
    candid_types::position::DecreaseLiquidityError,
    events::{Event, EventType},
    libraries::{balance_delta::BalanceDelta, slippage_check::validate_min_out},
    pool::{
        modify_liquidity::{modify_liquidity, ModifyLiquidityError, ModifyLiquidityParams},
        types::PoolId,
    },
    state::{mutate_state, read_state},
    validation::decrease_args::ValidatedDecreaseLiquidityArgs,
};

/// Executes the minting logic by computing liquidity and updating pool state.
pub fn execute_decrease_liquidity(
    caller: Principal,
    pool_id: PoolId,
    token0: Principal,
    token1: Principal,
    validated_args: ValidatedDecreaseLiquidityArgs,
    timestamp: u64,
) -> Result<BalanceDelta, DecreaseLiquidityError> {
    // Fetch pool state
    let pool =
        read_state(|s| s.get_pool(&pool_id)).ok_or(DecreaseLiquidityError::PoolNotInitialized)?;

    // Prepare and execute liquidity modification
    let modify_params = ModifyLiquidityParams {
        owner: caller,
        pool_id,
        tick_lower: validated_args.lower_tick,
        tick_upper: validated_args.upper_tick,
        liquidity_delta: validated_args.liquidity_delta,
        tick_spacing: pool.tick_spacing,
    };

    let success_result = modify_liquidity(modify_params).map_err(map_modify_liquidity_error)?;

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
    validate_min_out(
        success_result.balance_delta,
        validated_args.amount0_min,
        validated_args.amount1_min,
    )
    .map_err(|_| DecreaseLiquidityError::SlippageFailed)?;

    let final_balance = user_balance
        .add(success_result.fee_delta)
        .map_err(|_| DecreaseLiquidityError::FeeOverflow)?
        .add(success_result.balance_delta)
        .map_err(|_| DecreaseLiquidityError::AmountOverflow)?;

    // safe operation no overflow can happen since the balance_delta is always negative
    let amount0_received = success_result.balance_delta.amount0().abs().as_u256();
    let amount1_received = success_result.balance_delta.amount1().abs().as_u256();

    let event = Event {
        timestamp,
        payload: EventType::DecreasedLiquidity {
            modified_position: success_result.buffer_state.position.clone().unwrap().0,
            liquidity_delta: validated_args.liquidity_delta.abs() as u128,
            amount0_received,
            amount1_received,
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

    Ok(final_balance)
}

/// Maps ModifyLiquidityError to DecreaseLiquidityError.
fn map_modify_liquidity_error(error: ModifyLiquidityError) -> DecreaseLiquidityError {
    match error {
        ModifyLiquidityError::InvalidTick => DecreaseLiquidityError::InvalidTick,
        ModifyLiquidityError::TickNotAlignedWithTickSpacing => {
            panic!("Bug: Existing positions should have correct tick spacing")
        }
        ModifyLiquidityError::PoolNotInitialized => DecreaseLiquidityError::PoolNotInitialized,
        ModifyLiquidityError::LiquidityOverflow
        | ModifyLiquidityError::TickLiquidityOverflow
        | ModifyLiquidityError::PositionOverflow => DecreaseLiquidityError::LiquidityOverflow,
        ModifyLiquidityError::FeeOwedOverflow => DecreaseLiquidityError::FeeOverflow,
        ModifyLiquidityError::AmountDeltaOverflow => DecreaseLiquidityError::AmountOverflow,
        ModifyLiquidityError::InvalidTickSpacing | ModifyLiquidityError::ZeroLiquidityPosition => {
            panic!("Bug: Invalid tick spacing or zero liquidity in mint");
        }
    }
}
