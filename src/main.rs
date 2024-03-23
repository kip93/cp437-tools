//! Wrapper for all available subcommands in one single convinient place

use std::{env::args, process::ExitCode};

use cp437_tools::help;
#[path = "help/main.rs"]
mod help_cmd;
#[path = "to-png/main.rs"]
mod png_cmd;
#[path = "to-txt/main.rs"]
mod txt_cmd;

pub fn main() -> ExitCode {
    return run(args().collect());
}

#[inline]
fn run(args: Vec<String>) -> ExitCode {
    if args.len() < 2 {
        eprintln!("\x1B[31mERROR: Missing command\x1B[0m");
        help::print();
        return ExitCode::from(1);
    }

    let command = args[1].as_str();
    match command {
        "help" => {
            return help_cmd::run(without_command(args));
        }
        "to-png" => {
            return png_cmd::run(without_command(args));
        }
        "to-txt" => {
            return txt_cmd::run(without_command(args));
        }
        _ => {
            eprintln!("\x1B[31mERROR: Unknown command: {}\x1B[0m", command);
            help::print();
            return ExitCode::from(1);
        }
    }
}

#[inline]
fn without_command(args: Vec<String>) -> Vec<String> {
    return args
        .iter()
        .enumerate()
        .filter(|&(i, _)| return i != 1)
        .map(|(_, v)| return v.to_string())
        .collect();
}
