use ethnum::{I256, U256};
use num_bigint::BigUint;

pub fn big_uint_to_u256(biguint: BigUint) -> Result<U256, String> {
    let value_bytes = biguint.to_bytes_be();
    let mut value_u256 = [0u8; 32];
    if value_bytes.len() <= 32 {
        value_u256[32 - value_bytes.len()..].copy_from_slice(&value_bytes);
    } else {
        return Err(format!("does not fit in a U256: {}", biguint));
    }
    Ok(U256::from_be_bytes(value_u256))
}

pub fn big_uint_to_i256(biguint: BigUint) -> Result<I256, String> {
    let value_bytes = biguint.to_bytes_be();
    let mut value_u256 = [0u8; 32];
    if value_bytes.len() <= 32 {
        value_u256[32 - value_bytes.len()..].copy_from_slice(&value_bytes);
    } else {
        return Err(format!("does not fit in a U256: {}", biguint));
    }
    Ok(U256::from_be_bytes(value_u256)
        .try_into()
        .map_err(|_| String::from("does not fit in a I256"))?)
}

pub fn u256_to_big_uint(value: U256) -> BigUint {
    BigUint::from_bytes_be(&value.to_be_bytes())
}
