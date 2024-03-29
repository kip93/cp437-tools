use std::{
    fs::File,
    io::{stdout, Write},
};

use crate::{
    meta::{self, Meta},
    ExitCode,
};

#[doc(hidden)] // Internal impl detail
#[inline]
pub fn process<
    'a,
    F: for<'b> FnOnce(&'b mut File, &'b mut Box<dyn Write>, Option<Meta>) -> Result<(), ExitCode> + 'a,
>(
    input: &String,
    output: &Option<String>,
    callback: F,
) -> ExitCode {
    match wrapped_process(input, output, callback) {
        Ok(_) => {
            return ExitCode::OK;
        }
        Err(x) => {
            eprintln!("\x1B[31mERROR: {}\x1B[0m", x);
            return x;
        }
    }
}

#[inline]
fn wrapped_process<
    'a,
    F: for<'b> FnOnce(&'b mut File, &'b mut Box<dyn Write>, Option<Meta>) -> Result<(), ExitCode> + 'a,
>(
    input: &String,
    output: &Option<String>,
    callback: F,
) -> Result<(), ExitCode> {
    let mut input = File::open(input).map_err(|x| return ExitCode::ERROR(x.to_string()))?;
    let meta = meta::read(&mut input).map_err(|x| return ExitCode::FAIL(x))?;
    meta::check(&meta).map_err(|x| return ExitCode::FAIL(x))?;
    let mut output = match output {
        Some(filename) => {
            Box::new(File::create_new(filename).map_err(|x| return ExitCode::ERROR(x.to_string()))?)
                as Box<dyn Write>
        }
        None => Box::new(stdout()) as Box<dyn Write>,
    };

    return callback(&mut input, &mut output, meta);
}
