use std::{env::args, process::ExitCode};

use cp437_tools::help;

#[allow(dead_code)]
pub fn main() -> ExitCode {
    return run(args().collect());
}

#[inline]
pub fn run(_args: Vec<String>) -> ExitCode {
    help::print();
    return ExitCode::from(0);
}
