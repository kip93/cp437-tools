//! Wrapper for all available subcommands in one single convinient place.

use std::env::args;

use cp437_tools::internal::ExitCode;

#[path = "bins/check-meta/main.rs"]
mod cmd_check_meta;
#[cfg(feature = "_gen")]
#[path = "bins/gen/main.rs"]
mod cmd_gen;
#[path = "bins/help/main.rs"]
mod cmd_help;
#[path = "bins/read-meta/main.rs"]
mod cmd_read_meta;
#[path = "bins/remove-meta/main.rs"]
mod cmd_remove_meta;
#[path = "bins/set-meta/main.rs"]
mod cmd_set_meta;
#[path = "bins/to-png/main.rs"]
mod cmd_to_png;
#[path = "bins/to-svg/main.rs"]
mod cmd_to_svg;
#[path = "bins/to-txt/main.rs"]
mod cmd_to_txt;

#[must_use]
#[expect(missing_docs, reason = "Just an entry point")]
pub fn main() -> ExitCode {
    return exec(&args().collect::<Vec<String>>());
}

#[inline]
#[expect(clippy::missing_docs_in_private_items, reason = "Just an entry point")]
fn exec(args: &[String]) -> ExitCode {
    return if args.len() < 2 {
        ExitCode::USAGE(String::from("Missing command"))
    } else {
        let command = args[1].as_str();
        match command {
            "check-meta" => cmd_check_meta::exec(&without_command(args)),
            "help" => cmd_help::exec(&without_command(args)),
            "read-meta" => cmd_read_meta::exec(&without_command(args)),
            "remove-meta" => cmd_remove_meta::exec(&without_command(args)),
            "set-meta" => cmd_set_meta::exec(&without_command(args)),
            "to-png" => cmd_to_png::exec(&without_command(args)),
            "to-svg" => cmd_to_svg::exec(&without_command(args)),
            "to-txt" => cmd_to_txt::exec(&without_command(args)),
            #[cfg(feature = "_gen")]
            "gen" => cmd_gen::exec(&without_command(args)),
            _ => ExitCode::USAGE(format!("Unknown command: {command}")),
        }
    };
}

#[inline]
/// Changes the value of $0.
fn without_command(args: &[String]) -> Vec<String> {
    return [format!("cp437-{}", args[1])].iter().chain(args.iter().skip(2)).cloned().collect();
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;

    #[test]
    fn help() -> ExitCode {
        return exec(&[String::from("cp437-tools"), String::from("help")]);
    }

    #[test]
    fn no_command() {
        assert_eq!(exec(&[String::from("cp437-tools")]), ExitCode::USAGE(String::from("Missing command")));
    }

    #[test]
    fn unknown_command() {
        assert_eq!(
            exec(&[String::from("cp437-tools"), String::from("foo")]),
            ExitCode::USAGE(String::from("Unknown command: foo")),
        );
    }
}
