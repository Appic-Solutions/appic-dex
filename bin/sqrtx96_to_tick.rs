use appic_dex::libraries::tick_math;
use ethnum::U256;

fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 2 {
        //usage();
        std::process::exit(1);
    }

    let sqrt_x96_text = &args[1];

    let sqrt_x96 = U256::from_str_radix(sqrt_x96_text, 10_u32).expect("expected a number");

    let tick = tick_math::TickMath::get_tick_at_sqrt_ratio(sqrt_x96);

    println!("tick for sqrt_x96 price {} is {}", sqrt_x96, tick);
}
