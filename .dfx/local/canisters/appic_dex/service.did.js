export const idlFactory = ({ IDL }) => {
  const CandidPoolId = IDL.Record({
    'fee' : IDL.Nat,
    'token0' : IDL.Principal,
    'token1' : IDL.Principal,
  });
  const BurnPositionArgs = IDL.Record({
    'amount1_min' : IDL.Nat,
    'pool' : CandidPoolId,
    'amount0_min' : IDL.Nat,
    'tick_lower' : IDL.Int,
    'tick_upper' : IDL.Int,
  });
  const WithdrawError = IDL.Variant({
    'FeeUnknown' : IDL.Null,
    'TemporarilyUnavailable' : IDL.Text,
    'InvalidDestination' : IDL.Text,
    'InsufficientAllowance' : IDL.Record({ 'allowance' : IDL.Nat }),
    'InsufficientBalance' : IDL.Record({ 'balance' : IDL.Nat }),
    'AmountTooLow' : IDL.Record({ 'min_withdrawal_amount' : IDL.Nat }),
    'LockedPrincipal' : IDL.Null,
    'AmountOverflow' : IDL.Null,
  });
  const BurnPositionError = IDL.Variant({
    'PositionNotFound' : IDL.Null,
    'InvalidAmount' : IDL.Null,
    'InvalidPoolFee' : IDL.Null,
    'PoolNotInitialized' : IDL.Null,
    'InsufficientBalance' : IDL.Null,
    'LiquidityOverflow' : IDL.Null,
    'FeeOverflow' : IDL.Null,
    'BurntPositionWithdrawalFailed' : WithdrawError,
    'InvalidTick' : IDL.Null,
    'LockedPrincipal' : IDL.Null,
    'AmountOverflow' : IDL.Null,
  });
  const Result = IDL.Variant({ 'Ok' : IDL.Null, 'Err' : BurnPositionError });
  const CandidPositionKey = IDL.Record({
    'owner' : IDL.Principal,
    'pool' : CandidPoolId,
    'tick_lower' : IDL.Int,
    'tick_upper' : IDL.Int,
  });
  const CollectFeesSuccess = IDL.Record({
    'token0_collected' : IDL.Nat,
    'token1_collected' : IDL.Nat,
  });
  const CollectFeesError = IDL.Variant({
    'PositionNotFound' : IDL.Null,
    'FeeOverflow' : IDL.Null,
    'LockedPrincipal' : IDL.Null,
    'CollectedFeesWithdrawalFailed' : WithdrawError,
    'NoFeeToCollect' : IDL.Null,
  });
  const Result_1 = IDL.Variant({
    'Ok' : CollectFeesSuccess,
    'Err' : CollectFeesError,
  });
  const CreatePoolArgs = IDL.Record({
    'fee' : IDL.Nat,
    'sqrt_price_x96' : IDL.Nat,
    'token_a' : IDL.Principal,
    'token_b' : IDL.Principal,
  });
  const CreatePoolError = IDL.Variant({
    'InvalidSqrtPriceX96' : IDL.Null,
    'InvalidFeeAmount' : IDL.Null,
    'DuplicatedTokens' : IDL.Null,
    'InvalidToken' : IDL.Principal,
    'PoolAlreadyExists' : IDL.Null,
  });
  const Result_2 = IDL.Variant({
    'Ok' : CandidPoolId,
    'Err' : CreatePoolError,
  });
  const DecreaseLiquidityArgs = IDL.Record({
    'amount1_min' : IDL.Nat,
    'pool' : CandidPoolId,
    'liquidity' : IDL.Nat,
    'amount0_min' : IDL.Nat,
    'tick_lower' : IDL.Int,
    'tick_upper' : IDL.Int,
  });
  const DecreaseLiquidityError = IDL.Variant({
    'PositionNotFound' : IDL.Null,
    'InvalidAmount' : IDL.Null,
    'InvalidPoolFee' : IDL.Null,
    'PoolNotInitialized' : IDL.Null,
    'InsufficientBalance' : IDL.Null,
    'LiquidityOverflow' : IDL.Null,
    'FeeOverflow' : IDL.Null,
    'InvalidTick' : IDL.Null,
    'InvalidLiquidity' : IDL.Null,
    'LockedPrincipal' : IDL.Null,
    'AmountOverflow' : IDL.Null,
    'DecreasedPositionWithdrawalFailed' : WithdrawError,
  });
  const Result_3 = IDL.Variant({
    'Ok' : IDL.Null,
    'Err' : DecreaseLiquidityError,
  });
  const DepositArgs = IDL.Record({
    'token' : IDL.Principal,
    'from_subaccount' : IDL.Opt(IDL.Vec(IDL.Nat8)),
    'amount' : IDL.Nat,
  });
  const DepositError = IDL.Variant({
    'TemporarilyUnavailable' : IDL.Text,
    'InvalidDestination' : IDL.Text,
    'InsufficientAllowance' : IDL.Record({ 'allowance' : IDL.Nat }),
    'AmountTooLow' : IDL.Record({ 'min_withdrawal_amount' : IDL.Nat }),
    'LockedPrincipal' : IDL.Null,
    'AmountOverflow' : IDL.Null,
    'InsufficientFunds' : IDL.Record({ 'balance' : IDL.Nat }),
  });
  const Result_4 = IDL.Variant({ 'Ok' : IDL.Null, 'Err' : DepositError });
  const GetEventsArg = IDL.Record({
    'start' : IDL.Nat64,
    'length' : IDL.Nat64,
  });
  const SwapType = IDL.Variant({
    'ExactOutput' : IDL.Vec(CandidPoolId),
    'ExactInput' : IDL.Vec(CandidPoolId),
    'ExactOutputSingle' : CandidPoolId,
    'ExactInputSingle' : CandidPoolId,
  });
  const CandidEventType = IDL.Variant({
    'Swap' : IDL.Record({
      'principal' : IDL.Principal,
      'token_in' : IDL.Principal,
      'final_amount_in' : IDL.Nat,
      'final_amount_out' : IDL.Nat,
      'token_out' : IDL.Principal,
      'swap_type' : SwapType,
    }),
    'CreatedPool' : IDL.Record({
      'token0' : IDL.Principal,
      'token1' : IDL.Principal,
      'pool_fee' : IDL.Nat,
    }),
    'BurntPosition' : IDL.Record({
      'amount0_received' : IDL.Nat,
      'principal' : IDL.Principal,
      'burnt_position' : CandidPositionKey,
      'liquidity' : IDL.Nat,
      'amount1_received' : IDL.Nat,
    }),
    'IncreasedLiquidity' : IDL.Record({
      'principal' : IDL.Principal,
      'amount0_paid' : IDL.Nat,
      'liquidity_delta' : IDL.Nat,
      'amount1_paid' : IDL.Nat,
      'modified_position' : CandidPositionKey,
    }),
    'CollectedFees' : IDL.Record({
      'principal' : IDL.Principal,
      'amount1_collected' : IDL.Nat,
      'position' : CandidPositionKey,
      'amount0_collected' : IDL.Nat,
    }),
    'DecreasedLiquidity' : IDL.Record({
      'amount0_received' : IDL.Nat,
      'principal' : IDL.Principal,
      'liquidity_delta' : IDL.Nat,
      'amount1_received' : IDL.Nat,
      'modified_position' : CandidPositionKey,
    }),
    'MintedPosition' : IDL.Record({
      'principal' : IDL.Principal,
      'amount0_paid' : IDL.Nat,
      'liquidity' : IDL.Nat,
      'created_position' : CandidPositionKey,
      'amount1_paid' : IDL.Nat,
    }),
  });
  const CandidEvent = IDL.Record({
    'timestamp' : IDL.Nat64,
    'payload' : CandidEventType,
  });
  const GetEventsResult = IDL.Record({
    'total_event_count' : IDL.Nat64,
    'events' : IDL.Vec(CandidEvent),
  });
  const CandidPoolState = IDL.Record({
    'sqrt_price_x96' : IDL.Nat,
    'pool_reserves0' : IDL.Nat,
    'pool_reserves1' : IDL.Nat,
    'fee_protocol' : IDL.Nat,
    'token0_transfer_fee' : IDL.Nat,
    'swap_volume1_all_time' : IDL.Nat,
    'fee_growth_global_1_x128' : IDL.Nat,
    'tick' : IDL.Int,
    'liquidity' : IDL.Nat,
    'generated_swap_fee0' : IDL.Nat,
    'generated_swap_fee1' : IDL.Nat,
    'swap_volume0_all_time' : IDL.Nat,
    'fee_growth_global_0_x128' : IDL.Nat,
    'max_liquidity_per_tick' : IDL.Nat,
    'token1_transfer_fee' : IDL.Nat,
    'tick_spacing' : IDL.Int,
  });
  const CandidHistoryBucket = IDL.Record({
    'token0_reserves' : IDL.Nat,
    'end_timestamp' : IDL.Nat64,
    'swap_volume_token0_during_bucket' : IDL.Nat,
    'fee_generated_token1_during_bucket' : IDL.Nat,
    'fee_generated_token0_start' : IDL.Nat,
    'start_timestamp' : IDL.Nat64,
    'inrange_liquidity' : IDL.Nat,
    'fee_generated_token1_start' : IDL.Nat,
    'swap_volume_token0_start' : IDL.Nat,
    'swap_volume_token1_start' : IDL.Nat,
    'fee_generated_token0_during_bucket' : IDL.Nat,
    'last_sqrtx96_price' : IDL.Nat,
    'swap_volume_token1_during_bucket' : IDL.Nat,
    'token1_reserves' : IDL.Nat,
    'active_tick' : IDL.Int,
  });
  const CandidPoolHistory = IDL.Record({
    'hourly_frame' : IDL.Vec(CandidHistoryBucket),
    'monthly_frame' : IDL.Vec(CandidHistoryBucket),
    'yearly_frame' : IDL.Vec(CandidHistoryBucket),
    'daily_frame' : IDL.Vec(CandidHistoryBucket),
  });
  const CandidPositionInfo = IDL.Record({
    'fees_token0_owed' : IDL.Nat,
    'fee_growth_inside_1_last_x128' : IDL.Nat,
    'liquidity' : IDL.Nat,
    'fees_token1_owed' : IDL.Nat,
    'fee_growth_inside_0_last_x128' : IDL.Nat,
  });
  const IncreaseLiquidityArgs = IDL.Record({
    'amount1_max' : IDL.Nat,
    'pool' : CandidPoolId,
    'from_subaccount' : IDL.Opt(IDL.Vec(IDL.Nat8)),
    'amount0_max' : IDL.Nat,
    'tick_lower' : IDL.Int,
    'tick_upper' : IDL.Int,
  });
  const IncreaseLiquidity = IDL.Variant({
    'DepositError' : DepositError,
    'TickNotAlignedWithTickSpacing' : IDL.Null,
    'InvalidAmount' : IDL.Null,
    'InvalidPoolFee' : IDL.Null,
    'PoolNotInitialized' : IDL.Null,
    'InsufficientBalance' : IDL.Null,
    'LiquidityOverflow' : IDL.Null,
    'FeeOverflow' : IDL.Null,
    'InvalidTick' : IDL.Null,
    'PositionDoesNotExist' : IDL.Null,
    'LockedPrincipal' : IDL.Null,
    'AmountOverflow' : IDL.Null,
  });
  const Result_5 = IDL.Variant({ 'Ok' : IDL.Nat, 'Err' : IncreaseLiquidity });
  const MintPositionArgs = IDL.Record({
    'amount1_max' : IDL.Nat,
    'pool' : CandidPoolId,
    'from_subaccount' : IDL.Opt(IDL.Vec(IDL.Nat8)),
    'amount0_max' : IDL.Nat,
    'tick_lower' : IDL.Int,
    'tick_upper' : IDL.Int,
  });
  const MintPositionError = IDL.Variant({
    'DepositError' : DepositError,
    'TickNotAlignedWithTickSpacing' : IDL.Null,
    'InvalidAmount' : IDL.Null,
    'InvalidPoolFee' : IDL.Null,
    'PoolNotInitialized' : IDL.Null,
    'InsufficientBalance' : IDL.Null,
    'LiquidityOverflow' : IDL.Null,
    'FeeOverflow' : IDL.Null,
    'PositionAlreadyExists' : IDL.Null,
    'InvalidTick' : IDL.Null,
    'LockedPrincipal' : IDL.Null,
    'AmountOverflow' : IDL.Null,
  });
  const Result_6 = IDL.Variant({ 'Ok' : IDL.Nat, 'Err' : MintPositionError });
  const CandidPathKey = IDL.Record({
    'fee' : IDL.Nat,
    'intermediary_token' : IDL.Principal,
  });
  const QuoteExactParams = IDL.Record({
    'path' : IDL.Vec(CandidPathKey),
    'exact_token' : IDL.Principal,
    'exact_amount' : IDL.Nat,
  });
  const QuoteExactSingleParams = IDL.Record({
    'zero_for_one' : IDL.Bool,
    'pool_id' : CandidPoolId,
    'exact_amount' : IDL.Nat,
  });
  const QuoteArgs = IDL.Variant({
    'QuoteExactOutput' : QuoteExactParams,
    'QuoteExactOutputSingleParams' : QuoteExactSingleParams,
    'QuoteExactInputParams' : QuoteExactParams,
    'QuoteExactInputSingleParams' : QuoteExactSingleParams,
  });
  const QuoteError = IDL.Variant({
    'InvalidAmount' : IDL.Null,
    'PoolNotInitialized' : IDL.Null,
    'InvalidFee' : IDL.Null,
    'PriceLimitOutOfBounds' : IDL.Null,
    'InvalidPathLength' : IDL.Null,
    'IlliquidPool' : IDL.Null,
    'PriceLimitAlreadyExceeded' : IDL.Null,
    'InvalidFeeForExactOutput' : IDL.Null,
    'CalculationOverflow' : IDL.Null,
  });
  const Result_7 = IDL.Variant({ 'Ok' : IDL.Nat, 'Err' : QuoteError });
  const ExactOutputParams = IDL.Record({
    'amount_in_maximum' : IDL.Nat,
    'path' : IDL.Vec(CandidPathKey),
    'from_subaccount' : IDL.Opt(IDL.Vec(IDL.Nat8)),
    'amount_out' : IDL.Nat,
    'token_out' : IDL.Principal,
  });
  const ExactInputParams = IDL.Record({
    'token_in' : IDL.Principal,
    'path' : IDL.Vec(CandidPathKey),
    'from_subaccount' : IDL.Opt(IDL.Vec(IDL.Nat8)),
    'amount_out_minimum' : IDL.Nat,
    'amount_in' : IDL.Nat,
  });
  const ExactOutputSingleParams = IDL.Record({
    'amount_in_maximum' : IDL.Nat,
    'zero_for_one' : IDL.Bool,
    'from_subaccount' : IDL.Opt(IDL.Vec(IDL.Nat8)),
    'amount_out' : IDL.Nat,
    'pool_id' : CandidPoolId,
  });
  const ExactInputSingleParams = IDL.Record({
    'zero_for_one' : IDL.Bool,
    'from_subaccount' : IDL.Opt(IDL.Vec(IDL.Nat8)),
    'amount_out_minimum' : IDL.Nat,
    'amount_in' : IDL.Nat,
    'pool_id' : CandidPoolId,
  });
  const SwapArgs = IDL.Variant({
    'ExactOutput' : ExactOutputParams,
    'ExactInput' : ExactInputParams,
    'ExactOutputSingle' : ExactOutputSingleParams,
    'ExactInputSingle' : ExactInputSingleParams,
  });
  const CandidSwapSuccess = IDL.Record({
    'amount_out' : IDL.Nat,
    'amount_in' : IDL.Nat,
  });
  const SwapFailedReason = IDL.Variant({
    'TooMuchRequested' : IDL.Null,
    'InvalidAmount' : IDL.Null,
    'PoolNotInitialized' : IDL.Null,
    'InsufficientBalance' : IDL.Null,
    'PriceLimitOutOfBounds' : IDL.Null,
    'BalanceOverflow' : IDL.Null,
    'TooLittleReceived' : IDL.Null,
    'NoInRangeLiquidity' : IDL.Null,
    'PriceLimitAlreadyExceeded' : IDL.Null,
    'InvalidFeeForExactOutput' : IDL.Null,
    'CalculationOverflow' : IDL.Null,
  });
  const SwapError = IDL.Variant({
    'FailedToWithdraw' : IDL.Record({
      'amount_out' : IDL.Nat,
      'amount_in' : IDL.Nat,
      'reason' : WithdrawError,
    }),
    'InvalidAmountOut' : IDL.Null,
    'DepositError' : DepositError,
    'InvalidAmountIn' : IDL.Null,
    'InvalidAmountInMaximum' : IDL.Null,
    'InvalidAmountOutMinimum' : IDL.Null,
    'InvalidPoolFee' : IDL.Null,
    'PoolNotInitialized' : IDL.Null,
    'PathLengthTooSmall' : IDL.Record({
      'minimum' : IDL.Nat8,
      'received' : IDL.Nat8,
    }),
    'PathDuplicated' : IDL.Null,
    'PathLengthTooBig' : IDL.Record({
      'maximum' : IDL.Nat8,
      'received' : IDL.Nat8,
    }),
    'LockedPrincipal' : IDL.Null,
    'NoInRangeLiquidity' : IDL.Null,
    'SwapFailedRefunded' : IDL.Record({
      'refund_error' : IDL.Opt(WithdrawError),
      'refund_amount' : IDL.Opt(IDL.Nat),
      'failed_reason' : SwapFailedReason,
    }),
  });
  const Result_8 = IDL.Variant({ 'Ok' : CandidSwapSuccess, 'Err' : SwapError });
  const UserBalanceArgs = IDL.Record({
    'token' : IDL.Principal,
    'user' : IDL.Principal,
  });
  const Balance = IDL.Record({ 'token' : IDL.Principal, 'amount' : IDL.Nat });
  const Result_9 = IDL.Variant({ 'Ok' : IDL.Nat, 'Err' : WithdrawError });
  return IDL.Service({
    'burn' : IDL.Func([BurnPositionArgs], [Result], []),
    'collect_fees' : IDL.Func([CandidPositionKey], [Result_1], []),
    'create_pool' : IDL.Func([CreatePoolArgs], [Result_2], []),
    'decrease_liquidity' : IDL.Func([DecreaseLiquidityArgs], [Result_3], []),
    'deposit' : IDL.Func([DepositArgs], [Result_4], []),
    'get_events' : IDL.Func([GetEventsArg], [GetEventsResult], ['query']),
    'get_pool' : IDL.Func(
        [CandidPoolId],
        [IDL.Opt(CandidPoolState)],
        ['query'],
      ),
    'get_pool_history' : IDL.Func(
        [CandidPoolId],
        [IDL.Opt(CandidPoolHistory)],
        ['query'],
      ),
    'get_pools' : IDL.Func(
        [],
        [IDL.Vec(IDL.Tuple(CandidPoolId, CandidPoolState))],
        ['query'],
      ),
    'get_position' : IDL.Func(
        [CandidPositionKey],
        [IDL.Opt(CandidPositionInfo)],
        ['query'],
      ),
    'get_positions_by_owner' : IDL.Func(
        [IDL.Principal],
        [IDL.Vec(IDL.Tuple(CandidPositionKey, CandidPositionInfo))],
        ['query'],
      ),
    'increase_liquidity' : IDL.Func([IncreaseLiquidityArgs], [Result_5], []),
    'mint_position' : IDL.Func([MintPositionArgs], [Result_6], []),
    'quote' : IDL.Func([QuoteArgs], [Result_7], ['query']),
    'swap' : IDL.Func([SwapArgs], [Result_8], []),
    'user_balance' : IDL.Func([UserBalanceArgs], [IDL.Nat], ['query']),
    'user_balances' : IDL.Func([IDL.Principal], [IDL.Vec(Balance)], ['query']),
    'withdraw' : IDL.Func([Balance], [Result_9], []),
  });
};
export const init = ({ IDL }) => { return []; };
