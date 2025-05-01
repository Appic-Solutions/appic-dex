use candid::Principal;

use crate::pool::types::{PoolFee, PoolId};

#[derive(Debug, PartialEq)]
pub struct PathKey {
    intermediary_token: Principal,
    fee: PoolFee,
}

impl PathKey {
    /// # Returns
    /// `(pool, zero_for_one)`
    pub fn get_pool_and_swap_direction(&self, token_in: Principal) -> (PoolId, bool) {
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

        (pool_id, zero_for_one)
    }
}
