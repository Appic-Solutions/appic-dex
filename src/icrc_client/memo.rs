use candid::Principal;
use ethnum::U256;
use icrc_ledger_types::icrc1::transfer::Memo;
use minicbor;
use minicbor::{Decode, Encode, Encoder};

/// Encodes minter memo as a binary blob.
fn encode<T: minicbor::Encode<()>>(t: &T) -> Vec<u8> {
    let mut encoder = Encoder::new(Vec::new());
    encoder.encode(t).expect("minicbor encoding failed");
    encoder.into_writer()
}

#[derive(Decode, Encode, Debug, Eq, PartialEq, Clone)]
pub enum DepositMemo {
    /// The pool manager received funds to mint a new position.
    #[n(0)]
    MintPosition {
        #[cbor(n(0), with = "crate::cbor::u256")]
        /// amount
        amount: U256,
    },
    /// The pool manager received funds to increase liquidity of a position.
    #[n(1)]
    IncreasePosition {
        #[cbor(n(0), with = "crate::cbor::u256")]
        /// amount
        amount: U256,
    },
    /// The pool manager received funds to be swapped,
    #[n(2)]
    SwapIn {
        #[cbor(n(0), with = "crate::cbor::u256")]
        /// amount
        amount: U256,
    },
    /// The user deposits funds,
    #[n(3)]
    Deposit {
        #[cbor(n(0), with = "crate::cbor::u256")]
        /// amount
        amount: U256,
    },
}
impl DepositMemo {
    pub fn set_amount(&mut self, new_amount: U256) {
        match self {
            DepositMemo::MintPosition { amount } => *amount = new_amount,
            DepositMemo::IncreasePosition { amount } => *amount = new_amount,
            DepositMemo::SwapIn { amount } => *amount = new_amount,
            DepositMemo::Deposit { amount } => *amount = new_amount,
        }
    }
}

impl From<DepositMemo> for Memo {
    fn from(value: DepositMemo) -> Self {
        Memo::from(encode(&value))
    }
}

#[derive(Decode, Encode, Debug, Eq, PartialEq, Clone)]
pub enum WithdrawMemo {
    /// User received funds after position burnt.
    #[n(0)]
    BurnPosition {
        #[cbor(n(0), with = "crate::cbor::u256")]
        /// amount
        amount: U256,
    },
    /// The user received funds after decrease in liquidity of a position.
    #[n(1)]
    DecreasePosition {
        #[cbor(n(0), with = "crate::cbor::u256")]
        /// amount
        amount: U256,
    },
    /// The user received funds after swap,
    #[n(2)]
    SwapOut {
        #[cbor(n(0), with = "crate::cbor::u256")]
        /// amount
        amount: U256,
    },
    /// The user received funds from their internal pool manager balance,
    #[n(3)]
    WithdrawBalance {
        #[cbor(n(0), with = "crate::cbor::u256")]
        /// amount
        amount: U256,
    },
    #[n(4)]
    Refund {
        #[cbor(n(0), with = "crate::cbor::u256")]
        /// amount
        amount: U256,
    },
    #[n(5)]
    CollectFees {
        #[cbor(n(0), with = "crate::cbor::u256")]
        /// amount
        amount: U256,
    },
    #[n(6)]
    Withdraw {
        #[cbor(n(0), with = "crate::cbor::u256")]
        /// amount
        amount: U256,
    },
}

impl From<WithdrawMemo> for Memo {
    fn from(value: WithdrawMemo) -> Self {
        Memo::from(encode(&value))
    }
}

impl WithdrawMemo {
    pub fn set_amount(&mut self, new_amount: U256) {
        match self {
            WithdrawMemo::BurnPosition { amount } => *amount = new_amount,
            WithdrawMemo::DecreasePosition { amount } => *amount = new_amount,
            WithdrawMemo::SwapOut { amount } => *amount = new_amount,
            WithdrawMemo::WithdrawBalance { amount } => *amount = new_amount,
            WithdrawMemo::Refund { amount } => *amount = new_amount,
            WithdrawMemo::CollectFees { amount } => *amount = new_amount,
            WithdrawMemo::Withdraw { amount } => *amount = new_amount,
        }
    }
}
