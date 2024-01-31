use std::env;
use std::process;

use meta::Options;

fn handle_err(e: &str) -> ! {
    eprintln!("error: {e}");
    process::exit(1)
}

fn main() {
    let opts = Options::build(env::args()).unwrap_or_else(|err| handle_err(err));

    if let Err(e) = meta::run(opts) {
        handle_err(&e.to_string())
    }
}
