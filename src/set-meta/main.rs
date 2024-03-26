//! Add or change the metadata of a file

use regex::{Captures, Regex};
use std::{
    cmp::min,
    env::args,
    fs::File,
    io::{stdout, IsTerminal, Read, Seek, SeekFrom, Write},
    process::ExitCode,
};

use cp437_tools::{
    help,
    meta::{self, Meta},
    process, UTF8_TO_CP437,
};

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
    } else if args.len() < 3 {
        eprintln!("\x1B[31mERROR: Missing key\x1B[0m");
        help::print();
        return ExitCode::from(1);
    } else if args.len() < 4 {
        eprintln!("\x1B[31mERROR: Missing value\x1B[0m");
        help::print();
        return ExitCode::from(1);
    } else if args.len() > 5 {
        eprintln!("\x1B[31mERROR: Too many arguments\x1B[0m");
        help::print();
        return ExitCode::from(1);
    } else if args.len() == 4 && stdout().is_terminal() {
        eprintln!("\x1B[31mERROR: Refusing to write to terminal\x1B[0m");
        help::print();
        return ExitCode::from(1);
    }

    return process(
        &args[1],
        &args.get(4).map(|x| return x.to_string()),
        |i, o, m| return print(i, o, m, &args[2], &args[3]),
    );
}

fn print(
    input: &mut File,
    output: &mut Box<dyn Write>,
    meta: Option<Meta>,
    key: &String,
    value: &String,
) -> Result<(), String> {
    let mut meta = match meta {
        Some(meta) => meta,
        None => Meta {
            title: String::from(""),
            author: String::from(""),
            group: String::from(""),
            date: String::from(""),
            size: input.metadata().map_err(|x| return x.to_string())?.len() as u32,
            r#type: (1, 1),
            width: 80,
            height: 25,
            flags: 0x0D,
            font: String::from("IBM VGA"),
            notes: Vec::new(),
        },
    };
    let value =
        Regex::new(r"\\x([0-9A-Fa-f]{1,2})")
            .unwrap()
            .replace_all(&value, |groups: &Captures| {
                return (u8::from_str_radix(&groups[1], 16).unwrap() as char).to_string();
            });
    let value =
        Regex::new(r"\\u([0-9A-Fa-f]{1,6})")
            .unwrap()
            .replace_all(&value, |groups: &Captures| {
                return char::from_u32(u32::from_str_radix(&groups[1], 16).unwrap())
                    .unwrap()
                    .to_string();
            });
    let value = value
        .replace("\\0", "\0")
        .replace("\\t", "\t")
        .replace("\\n", "\n")
        .replace("\\r", "\r")
        .replace("\\\\", "\\");
    match key.as_str() {
        "title" => {
            meta.title = value.trim().to_string();
        }
        "author" => {
            meta.author = value.trim().to_string();
        }
        "group" => {
            meta.group = value.trim().to_string();
        }
        "date" => {
            meta.date = value.trim().to_string();
        }
        "type" => match value.to_lowercase().as_str() {
            "none" => {
                meta.r#type = (0, 0);
            }
            "character/ascii" => {
                meta.r#type = (1, 0);
            }
            "character/ansi" => {
                meta.r#type = (1, 1);
            }
            _ => {
                return Err(format!("Type is unsupported ({})", value));
            }
        },
        "width" => {
            meta.width = value
                .parse::<u16>()
                .map_err(|e| format!("Invalid width ({})", e))?;
        }
        "height" => {
            meta.height = value
                .parse::<u16>()
                .map_err(|e| format!("Invalid height ({})", e))?;
        }
        "flags" => {
            meta.flags = (if let Some(hex) = value.strip_prefix("0x") {
                u8::from_str_radix(&hex, 16)
            } else if let Some(bin) = value.strip_prefix("0b") {
                u8::from_str_radix(&bin, 2)
            } else {
                value.parse::<u8>()
            })
            .map_err(|e| format!("Invalid flags ({})", e))?;
        }
        "font" => {
            meta.font = value.trim().to_string();
        }
        "notes" => {
            meta.notes = value
                .split('\n')
                .map(|x| return x.trim().to_string())
                .collect();
        }
        _ => {
            return Err(format!("Unknown key: {}", key));
        }
    }

    meta::check(&Some(meta.clone()))?;

    let mut chunk = vec![0; 1 << 12]; // 4k chunks
    input
        .seek(SeekFrom::Start(0))
        .map_err(|x| return x.to_string())?;
    for i in 0..(meta.size as usize).div_ceil(chunk.len()) {
        let end = min(chunk.len(), (meta.size as usize) - (i * chunk.len()));
        input
            .read_exact(&mut chunk[..end])
            .map_err(|x| return x.to_string())?;
        output
            .write_all(&chunk[..end])
            .map_err(|x| return x.to_string())?;
    }
    output
        .write_all(b"\x1A")
        .map_err(|x| return x.to_string())?;
    if !meta.notes.is_empty() {
        output
            .write_all(b"COMNT")
            .map_err(|x| return x.to_string())?;
        for note in &meta.notes {
            output
                .write_all(
                    &format!("{:<64}", note)
                        .chars()
                        .map(|x| return UTF8_TO_CP437.get(&x).unwrap())
                        .cloned()
                        .collect::<Vec<u8>>(),
                )
                .map_err(|x| return x.to_string())?;
        }
    }
    output
        .write_all(b"SAUCE00")
        .map_err(|x| return x.to_string())?;
    output
        .write_all(
            &format!("{:<35}", meta.title)
                .chars()
                .map(|x| return UTF8_TO_CP437.get(&x).unwrap())
                .cloned()
                .collect::<Vec<u8>>(),
        )
        .map_err(|x| return x.to_string())?;
    output
        .write_all(
            &format!("{:<20}", meta.author)
                .chars()
                .map(|x| return UTF8_TO_CP437.get(&x).unwrap())
                .cloned()
                .collect::<Vec<u8>>(),
        )
        .map_err(|x| return x.to_string())?;
    output
        .write_all(
            &format!("{:<20}", meta.group)
                .chars()
                .map(|x| return UTF8_TO_CP437.get(&x).unwrap())
                .cloned()
                .collect::<Vec<u8>>(),
        )
        .map_err(|x| return x.to_string())?;
    output
        .write_all(
            &format!("{:<8}", meta.date)
                .chars()
                .map(|x| return UTF8_TO_CP437.get(&x).unwrap())
                .cloned()
                .collect::<Vec<u8>>(),
        )
        .map_err(|x| return x.to_string())?;
    output
        .write_all(&meta.size.to_le_bytes())
        .map_err(|x| return x.to_string())?;
    output
        .write_all(&[meta.r#type.0, meta.r#type.1])
        .map_err(|x| return x.to_string())?;
    output
        .write_all(&meta.width.to_le_bytes())
        .map_err(|x| return x.to_string())?;
    output
        .write_all(&meta.height.to_le_bytes())
        .map_err(|x| return x.to_string())?;
    output
        .write_all(&(0u32).to_le_bytes())
        .map_err(|x| return x.to_string())?;
    output
        .write_all(&[meta.notes.len() as u8])
        .map_err(|x| return x.to_string())?;
    output
        .write_all(&[meta.flags])
        .map_err(|x| return x.to_string())?;
    output
        .write_all(
            &format!("{:\0<22}", meta.font)
                .chars()
                .map(|x| return UTF8_TO_CP437.get(&x).unwrap())
                .cloned()
                .collect::<Vec<u8>>(),
        )
        .map_err(|x| return x.to_string())?;

    return Ok(());
}
