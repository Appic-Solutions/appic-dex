use core::panic;

use appic_dex::{
    balances::types::{UserBalance, UserBalanceKey},
    endpoints::{CreatePoolArgs, CreatePoolError, MintPositionArgs, MintPositionError},
    icrc_client::{memo::DepositMemo, LedgerClient},
    libraries::{
        balance_delta::{self, BalanceDelta},
        liquidity_amounts,
        safe_cast::u256_to_big_uint,
        slippage_check::BalanceDeltaValidationError,
        tick_math::{self, TickMath},
    },
    mint::execute_mint_position,
    pool::modify_liquidity::{self, modify_liquidity, ModifyLiquidityError, ModifyLiquidityParams},
    state::{mutate_state, read_state},
    validate_candid_args::{self, validate_mint_position_args, ValidatedMintPositionArgs},
};

use candid::Principal;
use ethnum::{I256, U256};
use ic_cdk::{call, query, update};
use icrc_ledger_types::icrc1::account::Account;
use num_traits::ToPrimitive;

fn validate_caller_not_anonymous() -> candid::Principal {
    let principal = ic_cdk::caller();
    if principal == candid::Principal::anonymous() {
        panic!("anonymous principal is not allowed");
    }
    principal
}

//#[update]
//pub fn create_pool(
//    CreatePoolArgs {
//        token_a,
//        token_b,
//        fee,
//        sqrt_price_x96,
//    }: CreatePoolArgs,
//) -> Result<(), CreatePoolError> {
//    // sort token_a and b, token 0 is always the smaller token
//    let (token0, token1) = if token_a < token_b {
//        (token_a, token_b)
//    } else {
//        (token_b, token_a)
//    };
//
//    let fee = PoolFee::try_from(fee).map_err(|_e| CreatePoolError::InvalidFeeAmount)?;
//
//    let pool_id = PoolId {
//        token0,
//        token1,
//        fee: fee.clone(),
//    };
//
//    if read_state(|s| s.get_pool(&pool_id)).is_some() {
//        return Err(CreatePoolError::PoolAlreadyExists);
//    }
//    let tick_spacing =
//        read_state(|s| s.get_tick_spacing(&fee)).ok_or(CreatePoolError::InvalidFeeAmount)?;
//
//    let sqrt_price_x96 = U256::from_str_radix(&sqrt_price_x96.0.to_str_radix(10), 10)
//        .map_err(|_e| CreatePoolError::InvalidSqrtPriceX96)?;
//
//    let tick = TickMath::get_tick_at_sqrt_ratio(sqrt_price_x96)
//        .map_err(|_e| CreatePoolError::InvalidSqrtPriceX96)?;
//
//    let max_liquidity_per_tick = tick::tick_spacing_to_max_liquidity_per_tick(tick_spacing.0);
//    let pool_state = PoolState {
//        sqrt_price_x96,
//        tick,
//        fee_growth_global_0_x128: U256::ZERO,
//        fee_growth_global_1_x128: U256::ZERO,
//        liquidity: 0,
//        tick_spacing,
//        max_liquidity_per_tick,
//        fee_protocol: todo!(),
//    };
//    mutate_state(|s| s.set_pool(pool_id, pool_state));
//
//    return Ok(());
//}
//

#[update]
async fn mint(args: MintPositionArgs) -> Result<(), MintPositionError> {
    // TODO: Principal Lock to be implemented

    // Validate inputs and caller
    let validated_args = validate_mint_position_args(args.clone())?;
    let caller = validate_caller_not_anonymous();

    let pool_id = validated_args.pool_id.clone();
    let token0 = args.pool.token0;
    let token1 = args.pool.token1;
    let max_deposit = BalanceDelta::new(validated_args.amount0_max, validated_args.amount1_max);

    let user_balance = read_state(|s| {
        BalanceDelta::new(
            s.get_user_balance(&UserBalanceKey {
                user: caller,
                token: token0,
            })
            .0
            .try_into()
            .unwrap_or(I256::MAX), // Safe due to balance constraints
            s.get_user_balance(&UserBalanceKey {
                user: caller,
                token: token1,
            })
            .0
            .try_into()
            .unwrap_or(I256::MAX),
        )
    });

    // Check if additional deposits are needed
    let deposit_needed = max_deposit
        .sub(user_balance)
        .map_err(|_| MintPositionError::InsufficientBalance)?;

    // Prepare account for deposits
    let mut from: Account = caller.into();
    if let Some(subaccount) = args.from_subaccount {
        from.subaccount = Some(subaccount);
    }

    // Perform deposits if needed
    deposit_if_needed(
        caller,
        token0,
        deposit_needed.amount0,
        &from,
        max_deposit.amount0,
        &mut DepositMemo::MintPotion {
            sender: caller,
            amount: U256::ZERO,
        },
    )
    .await?;
    deposit_if_needed(
        caller,
        token1,
        deposit_needed.amount1,
        &from,
        max_deposit.amount1,
        &mut DepositMemo::MintPotion {
            sender: caller,
            amount: U256::ZERO,
        },
    )
    .await?;

    // Execute minting
    execute_mint_position(caller, pool_id, token0, token1, validated_args)
}

/// Deposits tokens if the required amount is positive, updating user balance on success.
async fn deposit_if_needed(
    caller: Principal,
    token: Principal,
    amount: I256,
    from: &Account,
    max_amount: I256,
    memo: &mut DepositMemo,
) -> Result<(), MintPositionError> {
    if amount > I256::ZERO {
        let deposit_amount = u256_to_big_uint(amount.as_u256());
        memo.set_amount(amount.as_u256());
        LedgerClient::new(token)
            .deposit(*from, deposit_amount, memo.clone())
            .await
            .map_err(|e| MintPositionError::DepositError(e.into()))?;

        mutate_state(|s| {
            s.update_user_balance(
                UserBalanceKey {
                    user: caller,
                    token,
                },
                UserBalance(max_amount.as_u256()),
            );
        });
    }
    Ok(())
}

fn main() {
    println!("Hello, world!");
}
