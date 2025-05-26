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
    MintPotion {
        #[cbor(n(0), with = "crate::cbor::principal")]
        /// The sender of the token.
        sender: Principal,
        #[cbor(n(2), with = "crate::cbor::u256")]
        /// amount
        amount: U256,
    },
    /// The pool manager received funds to increase liquidity of a position.
    #[n(1)]
    IncreasePosition {
        #[cbor(n(0), with = "crate::cbor::principal")]
        /// The id corresponding to the withdrawal request.
        sender: Principal,
        #[cbor(n(2), with = "crate::cbor::u256")]
        /// amount
        amount: U256,
    },
    /// The pool manager received funds to be swapped,
    #[n(2)]
    SwapIn {
        #[cbor(n(0), with = "crate::cbor::principal")]
        /// The sender of the token.
        sender: Principal,
        #[cbor(n(2), with = "crate::cbor::u256")]
        /// amount
        amount: U256,
    },
    /// The user deposits funds,
    #[n(3)]
    Deposit {
        #[cbor(n(0), with = "crate::cbor::principal")]
        /// The sender of the token.
        sender: Principal,
        #[cbor(n(2), with = "crate::cbor::u256")]
        /// amount
        amount: U256,
    },
}
impl DepositMemo {
    pub fn set_amount(&mut self, new_amount: U256) {
        match self {
            DepositMemo::MintPotion { sender: _, amount } => *amount = new_amount,
            DepositMemo::IncreasePosition { sender: _, amount } => *amount = new_amount,
            DepositMemo::SwapIn { sender: _, amount } => *amount = new_amount,
            DepositMemo::Deposit { sender: _, amount } => *amount = new_amount,
        }
    }
}

impl From<DepositMemo> for Memo {
    fn from(value: DepositMemo) -> Self {
        Memo::from(encode(&value))
    }
}

#[derive(Decode, Encode, Debug, Eq, PartialEq, Clone)]
pub enum WithdrawalMemo {
    /// User received funds after position burnt.
    #[n(0)]
    BurnPotions {
        #[cbor(n(0), with = "crate::cbor::principal")]
        /// The receiver of the token.
        receiver: Principal,
        #[cbor(n(2), with = "crate::cbor::u256")]
        /// amount
        amount: U256,
    },
    /// The user received funds after decrease in liquidity of a position.
    #[n(1)]
    DecreasePosition {
        #[cbor(n(0), with = "crate::cbor::principal")]
        /// The receiver of the token.
        receiver: Principal,
        #[cbor(n(2), with = "crate::cbor::u256")]
        /// amount
        amount: U256,
    },
    /// The user received funds after swap,
    #[n(2)]
    SwapOut {
        #[cbor(n(0), with = "crate::cbor::principal")]
        /// The receiver of the token.
        receiver: Principal,
        #[cbor(n(2), with = "crate::cbor::u256")]
        /// amount
        amount: U256,
    },
    /// The user received funds from their internal pool manager balance,
    #[n(3)]
    WithdrawBalance {
        #[cbor(n(0), with = "crate::cbor::principal")]
        /// The receiver of the token.
        receiver: Principal,
        #[cbor(n(2), with = "crate::cbor::u256")]
        /// amount
        amount: U256,
    },
    #[n(4)]
    Refund {
        #[cbor(n(0), with = "crate::cbor::principal")]
        /// The receiver of the token.
        receiver: Principal,
        #[cbor(n(2), with = "crate::cbor::u256")]
        /// amount
        amount: U256,
    },
    #[n(5)]
    CollectFees {
        #[cbor(n(0), with = "crate::cbor::principal")]
        /// The receiver of the token.
        receiver: Principal,
        #[cbor(n(2), with = "crate::cbor::u256")]
        /// amount
        amount: U256,
    },
}

impl From<WithdrawalMemo> for Memo {
    fn from(value: WithdrawalMemo) -> Self {
        Memo::from(encode(&value))
    }
}

impl WithdrawalMemo {
    pub fn set_amount(&mut self, new_amount: U256) {
        match self {
            WithdrawalMemo::BurnPotions {
                receiver: _,
                amount,
            } => *amount = new_amount,
            WithdrawalMemo::DecreasePosition {
                receiver: _,
                amount,
            } => *amount = new_amount,
            WithdrawalMemo::SwapOut {
                receiver: _,
                amount,
            } => *amount = new_amount,
            WithdrawalMemo::WithdrawBalance {
                receiver: _,
                amount,
            } => *amount = new_amount,
            WithdrawalMemo::Refund {
                receiver: _,
                amount,
            } => *amount = new_amount,
            WithdrawalMemo::CollectFees {
                receiver: _,
                amount,
            } => *amount = new_amount,
        }
    }
}
