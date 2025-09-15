use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub struct CandidMinter {
    pub id: Principal,
    pub chain_id: u64,
    pub twin_usdc_principal: Principal,
    pub usdc_address: String,
}

#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub struct UpgradeArgs {
    pub upgrade_minters: Option<Vec<CandidMinter>>,
}
