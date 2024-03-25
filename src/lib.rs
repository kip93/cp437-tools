//! A small collection of tools to handle CP437 files.
//!
//! <div class="warning">
//!
//! This crate is primary written to supply CLI commands, not as a reusable
//! library.
//!
//! While I'll try and avoid making changes to the API, be warned that (at
//! least, at the moment) no guarantees are made about its stability.
//!
//! </div>
//!

#![deny(missing_docs)]

pub mod colour;
pub mod cp437;
pub mod fonts;
#[doc(hidden)]
pub mod help;
pub mod meta;

pub use self::{colour::COLOURS, cp437::*, meta::Meta};

use std::{
    fs::File,
    io::{stdout, Write},
    process::ExitCode,
};

#[doc(hidden)]
#[inline]
pub fn process<
    'a,
    F: for<'b> FnOnce(&'b mut File, &'b mut Box<dyn Write>, Option<Meta>) -> Result<(), String> + 'a,
>(
    input: &String,
    output: &Option<String>,
    callback: F,
) -> ExitCode {
    match wrapped_process(input, output, callback) {
        Ok(_) => {
            return ExitCode::from(0);
        }
        Err(msg) => {
            eprintln!("\x1B[31mERROR: {}\x1B[0m", msg);
            return ExitCode::from(2);
        }
    }
}

#[inline]
fn wrapped_process<
    'a,
    F: for<'b> FnOnce(&'b mut File, &'b mut Box<dyn Write>, Option<Meta>) -> Result<(), String> + 'a,
>(
    input: &String,
    output: &Option<String>,
    callback: F,
) -> Result<(), String> {
    let mut input = File::open(input).map_err(|x| return x.to_string())?;
    let meta = meta::read(&mut input)?;
    meta::check(&meta)?;
    let mut output = match output {
        Some(filename) => Box::new(File::create_new(filename).map_err(|x| return x.to_string())?)
            as Box<dyn Write>,
        None => Box::new(stdout()) as Box<dyn Write>,
    };

    return callback(&mut input, &mut output, meta);
}
