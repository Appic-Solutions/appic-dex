use appic_dex::libraries::{constants::Q96, safe_cast::big_uint_to_u256, tick_math};
use num_bigint::ToBigUint;

// we have 2 args, price and tick_spacing

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 6 {
        panic!("Expected 5 arguments: dec0 dec1 price direction tick_spacing");
    }

    let dec0: u8 = args[1]
        .parse()
        .expect("Invalid dec0: must be an integer (0-255)");
    let dec1: u8 = args[2]
        .parse()
        .expect("Invalid dec1: must be an integer (0-255)");
    let price: f64 = args[3]
        .parse()
        .expect("Invalid price: must be a positive float");
    let direction: u8 = args[4].parse().expect("Invalid direction: must be 0 or 1");
    let tick_spacing: u32 = args[5]
        .parse()
        .expect("Invalid tick_spacing: must be a positive integer");

    if price <= 0.0 {
        panic!("Price must be positive");
    }
    if direction > 1 {
        panic!("Direction must be 0 (token1/token0) or 1 (token0/token1)");
    }

    let q_96 = 2_f64.powi(96);

    // Calculate the decimal adjustment factor
    let factor = 10.0_f64.powi(dec1 as i32) / 10.0_f64.powi(dec0 as i32);

    // Compute price ratio based on direction (token1 base units per token0 base unit)
    let price_ratio = if direction == 0 {
        price * factor
    } else {
        (1.0 / price) * factor
    };

    let sqrt_price_x96_floating = price_ratio.sqrt() * q_96;

    println!("sqrt_price_x96_floating: {}", sqrt_price_x96_floating);

    let sqrt_price_x96_u256 = big_uint_to_u256(
        sqrt_price_x96_floating
            .to_biguint()
            .expect("Failed to convert sqrt_price_x96 to BigUint"),
    )
    .expect("Failed to parse sqrt_price_x96 into U256");

    let tick_not_aligned = tick_math::TickMath::get_tick_at_sqrt_ratio(sqrt_price_x96_u256);

    // Align tick with tick_spacing
    let tick = (tick_not_aligned / tick_spacing as i32) * tick_spacing as i32;

    println!(
        "tick for price {} (direction: {}) is {}, not_aligned tick: {}",
        price,
        if direction == 0 {
            "token1/token0"
        } else {
            "token0/token1"
        },
        tick,
        tick_not_aligned
    );
}
