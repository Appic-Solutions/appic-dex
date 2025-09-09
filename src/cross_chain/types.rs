use candid::Principal;
use ethnum::U256;
use minicbor::{Decode, Encode};

use crate::{
    address::Address,
    cross_chain::rlp_decoder::{CrossChainQuote, CrossChainStep},
    minter_client::minter_types::{DexOrderArgs, MinterKey, ReceivedSwapOrderEvent},
    swap_id::SwapTxId,
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
        #[n(5)]
        from_minter: MinterKey,
        #[n(6)]
        to_minter: MinterKey,
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
        #[n(5)]
        from_minter: MinterKey,
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
        #[n(5)]
        minter: MinterKey,
        #[n(6)]
        to_minter: MinterKey,
    },
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct RecievedSwapOrders {
    #[n(0)]
    quote: CrossChainQuote,
    #[n(1)]
    minter_request: Option<ReceivedSwapOrderEvent>,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct FailedMinterTransferNotifies {
    #[n(0)]
    swap_tx_id: SwapTxId,
    #[cbor(n(1), with = "crate::cbor::principal")]
    transfer_token: Principal,
    #[cbor(n(2), with = "crate::cbor::u256")]
    transfer_amount: U256,
    #[cbor(n(3), with = "crate::cbor::principal")]
    minter_id: Principal,
    #[n(4)]
    dex_order_to_send: DexOrderArgs,
}
