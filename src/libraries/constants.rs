use ethnum::U256;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref Q128: U256 = U256::from_words(1, 0); // 2^128;
    pub static ref Q96: U256 = U256::from(1u8) << 96; // 2^96 ;
}
