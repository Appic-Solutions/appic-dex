use ic_stable_structures::{storable::Bound, storable::Storable};
use std::borrow::Cow;

use crate::{
    balances::types::{UserBalance, UserBalanceKey},
    historical::types::{HistoryBucket, PoolHistory},
    pool::types::{PoolFee, PoolId, PoolState, PoolTickSpacing},
    position::types::{PositionInfo, PositionKey},
    tick::types::{BitmapWord, TickBitmapKey, TickInfo, TickKey},
};

macro_rules! impl_storable_minicbor {
    ($type:ty ) => {
        impl Storable for $type {
            fn to_bytes(&self) -> Cow<[u8]> {
                let mut buf = Vec::new();
                minicbor::encode(self, &mut buf).expect("minicbor encoding should always succeed");
                Cow::Owned(buf)
            }

            fn from_bytes(bytes: Cow<[u8]>) -> Self {
                minicbor::decode(bytes.as_ref()).unwrap_or_else(|e| {
                    panic!(
                        "failed to decode minicbor bytes {}: {}",
                        hex::encode(&bytes),
                        e
                    )
                })
            }
            const BOUND: Bound = Bound::Unbounded;
        }
    };
}

// Apply to your types
impl_storable_minicbor!(PoolState);
impl_storable_minicbor!(PoolId);
impl_storable_minicbor!(TickInfo);
impl_storable_minicbor!(PositionKey);
impl_storable_minicbor!(PositionInfo);
impl_storable_minicbor!(TickBitmapKey);
impl_storable_minicbor!(TickKey);
impl_storable_minicbor!(UserBalance);
impl_storable_minicbor!(UserBalanceKey);
impl_storable_minicbor!(BitmapWord);
impl_storable_minicbor!(PoolFee);
impl_storable_minicbor!(PoolTickSpacing);
impl_storable_minicbor!(HistoryBucket);
impl_storable_minicbor!(PoolHistory);
