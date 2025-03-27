use candid::{Nat, Principal};
use ethnum::{u256, U256};
use minicbor::{Decode, Encode};
use proptest::collection::vec as pvec;
use proptest::prelude::*;

pub fn check_roundtrip<T>(v: &T) -> Result<(), TestCaseError>
where
    for<'a> T: PartialEq + std::fmt::Debug + Encode<()> + Decode<'a, ()>,
{
    let mut buf = vec![];
    minicbor::encode(v, &mut buf).expect("encoding should succeed");
    let decoded = minicbor::decode(&buf).expect("decoding should succeed");
    prop_assert_eq!(v, &decoded);
    Ok(())
}

#[derive(Debug, PartialEq, Eq, Encode, Decode)]
struct U256Container {
    #[cbor(n(0), with = "crate::cbor::u256")]
    pub value: u256,
}

#[derive(Debug, PartialEq, Eq, Encode, Decode)]
struct U128Container {
    #[cbor(n(0), with = "crate::cbor::u128")]
    pub value: u128,
}

#[derive(Debug, PartialEq, Eq, Encode, Decode)]
struct I128Container {
    #[cbor(n(0), with = "crate::cbor::i128")]
    pub value: i128,
}

#[derive(Debug, PartialEq, Eq, Encode, Decode)]
struct NatContainer {
    #[cbor(n(0), with = "crate::cbor::nat")]
    pub value: Nat,
}

#[derive(Debug, PartialEq, Eq, Encode, Decode)]
struct OptNatContainer {
    #[cbor(n(0), with = "crate::cbor::nat::option")]
    pub value: Option<Nat>,
}

#[derive(Debug, PartialEq, Eq, Encode, Decode)]
struct PrincipalContainer {
    #[cbor(n(0), with = "crate::cbor::principal")]
    pub value: Principal,
}

#[derive(Debug, PartialEq, Eq, Encode, Decode)]
struct OptPrincipalContainer {
    #[cbor(n(0), with = "crate::cbor::principal::option")]
    pub value: Option<Principal>,
}

proptest! {
    #[test]
    fn u256_encoding_roundtrip((hi, lo) in (any::<u128>(), any::<u128>())) {
        check_roundtrip(&U256Container {
            value: U256([hi, lo]),
        })?;
    }

    #[test]
    fn u256_small_value_encoding_roundtrip(n in any::<u64>()) {
        check_roundtrip(&U256Container {
            value: u256::from(n),
        })?;
    }

    #[test]
    fn u128_encoding_roundtrip(n in any::<u128>()) {
        check_roundtrip(&U128Container {
            value: n,
        })?;
    }

    #[test]
    fn u128_small_value_encoding_roundtrip(n in any::<u64>()) {
        check_roundtrip(&U128Container {
            value: n as u128,
        })?;
    }

     #[test]
    fn i128_encoding_roundtrip(n in any::<i128>()) {
        check_roundtrip(&I128Container {
            value: n,
        })?;
    }

    #[test]
    fn i128_small_value_encoding_roundtrip(n in any::<i64>()) {
        check_roundtrip(&I128Container {
            value: n as i128,
        })?;
    }



    #[test]
    fn nat_encoding_roundtrip(n in any::<u128>()) {
        check_roundtrip(&NatContainer {
            value: Nat::from(n),
        })?;
    }

    #[test]
    fn opt_nat_encoding_roundtrip(n in proptest::option::of(any::<u128>())) {
        check_roundtrip(&OptNatContainer {
            value: n.map(Nat::from),
        })?;
    }

    #[test]
    fn principal_encoding_roundtrip(p in pvec(any::<u8>(), 0..30)) {
        check_roundtrip(&PrincipalContainer {
            value: Principal::from_slice(&p),
        })?;
    }

    #[test]
    fn opt_principal_encoding_roundtrip(p in proptest::option::of(pvec(any::<u8>(), 0..30))) {
        check_roundtrip(&OptPrincipalContainer {
            value: p.map(|principal| Principal::from_slice(&principal)),
        })?;
    }
}
