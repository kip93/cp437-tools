//! Wrapper for all available subcommands in one single convinient place

use std::env::args;

use cp437_tools::{help, ExitCode};
#[path = "remove-meta/main.rs"]
mod del_cmd;
#[path = "help/main.rs"]
mod help_cmd;
#[path = "to-png/main.rs"]
mod png_cmd;
#[path = "read-meta/main.rs"]
mod read_cmd;
#[path = "set-meta/main.rs"]
mod set_cmd;
#[path = "to-txt/main.rs"]
mod txt_cmd;

pub fn main() -> ExitCode {
    return run(args().collect());
}

#[inline]
fn run(args: Vec<String>) -> ExitCode {
    if args.len() < 2 {
        let msg = String::from("Missing command");
        eprintln!("\x1B[31mERROR: {}\x1B[0m", msg);
        help::print();
        return ExitCode::USAGE(msg);
    }

    let command = args[1].as_str();
    match command {
        "help" => {
            return help_cmd::run(without_command(args));
        }
        "read-meta" => {
            return read_cmd::run(without_command(args));
        }
        "remove-meta" => {
            return del_cmd::run(without_command(args));
        }
        "set-meta" => {
            return set_cmd::run(without_command(args));
        }
        "to-png" => {
            return png_cmd::run(without_command(args));
        }
        "to-txt" => {
            return txt_cmd::run(without_command(args));
        }
        _ => {
            let msg = format!("Unknown command: {}", command);
            eprintln!("\x1B[31mERROR: {}\x1B[0m", msg);
            help::print();
            return ExitCode::USAGE(msg);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn help() -> ExitCode {
        return run(vec![String::from("cp437-tools"), String::from("help")]);
    }

    #[test]
    fn no_command() {
        assert_eq!(
            run(vec![String::from("cp437-tools")]),
            ExitCode::USAGE(String::from("Missing command"))
        );
    }

    #[test]
    fn unknown_command() {
        assert_eq!(
            run(vec![String::from("cp437-tools"), String::from("foo")]),
            ExitCode::USAGE(String::from("Unknown command: foo"))
        );
    }
}
