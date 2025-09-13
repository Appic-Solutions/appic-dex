use minicbor::{Decode, Encode};

use crate::cross_chain::rlp_decoder::RlpDecodeError;

use super::*;

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
    InvalidToChain,
    LockedPrincipal,
    DepositError(DepositError),
    InvalidRecipient,
}
