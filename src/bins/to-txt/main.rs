//! Transpile a file to UTF-8.

use std::{cmp::Ordering, env::args};

use cp437_tools::{
    internal::{process, ExitCode, Input, Output},
    prelude::{Meta, CP437_TO_UTF8},
};

#[allow(dead_code)]
#[must_use]
#[allow(missing_docs, reason = "Just an entry point")]
#[allow(clippy::missing_docs_in_private_items, reason = "Just an entry point")]
pub fn main() -> ExitCode {
    return exec(&args().collect::<Vec<String>>());
}

#[inline]
#[must_use]
#[allow(missing_docs, reason = "Just an entry point")]
#[allow(clippy::missing_docs_in_private_items, reason = "Just an entry point")]
pub fn exec(args: &[String]) -> ExitCode {
    let exit_code = match args.len().cmp(&2) {
        Ordering::Less => ExitCode::USAGE(String::from("Missing input file")),
        Ordering::Greater => ExitCode::USAGE(String::from("Too many arguments")),
        Ordering::Equal => process(&args[1], run),
    };

    exit_code.print();
    return exit_code;
}

#[allow(missing_docs, reason = "Just an entry point")]
#[allow(clippy::missing_docs_in_private_items, reason = "Just an entry point")]
pub fn run(input: &mut Input, output: &mut Output) -> ExitCode {
    let meta = input.meta.clone().unwrap_or_else(|| {
        return Meta { size: input.size, ..Default::default() };
    });

    let mut control: Vec<u8> = vec![];
    let (mut x, mut y) = (0, 0);

    input.read_by_bytes(|byte| {
        if y >= meta.height() {
            return Ok(());
        }

        output.write(String::from(CP437_TO_UTF8[if byte > 0 { byte as usize } else { 32 }]).as_bytes())?;
        if !control.is_empty() {
            if control.len() > 1 && (0x40..=0x7E).contains(&byte) {
                control.clear();
            } else {
                control.push(byte);
            }
        } else if byte == 0x1B {
            control.push(byte);
        } else if byte == 0x0D {
            (x, y) = (0, y);
        } else if byte == 0x0A {
            (x, y) = (0, y + 1);
        } else {
            x += 1;
            if x >= meta.width() {
                output.write(b"\r\n")?;
                (x, y) = (0, y + 1);
            }
        }

        return Ok(());
    })?;

    return output.write(b"\x1B[0m").map(|_| return ExitCode::OK)?;
}

#[path = "."]
#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;

    #[path = "../../libs/internal/test_utils.rs"]
    mod test;

    #[test]
    fn no_input() {
        assert_eq!(exec(&[String::from("cp437-to-txt")]), ExitCode::USAGE(String::from("Missing input file")));
    }

    #[test]
    fn too_many_args() {
        assert_eq!(
            exec(&[String::from("cp437-to-txt"), String::from("a"), String::from("b")]),
            ExitCode::USAGE(String::from("Too many arguments")),
        );
    }

    #[test]
    fn simple() -> Result<(), String> {
        return test::file(run, "res/test/simple.ans", "res/test/simple.txt");
    }

    #[test]
    fn meta() -> Result<(), String> {
        return test::file(run, "res/test/meta.ans", "res/test/meta.txt");
    }

    #[test]
    fn background() -> Result<(), String> {
        return test::file(run, "res/test/background.ans", "res/test/background.txt");
    }
}
