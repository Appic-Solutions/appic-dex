use candid::{CandidType, Deserialize, Nat, Principal};
use serde::Serialize;

#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub struct CreatePoolArgs {
    pub token_a: Principal,
    pub token_b: Principal,
    pub fee: Nat,
    pub sqrt_price_x96: Nat,
}

#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub enum CreatePoolError {
    InvalidFeeAmount,
    InvalidSqrtPriceX96,
    InvalidToken(Principal),
    PoolAlreadyExists,
}
