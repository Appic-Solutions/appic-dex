pub mod address;
pub mod balances;
pub mod burn;
pub mod candid_types;
pub mod cbor;
pub mod collect_fees;
pub mod cross_chain;
pub mod decrease_liquidity;
pub mod deposit;
pub mod events;
pub mod guard;
pub mod historical;
pub mod icrc_client;
pub mod increase_liquidity;
pub mod libraries;
pub mod logs;
pub mod mint;
pub mod minter_client;
pub mod pool;
pub mod position;
pub mod proxy_canister;
pub mod quote;
pub mod state;
pub mod swap;
pub mod swap_id;
pub mod tick;
pub mod validation;
pub mod withdraw;

pub const APPIC_CONTROLLER: &str =
    "tb3vi-54bcb-4oudm-fmp2s-nntjp-rmhd3-ukvnq-lawfq-vk5vy-mnlc7-pae";

#[cfg(test)]
pub mod tests;
