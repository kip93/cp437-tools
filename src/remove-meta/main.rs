//! Purge the metadata of a file

use std::{
    cmp::min,
    env::args,
    fs::File,
    io::{stdout, IsTerminal, Read, Seek, SeekFrom, Write},
    process::ExitCode,
};

use cp437_tools::{help, process, Meta};

#[allow(dead_code)]
pub fn main() -> ExitCode {
    return run(args().collect());
}

#[inline]
pub fn run(args: Vec<String>) -> ExitCode {
    if args.len() < 2 {
        eprintln!("\x1B[31mERROR: Missing input file\x1B[0m");
        help::print();
        return ExitCode::from(1);
    } else if args.len() > 3 {
        eprintln!("\x1B[31mERROR: Too many arguments\x1B[0m");
        help::print();
        return ExitCode::from(1);
    } else if args.len() == 2 && stdout().is_terminal() {
        eprintln!("\x1B[31mERROR: Refusing to write to terminal\x1B[0m");
        help::print();
        return ExitCode::from(1);
    }

    return process(&args[1], &args.get(2).map(|x| return x.to_string()), print);
}

fn print(input: &mut File, output: &mut Box<dyn Write>, meta: Option<Meta>) -> Result<(), String> {
    let size = match meta {
        Some(meta) => meta.size as usize,
        None => input.metadata().map_err(|x| return x.to_string())?.len() as usize,
    };

    let mut chunk = vec![0; 1 << 12]; // 4k chunks
    input
        .seek(SeekFrom::Start(0))
        .map_err(|x| return x.to_string())?;
    for i in 0..size.div_ceil(chunk.len()) {
        let end = min(chunk.len(), size - (i * chunk.len()));
        input
            .read_exact(&mut chunk[..end])
            .map_err(|x| return x.to_string())?;
        output
            .write_all(&chunk[..end])
            .map_err(|x| return x.to_string())?;
    }

    return Ok(());
}
