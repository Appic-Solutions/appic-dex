use candid::Principal;
use ethnum::I256;

use crate::{
    balances::types::{UserBalance, UserBalanceKey},
    candid_types::position::IncreaseLiquidtyError,
    libraries::{balance_delta::BalanceDelta, slippage_check::validate_max_in},
    mint::calculate_liquidity,
    pool::{
        modify_liquidity::{modify_liquidity, ModifyLiquidityError, ModifyLiquidityParams},
        types::PoolId,
    },
    state::{mutate_state, read_state},
    validation::increase_args::ValidatedIncreaseLiquidityArgs,
};

/// Executes the minting logic by computing liquidity and updating pool state.
pub fn execute_increase_liquidty(
    caller: Principal,
    pool_id: PoolId,
    token0: Principal,
    token1: Principal,
    validated_args: ValidatedIncreaseLiquidityArgs,
) -> Result<u128, IncreaseLiquidtyError> {
    // Fetch pool state
    let pool =
        read_state(|s| s.get_pool(&pool_id)).ok_or(IncreaseLiquidtyError::PoolNotInitialized)?;

    // Compute liquidity for the position
    let liquidity_delta = calculate_liquidity(
        pool.sqrt_price_x96,
        validated_args.lower_tick,
        validated_args.upper_tick,
        validated_args.amount0_max,
        validated_args.amount1_max,
    )
    .map_err(|_| IncreaseLiquidtyError::LiquidityOverflow)?;

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
        user_balance.amount0().as_u256(),
        user_balance.amount1().as_u256(),
    )
    .unwrap();
    //.map_err(|_| IncreaseLiquidtyError::InsufficientBalance)?;

    let final_balance = user_balance
        .add(success_result.balance_delta)
        .map_err(|_| IncreaseLiquidtyError::AmountOverflow)?;

    // add generated fees
    let final_balance = final_balance
        .add(success_result.fee_delta)
        .map_err(|_| IncreaseLiquidtyError::AmountOverflow)?;

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
    });

    Ok(liquidity_delta as u128)
}

/// Maps ModifyLiquidityError to IncreaseLiquidtyError.
fn map_modify_liquidity_error(error: ModifyLiquidityError) -> IncreaseLiquidtyError {
    match error {
        ModifyLiquidityError::InvalidTick => IncreaseLiquidtyError::InvalidTick,
        ModifyLiquidityError::TickNotAlignedWithTickSpacing => {
            IncreaseLiquidtyError::TickNotAlignedWithTickSpacing
        }
        ModifyLiquidityError::PoolNotInitialized => IncreaseLiquidtyError::PoolNotInitialized,
        ModifyLiquidityError::LiquidityOverflow
        | ModifyLiquidityError::TickLiquidityOverflow
        | ModifyLiquidityError::PositionOverflow => IncreaseLiquidtyError::LiquidityOverflow,
        ModifyLiquidityError::FeeOwedOverflow => IncreaseLiquidtyError::FeeOverflow,
        ModifyLiquidityError::AmountDeltaOverflow => IncreaseLiquidtyError::AmountOverflow,
        ModifyLiquidityError::InvalidTickSpacing | ModifyLiquidityError::ZeroLiquidityPosition => {
            ic_cdk::trap("Bug: Invalid tick spacing or zero liquidity in mint");
        }
    }
}
