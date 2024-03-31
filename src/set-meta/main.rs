//! Add or change the metadata of a file

use regex::{Captures, Regex};
use std::{
    cmp::min,
    env::args,
    fs::File,
    io::{stdout, IsTerminal, Read, Seek, SeekFrom, Write},
};

use cp437_tools::{
    help,
    meta::{self, Meta},
    process, ExitCode, UTF8_TO_CP437,
};

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
    } else if args.len() < 3 {
        let msg = String::from("Missing key");
        eprintln!("\x1B[31mERROR: {}\x1B[0m", msg);
        help::print();
        return ExitCode::USAGE(msg);
    } else if args.len() < 4 {
        let msg = String::from("Missing value");
        eprintln!("\x1B[31mERROR: {}\x1B[0m", msg);
        help::print();
        return ExitCode::USAGE(msg);
    } else if args.len() > 5 {
        let msg = String::from("Too many arguments");
        eprintln!("\x1B[31mERROR: {}\x1B[0m", msg);
        help::print();
        return ExitCode::USAGE(msg);
    } else if args.len() == 4 && stdout().is_terminal() {
        let msg = String::from("Refusing to write to terminal");
        eprintln!("\x1B[31mERROR: {}\x1B[0m", msg);
        help::print();
        return ExitCode::USAGE(msg);
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
) -> Result<(), ExitCode> {
    let mut meta = match meta {
        Some(meta) => meta,
        None => Meta {
            size: input
                .metadata()
                .map_err(|x| return ExitCode::ERROR(x.to_string()))?
                .len() as u32,
            r#type: (1, 1),
            width: 80,
            height: 25,
            flags: 0x0D,
            font: String::from("IBM VGA"),
            ..Default::default()
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
        "size" => {
            return Err(ExitCode::USAGE(String::from("Size can't be changed")));
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
                return Err(ExitCode::USAGE(format!("Type is unsupported ({})", value)));
            }
        },
        "width" => {
            meta.width = value
                .parse::<u16>()
                .map_err(|x| return ExitCode::USAGE(format!("Invalid width ({})", x)))?;
        }
        "height" => {
            meta.height = value
                .parse::<u16>()
                .map_err(|x| return ExitCode::USAGE(format!("Invalid height ({})", x)))?;
        }
        "flags" => {
            meta.flags = (if let Some(hex) = value.strip_prefix("0x") {
                u8::from_str_radix(&hex, 16)
            } else if let Some(bin) = value.strip_prefix("0b") {
                u8::from_str_radix(&bin, 2)
            } else {
                value.parse::<u8>()
            })
            .map_err(|x| return ExitCode::USAGE(format!("Invalid flags ({})", x)))?;
        }
        "font" => {
            meta.font = value.trim().to_string();
        }
        "notes" => {
            if !value.is_empty() {
                meta.notes = value
                    .split('\n')
                    .map(|x| return x.trim().to_string())
                    .filter(|x| return !x.is_empty())
                    .collect();
            } else {
                meta.notes = vec![];
            }
        }
        _ => {
            return Err(ExitCode::USAGE(format!("Unknown key: {}", key)));
        }
    }

    meta::check(&Some(meta.clone())).map_err(|x| return ExitCode::FAIL(x))?;

    let mut chunk = vec![0; 1 << 12]; // 4k chunks
    input
        .seek(SeekFrom::Start(0))
        .map_err(|x| return ExitCode::ERROR(x.to_string()))?;
    for i in 0..(meta.size as usize).div_ceil(chunk.len()) {
        let end = min(chunk.len(), (meta.size as usize) - (i * chunk.len()));
        input
            .read_exact(&mut chunk[..end])
            .map_err(|x| return ExitCode::ERROR(x.to_string()))?;
        output
            .write_all(&chunk[..end])
            .map_err(|x| return ExitCode::ERROR(x.to_string()))?;
    }
    output
        .write_all(b"\x1A")
        .map_err(|x| return ExitCode::ERROR(x.to_string()))?;
    if !meta.notes.is_empty() {
        output
            .write_all(b"COMNT")
            .map_err(|x| return ExitCode::ERROR(x.to_string()))?;
        for note in &meta.notes {
            output
                .write_all(
                    &format!("{:<64}", note)
                        .chars()
                        .map(|x| return UTF8_TO_CP437.get(&x).unwrap())
                        .cloned()
                        .collect::<Vec<u8>>(),
                )
                .map_err(|x| return ExitCode::ERROR(x.to_string()))?;
        }
    }
    output
        .write_all(b"SAUCE00")
        .map_err(|x| return ExitCode::ERROR(x.to_string()))?;
    output
        .write_all(
            &format!("{:<35}", meta.title)
                .chars()
                .map(|x| return UTF8_TO_CP437.get(&x).unwrap())
                .cloned()
                .collect::<Vec<u8>>(),
        )
        .map_err(|x| return ExitCode::ERROR(x.to_string()))?;
    output
        .write_all(
            &format!("{:<20}", meta.author)
                .chars()
                .map(|x| return UTF8_TO_CP437.get(&x).unwrap())
                .cloned()
                .collect::<Vec<u8>>(),
        )
        .map_err(|x| return ExitCode::ERROR(x.to_string()))?;
    output
        .write_all(
            &format!("{:<20}", meta.group)
                .chars()
                .map(|x| return UTF8_TO_CP437.get(&x).unwrap())
                .cloned()
                .collect::<Vec<u8>>(),
        )
        .map_err(|x| return ExitCode::ERROR(x.to_string()))?;
    output
        .write_all(
            &format!("{:<8}", meta.date)
                .chars()
                .map(|x| return UTF8_TO_CP437.get(&x).unwrap())
                .cloned()
                .collect::<Vec<u8>>(),
        )
        .map_err(|x| return ExitCode::ERROR(x.to_string()))?;
    output
        .write_all(&meta.size.to_le_bytes())
        .map_err(|x| return ExitCode::ERROR(x.to_string()))?;
    output
        .write_all(&[meta.r#type.0, meta.r#type.1])
        .map_err(|x| return ExitCode::ERROR(x.to_string()))?;
    output
        .write_all(&meta.width.to_le_bytes())
        .map_err(|x| return ExitCode::ERROR(x.to_string()))?;
    output
        .write_all(&meta.height.to_le_bytes())
        .map_err(|x| return ExitCode::ERROR(x.to_string()))?;
    output
        .write_all(&(0u32).to_le_bytes())
        .map_err(|x| return ExitCode::ERROR(x.to_string()))?;
    output
        .write_all(&[meta.notes.len() as u8])
        .map_err(|x| return ExitCode::ERROR(x.to_string()))?;
    output
        .write_all(&[meta.flags])
        .map_err(|x| return ExitCode::ERROR(x.to_string()))?;
    output
        .write_all(
            &format!("{:\0<22}", meta.font)
                .chars()
                .map(|x| return UTF8_TO_CP437.get(&x).unwrap())
                .cloned()
                .collect::<Vec<u8>>(),
        )
        .map_err(|x| return ExitCode::ERROR(x.to_string()))?;

    return Ok(());
}

#[cfg(test)]
mod tests {
    use super::*;

    use tempfile::tempdir;

    #[test]
    fn no_input() {
        assert_eq!(
            run(vec![String::from("cp437-set-meta")]),
            ExitCode::USAGE(String::from("Missing input file"))
        );
    }

    #[test]
    fn no_key() {
        assert_eq!(
            run(vec![String::from("cp437-set-meta"), String::from("a")]),
            ExitCode::USAGE(String::from("Missing key"))
        );
    }

    #[test]
    fn unknown_key() -> Result<(), String> {
        return test_err("res/test/simple.ans", "foo", "bar", &|m: String| {
            return m == "Unknown key: foo";
        });
    }

    #[test]
    fn no_value() {
        assert_eq!(
            run(vec![
                String::from("cp437-set-meta"),
                String::from("a"),
                String::from("b")
            ]),
            ExitCode::USAGE(String::from("Missing value"))
        );
    }

    #[test]
    fn too_many_args() {
        assert_eq!(
            run(vec![
                String::from("cp437-set-meta"),
                String::from("a"),
                String::from("b"),
                String::from("c"),
                String::from("d"),
                String::from("e")
            ]),
            ExitCode::USAGE(String::from("Too many arguments"))
        );
    }

    #[test]
    fn stdout() {
        assert_eq!(
            run(vec![
                String::from("cp437-set-meta"),
                String::from("a"),
                String::from("b"),
                String::from("c")
            ]),
            ExitCode::USAGE(String::from("Refusing to write to terminal"))
        );
    }

    #[test]
    fn hex() -> Result<(), String> {
        return test("res/test/simple.ans", "title", "\\x40", &|m: Meta| {
            return m.title == "@";
        });
    }

    #[test]
    fn unicode() -> Result<(), String> {
        return test("res/test/simple.ans", "title", "\\u3B1", &|m: Meta| {
            return m.title == "α";
        });
    }

    #[test]
    fn title() -> Result<(), String> {
        return test("res/test/simple.ans", "title", "TITLE", &|m: Meta| {
            return m.title == "TITLE";
        });
    }

    #[test]
    fn author() -> Result<(), String> {
        return test("res/test/simple.ans", "author", "AUTHOR", &|m: Meta| {
            return m.author == "AUTHOR";
        });
    }

    #[test]
    fn group() -> Result<(), String> {
        return test("res/test/simple.ans", "group", "GROUP", &|m: Meta| {
            return m.group == "GROUP";
        });
    }

    #[test]
    fn date() -> Result<(), String> {
        return test("res/test/simple.ans", "date", "19700101", &|m: Meta| {
            return m.date == "19700101";
        });
    }

    #[test]
    fn size() -> Result<(), String> {
        return test_err("res/test/simple.ans", "size", "foo", &|m: String| {
            return m == "Size can't be changed";
        });
    }

    mod r#type {
        use super::*;

        #[test]
        fn none() -> Result<(), String> {
            return test("res/test/simple.ans", "type", "None", &|m: Meta| {
                return m.r#type == (0, 0);
            });
        }

        #[test]
        fn ascii() -> Result<(), String> {
            return test(
                "res/test/simple.ans",
                "type",
                "Character/ASCII",
                &|m: Meta| {
                    return m.r#type == (1, 0);
                },
            );
        }

        #[test]
        fn ansi() -> Result<(), String> {
            return test(
                "res/test/simple.ans",
                "type",
                "Character/ANSI",
                &|m: Meta| {
                    return m.r#type == (1, 1);
                },
            );
        }

        #[test]
        fn unsupported() -> Result<(), String> {
            return test_err("res/test/simple.ans", "type", "foo", &|m: String| {
                return m == "Type is unsupported (foo)";
            });
        }
    }

    #[test]
    fn width() -> Result<(), String> {
        return test("res/test/simple.ans", "width", "1", &|m: Meta| {
            return m.width == 1;
        });
    }

    #[test]
    fn height() -> Result<(), String> {
        return test("res/test/simple.ans", "height", "1", &|m: Meta| {
            return m.height == 1;
        });
    }

    mod flags {
        use super::*;

        #[test]
        fn b_0() -> Result<(), String> {
            return test_err("res/test/simple.ans", "flags", "0x00", &|m: String| {
                return m == "Blink mode is unsupported";
            });
        }

        #[test]
        fn b_1() -> Result<(), String> {
            return test("res/test/simple.ans", "flags", "0x01", &|m: Meta| {
                return m.flags == 0x01;
            });
        }

        #[test]
        fn ls_00() -> Result<(), String> {
            return test("res/test/simple.ans", "flags", "0x01", &|m: Meta| {
                return m.flags == 0x01;
            });
        }

        #[test]
        fn ls_01() -> Result<(), String> {
            return test("res/test/simple.ans", "flags", "0x03", &|m: Meta| {
                return m.flags == 0x03;
            });
        }

        #[test]
        fn ls_10() -> Result<(), String> {
            return test("res/test/simple.ans", "flags", "0x05", &|m: Meta| {
                return m.flags == 0x05;
            });
        }

        #[test]
        fn ls_11() -> Result<(), String> {
            return test_err("res/test/simple.ans", "flags", "0x07", &|m: String| {
                return m == "Invalid letter spacing";
            });
        }

        #[test]
        fn ar_00() -> Result<(), String> {
            return test("res/test/simple.ans", "flags", "0x01", &|m: Meta| {
                return m.flags == 0x01;
            });
        }

        #[test]
        fn ar_01() -> Result<(), String> {
            return test("res/test/simple.ans", "flags", "0x09", &|m: Meta| {
                return m.flags == 0x09;
            });
        }

        #[test]
        fn ar_10() -> Result<(), String> {
            return test("res/test/simple.ans", "flags", "0x11", &|m: Meta| {
                return m.flags == 0x11;
            });
        }

        #[test]
        fn ar_11() -> Result<(), String> {
            return test_err("res/test/simple.ans", "flags", "0x19", &|m: String| {
                return m == "Invalid aspect ratio";
            });
        }

        #[test]
        fn binary() -> Result<(), String> {
            return test("res/test/simple.ans", "flags", "0b00001", &|m: Meta| {
                return m.flags == 0x01;
            });
        }

        #[test]
        fn decimal() -> Result<(), String> {
            return test("res/test/simple.ans", "flags", "1", &|m: Meta| {
                return m.flags == 0x01;
            });
        }

        #[test]
        fn invalid() -> Result<(), String> {
            return test_err("res/test/simple.ans", "flags", "0x21", &|m: String| {
                return m == "Invalid flags";
            });
        }

        #[test]
        fn illegal() -> Result<(), String> {
            return test_err("res/test/simple.ans", "flags", "foo", &|m: String| {
                return m == "Invalid flags (invalid digit found in string)";
            });
        }
    }

    mod font {
        use super::*;

        #[test]
        fn valid() -> Result<(), String> {
            return test("res/test/simple.ans", "font", "IBM VGA 437", &|m: Meta| {
                return m.font == "IBM VGA 437";
            });
        }

        #[test]
        fn invalid() -> Result<(), String> {
            return test_err("res/test/simple.ans", "font", "foo", &|m: String| {
                return m == "Font is unsupported (foo)";
            });
        }
    }

    mod notes {
        use super::*;

        #[test]
        fn empty() -> Result<(), String> {
            return test("res/test/simple.ans", "notes", "", &|m: Meta| {
                return m.notes.is_empty();
            });
        }

        #[test]
        fn single() -> Result<(), String> {
            return test("res/test/simple.ans", "notes", "foo", &|m: Meta| {
                return m.notes == vec!["foo"];
            });
        }

        #[test]
        fn multiple() -> Result<(), String> {
            return test("res/test/simple.ans", "notes", "foo\\nbar", &|m: Meta| {
                return m.notes == vec!["foo", "bar"];
            });
        }

        #[test]
        fn trailing() -> Result<(), String> {
            return test("res/test/simple.ans", "notes", "foo\\n", &|m: Meta| {
                return m.notes == vec!["foo"];
            });
        }

        #[test]
        fn mixed() -> Result<(), String> {
            return test(
                "res/test/simple.ans",
                "notes",
                "foo\\n\\nbar",
                &|m: Meta| {
                    return m.notes == vec!["foo", "bar"];
                },
            );
        }
    }

    fn test<F: FnOnce(Meta) -> bool>(
        input: &str,
        key: &str,
        value: &str,
        check: F,
    ) -> Result<(), String> {
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
                String::from(key),
                String::from(value),
                target.clone(),
            ]),
            ExitCode::OK
        );
        assert!(tmp_dir.path().join("output.txt").exists());
        let meta = meta::get(&target)?;
        assert!(meta.is_some());
        assert!(check(meta.unwrap()));

        tmp_dir.close().map_err(|x| return x.to_string())?;

        return Ok(());
    }

    fn test_err<F: FnOnce(String) -> bool>(
        input: &str,
        key: &str,
        value: &str,
        check: F,
    ) -> Result<(), String> {
        let tmp_dir = tempdir().map_err(|x| return x.to_string())?;
        let target = tmp_dir
            .path()
            .join("output.txt")
            .to_string_lossy()
            .to_string();
        assert!(check(String::from(run(vec![
            String::from("cp437-to-txt"),
            String::from(input),
            String::from(key),
            String::from(value),
            target.clone(),
        ]))));

        tmp_dir.close().map_err(|x| return x.to_string())?;

        return Ok(());
    }
}
