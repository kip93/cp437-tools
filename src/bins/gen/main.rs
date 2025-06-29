//! Create some auto-generated files.

use std::{env::args, fs::remove_file, path::Path, process::Command};

use itertools::izip;
use strum::IntoEnumIterator as _;

use cp437_tools::{
    internal::{ExitCode, Input, Output},
    prelude::ColourScheme,
};

use crate::cmd_to_png;

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
pub fn exec(_args: &[String]) -> ExitCode {
    for scheme in ColourScheme::iter() {
        if let ColourScheme::CUSTOM(_) = scheme {
        } else {
            let exit_code = run(&scheme);
            exit_code.print();
            exit_code?;
        }
    }

    return ExitCode::OK;
}

#[allow(missing_docs, reason = "Just an entry point")]
#[allow(clippy::missing_docs_in_private_items, reason = "Just an entry point")]
pub fn run(scheme: &ColourScheme) -> ExitCode {
    generate_ans(scheme)?;
    generate_png(scheme)?;

    return ExitCode::OK;
}

/// Generate .ans files for each colour scheme.
fn generate_ans(scheme: &ColourScheme) -> ExitCode {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("res/schemes/{}.ans", scheme.name()));
    if path.exists() {
        remove_file(&path)?;
    }
    let mut output = Output::file(path)?;

    let colours = scheme.colours();
    for (index, left, right) in izip!(0..8, &colours[..8], &colours[8..]) {
        output.write(
            format!(
                "[0;4{}m  [0m [1;37m#{:02X}{:02X}{:02X}[0m [0;10{}m  [0m [1;37m#{:02X}{:02X}{:02X}[0m ",
                index, left[0], left[1], left[2], index, right[0], right[1], right[2],
            )
            .as_bytes(),
        )?;
    }

    output.write("COMNTCopyright: CC BY-NC-SA 4.0                                      ".as_bytes())?;
    output.write("SAUCE00".as_bytes())?;
    output.write(format!("{:<35}", format!("CP437 tools {} theme", scheme.name())).as_bytes())?;
    output.write(format!("{:<20}", "kip93").as_bytes())?;
    output.write(format!("{:<20}", "").as_bytes())?;
    output.write("19700101".as_bytes())?;
    output.write(&536_u32.to_le_bytes())?;
    output.write(&[1, 1])?;
    output.write(&22_u16.to_le_bytes())?;
    output.write(&8_u16.to_le_bytes())?;
    output.write(&[0, 0, 0, 0])?;
    output.write(&[1])?;
    output.write(&[0x0D])?;
    output.write(format!("{:\0<22}", "IBM VGA").as_bytes())?;

    return ExitCode::OK;
}

/// Generate .png files for each colour scheme.
fn generate_png(scheme: &ColourScheme) -> ExitCode {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("res/schemes/{}.ans", scheme.name()));
    let mut input = Input::new(&path)?;
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("res/schemes/{}.png", scheme.name()));
    if path.exists() {
        remove_file(&path)?;
    }
    let mut output = Output::file(&path)?;

    cmd_to_png::run(&mut input, &mut output, &scheme.name());

    assert!(Command::new("magick")
        .arg(&path)
        .arg("-resize")
        .arg("10%")
        .arg(&path)
        .status()
        .expect("Failed to execute process")
        .success());

    return ExitCode::OK;
}
