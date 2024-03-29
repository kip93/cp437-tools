//! Print help text and exit

use std::env::args;

use cp437_tools::{help, ExitCode};

#[allow(dead_code)]
pub fn main() -> ExitCode {
    return run(args().collect());
}

#[inline]
pub fn run(_args: Vec<String>) -> ExitCode {
    help::print();
    return ExitCode::OK;
}
