//! Show a command's help text and exit

use std::env::args;

use cp437_tools::internal::{help, ExitCode};

#[allow(dead_code)]
pub fn main() -> ExitCode {
    return run(args().collect());
}

#[inline]
pub fn run(args: Vec<String>) -> ExitCode {
    if args.len() > 2 {
        return ExitCode::USAGE(String::from("Too many arguments"));
    }

    let command = args.get(1).cloned().unwrap_or(String::from("cp437-tools"));
    return match help::get(command.clone()) {
        Some(text) => {
            eprintln!("{}", text);
            ExitCode::OK
        }
        None => ExitCode::USAGE(format!("Unknown command: {}", command)),
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;

    #[test]
    fn too_many_args() {
        assert_eq!(
            run(vec![
                String::from("cp437-help"),
                String::from("a"),
                String::from("b")
            ]),
            ExitCode::USAGE(String::from("Too many arguments"))
        );
    }

    #[test]
    fn no_args() {
        assert_eq!(run(vec![String::from("cp437-help")]), ExitCode::OK);
    }

    #[test]
    fn with_command() {
        assert_eq!(
            run(vec![String::from("cp437-help"), String::from("help")]),
            ExitCode::OK
        );
    }

    #[test]
    fn unknown() {
        assert_eq!(
            run(vec![String::from("cp437-help"), String::from("foo")]),
            ExitCode::USAGE(String::from("Unknown command: foo"))
        );
    }
}
