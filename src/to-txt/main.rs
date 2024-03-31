//! Transpile CP437 to UTF-8 while also stripping metadata

use std::{
    cmp::min,
    env::args,
    fs::File,
    io::{Read, Seek, SeekFrom, Write},
};

use cp437_tools::{help, process, ExitCode, Meta, CP437_TO_UTF8};

#[allow(dead_code)]
pub fn main() -> ExitCode {
    return run(args().collect());
}

#[inline]
pub fn run(args: Vec<String>) -> ExitCode {
    if args.len() < 2 {
        let msg = String::from("Missing input file");
        eprintln!("\x1B[31mERROR: {}\x1B[0m", msg);
        help::print();
        return ExitCode::USAGE(msg);
    } else if args.len() > 3 {
        let msg = String::from("Too many arguments");
        eprintln!("\x1B[31mERROR: {}\x1B[0m", msg);
        help::print();
        return ExitCode::USAGE(msg);
    }

    return process(&args[1], &args.get(2).map(|x| return x.to_string()), print);
}

fn print(
    input: &mut File,
    output: &mut Box<dyn Write>,
    meta: Option<Meta>,
) -> Result<(), ExitCode> {
    let size = match meta {
        Some(meta) => meta.size as usize,
        None => input
            .metadata()
            .map_err(|x| return ExitCode::ERROR(x.to_string()))?
            .len() as usize,
    };

    let mut chunk = vec![0; 1 << 12]; // 4k chunks
    input
        .seek(SeekFrom::Start(0))
        .map_err(|x| return ExitCode::ERROR(x.to_string()))?;
    for i in 0..size.div_ceil(chunk.len()) {
        let end = min(chunk.len(), size - (i * chunk.len()));
        input
            .read_exact(&mut chunk[..end])
            .map_err(|x| return ExitCode::ERROR(x.to_string()))?;
        output
            .write_all(
                chunk[..end]
                    .iter()
                    .map(|x| return CP437_TO_UTF8[*x as usize])
                    .collect::<String>()
                    .as_bytes(),
            )
            .map_err(|x| return ExitCode::ERROR(x.to_string()))?;
    }
    output
        .write_all(b"\x1B[0m")
        .map_err(|x| return ExitCode::ERROR(x.to_string()))?;

    return Ok(());
}

#[cfg(test)]
mod tests {
    use super::*;

    use file_diff::diff;
    use tempfile::tempdir;

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
                String::from("c")
            ]),
            ExitCode::USAGE(String::from("Too many arguments"))
        );
    }

    #[test]
    fn simple() -> Result<(), String> {
        return test("res/test/simple.ans", "res/test/simple.txt");
    }

    #[test]
    fn meta() -> Result<(), String> {
        return test("res/test/meta.ans", "res/test/meta.txt");
    }

    fn test(input: &str, output: &str) -> Result<(), String> {
        let tmp_dir = tempdir().map_err(|x| return x.to_string())?;
        let target = tmp_dir
            .path()
            .join("output.txt")
            .to_string_lossy()
            .to_string();
        assert_eq!(
            run(vec![
                String::from("cp437-to-txt"),
                String::from(input),
                target.clone(),
            ]),
            ExitCode::OK
        );
        assert!(tmp_dir.path().join("output.txt").exists());
        assert!(diff(&target, output));

        tmp_dir.close().map_err(|x| return x.to_string())?;

        return Ok(());
    }
}
