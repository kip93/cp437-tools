//! Check a file's metadata

use std::{cmp::Ordering, env::args};

use cp437_tools::{
    internal::{process, ExitCode, Input, Output},
    prelude::meta,
};

#[allow(dead_code)]
pub fn main() -> ExitCode {
    return run(args().collect());
}

#[inline]
pub fn run(args: Vec<String>) -> ExitCode {
    let exit_code = match args.len().cmp(&2) {
        Ordering::Less => ExitCode::USAGE(String::from("Missing input file")),
        Ordering::Greater => ExitCode::USAGE(String::from("Too many arguments")),
        Ordering::Equal => process(&args[1], check),
    };

    exit_code.print();
    return exit_code;
}

fn check(input: &mut Input, output: &mut Output) -> ExitCode {
    if let Err(msg) = meta::check(input.meta.as_ref()) {
        output.write(format!("\x1B[3;31m{}\x1B[0m\n", msg).as_bytes())?;
        return ExitCode::FAIL(msg);
    }

    return ExitCode::OK;
}

#[path = "."]
#[cfg(test)]
mod tests {
    use super::*;

    use indoc::indoc;
    use pretty_assertions::assert_eq;

    #[path = "../../libs/internal/test_utils.rs"]
    mod test;

    #[test]
    fn no_input() {
        assert_eq!(
            run(vec![String::from("cp437-check-meta")]),
            ExitCode::USAGE(String::from("Missing input file"))
        );
    }

    #[test]
    fn too_many_args() {
        assert_eq!(
            run(vec![
                String::from("cp437-check-meta"),
                String::from("a"),
                String::from("b"),
            ]),
            ExitCode::USAGE(String::from("Too many arguments"))
        );
    }

    #[test]
    fn ok() -> Result<(), String> {
        return test::ok(check, "res/test/meta.ans", indoc! {""});
    }

    #[test]
    fn no_meta() -> Result<(), String> {
        return test::ok(check, "res/test/simple.ans", indoc! {""});
    }

    #[test]
    fn title() -> Result<(), String> {
        return test::file_err(
            check,
            "res/test/bad_title.ans",
            indoc! {"
                \x1B[3;31mTitle contains illegal characters (0x00 is a control character)\x1B[0m
            "},
        );
    }

    #[test]
    fn author() -> Result<(), String> {
        return test::file_err(
            check,
            "res/test/bad_author.ans",
            indoc! {"
                \x1B[3;31mAuthor contains illegal characters (0x00 is a control character)\x1B[0m
            "},
        );
    }

    #[test]
    fn group() -> Result<(), String> {
        return test::file_err(
            check,
            "res/test/bad_group.ans",
            indoc! {"
                \x1B[3;31mGroup contains illegal characters (0x00 is a control character)\x1B[0m
            "},
        );
    }

    #[test]
    fn date() -> Result<(), String> {
        return test::file_err(
            check,
            "res/test/bad_date.ans",
            indoc! {"
                \x1B[3;31mDate format is wrong (input contains invalid characters)\x1B[0m
            "},
        );
    }

    #[test]
    fn r#type() -> Result<(), String> {
        return test::file_err(
            check,
            "res/test/bad_type.ans",
            indoc! {"
                \x1B[3;31mType is unsupported (Unknown 255/Unknown 255)\x1B[0m
            "},
        );
    }

    #[test]
    fn flags() -> Result<(), String> {
        return test::file_err(
            check,
            "res/test/bad_flags.ans",
            indoc! {"
                \x1B[3;31mInvalid letter spacing\x1B[0m
            "},
        );
    }

    #[test]
    fn font() -> Result<(), String> {
        return test::file_err(
            check,
            "res/test/bad_font.ans",
            indoc! {"
                \x1B[3;31mFont is unsupported (IBM FOO)\x1B[0m
            "},
        );
    }

    #[test]
    fn notes() -> Result<(), String> {
        return test::file_err(
            check,
            "res/test/bad_comment.ans",
            indoc! {"
                \x1B[3;31mNotes[0] contains illegal characters (0x00 is a control character)\x1B[0m
            "},
        );
    }
}
