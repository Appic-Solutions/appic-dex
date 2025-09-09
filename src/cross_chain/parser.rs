use std::{
    fmt::{Debug, Display, Formatter, LowerHex, UpperHex},
    str::FromStr,
};

use candid::Principal;
use minicbor::{Decode, Encode};
use serde::Deserialize;

use crate::{
    address::Address,
    cross_chain::{rlp_decoder::Blockchain, types::Recipient},
};

pub fn parse_recipient(
    recipient: String,
    destination_blockchain: Blockchain,
) -> Result<Recipient, String> {
    match destination_blockchain {
        Blockchain::ICP => Ok(Recipient::IcPrincipal(parse_principal(
            &FixedSizeData::from_str(&recipient)?,
        )?)),
        Blockchain::Evm(_) => Ok(Recipient::EvmAddress(parse_address(
            &FixedSizeData::from_str(&recipient)?,
        )?)),
    }
}

fn parse_address(address: &FixedSizeData) -> Result<Address, String> {
    Address::try_from(&address.0).map_err(|err| format!("Invalid address in log entry: {err}"))
}

fn parse_principal(principal: &FixedSizeData) -> Result<Principal, String> {
    parse_principal_from_slice(&principal.0)
}

#[derive(Clone, Deserialize, PartialOrd, Ord, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct FixedSizeData(pub [u8; 32]);

impl AsRef<[u8]> for FixedSizeData {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl std::str::FromStr for FixedSizeData {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.starts_with("0x") {
            return Err("Ethereum hex string doesn't start with 0x".to_string());
        }
        let mut bytes = [0u8; 32];
        hex::decode_to_slice(&s[2..], &mut bytes)
            .map_err(|e| format!("failed to decode hash from hex: {e}"))?;
        Ok(Self(bytes))
    }
}

impl Debug for FixedSizeData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:x}")
    }
}

impl Display for FixedSizeData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:x}")
    }
}

impl LowerHex for FixedSizeData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{}", hex::encode(self.0))
    }
}

impl UpperHex for FixedSizeData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{}", hex::encode_upper(self.0))
    }
}

/// Decode a candid::Principal from a slice of at most 32 bytes
/// encoded as follows
/// - the first byte is the number of bytes in the principal
/// - the next N bytes are the principal
/// - the remaining bytes are zero
///
/// Any other encoding will return an error.
/// Some specific valid [`Principal`]s are also not allowed
/// since the decoded principal will be used to receive twin tokens:
/// * the management canister principal
/// * the anonymous principal
///
/// This method MUST never panic (decode bytes from untrusted sources).
fn parse_principal_from_slice(slice: &[u8]) -> Result<Principal, String> {
    const ANONYMOUS_PRINCIPAL_BYTES: [u8; 1] = [4];

    if slice.is_empty() {
        return Err("slice too short".to_string());
    }
    if slice.len() > 32 {
        return Err(format!("Expected at most 32 bytes, got {}", slice.len()));
    }
    let num_bytes = slice[0] as usize;
    if num_bytes == 0 {
        return Err("management canister principal is not allowed".to_string());
    }
    if num_bytes > 29 {
        return Err(format!(
            "invalid number of bytes: expected a number in the range [1,29], got {num_bytes}",
        ));
    }
    if slice.len() < 1 + num_bytes {
        return Err("slice too short".to_string());
    }
    let (principal_bytes, trailing_zeroes) = slice[1..].split_at(num_bytes);
    if !trailing_zeroes
        .iter()
        .all(|trailing_zero| *trailing_zero == 0)
    {
        return Err("trailing non-zero bytes".to_string());
    }
    if principal_bytes == ANONYMOUS_PRINCIPAL_BYTES {
        return Err("anonymous principal is not allowed".to_string());
    }
    Principal::try_from_slice(principal_bytes).map_err(|err| err.to_string())
}
