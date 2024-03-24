//! Handling of file's metadata
//!
//! See <https://www.acid.org/info/sauce/sauce.htm>
//!

use endianness::{read_u16, read_u32, ByteOrder::LittleEndian};
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    str,
};

/// A structure representing a file's metadata
#[doc(alias = "Sauce")]
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
                title: String::from_utf8_lossy(&meta[meta.len() - 121..meta.len() - 86])
                    .trim_end_matches('\x20')
                    .to_string(),
                author: String::from_utf8_lossy(&meta[meta.len() - 86..meta.len() - 66])
                    .trim_end_matches('\x20')
                    .to_string(),
                group: String::from_utf8_lossy(&meta[meta.len() - 66..meta.len() - 46])
                    .trim_end_matches('\x20')
                    .to_string(),
                date: String::from_utf8_lossy(&meta[meta.len() - 46..meta.len() - 38])
                    .trim()
                    .to_string(),
                size: read_u32(&meta[meta.len() - 38..meta.len() - 34], LittleEndian).unwrap(),
                r#type: (meta[meta.len() - 34], meta[meta.len() - 33]),
                width: read_u16(&meta[meta.len() - 32..meta.len() - 30], LittleEndian).unwrap(),
                height: read_u16(&meta[meta.len() - 30..meta.len() - 28], LittleEndian).unwrap(),
                flags: meta[meta.len() - 23],
                font: String::from_utf8_lossy(&meta[meta.len() - 22..])
                    .trim_end_matches('\x00')
                    .to_string(),
                notes: (0..meta[meta.len() - 24] as usize)
                    .rev()
                    .map(|i| {
                        let offset = meta.len() - (i + 3) * 64;
                        return String::from_utf8_lossy(&meta[offset..offset + 64])
                            .trim_end_matches('\x20')
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
    if let Some(m) = meta {
        if m.r#type.0 != 1 {
            return Err(format!(
                "Can't handle type: {}",
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
                    _ => format!("Unknown ({})", m.r#type.0),
                }
            ));
        } else if ![0, 1].contains(&m.r#type.1) {
            return Err(format!(
                "Can't handle type: {}",
                match m.r#type.0 {
                    // 0 => String::from("Character/ASCII"),
                    // 1 => String::from("Character/ANSi"),
                    2 => String::from("Character/ANSiMation"),
                    3 => String::from("Character/RIPScript"),
                    4 => String::from("Character/PCBoardt"),
                    5 => String::from("Character/Avatar"),
                    6 => String::from("Character/HTML"),
                    7 => String::from("Character/Source"),
                    8 => String::from("Character/TundraDraw"),
                    _ => format!("Character/Unknown ({})", m.r#type.0),
                }
            ));
        } else if !["IBM VGA", "IBM VGA 437", ""].contains(&m.font.as_str()) {
            // IBM VGA is by far the most common font, haven't even tried to
            // support any others.
            return Err(format!("Can't handle font: {}", m.font));
        } else if m.flags & 0x01 == 0x00 {
            // Only intended for iCE colours
            return Err(String::from("Can't handle blink mode"));
        } else if m.flags & 0x06 == 0x06 {
            return Err(String::from("Invalid letter spacing"));
        } else if m.flags & 0x18 == 0x18 {
            return Err(String::from("Invalid aspect ratio"));
        } else {
            return Ok(());
        }
    } else {
        return Ok(());
    }
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
