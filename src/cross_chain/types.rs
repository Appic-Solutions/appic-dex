use candid::{Nat, Principal};
use ethnum::U256;
use minicbor::{Decode, Encode};

use crate::{
    address::Address, cross_chain::rlp_decoder::CrossChainStep,
    minter_client::minter_types::MinterKey, swap_id::SwapTxId,
    validation::swap_args::ValidatedSwapArgs,
};

#[derive(Clone, Debug, PartialEq, Encode, Decode, Eq, PartialOrd, Ord)]
pub enum Recipient {
    #[n(0)]
    EvmAddress(#[n(0)] Address),
    #[n(1)]
    IcPrincipal(#[cbor(n(0), with = "crate::cbor::principal")] Principal),
}

// in all of these swap, the first step of the swap has already been executed no matter of the swap
// direction
#[derive(Encode, Decode, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum CrosschainSwapOrder {
    #[n(0)]
    EvmToEvm {
        #[n(0)]
        tx_id: SwapTxId,
        #[n(1)]
        from_address: Address,
        #[n(2)]
        recipient: Recipient,
        #[n(3)]
        icp_swap_request: ValidatedSwapArgs,
        #[n(4)]
        evm_swap_step: CrossChainStep,
        #[n(5)]
        from_minter: MinterKey,
        #[n(6)]
        to_minter: MinterKey,
    },
    #[n(1)]
    EvmToIcp {
        #[n(0)]
        tx_id: SwapTxId,
        #[n(1)]
        from_address: Address,
        #[n(2)]
        recipient: Recipient,
        #[n(4)]
        icp_swap_request: ValidatedSwapArgs,
        #[n(5)]
        from_minter: MinterKey,
    },
    #[n(2)]
    IcpToEvm {
        #[n(0)]
        tx_id: SwapTxId,
        #[cbor(n(1), with = "crate::cbor::principal")]
        from: Principal,
        #[n(2)]
        recipient: Recipient,
        #[n(4)]
        icp_swap_request: ValidatedSwapArgs,
        #[n(5)]
        evm_swap_step: CrossChainStep,
        #[n(6)]
        to_minter: MinterKey,
    },
}

impl CrosschainSwapOrder {
    pub fn tx_id(&self) -> SwapTxId {
        match self {
            CrosschainSwapOrder::EvmToEvm { tx_id, .. } => tx_id.clone(),
            CrosschainSwapOrder::EvmToIcp { tx_id, .. } => tx_id.clone(),
            CrosschainSwapOrder::IcpToEvm { tx_id, .. } => tx_id.clone(),
        }
    }
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct RetryFailedDexOrder {
    #[n(0)]
    pub tx_id: SwapTxId,
    #[cbor(n(1), with = "crate::cbor::principal")]
    pub token_in: Principal,
    #[cbor(n(2), with = "crate::cbor::principal")]
    pub minter_id: Principal,
    #[cbor(n(3), with = "crate::cbor::u256")]
    pub amount_in: U256,
    #[cbor(n(4), with = "crate::cbor::u256")]
    pub min_amount_out: U256,
    #[n(5)]
    pub commands: Vec<u8>,
    #[n(6)]
    pub commands_data: Vec<String>,
    #[n(7)]
    pub max_gas_fee_usd: Option<String>,
    #[cbor(n(8), with = "crate::cbor::u256")]
    pub gas_limit: U256,
    #[cbor(n(9), with = "crate::cbor::u256")]
    pub deadline: U256,
    #[n(10)]
    pub recipient: String,
    #[cbor(n(11), with = "crate::cbor::nat::option")]
    pub erc20_ledger_burn_index: Option<Nat>,
    #[n(12)]
    pub is_refund: bool,
    #[n(13)]
    pub signing_fee: Option<String>,
}

#[derive(Encode, Decode, Clone, Debug, Eq, PartialEq)]
pub struct SwapOrderStatus {
    #[n(0)]
    pub is_refund: bool,
    #[cbor(n(1), with = "crate::cbor::u256::option")]
    pub icp_amount_out: Option<U256>,
}
