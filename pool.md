### Pool Management

- **create_pool**: Creates a new liquidity pool with specified tokens, fee, and initial price.

  - **Args**: `CreatePoolArgs { fee: nat, sqrt_price_x96: nat, token_a: principal, token_b: principal }`

  - **Returns**: `Result_2 { Ok: CandidPoolId, Err: CreatePoolError }`

  - **Example**:

    ```bash
    dfx canister call appic_dex create_pool '(record { fee = 3000 : nat; sqrt_price_x96 = 79228162514264337593543950336 : nat; token_a = principal "<token_a_principal>"; token_b = principal "<token_b_principal>" })'
    ```

  - **Errors**:

    ```candid
      type CreatePoolError = variant {
       InvalidSqrtPriceX96;
       InvalidFeeAmount; // fee not supported. supported fees (100, 500, 1000, 3000, 10000)
       DuplicatedTokens;
       InvalidToken : principal;
       PoolAlreadyExists;
    };
    ```

### Liquidity Management

- **mint_position**: Creates a new liquidity position in a pool within a specified price range.

  - **Args**: `MintPositionArgs { amount1_max: nat, pool: CandidPoolId, from_subaccount: opt blob, amount0_max: nat, tick_lower: int, tick_upper: int }`

  - **Returns**: `Result_6 { Ok: nat, Err: MintPositionError }`

  - **Example**:

    ```bash
    dfx canister call appic_dex mint_position '(record { amount1_max = 1000000 : nat; pool = record { fee = 3000 : nat; token0 = principal "<token0_principal>"; token1 = principal "<token1_principal>" }; from_subaccount = null; amount0_max = 500000 : nat; tick_lower = -1000 : int; tick_upper = 1000 : int })'
    ```

  - **Errors**:

    ```candid
      type MintPositionError = variant {
      DepositError : DepositError; // deposit failed
      TickNotAlignedWithTickSpacing; // ticks should be aligned with tick_spacing, tick % tick_spacing == 0
      InvalidAmount;
      InvalidPoolFee; // fee not supported. supported fees (100, 500, 1000, 3000, 10000)
      PoolNotInitialized;
      InsufficientBalance;
      LiquidityOverflow;
      FeeOverflow;
      PositionAlreadyExists;
      InvalidTick;
      LockedPrincipal; // at the same time another active operation for the same principal exists
      AmountOverflow; // math overflow
      SlippageFailed; // slippage check failed due to price movement
    };
    ```

- **increase_liquidity**: Adds liquidity to an existing position.

  - **Args**: `IncreaseLiquidityArgs { amount1_max: nat, pool: CandidPoolId, from_subaccount: opt blob, amount0_max: nat, tick_lower: int, tick_upper: int }`

  - **Returns**: `Result_5 { Ok: nat, Err: IncreaseLiquidity }`

  - **Example**:

    ```bash
    dfx canister call appic_dex increase_liquidity '(record { amount1_max = 500000 : nat; pool = record { fee = 3000 : nat; token0 = principal "<token0_principal>"; token1 = principal "<token1_principal>" }; from_subaccount = null; amount0_max = 250000 : nat; tick_lower = -1000 : int; tick_upper = 1000 : int })'
    ```

  - **Errors**:
    ```candid
      type MintPositionError = variant {
      DepositError : DepositError; // deposit failed
      TickNotAlignedWithTickSpacing; // ticks should be aligned with tick_spacing, tick % tick_spacing == 0
      InvalidAmount;
      InvalidPoolFee; // fee not supported. supported fees (100, 500, 1000, 3000, 10000)
      PoolNotInitialized;
      InsufficientBalance;
      LiquidityOverflow;
      FeeOverflow;
      PositionAlreadyExists;
      InvalidTick;
      LockedPrincipal; // at the same time another active operation for the same principal exists
      AmountOverflow; // math overflow
      SlippageFailed; // slippage check failed due to price movement
      DepositError : DepositError;
      PositionDoesNotExist;
    };
    ```

- **decrease_liquidity**: Removes liquidity from an existing position.

  - **Args**: `DecreaseLiquidityArgs { amount1_min: nat, pool: CandidPoolId, liquidity: nat, amount0_min: nat, tick_lower: int, tick_upper: int }`

  - **Returns**: `Result_3 { Ok, Err: DecreaseLiquidityError }`

  - **Example**:

    ```bash
    dfx canister call appic_dex decrease_liquidity '(record { amount1_min = 100000 : nat; pool = record { fee = 3000 : nat; token0 = principal "<token0_principal>"; token1 = principal "<token1_principal>" }; liquidity = 50000 : nat; amount0_min = 50000 : nat; tick_lower = -1000 : int; tick_upper = 1000 : int })'
    ```

  - **Errors**:
    ```candid
      type DecreaseLiquidityError = variant {
      InvalidAmount;
      InvalidPoolFee; // fee not supported. supported fees (100, 500, 1000, 3000, 10000)
      PoolNotInitialized;
      InsufficientBalance;
      LiquidityOverflow;
      FeeOverflow;
      InvalidTick;
      LockedPrincipal; // at the same time another active operation for the same principal exists
      AmountOverflow; // math overflow
      SlippageFailed; // slippage check failed due to price movement
      PositionNotFound;
      InvalidLiquidity;
      DecreasedPositionWithdrawalFailed : WithdrawError;
    };
    ```

- **burn**: Burns a liquidity position, removing it permanently.

  - **Args**: `BurnPositionArgs { amount1_min: nat, pool: CandidPoolId, amount0_min: nat, tick_lower: int, tick_upper: int }`

  - **Returns**: `Result { Ok, Err: BurnPositionError }`

  - **Example**:

    ```bash
    dfx canister call appic_dex burn '(record { amount1_min = 100000 : nat; pool = record { fee = 3000 : nat; token0 = principal "<token0_principal>"; token1 = principal "<token1_principal>" }; amount0_min = 50000 : nat; tick_lower = -1000 : int; tick_upper = 1000 : int })'
    ```

  - **Errors**:

    ```candid
      type DecreaseLiquidityError = variant {
      InvalidAmount;
      InvalidPoolFee; // fee not supported. supported fees (100, 500, 1000, 3000, 10000)
      PoolNotInitialized;
      InsufficientBalance;
      LiquidityOverflow;
      FeeOverflow;
      InvalidTick;
      LockedPrincipal; // at the same time another active operation for the same principal exists
      AmountOverflow; // math overflow
      SlippageFailed; // slippage check failed due to price movement
      PositionNotFound;
      InvalidLiquidity;
      BurntPositionWithdrawalFailed : WithdrawError;
    };
    ```

### Fee Collection

- **collect_fees**: Collects accumulated fees from a liquidity position.

  - **Args**: `CandidPositionKey { owner: principal, pool: CandidPoolId, tick_lower: int, tick_upper: int }`

  - **Returns**: `Result_1 { Ok: CollectFeesSuccess, Err: CollectFeesError }`

  - **Example**:

    ```bash
    dfx canister call appic_dex collect_fees '(record { owner = principal "<user_principal>"; pool = record { fee = 3000 : nat; token0 = principal "<token0_principal>"; token1 = principal "<token1_principal>" }; tick_lower = -1000 : int; tick_upper = 1000 : int })'
    ```

  - **Errors**:
    ```candid
      type CollectFeesError = variant {
        PositionNotFound;
        FeeOverflow;
        LockedPrincipal;
        CollectedFeesWithdrawalFailed : WithdrawError;
        NoFeeToCollect
      };
    ```
