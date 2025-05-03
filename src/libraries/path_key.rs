use candid::Principal;

use crate::pool::types::{PoolFee, PoolId};

#[derive(Debug, Clone)]
pub struct PathKey {
    pub intermediary_token: Principal,
    pub fee: PoolFee,
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Swap {
    pub pool_id: PoolId,
    // in case of exactOutput this will be oneForZero
    pub zero_for_one: bool,
}

impl PathKey {
    /// # Returns
    /// `(pool, zero_for_one)`
    pub fn get_pool_and_swap_direction(&self, token_in: Principal) -> Swap {
        let token_out = self.intermediary_token;
        let (token0, token1) = if token_in < token_out {
            (token_in, token_out)
        } else {
            (token_out, token_in)
        };

        let zero_for_one = token_in == token0;
        let pool_id = PoolId {
            token0,
            token1,
            fee: self.fee,
        };

        Swap {
            pool_id,
            zero_for_one,
        }
    }
}
