//! Set one field of a file's metadata

use std::{
    env::args,
    io::{stdout, IsTerminal},
};

use cp437_tools::{
    internal::{escape, process, ExitCode, Input, Output},
    prelude::{
        meta::{self, Meta},
        to_cp437,
    },
};

#[allow(dead_code)]
pub fn main() -> ExitCode {
    return run(args().collect());
}

#[inline]
pub fn run(args: Vec<String>) -> ExitCode {
    let exit_code = if args.len() < 2 {
        ExitCode::USAGE(String::from("Missing input file"))
    } else if args.len() < 3 {
        ExitCode::USAGE(String::from("Missing key"))
    } else if args.len() < 4 {
        ExitCode::USAGE(String::from("Missing value"))
    } else if args.len() > 5 {
        ExitCode::USAGE(String::from("Too many arguments"))
    } else if args.len() == 4 && stdout().is_terminal() {
        ExitCode::USAGE(String::from("Refusing to write to terminal"))
    } else {
        process(&args[1], |i, o| return print(i, o, &args[2], &args[3]))
    };

    exit_code.print();
    return exit_code;
}

fn print(input: &mut Input, output: &mut Output, key: &String, value: &String) -> ExitCode {
    let mut meta = input.meta.clone().unwrap_or(Meta {
        size: input.size,
        ..Default::default()
    });

    set_meta(&mut meta, key.to_string(), escape(value.to_string()))?;
    meta::check(Some(&meta))?;

    input.read_by_chunks(|chunk| {
        return output.write(chunk);
    })?;

    return write_meta(output, meta);
}

#[inline]
fn set_meta(meta: &mut Meta, key: String, value: String) -> ExitCode {
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
            return ExitCode::USAGE(String::from("Size can't be changed"));
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
                return ExitCode::USAGE(format!("Type is unsupported ({})", value));
            }
        },
        "width" => {
            meta.width = value
                .parse::<u16>()
                .map_err(|err| return ExitCode::USAGE(format!("Invalid width ({})", err)))?;
        }
        "height" => {
            meta.height = value
                .parse::<u16>()
                .map_err(|err| return ExitCode::USAGE(format!("Invalid height ({})", err)))?;
        }
        "flags" => {
            meta.flags = (if let Some(hex) = value.strip_prefix("0x") {
                u8::from_str_radix(&hex, 16)
            } else if let Some(bin) = value.strip_prefix("0b") {
                u8::from_str_radix(&bin, 2)
            } else {
                value.parse::<u8>()
            })
            .map_err(|err| return ExitCode::USAGE(format!("Invalid flags ({})", err)))?;
        }
        "font" => match value.as_str() {
            "" => {
                meta.font = value.trim().to_string();
            }
            "IBM VGA" => {
                meta.font = value.trim().to_string();
            }
            "IBM VGA 437" => {
                meta.font = value.trim().to_string();
            }
            _ => {
                return ExitCode::USAGE(format!("Font is unsupported ({})", value));
            }
        },
        "notes" => {
            if !value.is_empty() {
                meta.notes = value
                    .split('\n')
                    .map(|note| return note.trim().to_string())
                    .filter(|note| return !note.is_empty())
                    .collect();
            } else {
                meta.notes = vec![];
            }
        }
        _ => {
            return ExitCode::USAGE(format!("Unknown key: {}", key));
        }
    }

    return ExitCode::OK;
}

#[inline]
fn write_meta(output: &mut Output, meta: Meta) -> ExitCode {
    output.write(b"\x1A")?;
    if !meta.notes().is_empty() {
        output.write(b"COMNT")?;
        for note in meta.notes() {
            output.write(&to_cp437(format!("{:<64}", note))?)?;
        }
    }

    output.write(b"SAUCE00")?;
    output.write(&to_cp437(format!("{:<35}", meta.title))?)?;
    output.write(&to_cp437(format!("{:<20}", meta.author))?)?;
    output.write(&to_cp437(format!("{:<20}", meta.group))?)?;
    output.write(&to_cp437(format!("{:<8}", meta.date))?)?;
    output.write(&meta.size.to_le_bytes())?;
    output.write(&[meta.r#type.0, meta.r#type.1])?;
    output.write(&meta.width.to_le_bytes())?;
    output.write(&meta.height.to_le_bytes())?;
    output.write(&0u32.to_le_bytes())?;
    output.write(&[meta.notes().len() as u8])?;
    output.write(&[meta.flags])?;
    output.write(&to_cp437(format!("{:\0<22}", meta.font))?)?;

    return ExitCode::OK;
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
    fn unknown_key() -> Result<(), String> {
        return test::err(
            |i, o| return print(i, o, &String::from("foo"), &String::from("bar")),
            "res/test/simple.ans",
            "Unknown key: foo",
        );
    }

    #[test]
    fn illegal() -> Result<(), String> {
        return test::err(
            |i, o| return print(i, o, &String::from("title"), &String::from("🚫")),
            "res/test/simple.ans",
            "Title contains illegal characters (🚫 (U+1F6AB) is not a valid CP437 character)",
        );
    }

    #[test]
    fn hex() -> Result<(), String> {
        return test::file_meta(
            |i, o| return print(i, o, &String::from("title"), &String::from("\\x40")),
            "res/test/simple.ans",
            Some(Meta {
                title: String::from("@"),
                size: 416,
                ..Default::default()
            }),
        );
    }

    #[test]
    fn unicode() -> Result<(), String> {
        return test::file_meta(
            |i, o| return print(i, o, &String::from("title"), &String::from("\\u3B1")),
            "res/test/simple.ans",
            Some(Meta {
                title: String::from("α"),
                size: 416,
                ..Default::default()
            }),
        );
    }

    #[test]
    fn lf() -> Result<(), String> {
        return test::file_meta(
            |i, o| return print(i, o, &String::from("title"), &String::from("\\n")),
            "res/test/simple.ans",
            Some(Meta {
                title: String::from(""),
                size: 416,
                ..Default::default()
            }),
        );
    }

    #[test]
    fn title() -> Result<(), String> {
        return test::file_meta(
            |i, o| return print(i, o, &String::from("title"), &String::from("TITLE")),
            "res/test/simple.ans",
            Some(Meta {
                title: String::from("TITLE"),
                size: 416,
                ..Default::default()
            }),
        );
    }

    #[test]
    fn author() -> Result<(), String> {
        return test::file_meta(
            |i, o| return print(i, o, &String::from("author"), &String::from("AUTHOR")),
            "res/test/simple.ans",
            Some(Meta {
                author: String::from("AUTHOR"),
                size: 416,
                ..Default::default()
            }),
        );
    }

    #[test]
    fn group() -> Result<(), String> {
        return test::file_meta(
            |i, o| return print(i, o, &String::from("group"), &String::from("GROUP")),
            "res/test/simple.ans",
            Some(Meta {
                group: String::from("GROUP"),
                size: 416,
                ..Default::default()
            }),
        );
    }

    mod date {
        use super::*;

        #[test]
        fn valid() -> Result<(), String> {
            return test::file_meta(
                |i, o| return print(i, o, &String::from("date"), &String::from("19700101")),
                "res/test/simple.ans",
                Some(Meta {
                    date: String::from("19700101"),
                    size: 416,
                    ..Default::default()
                }),
            );
        }

        #[test]
        fn invalid() -> Result<(), String> {
            return test::err(
                |i, o| return print(i, o, &String::from("date"), &String::from("YYYYMMDD")),
                "res/test/simple.ans",
                "Date format is wrong (input contains invalid characters)",
            );
        }
    }

    #[test]
    fn size() -> Result<(), String> {
        return test::err(
            |i, o| return print(i, o, &String::from("size"), &String::from("1")),
            "res/test/simple.ans",
            "Size can't be changed",
        );
    }

    mod r#type {
        use super::*;

        #[test]
        fn none() -> Result<(), String> {
            return test::file_meta(
                |i, o| return print(i, o, &String::from("type"), &String::from("None")),
                "res/test/simple.ans",
                Some(Meta {
                    r#type: (0, 0),
                    size: 416,
                    ..Default::default()
                }),
            );
        }

        #[test]
        fn ascii() -> Result<(), String> {
            return test::file_meta(
                |i, o| {
                    return print(
                        i,
                        o,
                        &String::from("type"),
                        &String::from("Character/ASCII"),
                    );
                },
                "res/test/simple.ans",
                Some(Meta {
                    r#type: (1, 0),
                    size: 416,
                    ..Default::default()
                }),
            );
        }

        #[test]
        fn ansi() -> Result<(), String> {
            return test::file_meta(
                |i, o| {
                    return print(i, o, &String::from("type"), &String::from("Character/ANSI"));
                },
                "res/test/simple.ans",
                Some(Meta {
                    r#type: (1, 1),
                    size: 416,
                    ..Default::default()
                }),
            );
        }

        #[test]
        fn unsupported() -> Result<(), String> {
            return test::err(
                |i, o| return print(i, o, &String::from("type"), &String::from("foo")),
                "res/test/simple.ans",
                "Type is unsupported (foo)",
            );
        }
    }

    #[test]
    fn width() -> Result<(), String> {
        return test::file_meta(
            |i, o| {
                return print(i, o, &String::from("width"), &String::from("1"));
            },
            "res/test/simple.ans",
            Some(Meta {
                width: 1,
                size: 416,
                ..Default::default()
            }),
        );
    }

    #[test]
    fn height() -> Result<(), String> {
        return test::file_meta(
            |i, o| {
                return print(i, o, &String::from("height"), &String::from("1"));
            },
            "res/test/simple.ans",
            Some(Meta {
                height: 1,
                size: 416,
                ..Default::default()
            }),
        );
    }

    mod flags {
        use super::*;

        #[test]
        fn valid() -> Result<(), String> {
            return test::file_meta(
                |i, o| {
                    return print(i, o, &String::from("flags"), &String::from("0x01"));
                },
                "res/test/simple.ans",
                Some(Meta {
                    flags: 0x01,
                    size: 416,
                    ..Default::default()
                }),
            );
        }

        #[test]
        fn binary() -> Result<(), String> {
            return test::file_meta(
                |i, o| {
                    return print(i, o, &String::from("flags"), &String::from("0b00011"));
                },
                "res/test/simple.ans",
                Some(Meta {
                    flags: 0x03,
                    size: 416,
                    ..Default::default()
                }),
            );
        }

        #[test]
        fn hex() -> Result<(), String> {
            return test::file_meta(
                |i, o| {
                    return print(i, o, &String::from("flags"), &String::from("0x03"));
                },
                "res/test/simple.ans",
                Some(Meta {
                    flags: 0x03,
                    size: 416,
                    ..Default::default()
                }),
            );
        }

        #[test]
        fn decimal() -> Result<(), String> {
            return test::file_meta(
                |i, o| {
                    return print(i, o, &String::from("flags"), &String::from("3"));
                },
                "res/test/simple.ans",
                Some(Meta {
                    flags: 0x03,
                    size: 416,
                    ..Default::default()
                }),
            );
        }

        #[test]
        fn unsupported() -> Result<(), String> {
            return test::err(
                |i, o| return print(i, o, &String::from("flags"), &String::from("0x00")),
                "res/test/simple.ans",
                "Blink mode is unsupported",
            );
        }

        #[test]
        fn illegal() -> Result<(), String> {
            return test::err(
                |i, o| return print(i, o, &String::from("flags"), &String::from("x")),
                "res/test/simple.ans",
                "Invalid flags (invalid digit found in string)",
            );
        }
    }

    mod font {
        use super::*;

        #[test]
        fn valid() -> Result<(), String> {
            return test::file_meta(
                |i, o| {
                    return print(i, o, &String::from("font"), &String::from("IBM VGA 437"));
                },
                "res/test/simple.ans",
                Some(Meta {
                    font: String::from("IBM VGA 437"),
                    size: 416,
                    ..Default::default()
                }),
            );
        }

        #[test]
        fn unsupported() -> Result<(), String> {
            return test::err(
                |i, o| return print(i, o, &String::from("font"), &String::from("foo")),
                "res/test/simple.ans",
                "Font is unsupported (foo)",
            );
        }
    }

    mod notes {
        use super::*;

        #[test]
        fn empty() -> Result<(), String> {
            return test::file_meta(
                |i, o| {
                    return print(i, o, &String::from("notes"), &String::from(""));
                },
                "res/test/simple.ans",
                Some(Meta {
                    notes: vec![],
                    size: 416,
                    ..Default::default()
                }),
            );
        }

        #[test]
        fn single() -> Result<(), String> {
            return test::file_meta(
                |i, o| {
                    return print(i, o, &String::from("notes"), &String::from("foo"));
                },
                "res/test/simple.ans",
                Some(Meta {
                    notes: vec![String::from("foo")],
                    size: 416,
                    ..Default::default()
                }),
            );
        }

        #[test]
        fn multiple() -> Result<(), String> {
            return test::file_meta(
                |i, o| {
                    return print(i, o, &String::from("notes"), &String::from("foo\\nbar"));
                },
                "res/test/simple.ans",
                Some(Meta {
                    notes: vec![String::from("foo"), String::from("bar")],
                    size: 416,
                    ..Default::default()
                }),
            );
        }

        #[test]
        fn trailing() -> Result<(), String> {
            return test::file_meta(
                |i, o| {
                    return print(i, o, &String::from("notes"), &String::from("foo\\n"));
                },
                "res/test/simple.ans",
                Some(Meta {
                    notes: vec![String::from("foo")],
                    size: 416,
                    ..Default::default()
                }),
            );
        }

        #[test]
        fn infix_empty() -> Result<(), String> {
            return test::file_meta(
                |i, o| {
                    return print(i, o, &String::from("notes"), &String::from("foo\\n\\nbar"));
                },
                "res/test/simple.ans",
                Some(Meta {
                    notes: vec![String::from("foo"), String::from("bar")],
                    size: 416,
                    ..Default::default()
                }),
            );
        }
    }
}
