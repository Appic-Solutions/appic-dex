use candid::Principal;
use ethnum::U256;
use minicbor::{Decode, Encode};

use crate::{
    address::Address, cross_chain::rlp_decoder::CrossChainStep, swap_id::SwapTxId,
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
        swap_tx_id: SwapTxId,
        #[n(1)]
        from_address: Address,
        #[n(2)]
        recipient: Recipient,
        #[n(3)]
        icp_swap_request: ValidatedSwapArgs,
        #[n(4)]
        evm_swap_step: CrossChainStep,
    },
    #[n(1)]
    EvmToIcp {
        #[n(0)]
        swap_tx_id: SwapTxId,
        #[n(1)]
        from_address: Address,
        #[n(2)]
        recipient: Recipient,
        #[cbor(n(3), with = "crate::cbor::u256")]
        amount_in: U256,
        #[n(4)]
        icp_swap_step: ValidatedSwapArgs,
    },
    #[n(2)]
    IcpToEvm {
        #[n(0)]
        swap_tx_id: SwapTxId,
        #[cbor(n(1), with = "crate::cbor::principal")]
        from: Principal,
        #[n(2)]
        recipient: Recipient,
        #[cbor(n(3), with = "crate::cbor::u256")]
        amount_in: U256,
        #[n(4)]
        evm_swap_step: CrossChainStep,
    },
}
