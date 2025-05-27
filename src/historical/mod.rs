// a module for keeping track of historical data in different time pre aggregated buckets to make
// it easier to show to useres in appic explorer
// Stored data include swap_volume, fee generation, price movement, and liquidty changes
// Time buckets include Ten minutes ,Hourly, Daily, Monthly, and Yearly

// to prevent storage overflow a mechanism automatically removes old data
// historical storage keeps at max 144 ten minute buckets, 72 hourly bucket, 90 daily bucket, 36 monthly bucket, and 12 yealry
// buckets

use ethnum::U256;
use minicbor::{Decode, Encode};

#[derive(Encode, Decode, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct HistoryBucket {
    #[cbor(n(0), with = "crate::cbor::u128")]
    pub time_stamp: u128,
    #[cbor(n(2), with = "crate::cbor::u256")]
    pub swap_volume_token0: U256,
    #[cbor(n(3), with = "crate::cbor::u256")]
    pub swap_volume_token1: U256,
    #[cbor(n(4), with = "crate::cbor::u256")]
    pub fee_generated_token0: U256,
    #[cbor(n(5), with = "crate::cbor::u256")]
    pub fee_generated_token1: U256,
    #[cbor(n(6), with = "crate::cbor::u256")]
    pub token0_reserves: U256,
    #[cbor(n(7), with = "crate::cbor::u256")]
    pub token1_reserves: U256,
    #[cbor(n(8), with = "crate::cbor::u256")]
    pub sqrtx96_price: U256,
    #[cbor(n(9), with = "crate::cbor::u256")]
    pub inrange_liquidty: U256,
    #[n(10)]
    pub active_tick: i32,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct PoolHistory {
    #[n(0)]
    ten_minute_frame: Vec<HistoryBucket>,
    #[n(1)]
    hourly_frame: Vec<HistoryBucket>,
    #[n(2)]
    daily_frame: Vec<HistoryBucket>,
    #[n(3)]
    monthly_frame: Vec<HistoryBucket>,
    #[n(4)]
    yearly_frame: Vec<HistoryBucket>,
}

pub enum TimeBucket {
    TenMinute,
    Hourly,
    Daily,
    Monthly,
    Yealry,
}

// Align timestamp to bucket start
fn align_timestamp_to_bucket(timestamp: u64, bucket: &TimeBucket) -> u64 {
    match bucket {
        TimeBucket::TenMinute => timestamp - (timestamp % 600),
        TimeBucket::Hourly => timestamp - (timestamp % 3600),
        TimeBucket::Daily => timestamp - (timestamp % 86400),
        TimeBucket::Monthly => timestamp - (timestamp % (86400 * 30)),
        TimeBucket::Yealry => timestamp - (timestamp % (86400 * 365)),
    }
}
