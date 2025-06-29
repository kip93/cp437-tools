#![expect(missing_docs, reason = "Just a humble build script")]
#![expect(clippy::missing_docs_in_private_items, reason = "Just a humble build script")]
#![expect(clippy::too_many_lines, reason = "Not much that can be done here")]
#![expect(clippy::unwrap_used, reason = "These are build-time panics")]

use indoc::indoc;
use std::{
    env,
    ffi::{OsStr, OsString},
    fs::{copy, create_dir_all, remove_dir_all, File},
    io::{self, Write as _},
    path::Path,
    process::{Command, Stdio},
};

pub fn main() -> Result<(), io::Error> {
    let binding = env::var("OUT_DIR").unwrap();
    let out_dir = Path::new(&binding);
    man(out_dir)?;
    doc(out_dir)?;

    return Ok(());
}

fn man(out_dir: &Path) -> Result<(), io::Error> {
    let src_dir = Path::new("res/man").canonicalize()?;
    let dst_dir = out_dir.join("man");
    if dst_dir.exists() {
        remove_dir_all(&dst_dir)?;
    }
    create_dir_all(dst_dir.join("tmp"))?;

    let mut entries = src_dir
        .read_dir()?
        .map(|result| return result.map(|entry| return entry.path()))
        .collect::<Result<Vec<_>, io::Error>>()?;
    entries.sort();
    for raw_path in &entries {
        let man_path = dst_dir.join(raw_path.strip_prefix(&src_dir).unwrap());
        copy(raw_path, &man_path)?;
        let mut man_file = File::options().append(true).open(&man_path)?;
        write!(
            &mut man_file,
            indoc! {"
                .\" -----------------------------------------------------------------------------
                .SH COPYRIGHT
                License GPLv3+: GNU GPL version 3 or later <https://gnu.org/licenses/gpl.html>
                .PP
                This is free software: you are free to change and redistribute it. There is NO
                WARRANTY, to the extent permitted by law.
            "},
        )?;

        write!(
            &mut man_file,
            indoc! {"
                .\" -----------------------------------------------------------------------------
                .SH SEE ALSO
            "},
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
                if i + 2 < entries.len() { "," } else { "" },
            )?;
        }
        write!(
            &mut man_file,
            indoc! {"
                .\" -----------------------------------------------------------------------------
                .pl \\n[nl]u
            "},
        )?;

        let man_gz_file = File::create(man_path.with_extension({
            let mut old_extension = OsString::from(man_path.extension().unwrap_or(OsStr::new("")));
            old_extension.push(OsStr::new(".gz"));

            old_extension
        }))?;
        assert!(
            Command::new("gzip")
                .arg("-f9ck")
                .arg(man_path)
                .stdout(Stdio::from(man_gz_file))
                .status()
                .expect("Failed to gzip manpage")
                .success(),
            "Failed to gzip manpage",
        );

        if raw_path.extension().unwrap_or(OsStr::new("")) == "1" {
            let tmp_path = dst_dir.join("tmp").join(raw_path.strip_prefix(&src_dir).unwrap());
            let mut tmp_file = File::create(&tmp_path)?;
            write!(
                &mut tmp_file,
                indoc! {"
                    .de PT
                    ..
                    .de BT
                    ..
                "},
            )?;
            io::copy(&mut File::open(raw_path)?, &mut tmp_file)?;
            write!(
                &mut tmp_file,
                indoc! {"
                    .\" -----------------------------------------------------------------------------
                    .pl \\n[nl]u
                "},
            )?;
            tmp_file.flush()?;
            let txt_path = dst_dir.join(raw_path.strip_prefix(&src_dir).unwrap()).with_extension(OsStr::new("txt"));
            let txt_file = File::create(txt_path)?;
            assert!(
                Command::new("groff")
                    .arg("-man")
                    .arg("-tTutf8")
                    .arg(tmp_path)
                    .stdout(Stdio::from(txt_file))
                    .status()
                    .expect("Failed to render manpage")
                    .success(),
                "Failed to render manpage",
            );
        }
    }

    remove_dir_all(dst_dir.join("tmp"))?;

    return Ok(());
}

fn doc(out_dir: &Path) -> Result<(), io::Error> {
    let src = Path::new("FILE_ID.DIZ").canonicalize()?;
    let dst = out_dir.join("doc");
    if dst.exists() {
        remove_dir_all(&dst)?;
    }
    create_dir_all(&dst)?;
    copy(src, dst.join("FILE_ID.DIZ"))?;

    return Ok(());
}
