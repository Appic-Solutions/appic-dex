use ethnum::U256;

/// Denominator for fee calculations, in hundredths of a bip (100% = 1,000,000).
pub const PIPS_DENOMINATOR: u32 = 1_000_000;

/// notice Max protocol fee is 0.1% (1000 pips)
/// Increasing these values could lead to overflow in Pool.swap
const MAX_PROTOCOL_FEE: u16 = 1_000;

/// Calculates the total swap fee by combining protocol and LP fees.
///
/// The protocol fee is taken from the input amount first, then the LP fee is applied to the remaining amount.
/// The total fee is capped at 100% and computed as:
/// `swap_fee = protocol_fee + lp_fee - (protocol_fee * lp_fee / 1_000_000)`, rounded up.
/// Both fees are expressed in hundredths of a bip (e.g., 3000 for 0.3%).
///
/// # Arguments
/// * `protocol_fee` - The protocol fee in pips (1 pip = 0.0001), typically up to 1000 pips (0.1%).
/// * `lp_fee` - The liquidity provider fee in hundredths of a bip (e.g., 3000 for 0.3%).
///
/// # Returns
/// The total swap fee in hundredths of a bip as a `u32`.
///
/// # Errors
/// Returns `ComputeSwapError::InvalidFee` if `protocol_fee` or `lp_fee` exceeds valid bounds or results in an overflow.
pub fn calculate_swap_fee(protocol_fee: u16, lp_fee: u32) -> u32 {
    // Convert to U256 for safe arithmetic
    let protocol_fee_u256 = U256::from(protocol_fee);
    let lp_fee_u256 = U256::from(lp_fee);
    let denominator = U256::from(PIPS_DENOMINATOR);

    // Compute: protocol_fee + lp_fee - (protocol_fee * lp_fee / PIPS_DENOMINATOR)
    let numerator = protocol_fee_u256 * lp_fee_u256;
    let fee_subtraction = numerator / denominator; // Rounds down
    let swap_fee = protocol_fee_u256 + lp_fee_u256 - fee_subtraction;

    swap_fee.as_u32()
}

#[cfg(test)]
mod tests {
    use super::*;

    const MAX_LP_FEE: u32 = 1_000_000; // 100% in hundredths of a bip

    #[test]
    fn test_calculate_swap_fee() {
        assert_eq!(calculate_swap_fee(MAX_PROTOCOL_FEE, MAX_LP_FEE), MAX_LP_FEE);

        // 1000 + 3000 - (1000 * 3000 / 1_000_000) = 3997
        assert_eq!(calculate_swap_fee(MAX_PROTOCOL_FEE, 3000), 3997);

        assert_eq!(
            calculate_swap_fee(MAX_PROTOCOL_FEE, 0),
            MAX_PROTOCOL_FEE as u32
        );

        assert_eq!(calculate_swap_fee(0, 0), 0);

        assert_eq!(calculate_swap_fee(0, 1000), 1000);
    }
}
