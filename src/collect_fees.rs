use candid::Principal;
use ethnum::I256;

use crate::{
    balances::types::{UserBalance, UserBalanceKey},
    candid_types::position::CollectFeesError,
    events::{Event, EventType},
    libraries::balance_delta::BalanceDelta,
    pool::{
        modify_liquidity::{modify_liquidity, ModifyLiquidityParams},
        types::PoolTickSpacing,
    },
    position::types::PositionKey,
    state::{mutate_state, read_state},
};

/// modifies the liquidity with zero delta liquidity to get only the fees owed to the user
/// #returns fee delta in case of success and CollectFeesError in case of failure
pub fn execute_collect_fees(
    caller: Principal,
    position_key: &PositionKey,
    tick_spacing: PoolTickSpacing,
) -> Result<BalanceDelta, CollectFeesError> {
    let modify_liquidity_params = ModifyLiquidityParams {
        owner: position_key.owner,
        pool_id: position_key.pool_id.clone(),
        tick_lower: position_key.tick_lower,
        tick_upper: position_key.tick_upper,
        liquidity_delta: 0,
        tick_spacing,
    };

    let success_result =
        modify_liquidity(modify_liquidity_params).map_err(|_| CollectFeesError::FeeOverflow)?;

    // Update user balances
    let user_balance = read_state(|s| {
        BalanceDelta::new(
            s.get_user_balance(&UserBalanceKey {
                user: caller,
                token: position_key.pool_id.token0,
            })
            .0
            .try_into()
            .unwrap_or(I256::MAX),
            s.get_user_balance(&UserBalanceKey {
                user: caller,
                token: position_key.pool_id.token1,
            })
            .0
            .try_into()
            .unwrap_or(I256::MAX),
        )
    });

    let final_balance = user_balance
        .add(success_result.fee_delta)
        .map_err(|_| CollectFeesError::FeeOverflow)?;

    // safe operation no overflow can happen since the balance_delta is always negative
    let amount0_collected = success_result.fee_delta.amount0().abs().as_u256();
    let amount1_collected = success_result.fee_delta.amount1().abs().as_u256();

    let event = Event {
        timestamp: ic_cdk::api::time(),
        payload: EventType::CollectedFees {
            position: position_key.clone(),
            amount0_collected,
            amount1_collected,
            principal: caller,
        },
    };
    //Batch state updates
    mutate_state(|s| {
        s.update_user_balance(
            UserBalanceKey {
                user: caller,
                token: position_key.pool_id.token0,
            },
            UserBalance(final_balance.amount0().as_u256()),
        );
        s.update_user_balance(
            UserBalanceKey {
                user: caller,
                token: position_key.pool_id.token1,
            },
            UserBalance(final_balance.amount1().as_u256()),
        );
        s.apply_modify_liquidity_buffer_state(success_result.buffer_state);

        s.record_event(event);
    });

    Ok(success_result.fee_delta)
}
