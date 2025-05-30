### Trading

- **swap**: Executes a token swap, supporting exact input/output and single/multi-hop paths.

  - **Args**: `SwapArgs { ExactInput: ExactInputParams, ExactOutput: ExactOutputParams, ExactInputSingle: ExactInputSingleParams, ExactOutputSingle: ExactOutputSingleParams }`

  - **Returns**: `Result_8 { Ok: CandidSwapSuccess, Err: SwapError }`

  - **Example (Exact Input Single)**:

    ```bash
    dfx canister call appic_dex swap '(variant { ExactInputSingle = record { zero_for_one = true; from_subaccount = null; amount_out_minimum = 100000 : nat; amount_in = 500000 : nat; pool_id = record { fee = 3000 : nat; token0 = principal "<token0_principal>"; token1 = principal "<token1_principal>" } } })'
    ```

  - **SuccessResult**

    ```candid
        type CandidSwapSuccess = record { amount_out : nat; amount_in : nat };
    ```

  - **Error**

    ```candid
        type SwapError = variant {
            InvalidAmountOut; // in exact output cases
            InvalidAmountIn; // in exact input cases
            DepositError : DepositError; // Failed to deposit tokens into dex
            InvalidAmountInMaximum; // in exact output cases
            InvalidAmountOutMinimum; // in exact input cases
            InvalidPoolFee; // fee not supported. supported fees (100, 500, 1000, 3000, 10000)
            PoolNotInitialized;
            PathLengthTooSmall : record { minimum : nat8; received : nat8 }; // in multi-hop swaps min swap length is 2
            PathLengthTooBig : record { maximum : nat8; received : nat8 }; // in multi-hop swaps max swap length is 4
            PathDuplicated; // in multi-hop swaps a single pool can not be repeated twice
            LockedPrincipal; // at the same time another active operation for the same principal exists
            NoInRangeLiquidity; // pool is illiquid
            FailedToWithdraw : record { // swap successful but withdraw failed
                amount_out : nat;
                amount_in : nat;
                reason : WithdrawError
            };
            SwapFailedRefunded : record { // swap failed and refunded
                refund_error : opt WithdrawError; // if presented, refund failed as well
                refund_amount : opt nat;
                failed_reason : SwapFailedReason
            }
        };
    ```

- **quote**: Estimates the output or input amount for a swap without executing it.

  - **Args**: `QuoteArgs { QuoteExactInputParams: QuoteExactParams, QuoteExactOutput: QuoteExactParams, QuoteExactInputSingleParams: QuoteExactSingleParams, QuoteExactOutputSingleParams: QuoteExactSingleParams }`

  - **Returns**: `Result_7 { Ok: nat, Err: QuoteError }`

  - **Example (Quote Exact Input Single)**:

    ```bash
    dfx canister call appic_dex quote '(variant { QuoteExactInputSingleParams = record { zero_for_one = true; pool_id = record { fee = 3000 : nat; token0 = principal "<token0_principal>"; token1 = principal "<token1_principal>" }; exact_amount = 500000 : nat } })'
    ```

  - **Error**

    ```candid
    type QuoteError = variant {
        InvalidAmount; // invalid amount in or out
        PoolNotInitialized;
        InvalidFee; // fee not supported. supported fees (100, 500, 1000, 3000, 10000)
        PriceLimitOutOfBounds;
        InvalidPathLength; // in multi-hop swaps min swap length is 2 and max is 4
        IlliquidPool;
        PriceLimitAlreadyExceeded;
        InvalidFeeForExactOutput;
        CalculationOverflow
    };
    ```
