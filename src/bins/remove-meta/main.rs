//! Remove a file's metadata

use std::{
    cmp::Ordering,
    env::args,
    io::{stdout, IsTerminal},
};

use cp437_tools::internal::{process, ExitCode, Input, Output};

#[allow(dead_code)]
pub fn main() -> ExitCode {
    return run(args().collect());
}

#[inline]
pub fn run(args: Vec<String>) -> ExitCode {
    let exit_code = match args.len().cmp(&2) {
        Ordering::Less => ExitCode::USAGE(String::from("Missing input file")),
        Ordering::Greater => ExitCode::USAGE(String::from("Too many arguments")),
        Ordering::Equal => {
            if stdout().is_terminal() {
                ExitCode::USAGE(String::from("Refusing to write to terminal"))
            } else {
                process(&args[1], print)
            }
        }
    };

    exit_code.print();
    return exit_code;
}

fn print(input: &mut Input, output: &mut Output) -> ExitCode {
    return input
        .read_by_chunks(|chunk| {
            return output.write(chunk);
        })
        .map(|_| return ExitCode::OK)?;
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
            run(vec![String::from("cp437-remove-meta")]),
            ExitCode::USAGE(String::from("Missing input file"))
        );
    }

    #[test]
    fn too_many_args() {
        assert_eq!(
            run(vec![
                String::from("cp437-remove-meta"),
                String::from("a"),
                String::from("b"),
            ]),
            ExitCode::USAGE(String::from("Too many arguments"))
        );
    }

    #[test]
    fn stdout() {
        assert_eq!(
            run(vec![String::from("cp437-remove-meta"), String::from("a"),]),
            ExitCode::USAGE(String::from("Refusing to write to terminal"))
        );
    }

    #[test]
    fn simple() -> Result<(), String> {
        return test::file(print, "res/test/simple.ans", "res/test/simple.ans");
    }

    #[test]
    fn empty() -> Result<(), String> {
        return test::file(print, "res/test/empty.ans", "res/test/empty.ans");
    }

    #[test]
    fn meta() -> Result<(), String> {
        return test::file(print, "res/test/meta.ans", "res/test/simple.ans");
    }
}
