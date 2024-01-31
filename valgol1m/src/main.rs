use std::env;
use std::process;

use valgol1m::Options;

fn handle_err(e: &str) -> ! {
    eprintln!("error: {e}");
    process::exit(1)
}

fn main() {
    let opts = Options::build(env::args()).unwrap_or_else(|err| handle_err(err));

    if let Err(e) = valgol1m::run(opts) {
        handle_err(&e.to_string())
    }
}
