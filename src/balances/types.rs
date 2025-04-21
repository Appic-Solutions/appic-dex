use candid::Principal;
use ethnum::U256;
use minicbor::{Decode, Encode};

/// Used for storing X token balance of U user
#[derive(Encode, Decode, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct UserBalanceKey {
    #[cbor(n(0), with = "crate::cbor::principal")]
    pub user: Principal,
    #[cbor(n(1), with = "crate::cbor::principal")]
    pub token: Principal,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct UserBalance(#[cbor(n(0), with = "crate::cbor::u256")] U256);
