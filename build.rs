use indoc::indoc;
use std::{
    ffi::{OsStr, OsString},
    fs::{self, create_dir_all, remove_dir_all, File},
    io::{self, Write},
    path::Path,
    process::{Command, Stdio},
};

fn main() -> Result<(), io::Error> {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=res/man");

    // This does not build on docs.rs
    if std::env::var("DOCS_RS").is_err() {
        let man_dir = Path::new("res/man").canonicalize()?;
        if Path::new("target/man").exists() {
            remove_dir_all("target/man")?;
        }
        create_dir_all("target/man/tmp")?;

        let mut entries = man_dir
            .read_dir()?
            .map(|result| return result.map(|entry| return entry.path()))
            .collect::<Result<Vec<_>, io::Error>>()?;
        entries.sort();
        for raw_path in &entries {
            let man_path =
                Path::new("target/man").join(raw_path.strip_prefix(man_dir.clone()).unwrap());
            fs::copy(raw_path.clone(), man_path.clone())?;
            let mut man_file = File::options().append(true).open(man_path.clone())?;
            write!(
                &mut man_file,
                indoc! {"
                    .\" -----------------------------------------------------------------------------
                    .SH COPYRIGHT
                    License GPLv3+: GNU GPL version 3 or later <https://gnu.org/licenses/gpl.html>
                    .PP
                    This is free software: you are free to change and redistribute it. There is NO
                    WARRANTY, to the extent permitted by law.
                "}
            )?;

            write!(
                &mut man_file,
                indoc! {"
                    .\" -----------------------------------------------------------------------------
                    .SH SEE ALSO
                "}
            )?;
            for (i, ref_path) in entries
                .iter()
                .filter(|ref_path| {
                    return ref_path.file_name().unwrap() != raw_path.file_name().unwrap();
                })
                .enumerate()
            {
                writeln!(
                    &mut man_file,
                    ".BR {} ({}){}",
                    ref_path.file_stem().unwrap().to_str().unwrap(),
                    ref_path.extension().unwrap().to_str().unwrap(),
                    if i + 2 < entries.len() { "," } else { "" }
                )?;
            }
            write!(
                &mut man_file,
                indoc! {"
                    .\" -----------------------------------------------------------------------------
                    .pl \\n[nl]u
                "}
            )?;

            let man_gz_file = File::create(man_path.with_extension({
                let mut old_extension =
                    OsString::from(man_path.extension().unwrap_or(OsStr::new("")));
                old_extension.push(OsStr::new(".gz"));

                old_extension
            }))?;
            assert!(
                Command::new("gzip")
                    .arg("-f9ck")
                    .arg(man_path.clone())
                    .stdout(Stdio::from(man_gz_file))
                    .status()
                    .expect("Failed to gzip manpage")
                    .success(),
                "Failed to gzip manpage"
            );

            if raw_path.extension().unwrap_or(OsStr::new("")) == "1" {
                let tmp_path = Path::new("target/man/tmp")
                    .join(raw_path.strip_prefix(man_dir.clone()).unwrap());
                let mut tmp_file = File::create(tmp_path.clone())?;
                write!(
                    &mut tmp_file,
                    indoc! {"
                        .de PT
                        ..
                        .de BT
                        ..
                    "}
                )?;
                io::copy(&mut File::open(raw_path.clone())?, &mut tmp_file)?;
                write!(
                    &mut tmp_file,
                    indoc! {"
                        .\" -----------------------------------------------------------------------------
                        .pl \\n[nl]u
                    "}
                )?;
                tmp_file.flush()?;
                let txt_path = Path::new("target/man")
                    .join(raw_path.strip_prefix(man_dir.clone()).unwrap())
                    .with_extension(OsStr::new("txt"));
                let txt_file = File::create(txt_path)?;
                assert!(
                    Command::new("groff")
                        .arg("-man")
                        .arg("-tTutf8")
                        .arg(tmp_path.clone())
                        .stdout(Stdio::from(txt_file))
                        .status()
                        .expect("Failed to render manpage")
                        .success(),
                    "Failed to render manpage"
                );
            }
        }

        remove_dir_all("target/man/tmp")?;
    }

    return Ok(());
}
