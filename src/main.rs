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
    pool::modify_liquidity::{self, modify_liquidity, ModifyLiquidityError, ModifyLiquidityParams},
    state::{mutate_state, read_state},
    validate_candid_args::{self, validate_mint_position_args, ValidatedMintPosiotnArgs},
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
    // Principal Lock to be implemented
    let validated_args = validate_mint_position_args(args.clone())?;

    let caller = validate_caller_not_anonymous();

    // check user internal balance if there are enough funds, deduct from user balance
    // if not enough tokens trigger an on-chain transfer using ledger client
    let token0 = args.pool.token0;
    let token1 = args.pool.token1;

    let user_balalnce = read_state(|s| {
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
    let max_amounts = BalanceDelta::new(validated_args.amount0_max, validated_args.amount1_max);

    let balance_delta = max_amounts
        .sub(user_balalnce)
        .expect("Bug: Amounts are already checked so overflow or underflow should not happen");

    let mut from: Account = ic_cdk::caller().into();
    if let Some(subaccount) = args.from_subaccount {
        from.subaccount = Some(subaccount);
    };

    if balance_delta.amount0 > 0 {
        match LedgerClient::new(token0)
            .deposit(
                from,
                u256_to_big_uint(balance_delta.amount0().as_u256()),
                DepositMemo::MintPotion {
                    sender: caller,
                    amount: balance_delta.amount0.as_u256(),
                },
            )
            .await
        {
            Ok(_block_index) => mutate_state(|s| {
                s.update_user_balance(
                    UserBalanceKey {
                        user: caller,
                        token: token0,
                    },
                    UserBalance(max_amounts.amount0.as_u256()),
                )
            }),
            Err(transfer_error) => {
                return Err(MintPositionError::DepositError(transfer_error.into()))
            }
        };
    }

    if balance_delta.amount1 > 0 {
        match LedgerClient::new(token1)
            .deposit(
                from,
                u256_to_big_uint(balance_delta.amount1().as_u256()),
                DepositMemo::MintPotion {
                    sender: caller,
                    amount: balance_delta.amount1.as_u256(),
                },
            )
            .await
        {
            Ok(_block_index) => mutate_state(|s| {
                s.update_user_balance(
                    UserBalanceKey {
                        user: caller,
                        token: token1,
                    },
                    UserBalance(max_amounts.amount1.as_u256()),
                )
            }),
            Err(transfer_error) => {
                return Err(MintPositionError::DepositError(transfer_error.into()))
            }
        };
    }

    // Now we are sure that there is amount0_max and amount1_max available in the canister
    process_mint_potions(caller, token0, token1, validated_args)
}

fn process_mint_potions(
    caller: Principal,
    token0: Principal,
    token1: Principal,
    validated_args: ValidatedMintPosiotnArgs,
) -> Result<(), MintPositionError> {
    let pool = read_state(|s| s.get_pool(&validated_args.pool_id))
        .expect("Bug: Already validate, pool should be intialized by now");
    let sqrt_price_x96 = pool.sqrt_price_x96;
    let sqrt_price_a_x96 = tick_math::TickMath::get_sqrt_ratio_at_tick(validated_args.lower_tick);
    let sqrt_price_b_x96 = tick_math::TickMath::get_sqrt_ratio_at_tick(validated_args.upper_tick);

    let liquidity_delta = liquidity_amounts::get_liquidity_for_amounts(
        sqrt_price_x96,
        sqrt_price_a_x96,
        sqrt_price_b_x96,
        validated_args.amount0_max.as_u256(),
        validated_args.amount1_max.as_u256(),
    )
    .map_err(|_e| MintPositionError::LiquidityOverflow)?
    .to_i128()
    .ok_or(MintPositionError::LiquidityOverflow)?;

    let modify_liquidity_params = ModifyLiquidityParams {
        owner: caller,
        pool_id: validated_args.pool_id,
        tick_lower: validated_args.lower_tick,
        tick_upper: validated_args.upper_tick,
        liquidity_delta,
        tick_spacing: pool.tick_spacing,
    };

    match modify_liquidity(modify_liquidity_params) {
        Ok(success_result) => {
            let user_balalnce = read_state(|s| {
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

            let balance = user_balalnce
                .sub(success_result.balance_delta)
                .expect("Bug: deducting from balance should notfail at this point")
                .add(success_result.fee_delta)
                .map_err(|_e| MintPositionError::FeeOverflow)?;

            mutate_state(|s| {
                s.update_user_balance(
                    UserBalanceKey {
                        user: caller,
                        token: token0,
                    },
                    UserBalance(balance.amount0.as_u256()),
                )
            });
            mutate_state(|s| {
                s.update_user_balance(
                    UserBalanceKey {
                        user: caller,
                        token: token1,
                    },
                    UserBalance(balance.amount1.as_u256()),
                )
            });

            mutate_state(|s| s.apply_modify_liquidity_buffer_state(success_result.buffer_state));
        }
        Err(reason) => match reason {
            ModifyLiquidityError::InvalidTick => return Err(MintPositionError::InvalidTick),
            ModifyLiquidityError::TickNotAlignedWithTickSpacing => {
                return Err(MintPositionError::TickNotAlignedWithTickSpacing)
            }
            ModifyLiquidityError::PoolNotInitialized => {
                return Err(MintPositionError::PoolNotInitialized)
            }
            ModifyLiquidityError::LiquidityOverflow => {
                return Err(MintPositionError::LiquidityOverflow)
            }
            ModifyLiquidityError::TickLiquidityOverflow => {
                return Err(MintPositionError::LiquidityOverflow)
            }
            ModifyLiquidityError::PositionOverflow => {
                return Err(MintPositionError::LiquidityOverflow)
            }
            ModifyLiquidityError::FeeOwedOverflow => return Err(MintPositionError::FeeOverflow),
            ModifyLiquidityError::AmountDeltaOverflow => {
                return Err(MintPositionError::AmountOverflow)
            }
            ModifyLiquidityError::InvalidTickSpacing => {
                panic!("Bug: tick spacing should not be zero")
            }
            ModifyLiquidityError::ZeroLiquidityPosition => {
                panic!("Bug: In minting liquidity delta is always bigger than 0")
            }
        },
    };

    Ok(())
}
fn main() {
    println!("Hello, world!");
}
