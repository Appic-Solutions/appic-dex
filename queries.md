### Queries

- **get_pool**: Retrieves the state of a specific pool.

  - **Args**: `CandidPoolId { fee: nat, token0: principal, token1: principal }`

  - **Returns**: `opt CandidPoolState`

  - **Example**:

    ```bash
    dfx canister call appic_dex get_pool '(record { fee = 3000 : nat; token0 = principal "<token0_principal>"; token1 = principal "<token1_principal>" })'
    ```

- **get_pools**: Lists all pools with their states.

  - **Args**: `()`

  - **Returns**: `vec record { CandidPoolId; CandidPoolState }`

  - **Example**:

    ```bash
    dfx canister call appic_dex get_pools '()'
    ```

- **get_pool_history**: Retrieves historical data for a pool (hourly, daily, monthly, yearly).

  - **Args**: `CandidPoolId { fee: nat, token0: principal, token1: principal }`

  - **Returns**: `opt CandidPoolHistory`

  - **Example**:

    ```bash
    dfx canister call appic_dex get_pool_history '(record { fee = 3000 : nat; token0 = principal "<token0_principal>"; token1 = principal "<token1_principal>" })'
    ```

- **get_position**: Retrieves information about a specific liquidity position.

  - **Args**: `CandidPositionKey { owner: principal, pool: CandidPoolId, tick_lower: int, tick_upper: int }`

  - **Returns**: `opt CandidPositionInfo`

  - **Example**:

    ```bash
    dfx canister call appic_dex get_position '(record { owner = principal "<user_principal>"; pool = record { fee = 3000 : nat; token0 = principal "<token0_principal>"; token1 = principal "<token1_principal>" }; tick_lower = -1000 : int; tick_upper = 1000 : int })'
    ```

- **get_positions_by_owner**: Lists all positions for a specific user.

  - **Args**: `principal`

  - **Returns**: `vec record { CandidPositionKey; CandidPositionInfo }`

  - **Example**:

    ```bash
    dfx canister call appic_dex get_positions_by_owner '(principal "<user_principal>")'
    ```

- **get_events**: Retrieves a list of events (e.g., swaps, pool creation, liquidity changes) within a time range.

  - **Args**: `GetEventsArg { start: nat64, length: nat64 }`

  - **Returns**: `GetEventsResult { total_event_count: nat64, events: vec CandidEvent }`

  - **Example**:

    ```bash
    dfx canister call appic_dex get_events '(record { start = 0 : nat64; length = 100 : nat64 })'
    ```

- **user_balance**: Retrieves a user's balance for a specific token.

  - **Args**: `UserBalanceArgs { token: principal, user: principal }`

  - **Returns**: `nat`

  - **Example**:

    ```bash
    dfx canister call appic_dex user_balance '(record { token = principal "<token_principal>"; user = principal "<user_principal>" })'
    ```

- **user_balances**: Retrieves all token balances for a user.

  - **Args**: `principal`

  - **Returns**: `vec Balance`

  - **Example**:

    ```bash
    dfx canister call appic_dex user_balances '(principal "<user_principal>")'
    ```
