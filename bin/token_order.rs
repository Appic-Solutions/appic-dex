use candid::Principal;

/// we have 2 args, token_a:Principal, token_b:Principal
/// returns token_0 and token_1
fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 3 {
        panic!("no argument provided");
    }

    let token_a = Principal::from_text(&args[1]).expect("expected a valid princiapl");
    let token_b = Principal::from_text(&args[2]).expect("expected a valid princiapl");

    // sort token_a and b, token 0 is always the smaller token
    let (token0, token1) = if token_a < token_b {
        (token_a, token_b)
    } else {
        (token_b, token_a)
    };

    println!("token_0: {} , token_1:{}", token0, token1);
}
