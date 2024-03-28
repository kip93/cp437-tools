//! Handling of file's metadata
//!
//! See <https://www.acid.org/info/sauce/sauce.htm>
//!

use chrono::NaiveDate;
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    str,
};

use crate::cp437::*;

/// A structure representing a file's metadata
#[doc(alias = "Sauce")]
#[derive(Clone)]
pub struct Meta {
    /// The image's title
    pub title: String,
    /// The image's author
    pub author: String,
    /// The image author's team or group
    #[doc(alias = "team")]
    pub group: String,
    /// The image creation date, in the YYYYMMDD format
    pub date: String,
    /// The size of the file, sans this metadata
    pub size: u32,
    /// The type of this file
    ///
    /// Only supported values are
    /// * `(1, 0)` → `Character/ASCII`
    /// * `(1, 1)` → `Character/ANSI`
    ///
    /// See <https://www.acid.org/info/sauce/sauce.htm#FileType>
    ///
    pub r#type: (u8, u8),
    /// Width of the image
    pub width: u16,
    /// Height of the image
    pub height: u16,
    /// A bitfield of flags that define how to process an image
    ///
    /// See <https://www.acid.org/info/sauce/sauce.htm#ANSiFlags>
    ///
    #[doc(alias = "AR")]
    #[doc(alias = "aspect ratio")]
    #[doc(alias = "LS")]
    #[doc(alias = "letter spacing")]
    #[doc(alias = "B")]
    #[doc(alias = "ice colour")]
    #[doc(alias = "non-blink mode")]
    pub flags: u8,
    /// The name of the font this image uses
    ///
    /// Only IBM VGA is supported.
    ///
    pub font: String,
    /// A list of comments on this image
    #[doc(alias = "comments")]
    pub notes: Vec<String>,
}

/// An empty meta
///
/// Only field set is the iCE colours flag, since blinking mode is not
/// supported.
///
impl Default for Meta {
    fn default() -> Meta {
        return Meta {
            title: String::from(""),
            author: String::from(""),
            group: String::from(""),
            date: String::from(""),
            size: 0,
            r#type: (0, 0),
            width: 0,
            height: 0,
            flags: 1,
            font: String::from(""),
            notes: vec![],
        };
    }
}

/// Get a file's metadata via its path
///
/// Arguments:
/// * `path`: Path pointing to file. Can be relative to cwd.
///
pub fn get(path: &str) -> Result<Option<Meta>, String> {
    return read(&mut File::open(path).map_err(|x| return x.to_string())?);
}

/// Get a file's metadata via a file reference
///
/// Arguments:
/// * `file`: File to read
///
pub fn read(file: &mut File) -> Result<Option<Meta>, String> {
    return read_raw(file).map(|x| {
        return x.map(|meta| {
            return Meta {
                title: meta[meta.len() - 121..meta.len() - 86]
                    .iter()
                    .map(|x| return CP437_TO_UTF8[*x as usize])
                    .collect::<String>()
                    .trim()
                    .to_string(),
                author: meta[meta.len() - 86..meta.len() - 66]
                    .iter()
                    .map(|x| return CP437_TO_UTF8[*x as usize])
                    .collect::<String>()
                    .trim()
                    .to_string(),
                group: meta[meta.len() - 66..meta.len() - 46]
                    .iter()
                    .map(|x| return CP437_TO_UTF8[*x as usize])
                    .collect::<String>()
                    .trim()
                    .to_string(),
                date: meta[meta.len() - 46..meta.len() - 38]
                    .iter()
                    .map(|x| return CP437_TO_UTF8[*x as usize])
                    .collect::<String>()
                    .trim()
                    .to_string(),
                size: u32::from_le_bytes(
                    meta[meta.len() - 38..meta.len() - 34].try_into().unwrap(),
                ),
                r#type: (meta[meta.len() - 34], meta[meta.len() - 33]),
                width: u16::from_le_bytes(
                    meta[meta.len() - 32..meta.len() - 30].try_into().unwrap(),
                ),
                height: u16::from_le_bytes(
                    meta[meta.len() - 30..meta.len() - 28].try_into().unwrap(),
                ),
                flags: meta[meta.len() - 23],
                font: meta[meta.len() - 22..]
                    .iter()
                    .map(|x| return CP437_TO_UTF8[*x as usize])
                    .collect::<String>()
                    .trim()
                    .to_string(),
                notes: (0..meta[meta.len() - 24] as usize)
                    .rev()
                    .map(|i| {
                        let offset = meta.len() - (i + 3) * 64;
                        return meta[offset..offset + 64]
                            .iter()
                            .map(|x| return CP437_TO_UTF8[*x as usize])
                            .collect::<String>()
                            .trim()
                            .to_string();
                    })
                    .collect(),
            };
        });
    });
}

/// Check that a given file's metadata is valid and supported
///
/// Arguments:
/// * `meta`: The metadata to check
///
pub fn check(meta: &Option<Meta>) -> Result<(), String> {
    check_title(meta)?;
    check_author(meta)?;
    check_group(meta)?;
    check_date(meta)?;
    check_type(meta)?;
    check_flags(meta)?;
    check_font(meta)?;
    check_notes(meta)?;

    return Ok(());
}

/// Check that the title is valid
///
/// Arguments:
/// * `meta`: The metadata to check
///
pub fn check_title(meta: &Option<Meta>) -> Result<(), String> {
    return meta
        .as_ref()
        .map(|m| return check_str(&m.title, "Title", 35))
        .unwrap_or(Ok(()));
}

/// Check that the author is valid
///
/// Arguments:
/// * `meta`: The metadata to check
///
pub fn check_author(meta: &Option<Meta>) -> Result<(), String> {
    return meta
        .as_ref()
        .map(|m| return check_str(&m.author, "Author", 20))
        .unwrap_or(Ok(()));
}

/// Check that the group is valid
///
/// Arguments:
/// * `meta`: The metadata to check
///
pub fn check_group(meta: &Option<Meta>) -> Result<(), String> {
    return meta
        .as_ref()
        .map(|m| return check_str(&m.group, "Group", 20))
        .unwrap_or(Ok(()));
}

/// Check that the date is valid
///
/// Arguments:
/// * `meta`: The metadata to check
///
pub fn check_date(meta: &Option<Meta>) -> Result<(), String> {
    if let Some(m) = meta {
        if !m.date.is_empty() {
            if m.date.len() != 8 {
                return Err(format!(
                    "Date length is wrong (expected =8, got {})",
                    m.date.len()
                ));
            } else if let Err(e) = NaiveDate::parse_from_str(&m.date, "%Y%m%d") {
                return Err(format!("Date is wrong ({})", e));
            }
        }
    }

    return Ok(());
}

/// Check that the type is valid and supported
///
/// Arguments:
/// * `meta`: The metadata to check
///
pub fn check_type(meta: &Option<Meta>) -> Result<(), String> {
    if let Some(m) = meta {
        if ![0, 1].contains(&m.r#type.0) {
            return Err(format!(
                "Type is unsupported ({})",
                match m.r#type.0 {
                    // 0 => String::from("None"),
                    // 1 => String::from("Character"),
                    2 => String::from("Bitmap"),
                    3 => String::from("Vector"),
                    4 => String::from("Audio"),
                    5 => String::from("BinaryText"),
                    6 => String::from("XBin"),
                    7 => String::from("Archive"),
                    8 => String::from("Executable"),
                    _ => format!("Unknown {}", m.r#type.0),
                }
            ));
        } else if ![0, 1].contains(&m.r#type.1) {
            return Err(format!(
                "Type is unsupported ({})",
                match m.r#type.1 {
                    // 0 => String::from("Character/ASCII"),
                    // 1 => String::from("Character/ANSi"),
                    2 => String::from("Character/ANSiMation"),
                    3 => String::from("Character/RIPScript"),
                    4 => String::from("Character/PCBoard"),
                    5 => String::from("Character/Avatar"),
                    6 => String::from("Character/HTML"),
                    7 => String::from("Character/Source"),
                    8 => String::from("Character/TundraDraw"),
                    _ => format!("Character/Unknown {}", m.r#type.1),
                }
            ));
        }
    }

    return Ok(());
}

/// Check that the flags are valid
///
/// Arguments:
/// * `meta`: The metadata to check
///
pub fn check_flags(meta: &Option<Meta>) -> Result<(), String> {
    if let Some(m) = meta {
        if m.flags & 0x01 == 0x00 {
            // Only intended to support iCE colours
            return Err(String::from("Blink mode is unsupported"));
        } else if m.flags & 0x06 == 0x06 {
            return Err(String::from("Invalid letter spacing"));
        } else if m.flags & 0x18 == 0x18 {
            return Err(String::from("Invalid aspect ratio"));
        } else if m.flags > 0x1F {
            return Err(String::from("Invalid flags"));
        }
    }

    return Ok(());
}

/// Check that the font is valid and supported
///
/// Arguments:
/// * `meta`: The metadata to check
///
pub fn check_font(meta: &Option<Meta>) -> Result<(), String> {
    if let Some(m) = meta {
        if !["IBM VGA", "IBM VGA 437", ""].contains(&m.font.as_str()) {
            // IBM VGA is by far the most common font, haven't even tried to
            // support any others.
            return Err(format!("Font is unsupported ({})", m.font));
        }
    }

    return Ok(());
}

/// Check that the notes are valid
///
/// Arguments:
/// * `meta`: The metadata to check
///
pub fn check_notes(meta: &Option<Meta>) -> Result<(), String> {
    if let Some(m) = meta {
        for (i, note) in m.notes.iter().enumerate() {
            check_str(
                &note,
                &format!(
                    "Notes[{:0width$}]",
                    i,
                    width = (m.notes.len() as f32).log10().ceil() as usize
                ),
                64,
            )?;
        }
    }

    return Ok(());
}

fn check_str(string: &String, name: &str, max_length: usize) -> Result<(), String> {
    if string.len() > max_length {
        return Err(format!(
            "{} is too long (expected <={}, got {})",
            name,
            max_length,
            string.len()
        ));
    } else if !string.chars().all(valid_char) {
        return Err(format!("{} contains illegal characters", name,));
    }

    return Ok(());
}

fn valid_char(c: char) -> bool {
    return UTF8_TO_CP437[0x20..=0xFE]
        .keys()
        .cloned()
        .collect::<Vec<char>>()
        .contains(&c);
}

fn read_raw(file: &mut File) -> Result<Option<Vec<u8>>, String> {
    let mut sauce = vec![0; 128];
    file.seek(SeekFrom::End(-128))
        .map_err(|x| return x.to_string())?;
    file.read_exact(&mut sauce)
        .map_err(|x| return x.to_string())?;

    if &sauce[..7] != "SAUCE00".as_bytes() {
        return Ok(None);
    } else {
        let offset = sauce[104] as usize * 64 + (if sauce[104] > 0 { 134 } else { 129 });
        file.seek(SeekFrom::End(-(offset as i64)))
            .map_err(|x| return x.to_string())?;
        let mut raw = vec![0; offset];
        file.read_exact(&mut raw)
            .map_err(|x| return x.to_string())?;
        if raw[0] != 0x1A || (offset > 129 && &raw[1..6] != "COMNT".as_bytes()) {
            return Ok(None);
        }
        return Ok(Some(raw));
    }
}
