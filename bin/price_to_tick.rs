use appic_dex::libraries::{constants::Q96, safe_cast::big_uint_to_u256, tick_math};
use ethnum::U256;
use num_bigint::ToBigUint;

// we have 2 args, price and tick_spacing

fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 3 {
        panic!("no argument provided");
        std::process::exit(1);
    }

    let q_96 = 2_f64.powi(96);

    let price = &args[1].parse::<f64>().expect("expected a floating number");

    let tick_spacing = args[2]
        .parse::<u32>()
        .expect("expected a valid tick_spacing");

    let sqrtx96_floating = price.sqrt() * q_96;

    println!("{ }", sqrtx96_floating);

    let sqrtx96_u256 = big_uint_to_u256(
        sqrtx96_floating
            .to_biguint()
            .expect("failed to convert to biguint"),
    )
    .expect("failed to parse into u256");

    let tick_not_aligned = tick_math::TickMath::get_tick_at_sqrt_ratio(sqrtx96_u256);

    // align tick with tick_spacing
    let tick = (tick_not_aligned / tick_spacing as i32) * tick_spacing as i32;

    println!(
        "tick for sqrt_x96 price {} is {} and not_aligned tick is {}",
        price, tick, tick_not_aligned
    );
}
