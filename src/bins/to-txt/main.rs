//! Transpile CP437 to UTF-8 while also stripping metadata

use std::{cmp::Ordering, env::args};

use cp437_tools::{
    internal::{process, ExitCode, Input, Output},
    prelude::{Meta, CP437_TO_UTF8},
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
        Ordering::Equal => process(&args[1], print),
    };

    exit_code.print();
    return exit_code;
}

fn print(input: &mut Input, output: &mut Output) -> ExitCode {
    let meta = input.meta.clone().unwrap_or_else(|| {
        return Meta {
            size: input.size,
            ..Default::default()
        };
    });

    let mut control: Vec<u8> = vec![];
    let (mut x, mut y) = (0, 0);

    input.read_by_bytes(|byte| {
        if y >= meta.height() {
            return Ok(());
        }

        output.write(
            String::from(CP437_TO_UTF8[if byte > 0 { byte as usize } else { 32 }]).as_bytes(),
        )?;
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
        assert_eq!(
            run(vec![String::from("cp437-to-txt")]),
            ExitCode::USAGE(String::from("Missing input file"))
        );
    }

    #[test]
    fn too_many_args() {
        assert_eq!(
            run(vec![
                String::from("cp437-to-txt"),
                String::from("a"),
                String::from("b"),
            ]),
            ExitCode::USAGE(String::from("Too many arguments"))
        );
    }

    #[test]
    fn simple() -> Result<(), String> {
        return test::file(print, "res/test/simple.ans", "res/test/simple.txt");
    }

    #[test]
    fn meta() -> Result<(), String> {
        return test::file(print, "res/test/meta.ans", "res/test/meta.txt");
    }

    #[test]
    fn background() -> Result<(), String> {
        return test::file(print, "res/test/background.ans", "res/test/background.txt");
    }
}
