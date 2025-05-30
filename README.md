# Appic DEX - Decentralized Exchange on ICP

## Overview

Appic DEX is a decentralized exchange built in Rust, deployed on the Internet Computer (ICP). It enables trustless, permission-less trading of tokens with features like automated market maker (AMM) pools, liquidity provision, and fee collection. This project leverages the scalability and security of ICP to provide a fast and cost-efficient trading experience.

## Features

- **Pool Creation and Management**: Create and manage AMM pools with customizable fees and initial price settings.
- **Token Swapping**: Perform single or multi-hop swaps with exact input or output amounts, supporting flexible trading paths.
- **Quoting**: Performs quoting for single or multi-hop swaps in both exact input or exact output directions.
- **Liquidity Management**: Add, increase, decrease, or burn liquidity positions with precise control over price ranges (ticks).
- **Fee Collection**: Collect accumulated fees from liquidity positions in both tokens of a pool.
- **Comprehensive Queries**: Access pool states,, historical data, user balances, and position details for transparency and analysis.
- **Security**: Built with Rust for memory safety and designed with robust error handling for reliable operation.

## Prerequisites

To run or develop this DEX, ensure you have the following installed:

- Rust
- DFX (Internet Computer SDK, version 0.20.0 or later)
- Make (for simplified build and test process; see installation guide below)

## Installing Make

To use the `make build` and `make test` commands, install `make`:

### On macOS

```bash
brew install make
```

### On Ubuntu/Debian

```bash
sudo apt update && sudo apt install make
```

### Verify Installation

```bash
make --version
```

## Installation

1. **Clone the Repository**:

   ```bash
   git clone https://github.com/Appic-Solutions/appic-dex.git
   cd appic-dex
   ```

2. **Build the Project**: Run the following command to build the project:

   ```bash
   make build
   ```

3. **Start DFX**: Ensure DFX is running in the background:

   ```bash
   dfx start --background
   ```

4. **Deploy the DEX Canister**: Deploy the DEX canister using:

   ```bash
   dfx deploy appic_dex
   ```

5. **Interact with the DEX**: Use the Candid UI or a custom frontend to interact with the deployed canister. Access the canister ID from the deployment output.

## Testing

Run all tests (unit and integration) with:

```bash
make test
```

## Usage

1. [**Pool and Position management guide**](./pool.md)
2. [**Swapping guide**](./swap.md)
3. [**Dex Queries guide**](./queries.md)

## Project Structure

```
├── src/                          # Rust source code for the DEX logic
│   ├── balances/                 # Handles user balances and token accounting
│   ├── candid_types/             # Defines Candid types for canister interfaces
│   ├── cbor/                     # CBOR encoding/decoding for stable memory
│   ├── events/                   # Manages event logging
│   ├── guard/                    # Implements principal guard
│   ├── historical/               # Stores historical data
│   ├── icrc_client/              # Interacts with ICRC token standards on ICP
│   ├── libraries/                # Reusable libraries and utility functions
│   ├── pool/                     # Manages liquidity pools
│   ├── position/                 # Handles liquidity positions
│   ├── state/                    # Manages the canister's state
│   ├── tests/                    # Unit and integration tests
│   ├── tick/                     # Manages tick data for the AMM
│   ├── validation/               # Validation logic for various actions
│   ├── lib.rs                    # Library module
│   ├── logs.rs                   # Logging utilities
│   ├── main.rs                   # Main
│   ├── mint.rs                   # Mint position execution
│   ├── burn.rs                   # Burn position execution
│   ├── increase_liquidity.rs     # Increase liquidity on a position execution
│   ├── decrease_liquidity.rs     # Decrease liquidity on a position execution
│   ├── quote.rs                  # Quoting execution
│   └── swap.rs                   # Swap execution
├── Cargo.toml                    # Rust dependencies and configuration
├── dfx.json                      # DFX configuration for canister deployment
├── makefile                      # Makefile for simplified build and test process
└── appic_dex.did                 # Candid interface definition for the DEX canister
```

## Contributing

We welcome contributions! To contribute:

1. Fork the repository.
2. Create a feature branch (`git checkout -b feature/your-feature`).
3. Commit changes (`git commit -m 'Add your feature'`).
4. Push to the branch (`git push origin feature/your-feature`).
5. Open a pull request.

Please follow the Code of Conduct and ensure tests pass before submitting.

## Security

This project has been developed with following security best practices including [how to audit a canister](https://www.joachim-breitner.de/blog/788-How_to_audit_an_Internet_Computer_canister), [security best practices](https://internetcomputer.org/docs/building-apps/security/overview), [effective rust canisters](https://mmapped.blog/posts/01-effective-rust-canisters.html), and leveraging Rust’s memory safety guarantees. If you find vulnerabilities, please report them responsibly via tech@appicdao.com or open an issue.

## License

This project is licensed under the Apache-2.0 License. See the LICENSE file for details.

## Acknowledgments

- Built with Rust and Internet Computer.
- Thanks to the ICP community for all the amazing support.
