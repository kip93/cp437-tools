#![allow(dead_code)]

use file_diff::diff;
use pretty_assertions::assert_eq;
use std::{fs::File, io::Read};
use tempfile::tempdir;

use cp437_tools::{
    internal::{ExitCode, Input, Output},
    prelude::meta::{self, Meta},
};

pub fn ok<F: for<'a> FnOnce(&'a mut Input, &'a mut Output) -> ExitCode>(
    callback: F,
    input: &str,
    output: &str,
) -> Result<(), String> {
    let tmp_dir = tempdir().map_err(|err| return err.to_string())?;
    let target = tmp_dir.path().join("output.txt").to_string_lossy().to_string();

    assert!(callback(&mut Input::new(&String::from(input))?, &mut Output::file(&target)?).is_ok());
    assert!(tmp_dir.path().join("output.txt").exists());
    let mut buffer = String::new();
    File::open(tmp_dir.path().join("output.txt"))
        .map_err(|err| return err.to_string())?
        .read_to_string(&mut buffer)
        .map_err(|err| return err.to_string())?;
    assert_eq!(buffer, output);

    tmp_dir.close().map_err(|err| return err.to_string())?;

    return Ok(());
}

pub fn err<F: for<'a> FnOnce(&'a mut Input, &'a mut Output) -> ExitCode>(
    callback: F,
    input: &str,
    output: &str,
) -> Result<(), String> {
    let tmp_dir = tempdir().map_err(|err| return err.to_string())?;
    let target = tmp_dir.path().join("output.txt").to_string_lossy().to_string();

    let result = callback(&mut Input::new(&String::from(input))?, &mut Output::file(&target)?);
    assert!(result.is_err());
    assert_eq!(String::from(result), output);

    tmp_dir.close().map_err(|err| return err.to_string())?;

    return Ok(());
}

pub fn file<F: for<'a> FnOnce(&'a mut Input, &'a mut Output) -> ExitCode>(
    callback: F,
    input: &str,
    output: &str,
) -> Result<(), String> {
    let tmp_dir = tempdir().map_err(|err| return err.to_string())?;
    let target = tmp_dir.path().join("output.txt").to_string_lossy().to_string();

    assert!(callback(&mut Input::new(&String::from(input))?, &mut Output::file(&target)?).is_ok());
    assert!(tmp_dir.path().join("output.txt").exists());
    assert!(diff(&target, output));

    tmp_dir.close().map_err(|err| return err.to_string())?;

    return Ok(());
}

pub fn file_meta<F: for<'a> FnOnce(&'a mut Input, &'a mut Output) -> ExitCode>(
    callback: F,
    input: &str,
    output: Option<Meta>,
) -> Result<(), String> {
    let tmp_dir = tempdir().map_err(|err| return err.to_string())?;
    let target = tmp_dir.path().join("output.txt").to_string_lossy().to_string();

    assert!(callback(&mut Input::new(&String::from(input))?, &mut Output::file(&target)?).is_ok());
    assert!(tmp_dir.path().join("output.txt").exists());
    assert_eq!(meta::get(&target)?, output);

    tmp_dir.close().map_err(|err| return err.to_string())?;

    return Ok(());
}

pub fn file_err<F: for<'a> FnOnce(&'a mut Input, &'a mut Output) -> ExitCode>(
    callback: F,
    input: &str,
    output: &str,
) -> Result<(), String> {
    let tmp_dir = tempdir().map_err(|err| return err.to_string())?;
    let target = tmp_dir.path().join("output.txt").to_string_lossy().to_string();

    assert!(callback(&mut Input::new(&String::from(input))?, &mut Output::file(&target)?).is_err());
    assert!(tmp_dir.path().join("output.txt").exists());
    let mut buffer = String::new();
    File::open(tmp_dir.path().join("output.txt"))
        .map_err(|err| return err.to_string())?
        .read_to_string(&mut buffer)
        .map_err(|err| return err.to_string())?;
    assert_eq!(buffer, output);

    tmp_dir.close().map_err(|err| return err.to_string())?;

    return Ok(());
}
