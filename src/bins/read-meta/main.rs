//! Read a file's metadata.

use humansize::{format_size, BINARY};
use std::{cmp::Ordering, env::args};

use cp437_tools::{
    internal::{process, ExitCode, Input, Output},
    prelude::meta::{self, Meta},
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
    output.write(
        format!(
            "\x1B[{}mMetadata\x1B[0m:\n",
            match input.meta {
                Some(_) => "4",
                None => "4;33",
            }
        )
        .as_bytes(),
    )?;

    let meta = input.meta.clone().unwrap_or(Meta {
        size: input.size,
        r#type: (0, 0),
        width: 0,
        height: 0,
        flags: 0x01,
        font: String::default(),
        ..Default::default()
    });

    print_title(output, &meta)?;
    print_author(output, &meta)?;
    print_group(output, &meta)?;
    print_date(output, &meta)?;
    print_size(output, &meta)?;
    print_type(output, &meta)?;
    print_width(output, &meta)?;
    print_height(output, &meta)?;
    print_flags(output, &meta)?;
    print_font(output, &meta)?;
    print_notes(output, &meta)?;

    return ExitCode::OK;
}

/// Show the file's title if present.
#[inline]
fn print_title(output: &mut Output, meta: &Meta) -> ExitCode {
    if meta.title().is_some() {
        output.write(
            format!(
                "* \x1B[1mTitle\x1B[0m: \x1B[{}m{:?}\x1B[0m\n",
                if meta::check_title(Some(meta)).is_err() { "1;3;31" } else { "3;32" },
                meta.title,
            )
            .as_bytes(),
        )?;
    }

    return ExitCode::OK;
}

/// Show the file's author if present.
#[inline]
fn print_author(output: &mut Output, meta: &Meta) -> ExitCode {
    if meta.author().is_some() {
        output.write(
            format!(
                "* \x1B[1mAuthor\x1B[0m: \x1B[{}m{:?}\x1B[0m\n",
                if meta::check_author(Some(meta)).is_err() { "1;3;31" } else { "3;32" },
                meta.author,
            )
            .as_bytes(),
        )?;
    }

    return ExitCode::OK;
}

/// Show the file's author's team or group if present.
#[inline]
fn print_group(output: &mut Output, meta: &Meta) -> ExitCode {
    if meta.group().is_some() {
        output.write(
            format!(
                "* \x1B[1mGroup\x1B[0m: \x1B[{}m{:?}\x1B[0m\n",
                if meta::check_group(Some(meta)).is_err() { "1;3;31" } else { "3;32" },
                meta.group,
            )
            .as_bytes(),
        )?;
    }

    return ExitCode::OK;
}

/// Show the file's date if present.
#[inline]
fn print_date(output: &mut Output, meta: &Meta) -> ExitCode {
    if meta.date().is_some() {
        output.write(
            format!(
                "* \x1B[1mDate\x1B[0m: \x1B[{}m{}/{}/{}\x1B[0m\n",
                if meta::check_date(Some(meta)).is_err() { "1;3;31" } else { "3;32" },
                &meta.date[0..4],
                &meta.date[4..6],
                &meta.date[6..8],
            )
            .as_bytes(),
        )?;
    }

    return ExitCode::OK;
}

/// Show the file's size.
#[inline]
fn print_size(output: &mut Output, meta: &Meta) -> ExitCode {
    output.write(format!("* \x1B[1mSize\x1B[0m: \x1B[3m{}\x1B[0m\n", format_size(meta.size, BINARY)).as_bytes())?;

    return ExitCode::OK;
}

/// Show the file's type.
#[inline]
fn print_type(output: &mut Output, meta: &Meta) -> ExitCode {
    output.write(
        format!(
            "* \x1B[1mType\x1B[0m: {}\x1B[0m\n",
            match meta.r#type {
                (0, _) => format!("\x1B[1;3;33mNone ({})", meta::type_name(Meta::default().r#type)),
                (1, 0 | 1) => format!("\x1B[3;32m{}", meta::type_name(meta.r#type)),
                _ => format!("\x1B[1;3;31m{}", meta::type_name(meta.r#type)),
            },
        )
        .as_bytes(),
    )?;

    return ExitCode::OK;
}

/// Show the file's width.
#[inline]
fn print_width(output: &mut Output, meta: &Meta) -> ExitCode {
    output.write(
        format!(
            "* \x1B[1mWidth\x1B[0m: \x1B[{}m{} chars\x1B[0m\n",
            if meta.width > 0 { "3;32" } else { "1;3;33" },
            meta.width(),
        )
        .as_bytes(),
    )?;

    return ExitCode::OK;
}

/// Show the file's height.
#[inline]
fn print_height(output: &mut Output, meta: &Meta) -> ExitCode {
    output.write(
        format!(
            "* \x1B[1mHeight\x1B[0m: \x1B[{}m{} chars\x1B[0m\n",
            if meta.height > 0 { "3;32" } else { "1;3;33" },
            meta.height(),
        )
        .as_bytes(),
    )?;

    return ExitCode::OK;
}

/// Show the file's flags.
#[inline]
fn print_flags(output: &mut Output, meta: &Meta) -> ExitCode {
    output.write(
        format!(
            "* \x1B[1mFlags\x1B[0m: {}\x1B[0m\n",
            if meta.flags().0 == 0b11 || meta.flags().1 == 0b11 || meta.flags().2 == 0b0 {
                format!("\x1B[3;31m{:02X}h", meta.flags)
            } else if meta.flags().0 == 0b00 || meta.flags().1 == 0b00 {
                format!(
                    "\x1B[3;33m{:02X}h ({:02X}h)",
                    meta.flags,
                    meta.flags
                        | (if meta.flags().0 == 0b00 { 0x08 } else { 0x00 })
                        | (if meta.flags().1 == 0b00 { 0x04 } else { 0x00 }),
                )
            } else {
                format!("\x1B[3;32m{:02X}h", meta.flags)
            },
        )
        .as_bytes(),
    )?;

    return ExitCode::OK;
}

/// Show the file's font.
#[inline]
fn print_font(output: &mut Output, meta: &Meta) -> ExitCode {
    output.write(
        format!(
            "* \x1B[1mFont\x1B[0m: {}\x1B[0m\n",
            if let Some(font) = meta.font() {
                if ["IBM VGA", "IBM VGA 437"].contains(&font.as_str()) {
                    format!("\x1B[3;32m{font:?}")
                } else {
                    format!("\x1B[1;3;31m{font:?}")
                }
            } else {
                format!("\x1B[1;3;33m<N/A> ({})", Meta::default().font)
            },
        )
        .as_bytes(),
    )?;

    return ExitCode::OK;
}

/// Show the file's comments if present.
#[inline]
#[expect(clippy::cast_possible_truncation, reason = "Range is [0,3]")]
#[expect(clippy::cast_sign_loss, reason = "Range is [0,3]")]
#[expect(clippy::cast_precision_loss, reason = "Range is [0,3]")]
fn print_notes(output: &mut Output, meta: &Meta) -> ExitCode {
    for (i, note) in meta.notes().iter().enumerate() {
        output.write(
            format!(
                "* \x1B[1mNotes[{:0width$}]\x1B[0m: \x1B[{}m{:?}\x1B[0m\n",
                i,
                if meta::check_note(Some(meta), i).is_err() { "1;3;31" } else { "3;32" },
                note,
                width = (meta.notes().len() as f32).log10().ceil() as usize,
            )
            .as_bytes(),
        )?;
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
        assert_eq!(exec(&[String::from("cp437-read-meta")]), ExitCode::USAGE(String::from("Missing input file")));
    }

    #[test]
    fn too_many_args() {
        assert_eq!(
            exec(&[String::from("cp437-read-meta"), String::from("a"), String::from("b")]),
            ExitCode::USAGE(String::from("Too many arguments")),
        );
    }

    #[test]
    fn simple() -> Result<(), String> {
        return test::ok(
            run,
            "res/test/simple.ans",
            indoc! {"
                \x1B[4;33mMetadata\x1B[0m:
                * \x1B[1mSize\x1B[0m: \x1B[3m416 B\x1B[0m
                * \x1B[1mType\x1B[0m: \x1B[1;3;33mNone (Character/ANSi)\x1B[0m
                * \x1B[1mWidth\x1B[0m: \x1B[1;3;33m80 chars\x1B[0m
                * \x1B[1mHeight\x1B[0m: \x1B[1;3;33m25 chars\x1B[0m
                * \x1B[1mFlags\x1B[0m: \x1B[3;33m01h (0Dh)\x1B[0m
                * \x1B[1mFont\x1B[0m: \x1B[1;3;33m<N/A> (IBM VGA)\x1B[0m
            "},
        );
    }

    #[test]
    fn meta() -> Result<(), String> {
        return test::ok(
            run,
            "res/test/meta.ans",
            indoc! {"
                \x1B[4mMetadata\x1B[0m:
                * \x1B[1mTitle\x1B[0m: \x1B[3;32m\"TITLE\"\x1B[0m
                * \x1B[1mAuthor\x1B[0m: \x1B[3;32m\"AUTHOR\"\x1B[0m
                * \x1B[1mGroup\x1B[0m: \x1B[3;32m\"GROUP\"\x1B[0m
                * \x1B[1mDate\x1B[0m: \x1B[3;32m1970/01/01\x1B[0m
                * \x1B[1mSize\x1B[0m: \x1B[3m416 B\x1B[0m
                * \x1B[1mType\x1B[0m: \x1B[3;32mCharacter/ANSi\x1B[0m
                * \x1B[1mWidth\x1B[0m: \x1B[3;32m32 chars\x1B[0m
                * \x1B[1mHeight\x1B[0m: \x1B[3;32m8 chars\x1B[0m
                * \x1B[1mFlags\x1B[0m: \x1B[3;33m01h (0Dh)\x1B[0m
                * \x1B[1mFont\x1B[0m: \x1B[3;32m\"IBM VGA\"\x1B[0m
            "},
        );
    }

    #[test]
    fn notes() -> Result<(), String> {
        return test::ok(
            run,
            "res/test/comments.ans",
            indoc! {"
                \x1B[4mMetadata\x1B[0m:
                * \x1B[1mTitle\x1B[0m: \x1B[3;32m\"TITLE\"\x1B[0m
                * \x1B[1mAuthor\x1B[0m: \x1B[3;32m\"AUTHOR\"\x1B[0m
                * \x1B[1mGroup\x1B[0m: \x1B[3;32m\"GROUP\"\x1B[0m
                * \x1B[1mDate\x1B[0m: \x1B[3;32m1970/01/01\x1B[0m
                * \x1B[1mSize\x1B[0m: \x1B[3m416 B\x1B[0m
                * \x1B[1mType\x1B[0m: \x1B[3;32mCharacter/ANSi\x1B[0m
                * \x1B[1mWidth\x1B[0m: \x1B[3;32m32 chars\x1B[0m
                * \x1B[1mHeight\x1B[0m: \x1B[3;32m8 chars\x1B[0m
                * \x1B[1mFlags\x1B[0m: \x1B[3;33m01h (0Dh)\x1B[0m
                * \x1B[1mFont\x1B[0m: \x1B[3;32m\"IBM VGA\"\x1B[0m
                * \x1B[1mNotes[0]\x1B[0m: \x1B[3;32m\"Lorem\"\x1B[0m
                * \x1B[1mNotes[1]\x1B[0m: \x1B[3;32m\"ipsum\"\x1B[0m
                * \x1B[1mNotes[2]\x1B[0m: \x1B[3;32m\"dolor\"\x1B[0m
                * \x1B[1mNotes[3]\x1B[0m: \x1B[3;32m\"sit\"\x1B[0m
                * \x1B[1mNotes[4]\x1B[0m: \x1B[3;32m\"amet\"\x1B[0m
            "},
        );
    }
}
