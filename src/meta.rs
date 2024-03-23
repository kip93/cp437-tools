// https://www.acid.org/info/sauce/sauce.htm

use endianness::{read_u16, read_u32, ByteOrder::LittleEndian};
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    str,
};

#[derive(Debug)]
pub struct Meta {
    pub title: String,
    pub author: String,
    pub group: String,
    pub date: String,
    pub size: u32,
    pub r#type: (u8, u8),
    pub width: u16,
    pub height: u16,
    pub notes: Vec<String>,
    pub flags: u8,
    pub font: String,
}

pub fn get(path: &str) -> Result<Option<Meta>, String> {
    return read(&mut File::open(path).map_err(|x| return x.to_string())?);
}

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
                date: String::from_utf8_lossy(&meta[meta.len() - 46..meta.len() - 38]).to_string(),
                size: read_u32(&meta[meta.len() - 38..meta.len() - 34], LittleEndian).unwrap(),
                r#type: (meta[meta.len() - 34], meta[meta.len() - 33]),
                width: read_u16(&meta[meta.len() - 32..meta.len() - 30], LittleEndian).unwrap(),
                height: read_u16(&meta[meta.len() - 30..meta.len() - 28], LittleEndian).unwrap(),
                notes: (0..meta[meta.len() - 24] as usize)
                    .rev()
                    .map(|i| {
                        let offset = meta.len() - (i + 3) * 64;
                        return String::from_utf8_lossy(&meta[offset..offset + 64])
                            .trim_end_matches('\x20')
                            .to_string();
                    })
                    .collect(),
                flags: meta[meta.len() - 23],
                font: String::from_utf8_lossy(&meta[meta.len() - 22..])
                    .trim_end_matches('\x00')
                    .to_string(),
            };
        });
    });
}

pub fn check(meta: &Option<Meta>) -> Result<(), String> {
    if let Some(m) = meta {
        if m.r#type.0 != 1 {
            return Err(format!(
                "Can't handle type: {}",
                match m.r#type.0 {
                    0 => String::from("None"),
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
