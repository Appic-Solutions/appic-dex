use std::fmt::Debug;

use candid::{CandidType, Deserialize, Nat, Principal};
use minicbor::{Decode, Encode};

use crate::{
    address::Address, candid_types::swap::SwapError, cross_chain::rlp_decoder::RlpDecodeError,
};

// Dex orders type to be sent to minter

#[derive(CandidType, Deserialize, Clone, Debug, Encode, Decode, Eq, PartialEq)]
pub struct DexOrderArgs {
    #[n(0)]
    pub tx_id: String,
    #[cbor(n(1), with = "crate::cbor::nat")]
    pub amount_in: Nat,
    #[cbor(n(2), with = "crate::cbor::nat")]
    pub min_amount_out: Nat,
    #[n(3)]
    pub commands: Vec<u8>,
    #[n(4)]
    pub commands_data: Vec<String>,
    #[n(5)]
    pub max_gas_fee_usd: Option<String>,
    #[n(6)]
    pub signing_fee: Option<String>,
    #[cbor(n(7), with = "crate::cbor::nat")]
    pub gas_limit: Nat,
    #[cbor(n(8), with = "crate::cbor::nat")]
    pub deadline: Nat,
    #[n(9)]
    pub recipient: String,
    #[cbor(n(10), with = "crate::cbor::nat")]
    pub erc20_ledger_burn_index: Nat,
    #[n(11)]
    pub is_refund: bool,
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq, Encode, Decode)]
pub enum DexOrderError {
    #[n(0)]
    InvalidAmount,
    #[n(1)]
    InvalidMinAmountIn,
    #[n(2)]
    TemporarilyUnavailable(#[n(0)] String),
    #[n(3)]
    InvalidMaxUsdFeeAmount(#[n(0)] String),
    #[n(4)]
    MaxUsdFeeTooLow,
    #[n(5)]
    UsdcAmountInTooLow,
    #[n(6)]
    InvalidCommand(#[n(0)] String),
    #[n(7)]
    InvalidCommandData(#[n(0)] String),
    #[n(8)]
    InvalidRecipient(#[n(0)] String),
    #[n(9)]
    InvalidGasLimit(#[n(0)] String),
    #[n(10)]
    InvalidDeadline(#[n(0)] String),
    #[n(11)]
    NotEnoughGasInGasTank {
        #[cbor(n(0), with = "crate::cbor::nat")]
        requested: Nat,
        #[cbor(n(1), with = "crate::cbor::nat")]
        available: Nat,
    },
}

// a fetched swap event from the swap contract logs
#[derive(CandidType, Deserialize, PartialEq, Eq, PartialOrd, Ord, Encode, Decode, Debug, Clone)]
pub struct ReceivedSwapOrderEvent {
    #[n(0)]
    pub from_address: String,
    #[n(1)]
    // recipient can be either an EVM address or an ICP principal id or an BTC address
    pub recipient: String,
    // token in on the initial evm swap
    #[n(2)]
    pub token_in: String,
    #[n(3)]
    pub token_out: String,

    #[cbor(n(4), with = "crate::cbor::nat")]
    // amount in on the initial swap
    pub amount_in: Nat,
    #[cbor(n(5), with = "crate::cbor::nat")]
    pub amount_out: Nat,

    #[n(6)]
    // the whole encoded swap transaction flow
    pub encoded_swap_data: String,

    #[n(7)]
    pub tx_id: String,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Encode, Decode, Debug, CandidType, Deserialize)]
pub struct MinterKey {
    #[n(0)]
    pub chain_id: u64,
    #[cbor(n(1), with = "crate::cbor::principal")]
    pub id: Principal,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Encode, Decode, Debug)]
pub struct Minter {
    #[cbor(n(0), with = "crate::cbor::principal")]
    pub twin_usdc_principal: Principal,
    #[n(1)]
    pub usdc_address: Address,
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
pub enum SwapOrderCreationError {
    InvalidMinter,
    InvalidAmountOut,
    InvalidFromAddress,
    InvalidOriginChain,
    InvalidToChain,
    InvalidOriginAndDestinationChain,
    FailedRlpDecoding,
    InvalidIcpSwapStep(SwapError),
    InvalidRecipient(String),
    InvalidRlpData(RlpDecodeError),
    InvalidTokenIn,
}
