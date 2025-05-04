use ethnum::U256;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref Q128: U256 = U256::from_words(1, 0); // 2^128;
    pub static ref Q96: U256 = U256::from(1u8) << 96; // 2^96 ;
    pub static ref Q160: U256 = U256::from(1u8) << 160; // 2^160;
     pub static ref U160_MAX: U256 = (U256::from(1u8) << 160) - U256::ONE; // 2^160;



    pub static ref MIN_SQRT_RATIO: U256 = U256::from_str_radix("4295128739", 10).unwrap();
    pub static ref MAX_SQRT_RATIO: U256 =
        U256::from_str_radix("1461446703485210103287273052203988822378723970342", 10).unwrap();

    pub static ref DEFAULT_PROTOCOL_FEE: u16 = 0;
}

pub const MIN_TICK: i32 = -887272;
pub const MAX_TICK: i32 = 887272;
