use minicbor::{Decode, Encode};

use crate::{
    candid_types::{events::SwapType, pool::CandidPoolId},
    cross_chain::{
        rlp_decoder::{Blockchain, CrossChainStep, PoolHop, RlpDecodeError},
        types::{CrosschainSwapOrder, Recipient},
    },
    libraries::safe_cast::u256_to_nat,
    minter_client::minter_types::MinterKey,
    validation::swap_args::ValidatedSwapArgs,
};

use super::*;

#[derive(CandidType, Deserialize, PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
//the amount that was refunded to swapped out
pub enum CrosschainSwapStatus {
    Successful(Nat),
    Refunded,
    Pending,
}

// a fetched swap event from the swap contract logs
#[derive(CandidType, Deserialize, PartialEq, Eq, PartialOrd, Ord, Encode, Decode, Debug, Clone)]
pub struct CrosschainSwapArgs {
    // recipient can be either an EVM address or BTC address
    #[n(0)]
    pub recipient: String,
    #[n(2)]
    // the whole encoded swap transaction flow
    pub encoded_swap_data: String,
}

#[derive(CandidType, Deserialize, PartialEq, Eq, Debug, Clone)]
pub enum CrosschainSwapError {
    InvalidEncodedData(RlpDecodeError),
    InvalidIcpSwapStep,
    InvalidTokenIn,
    InvalidTokenOut,
    InvalidToChain,
    LockedPrincipal,
    DepositError(DepositError),
    InvalidRecipient,
}

#[derive(CandidType, Deserialize, PartialEq, Eq, Debug, Clone)]
pub enum CandidRecipient {
    EvmAddress(String),
    IcPrincipal(Principal),
}

#[derive(CandidType, Deserialize, Debug, Clone)]
pub enum CandidCrosschainSwapOrder {
    EvmToEvm {
        tx_id: String,
        from_address: String,
        recipient: CandidRecipient,
        icp_swap_request: SwapType,
        evm_swap_step: CandidCrosschainStep,
        from_minter: MinterKey,
        to_minter: MinterKey,
    },
    EvmToIcp {
        tx_id: String,
        from_address: String,
        recipient: CandidRecipient,
        icp_swap_request: SwapType,
        from_minter: MinterKey,
    },
    IcpToEvm {
        tx_id: String,
        from: Principal,
        recipient: CandidRecipient,
        icp_swap_request: SwapType,
        evm_swap_step: CandidCrosschainStep,
        to_minter: MinterKey,
    },
}

#[derive(CandidType, Deserialize, PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
pub struct CandidCrosschainStep {
    pub chain_id: Blockchain,
    pub amount_in: Nat,
    pub amount_out: Nat,
    pub min_amount_out: Option<Nat>,
    pub slippage: Option<String>,
    pub gas_limit: Option<Nat>,
    pub max_gas_fee: Option<Nat>,
    pub gas_price_usd: Option<String>,
    pub canister_fee_usd: Option<String>,
    pub route: Vec<PoolHop>,
}

impl From<ValidatedSwapArgs> for SwapType {
    fn from(value: ValidatedSwapArgs) -> Self {
        match value {
            ValidatedSwapArgs::ExactInputSingle {
                pool_id,
                zero_for_one: _,
                amount_in: _,
                amount_out_minimum: _,
                from_subaccount: _,
                token_in: _,
                token_out: _,
            } => SwapType::ExactInputSingle(pool_id.into()),
            ValidatedSwapArgs::ExactInput {
                path,
                amount_in: _,
                amount_out_minimum: _,
                from_subaccount: _,
                token_in: _,
                token_out: _,
            } => SwapType::ExactInput(
                path.into_iter()
                    .map(|swap| CandidPoolId::from(swap.pool_id))
                    .collect(),
            ),
            ValidatedSwapArgs::ExactOutputSingle {
                pool_id,
                zero_for_one: _,
                amount_out: _,
                amount_in_maximum: _,
                from_subaccount: _,
                token_in: _,
                token_out: _,
            } => SwapType::ExactOutputSingle(pool_id.into()),
            ValidatedSwapArgs::ExactOutput {
                path,
                amount_out: _,
                amount_in_maximum: _,
                from_subaccount: _,
                token_in: _,
                token_out: _,
            } => SwapType::ExactOutput(
                path.into_iter()
                    .map(|swap| CandidPoolId::from(swap.pool_id))
                    .collect(),
            ),
            ValidatedSwapArgs::NoSwapNeeded {
                token: _,
                amount: _,
            } => SwapType::NoSwapNeeded,
        }
    }
}

impl From<Recipient> for CandidRecipient {
    fn from(value: Recipient) -> Self {
        match value {
            Recipient::EvmAddress(address) => Self::EvmAddress(address.to_string()),
            Recipient::IcPrincipal(principal) => Self::IcPrincipal(principal),
        }
    }
}

impl From<CrossChainStep> for CandidCrosschainStep {
    fn from(value: CrossChainStep) -> Self {
        Self {
            chain_id: value.chain_id,
            amount_in: u256_to_nat(value.amount_in),
            amount_out: u256_to_nat(value.amount_out),
            min_amount_out: value.min_amount_out.map(u256_to_nat),
            slippage: value.slippage,
            gas_limit: value.gas_limit.map(u256_to_nat),
            max_gas_fee: value.max_gas_fee.map(u256_to_nat),
            gas_price_usd: value.gas_price_usd,
            canister_fee_usd: value.canister_fee_usd,
            route: value.route,
        }
    }
}

impl From<CrosschainSwapOrder> for CandidCrosschainSwapOrder {
    fn from(value: CrosschainSwapOrder) -> Self {
        match value {
            CrosschainSwapOrder::EvmToEvm {
                tx_id,
                from_address,
                recipient,
                icp_swap_request,
                evm_swap_step,
                from_minter,
                to_minter,
            } => CandidCrosschainSwapOrder::EvmToEvm {
                tx_id: tx_id.0,
                from_address: from_address.to_string(),
                recipient: recipient.into(),
                icp_swap_request: icp_swap_request.into(),
                evm_swap_step: evm_swap_step.into(),
                from_minter,
                to_minter,
            },
            CrosschainSwapOrder::EvmToIcp {
                tx_id,
                from_address,
                recipient,
                icp_swap_request,
                from_minter,
            } => CandidCrosschainSwapOrder::EvmToIcp {
                tx_id: tx_id.0,
                from_address: from_address.to_string(),
                recipient: recipient.into(),
                icp_swap_request: icp_swap_request.into(),
                from_minter,
            },
            CrosschainSwapOrder::IcpToEvm {
                tx_id,
                from,
                recipient,
                icp_swap_request,
                evm_swap_step,
                to_minter,
            } => CandidCrosschainSwapOrder::IcpToEvm {
                tx_id: tx_id.0,
                from,
                recipient: recipient.into(),
                icp_swap_request: icp_swap_request.into(),
                evm_swap_step: evm_swap_step.into(),
                to_minter,
            },
        }
    }
}
