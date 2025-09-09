use std::fmt::Debug;

use candid::{CandidType, Nat, Principal};
use minicbor::{Decode, Encode};
use serde::Deserialize;

use crate::address::Address;

// candid file designed for operations sent by appic dex
#[derive(CandidType, Deserialize, Clone, Debug, Encode, Decode, Eq, PartialEq, PartialOrd, Ord)]
pub enum DexOrderArgs {
    #[n(0)]
    Swap(#[n(0)] DexSwapOrderArgs),
    #[n(1)]
    Bridge(#[n(0)] DexBridgeOrderArgs),
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq, Eq, Encode, Decode, PartialOrd, Ord)]
pub struct DexSwapOrderArgs {
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
    pub max_gas_fee_usd: String,
    #[cbor(n(6), with = "crate::cbor::nat")]
    pub gas_limit: Nat,
    #[cbor(n(7), with = "crate::cbor::nat")]
    pub deadline: Nat,
    #[n(8)]
    pub recipient: String,
    #[cbor(n(9), with = "crate::cbor::nat")]
    pub erc20_ledger_burn_index: Nat,
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq, Eq, Encode, Decode, PartialOrd, Ord)]
pub struct DexBridgeOrderArgs {
    #[n(0)]
    pub tx_id: String,
    #[n(2)]
    pub recipient: String,
    #[cbor(n(3), with = "crate::cbor::nat")]
    pub amount: Nat,
    #[n(4)]
    pub max_gas_fee_usd: String,
    #[cbor(n(5), with = "crate::cbor::nat")]
    pub gas_limit: Nat,
    #[cbor(n(6), with = "crate::cbor::nat")]
    pub deadline: Nat,
    #[cbor(n(7), with = "crate::cbor::nat")]
    pub erc20_ledger_burn_index: Nat,
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
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Encode, Decode, Debug)]
pub struct ReceivedSwapOrderEvent {
    #[n(0)]
    pub transaction_hash: String,
    #[cbor(n(1), with = "crate::cbor::nat")]
    pub block_number: Nat,
    #[cbor(n(2), with = "crate::cbor::nat")]
    pub log_index: Nat,
    #[n(3)]
    pub from_address: String,
    #[n(4)]
    // recipient can be either an EVM address or an ICP principal id or an BTC address
    pub recipient: String,
    // token in on the initial evm swap
    #[n(5)]
    pub token_in: String,
    #[n(6)]
    pub token_out: String,
    #[cbor(n(7), with = "crate::cbor::nat")]
    // amount in on the initial swap
    pub amount_in: Nat,
    #[cbor(n(8), with = "crate::cbor::nat")]
    pub amount_out: Nat,
    #[n(9)]
    // specifies if funds were bridged to minter to initiate a corsschain swap or not
    pub bridged_to_minter: bool,
    #[n(10)]
    // the whole encoded swap transaction flow
    pub encoded_swap_data: String,

    #[n(11)]
    pub tx_swap_id: String,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Encode, Decode, Debug)]
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
