//! Show the metadata of a file

use humansize::{format_size, BINARY};
use std::{cmp::Ordering, env::args, fs::File, io::Write, process::ExitCode};

use cp437_tools::{
    help,
    meta::{self, Meta},
    process,
};

#[allow(dead_code)]
pub fn main() -> ExitCode {
    return run(args().collect());
}

#[inline]
pub fn run(args: Vec<String>) -> ExitCode {
    match args.len().cmp(&2) {
        Ordering::Less => {
            eprintln!("\x1B[31mERROR: Missing input file\x1B[0m");
            help::print();
            return ExitCode::from(1);
        }
        Ordering::Greater => {
            eprintln!("\x1B[31mERROR: Too many arguments\x1B[0m");
            help::print();
            return ExitCode::from(1);
        }
        Ordering::Equal => {
            return process(&args[1], &None, print);
        }
    }
}

fn print(input: &mut File, output: &mut Box<dyn Write>, meta: Option<Meta>) -> Result<(), String> {
    match meta {
        Some(meta) => {
            output
                .write_all(b"\x1B[4mMetadata\x1B[0m:\n")
                .map_err(|x| return x.to_string())?;
            if !meta.title.is_empty() {
                output
                    .write_all(
                        format!("* \x1B[1mTitle\x1B[0m: \x1B[3m{:?}\x1B[0m\n", meta.title).as_bytes(),
                    )
                    .map_err(|x| return x.to_string())?;
            }
            if !meta.group.is_empty() {
                output
                    .write_all(
                        format!("* \x1B[1mGroup\x1B[0m: \x1B[3m{:?}\x1B[0m\n", meta.group).as_bytes(),
                    )
                    .map_err(|x| return x.to_string())?;
            }
            if !meta.author.is_empty() {
                output
                    .write_all(
                        format!("* \x1B[1mAuthor\x1B[0m: \x1B[3m{:?}\x1B[0m\n", meta.author)
                            .as_bytes(),
                    )
                    .map_err(|x| return x.to_string())?;
            }
            if !meta.date.is_empty() {
                output
                    .write_all(
                        format!(
                            "* \x1B[1mDate\x1B[0m: \x1B[3m{}/{}/{}\x1B[0m\n",
                            &meta.date[0..4],
                            &meta.date[4..6],
                            &meta.date[6..8]
                        )
                        .as_bytes(),
                    )
                    .map_err(|x| return x.to_string())?;
            }
            output
                .write_all(
                    format!(
                        "* \x1B[1mSize\x1B[0m: \x1B[3m{}\x1B[0m\n",
                        format_size(meta.size, BINARY)
                    )
                    .as_bytes(),
                )
                .map_err(|x| return x.to_string())?;
            output
                .write_all(
                    format!(
                        "* \x1B[1mType\x1B[0m: \x1B[3m{}\x1B[0m\n",
                        match meta.r#type {
                            (0, _) => String::from("\x1B[1;3;33mNone (Character/ANSi)\x1B[0m"),
                            (1, 0) => String::from("\x1B[3;32mCharacter/ASCII\x1B[0m"),
                            (1, 1) => String::from("\x1B[3;32mCharacter/ANSi\x1B[0m"),
                            (1, 2) => String::from("\x1B[1;3;31mCharacter/ANSiMation\x1B[0m"),
                            (1, 3) => String::from("\x1B[1;3;31mCharacter/RIPScript\x1B[0m"),
                            (1, 4) => String::from("\x1B[1;3;31mCharacter/PCBoard\x1B[0m"),
                            (1, 5) => String::from("\x1B[1;3;31mCharacter/Avatar\x1B[0m"),
                            (1, 6) => String::from("\x1B[1;3;31mCharacter/HTML\x1B[0m"),
                            (1, 7) => String::from("\x1B[1;3;31mCharacter/Source\x1B[0m"),
                            (1, 8) => String::from("\x1B[1;3;31mCharacter/TundraDraw\x1B[0m"),
                            (1, _) =>
                                format!("\x1B[1;3;31mCharacter/Unknown {}\x1B[0m", meta.r#type.1),
                            (2, _) => String::from("\x1B[1;3;31mBitmap\x1B[0m"),
                            (3, _) => String::from("\x1B[1;3;31mVector\x1B[0m"),
                            (4, _) => String::from("\x1B[1;3;31mAudio\x1B[0m"),
                            (5, _) => String::from("\x1B[1;3;31mBinaryText\x1B[0m"),
                            (6, _) => String::from("\x1B[1;3;31mXBin\x1B[0m"),
                            (7, _) => String::from("\x1B[1;3;31mArchive\x1B[0m"),
                            (8, _) => String::from("\x1B[1;3;31mExecutable\x1B[0m"),
                            _ => format!(
                                "\x1B[1;3;31mUnknown {}/Unknown {}\x1B[0m",
                                meta.r#type.0, meta.r#type.1
                            ),
                        }
                    )
                    .as_bytes(),
                )
                .map_err(|x| return x.to_string())?;
            output
                .write_all(
                    format!(
                        "* \x1B[1mWidth\x1B[0m: {}\n",
                        if meta.width > 0 {
                            format!("\x1B[3;32m{} chars\x1B[0m", meta.width)
                        } else {
                            String::from("\x1B[1;3;33m0 chars (80 chars)\x1B[0m")
                        }
                    )
                    .as_bytes(),
                )
                .map_err(|x| return x.to_string())?;
            output
                .write_all(
                    format!(
                        "* \x1B[1mHeight\x1B[0m: {}\n",
                        if meta.height > 0 {
                            format!("\x1B[3;32m{} chars\x1B[0m", meta.height)
                        } else {
                            String::from("\x1B[1;3;33m0 chars (25 chars)\x1B[0m")
                        }
                    )
                    .as_bytes(),
                )
                .map_err(|x| return x.to_string())?;
            output
                .write_all(
                    format!(
                        "* \x1B[1mFlags\x1B[0m: \x1B[3m{}\x1B[0m\n",
                        if meta.flags & 0x01 == 0x00
                            || meta.flags & 0x06 == 0x06
                            || meta.flags & 0x18 == 0x18
                        {
                            format!("\x1B[3;31m{:02X}h\x1B[0m", meta.flags)
                        } else if meta.flags & 0x06 == 0x00 || meta.flags & 0x18 == 0x00 {
                            format!(
                                "\x1B[3;33m{:02X}h ({:02X}h)\x1B[0m",
                                meta.flags,
                                meta.flags
                                    | (if meta.flags & 0x06 == 0x00 {
                                        0x04
                                    } else {
                                        0x00
                                    })
                                    | (if meta.flags & 0x18 == 0x00 {
                                        0x08
                                    } else {
                                        0x00
                                    })
                            )
                        } else {
                            format!("\x1B[3;32m{:02X}h\x1B[0m", meta.flags)
                        }
                    )
                    .as_bytes(),
                )
                .map_err(|x| return x.to_string())?;
            output
                .write_all(
                    format!(
                        "* \x1B[1mFont\x1B[0m: {}\n",
                        if meta.font.is_empty() {
                            String::from("\x1B[1;3;33m<N/A> (IBM VGA)\x1B[0m")
                        } else if ["IBM VGA", "IBM VGA 437"].contains(&meta.font.as_str()) {
                            format!("\x1B[3;32m{:?}\x1B[0m", meta.font)
                        } else {
                            format!("\x1B[1;3;31m{:?}\x1B[0m", meta.font)
                        }
                    )
                    .as_bytes(),
                )
                .map_err(|x| return x.to_string())?;
            for (i, note) in meta.notes.iter().enumerate() {
                output
                    .write_all(
                        format!(
                            "* \x1B[1mNotes[{:0width$}]\x1B[0m: \x1B[3m{:?}\x1B[0m\n",
                            i,
                            note,
                            width = (meta.notes.len() as f32).log10().ceil() as usize
                        )
                        .as_bytes(),
                    )
                    .map_err(|x| return x.to_string())?;
            }

            return meta::check(&Some(meta));
        }
        None => {
            output
                .write_all(b"\x1B[4;33mNo metadata\x1B[0m:\n")
                .map_err(|x| return x.to_string())?;
            output
                .write_all(
                    format!(
                        "* \x1B[1mSize\x1B[0m: {}\n",
                        format_size(
                            input.metadata().map_err(|x| return x.to_string())?.len(),
                            BINARY
                        )
                    )
                    .as_bytes(),
                )
                .map_err(|x| return x.to_string())?;
            output
                .write_all(b"* \x1B[1mType\x1B[0m: \x1B[3;33mCharacter/ANSi\x1B[0m\n")
                .map_err(|x| return x.to_string())?;
            output
                .write_all(b"* \x1B[1mWidth\x1B[0m: \x1B[3;33m80 chars\x1B[0m\n")
                .map_err(|x| return x.to_string())?;
            output
                .write_all(b"* \x1B[1mHeight\x1B[0m: \x1B[3;33m25 chars\x1B[0m\n")
                .map_err(|x| return x.to_string())?;
            output
                .write_all(b"* \x1B[1mFlags\x1B[0m: \x1B[3;33m0Dh\x1B[0m\n")
                .map_err(|x| return x.to_string())?;
            output
                .write_all(b"* \x1B[1mFont\x1B[0m: \x1B[3;33m\"IBM VGA\"\x1B[0m\n")
                .map_err(|x| return x.to_string())?;
            return Ok(());
        }
    };
}
