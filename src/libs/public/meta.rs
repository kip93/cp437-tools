//! Handling of file's metadata
//!
//! See <https://web.archive.org/web/20250427042053id_/https://www.acid.org/info/sauce/sauce.htm>
//!

use chrono::NaiveDate;
use std::{
    array::TryFromSliceError,
    fs::File,
    io::{Read as _, Seek as _, SeekFrom},
    str,
};
use ttf_parser::Face;

use crate::{
    fonts,
    prelude::{to_utf8, CP437_TO_UTF8},
};

/// A structure representing a file's metadata.
#[doc(alias = "Sauce")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Meta {
    /// The image's title.
    pub title: String,
    /// The image's author.
    pub author: String,
    /// The image author's team or group.
    #[doc(alias = "team")]
    pub group: String,
    /// The image creation date, in the YYYYMMDD format.
    pub date: String,
    /// The size of the file, sans this metadata.
    pub size: u32,
    /// The type of this file.
    ///
    /// Only supported values are:
    /// * `(0, 0)` → `None` (effectively, `Character/ANSI`)
    /// * `(1, 0)` → `Character/ASCII`
    /// * `(1, 1)` → `Character/ANSI`
    ///
    /// See <https://web.archive.org/web/20250427042053id_/https://www.acid.org/info/sauce/sauce.htm#FileType>
    ///
    pub r#type: (u8, u8),
    /// Width of the image.
    pub width: u16,
    /// Height of the image.
    pub height: u16,
    /// A bitfield of flags that define how to process an image.
    ///
    /// See <https://web.archive.org/web/20250427042053id_/https://www.acid.org/info/sauce/sauce.htm#ANSiFlags>
    ///
    #[doc(alias = "AR")]
    #[doc(alias = "aspect ratio")]
    #[doc(alias = "LS")]
    #[doc(alias = "letter spacing")]
    #[doc(alias = "B")]
    #[doc(alias = "iCE colour")]
    #[doc(alias = "non-blink mode")]
    pub flags: u8,
    /// The name of the font this image uses.
    ///
    /// Only IBM VGA is supported.
    ///
    pub font: String,
    /// A list of comments on this image.
    #[doc(alias = "comments")]
    pub notes: Vec<String>,
}

/// A minimal meta.
///
/// Sets all defaults as interpreted when undefined.
///
impl Default for Meta {
    /// Get default meta values.
    fn default() -> Meta {
        return Meta {
            title: String::new(),
            author: String::new(),
            group: String::new(),
            date: String::new(),
            size: 0,
            r#type: (1, 1),
            width: 80,
            height: 25,
            flags: 0x0D,
            font: String::from("IBM VGA"),
            notes: vec![],
        };
    }
}

impl Meta {
    /// Wrap the title in an [`Option`].
    ///
    /// See [`title` field](#structfield.title)
    ///
    #[must_use]
    pub fn title(&self) -> Option<&String> {
        return if self.title.is_empty() { None } else { Some(&self.title) };
    }

    /// Wrap the author in an [`Option`].
    ///
    /// See [`author` field](#structfield.author)
    ///
    #[must_use]
    pub fn author(&self) -> Option<&String> {
        return if self.author.is_empty() { None } else { Some(&self.author) };
    }

    /// Wrap the group in an [`Option`].
    ///
    /// See [`group` field](#structfield.group)
    ///
    #[must_use]
    pub fn group(&self) -> Option<&String> {
        return if self.group.is_empty() { None } else { Some(&self.group) };
    }

    /// Wrap the date in an [`Option`].
    ///
    /// See [`date` field](#structfield.date)
    ///
    #[must_use]
    pub fn date(&self) -> Option<&String> {
        return if self.date.is_empty() { None } else { Some(&self.date) };
    }

    /// Fetch the size.
    ///
    /// See [`size` field](#structfield.size)
    ///
    #[inline]
    #[must_use]
    pub fn size(&self) -> u32 {
        return self.size;
    }

    /// Fetch the type if `type != (0, 0)`, otherwise the default.
    ///
    /// See [`type` field](#structfield.type)
    ///
    /// See [`Meta::default`]
    ///
    #[must_use]
    pub fn r#type(&self) -> (u8, u8) {
        return if self.r#type == (0, 0) { Meta::default().r#type } else { self.r#type };
    }

    /// Fetch the width if `width > 0`, otherwise the default.
    ///
    /// See [`width` field](#structfield.width)
    ///
    /// See [`Meta::default`]
    ///
    #[must_use]
    pub fn width(&self) -> u16 {
        return if self.width > 0 { self.width } else { Meta::default().width };
    }

    /// Fetch the height if `height > 0`, otherwise the default.
    ///
    /// See [`height` field](#structfield.height)
    ///
    /// See [`Meta::default`]
    ///
    #[must_use]
    pub fn height(&self) -> u16 {
        return if self.height > 0 { self.height } else { Meta::default().height };
    }

    /// Get both the width and the height.
    ///
    /// See [`width` method](#method.width)
    ///
    /// See [`height` method](#method.height)
    ///
    #[inline]
    #[must_use]
    pub fn dimensions(&self) -> (u16, u16) {
        return (self.width(), self.height());
    }

    /// Fetch the flags, split into `(AR, LS, B)`.
    ///
    /// See [`flags` field](#structfield.flags)
    ///
    #[must_use]
    pub fn flags(&self) -> (u8, u8, u8) {
        return ((self.flags >> 3) & 3, (self.flags >> 1) & 3, self.flags & 1);
    }

    /// Fetch the font if `font != ""`, otherwise the default.
    ///
    /// See [`font` field](#structfield.font)
    ///
    #[must_use]
    pub fn font(&self) -> Option<&String> {
        return if self.font.is_empty() { None } else { Some(&self.font) };
    }

    /// Font face, in OTB format.
    ///
    /// See [`font` field](#structfield.font)
    ///
    #[must_use]
    pub fn font_face_otb(&self) -> &Face<'_> {
        return if self.font_width() == 8 { &fonts::VGA_8X16 as &Face } else { &fonts::VGA_9X16 as &Face };
    }

    /// Font face, in WOFF format.
    ///
    /// See [`font` field](#structfield.font)
    ///
    #[must_use]
    pub fn font_face_woff(&self) -> &[u8] {
        return if self.font_width() == 8 { &fonts::VGA_8X16_WOFF } else { &fonts::VGA_9X16_WOFF };
    }

    /// Fetch the notes.
    ///
    /// See [`notes` field](#structfield.notes)
    ///
    #[inline]
    #[must_use]
    pub fn notes(&self) -> &Vec<String> {
        return &self.notes;
    }

    /// Compute the stretch required for a given aspect ratio.
    ///
    /// See [`aspect_ratio` method](#method.aspect_ratio)
    ///
    #[inline]
    #[must_use]
    pub fn stretch(&self) -> f64 {
        let ar = self.aspect_ratio();
        return f64::from(ar.1) / f64::from(ar.0);
    }

    /// Compute the aspect ratio.
    ///
    /// See [`flags` field](#structfield.flags)
    ///
    #[must_use]
    pub fn aspect_ratio(&self) -> (u8, u8) {
        return if self.flags().0 == 0b10 {
            (1, 1)
        } else if self.flags().1 == 0b01 {
            (5, 6)
        } else {
            (20, 27)
        };
    }

    /// Font width.
    ///
    /// See [`flags` field](#structfield.flags)
    ///
    #[must_use]
    pub fn font_width(&self) -> u8 {
        return if self.flags().1 == 0b01 { 8 } else { 9 };
    }

    /// Font height.
    #[inline]
    #[must_use]
    pub fn font_height(&self) -> u8 {
        return 16;
    }

    /// Font dimensions.
    ///
    /// See [`font_width` method](#method.font_width)
    ///
    /// See [`font_height` method](#method.font_height)
    ///
    #[inline]
    #[must_use]
    pub fn font_size(&self) -> (u8, u8) {
        return (self.font_width(), self.font_height());
    }
}

/// Get a file's metadata via its path.
///
/// # Arguments
///
/// * `path`: Path pointing to file. Can be relative to cwd.
///
/// # Errors
///
/// Fails when there's problems reading the file.
///
#[inline]
pub fn get(path: &str) -> Result<Option<Meta>, String> {
    return read(&mut File::open(path).map_err(|err| return err.to_string())?);
}

/// Get a file's metadata via a file reference.
///
/// # Arguments
///
/// * `file`: File to read.
///
/// # Errors
///
/// Fails when there's problems reading the file.
///
pub fn read(file: &mut File) -> Result<Option<Meta>, String> {
    return read_raw(file).map(|maybe_raw| {
        return maybe_raw
            .map(|raw| {
                return Ok(Meta {
                    title: to_utf8(&(raw[raw.len() - 121..raw.len() - 86])).trim_matches('\x20').to_string(),
                    author: to_utf8(&(raw[raw.len() - 86..raw.len() - 66])).trim_matches('\x20').to_string(),
                    group: to_utf8(&(raw[raw.len() - 66..raw.len() - 46])).trim_matches('\x20').to_string(),
                    date: to_utf8(&(raw[raw.len() - 46..raw.len() - 38])).trim_matches('\x20').to_string(),
                    size: u32::from_le_bytes(
                        raw[raw.len() - 38..raw.len() - 34]
                            .try_into()
                            .map_err(|err: TryFromSliceError| return err.to_string())?,
                    ),
                    r#type: (raw[raw.len() - 34], raw[raw.len() - 33]),
                    width: u16::from_le_bytes(
                        raw[raw.len() - 32..raw.len() - 30]
                            .try_into()
                            .map_err(|err: TryFromSliceError| return err.to_string())?,
                    ),
                    height: u16::from_le_bytes(
                        raw[raw.len() - 30..raw.len() - 28]
                            .try_into()
                            .map_err(|err: TryFromSliceError| return err.to_string())?,
                    ),
                    flags: raw[raw.len() - 23],
                    font: to_utf8(&(raw[raw.len() - 22..])).trim_matches('\x00').to_string(),
                    notes: (0..raw[raw.len() - 24] as usize)
                        .rev()
                        .map(|i| {
                            let offset = raw.len() - (i + 3) * 64;
                            return to_utf8(&(raw[offset..offset + 64])).trim_matches('\x20').to_string();
                        })
                        .collect(),
                });
            })
            .transpose();
    })?;
}

/// Get a human readable type name.
///
/// # Arguments
///
/// * `type`: The type to get the name for.
///
#[must_use]
pub fn type_name(r#type: (u8, u8)) -> String {
    return match r#type {
        (0, _) => String::from("None"),
        (1, 0) => String::from("Character/ASCII"),
        (1, 1) => String::from("Character/ANSi"),
        (1, 2) => String::from("Character/ANSiMation"),
        (1, 3) => String::from("Character/RIPScript"),
        (1, 4) => String::from("Character/PCBoard"),
        (1, 5) => String::from("Character/Avatar"),
        (1, 6) => String::from("Character/HTML"),
        (1, 7) => String::from("Character/Source"),
        (1, 8) => String::from("Character/TundraDraw"),
        (1, _) => format!("Character/Unknown {}", r#type.1),
        (2, _) => String::from("Bitmap"),
        (3, _) => String::from("Vector"),
        (4, _) => String::from("Audio"),
        (5, _) => String::from("BinaryText"),
        (6, _) => String::from("XBin"),
        (7, _) => String::from("Archive"),
        (8, _) => String::from("Executable"),
        _ => format!("Unknown {}/Unknown {}", r#type.0, r#type.1),
    };
}

/// Check that a given file's metadata is valid and supported.
///
/// # Arguments
///
/// * `meta`: The metadata to check.
///
#[expect(clippy::missing_errors_doc, reason = "That's like the whole purpose of this function")]
pub fn check(meta: Option<&Meta>) -> Result<(), String> {
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

/// Check that the title is valid.
///
/// # Arguments
///
/// * `meta`: The metadata to check.
///
#[expect(clippy::missing_errors_doc, reason = "That's like the whole purpose of this function")]
pub fn check_title(meta: Option<&Meta>) -> Result<(), String> {
    return meta.as_ref().map_or(Ok(()), |m| return check_str(&m.title, "Title", 35));
}

/// Check that the author is valid.
///
/// # Arguments
///
/// * `meta`: The metadata to check.
///
#[expect(clippy::missing_errors_doc, reason = "That's like the whole purpose of this function")]
pub fn check_author(meta: Option<&Meta>) -> Result<(), String> {
    return meta.as_ref().map_or(Ok(()), |m| return check_str(&m.author, "Author", 20));
}

/// Check that the group is valid.
///
/// # Arguments
///
/// * `meta`: The metadata to check.
///
#[expect(clippy::missing_errors_doc, reason = "That's like the whole purpose of this function")]
pub fn check_group(meta: Option<&Meta>) -> Result<(), String> {
    return meta.as_ref().map_or(Ok(()), |m| return check_str(&m.group, "Group", 20));
}

/// Check that the date is valid.
///
/// # Arguments
///
/// * `meta`: The metadata to check.
///
#[expect(clippy::missing_errors_doc, reason = "That's like the whole purpose of this function")]
pub fn check_date(meta: Option<&Meta>) -> Result<(), String> {
    if let Some(m) = meta {
        if !m.date.is_empty() {
            if m.date.len() != 8 {
                return Err(format!("Date length is wrong (expected =8, got {})", m.date.len()));
            } else if let Err(err) = NaiveDate::parse_from_str(&m.date, "%Y%m%d") {
                return Err(format!("Date format is wrong ({err})"));
            }
        }
    }

    return Ok(());
}

/// Check that the type is valid and supported.
///
/// # Arguments
///
/// * `meta`: The metadata to check.
///
#[expect(clippy::missing_errors_doc, reason = "That's like the whole purpose of this function")]
pub fn check_type(meta: Option<&Meta>) -> Result<(), String> {
    if let Some(m) = meta {
        if ![0, 1].contains(&m.r#type.0) || ![0, 1].contains(&m.r#type.1) {
            return Err(format!("Type is unsupported ({})", type_name(m.r#type)));
        }
    }

    return Ok(());
}

/// Check that the flags are valid.
///
/// # Arguments
///
/// * `meta`: The metadata to check.
///
#[expect(clippy::missing_errors_doc, reason = "That's like the whole purpose of this function")]
pub fn check_flags(meta: Option<&Meta>) -> Result<(), String> {
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

/// Check that the font is valid and supported.
///
/// # Arguments
///
/// * `meta`: The metadata to check.
///
#[expect(clippy::missing_errors_doc, reason = "That's like the whole purpose of this function")]
pub fn check_font(meta: Option<&Meta>) -> Result<(), String> {
    if let Some(m) = meta {
        if !["IBM VGA", "IBM VGA 437", ""].contains(&m.font.as_str()) {
            // IBM VGA is by far the most common font, haven't even tried to
            // support any others.
            return Err(format!("Font is unsupported ({})", m.font));
        }
    }

    return Ok(());
}

/// Check that the notes are valid.
///
/// # Arguments
///
/// * `meta`: The metadata to check.
///
#[expect(clippy::missing_errors_doc, reason = "That's like the whole purpose of this function")]
pub fn check_notes(meta: Option<&Meta>) -> Result<(), String> {
    if let Some(m) = meta {
        if m.notes.len() > 255 {
            return Err(format!("Too many notes (expected <= 255, got {})", m.notes.len()));
        }

        for i in 0..m.notes.len() {
            check_note(meta, i)?;
        }
    }

    return Ok(());
}

/// Check that a single note is valid.
///
/// # Arguments
///
/// * `meta`: The metadata to check.
/// * `i`: The index of the note.
///
#[expect(clippy::cast_possible_truncation, reason = "Range is [0,3]")]
#[expect(clippy::cast_sign_loss, reason = "Range is [0,3]")]
#[expect(clippy::cast_precision_loss, reason = "Range is [0,3]")]
#[expect(clippy::missing_errors_doc, reason = "That's like the whole purpose of this function")]
pub fn check_note(meta: Option<&Meta>, i: usize) -> Result<(), String> {
    if let Some(m) = meta {
        check_str(
            &m.notes[i],
            &format!("Notes[{:0width$}]", i, width = (m.notes.len() as f32).log10().ceil() as usize),
            64,
        )?;
    }

    return Ok(());
}

fn check_str(string: &str, name: &str, max_length: usize) -> Result<(), String> {
    if string.len() > max_length {
        return Err(format!("{} is too long (expected <={}, got {})", name, max_length, string.len()));
    }

    return string.chars().try_for_each(|r#char| {
        return check_char(r#char).map_err(|msg| return format!("{name} contains illegal characters ({msg})"));
    });
}

fn check_char(r#char: char) -> Result<(), String> {
    if ['\x00', '\x0A', '\x0D', '\x1A', '\x1B'].contains(&r#char) {
        return Err(format!("0x{:02X} is a control character", r#char as u8));
    } else if !CP437_TO_UTF8.contains(&r#char) {
        return Err(format!("{} (U+{:X}) is not a valid CP437 character", r#char, r#char as u32));
    }

    return Ok(());
}

fn read_raw(file: &mut File) -> Result<Option<Vec<u8>>, String> {
    if file.metadata().map_err(|err| return err.to_string())?.len() < 129 {
        return Ok(None);
    }

    let mut sauce = vec![0; 128];
    file.seek(SeekFrom::End(-128)).map_err(|err| return err.to_string())?;
    file.read_exact(&mut sauce).map_err(|err| return err.to_string())?;

    if &sauce[..7] != "SAUCE00".as_bytes() {
        return Ok(None);
    }

    let offset = sauce[104] as usize * 64 + (if sauce[104] > 0 { 134 } else { 129 });
    #[expect(clippy::cast_possible_wrap, reason = "Range is [0,16454]")]
    file.seek(SeekFrom::End(-(offset as i64))).map_err(|err| return err.to_string())?;
    let mut raw = vec![0; offset];
    file.read_exact(&mut raw).map_err(|err| return err.to_string())?;
    if raw[0] != 0x1A || (offset > 129 && &raw[1..6] != "COMNT".as_bytes()) {
        return Ok(None);
    }

    return Ok(Some(raw));
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;

    #[test]
    fn default() {
        let meta = Meta::default();
        assert_eq!(meta.title(), None);
        assert_eq!(meta.author(), None);
        assert_eq!(meta.group(), None);
        assert_eq!(meta.date(), None);
        assert_eq!(meta.size(), 0);
        assert_eq!(meta.r#type(), (1, 1));
        assert_eq!(meta.dimensions(), (80, 25));
        assert_eq!(meta.flags(), (0b01, 0b10, 0b1));
        assert_eq!(meta.font(), Some(&String::from("IBM VGA")));
        assert_eq!(meta.notes(), &Vec::<String>::new());
    }

    #[test]
    fn none() -> Result<(), String> {
        let meta = get("res/test/simple.ans")?;
        assert!(meta.is_none());

        return Ok(());
    }

    #[test]
    fn some() -> Result<(), String> {
        let meta = get("res/test/meta.ans")?;
        assert!(meta.is_some());
        let meta = meta.unwrap();
        assert_eq!(meta.title(), Some(&String::from("TITLE")));
        assert_eq!(meta.author(), Some(&String::from("AUTHOR")));
        assert_eq!(meta.group(), Some(&String::from("GROUP")));
        assert_eq!(meta.date(), Some(&String::from("19700101")));
        assert_eq!(meta.size(), 416);
        assert_eq!(meta.r#type(), (1, 1));
        assert_eq!(meta.dimensions(), (32, 8));
        assert_eq!(meta.flags(), (0, 0, 1));
        assert_eq!(meta.font(), Some(&String::from("IBM VGA")));
        assert_eq!(meta.notes(), &Vec::<String>::new());

        return Ok(());
    }

    #[test]
    fn notes() -> Result<(), String> {
        let meta = get("res/test/comments.ans")?;
        assert!(meta.is_some());
        let meta = meta.unwrap();
        assert_eq!(meta.title(), Some(&String::from("TITLE")));
        assert_eq!(meta.author(), Some(&String::from("AUTHOR")));
        assert_eq!(meta.group(), Some(&String::from("GROUP")));
        assert_eq!(meta.date(), Some(&String::from("19700101")));
        assert_eq!(meta.size(), 416);
        assert_eq!(meta.r#type(), (1, 1));
        assert_eq!(meta.dimensions(), (32, 8));
        assert_eq!(meta.flags(), (0, 0, 1));
        assert_eq!(meta.font(), Some(&String::from("IBM VGA")));
        assert_eq!(meta.notes(), &vec!["Lorem", "ipsum", "dolor", "sit", "amet"]);

        return Ok(());
    }

    #[test]
    fn empty() -> Result<(), String> {
        let meta = get("res/test/empty.ans")?;
        assert!(meta.is_none());

        return Ok(());
    }

    #[test]
    fn no_data() -> Result<(), String> {
        let meta = get("res/test/no_data.ans")?;
        assert!(meta.is_some());
        let meta = meta.unwrap();
        assert_eq!(meta.size(), 0);

        return Ok(());
    }

    #[test]
    fn one_hundred_twenty_eight_bytes() -> Result<(), String> {
        let meta = get("res/test/128_bytes.ans")?;
        assert!(meta.is_none());

        return Ok(());
    }

    mod raw {
        use super::*;

        use pretty_assertions::assert_eq;

        #[test]
        fn none() -> Result<(), String> {
            let meta = read_raw(&mut File::open("res/test/simple.ans").map_err(|err| return err.to_string())?)?;
            assert!(meta.is_none());

            return Ok(());
        }

        #[test]
        fn some() -> Result<(), String> {
            let meta = read_raw(&mut File::open("res/test/meta.ans").map_err(|err| return err.to_string())?)?;
            assert!(meta.is_some());
            assert_eq!(
                meta.unwrap(),
                b"\x1ASAUCE00"
                    .iter()
                    .cloned()
                    .chain(format!("{:<35}", "TITLE").bytes()) // Title
                    .chain(format!("{:<20}", "AUTHOR").bytes()) // Author
                    .chain(format!("{:<20}", "GROUP").bytes()) // Group
                    .chain(b"19700101".iter().cloned()) // Date
                    .chain(416u32.to_le_bytes()) // Size
                    .chain([1u8, 1u8]) // Type
                    .chain(32u16.to_le_bytes()) // Width
                    .chain(8u16.to_le_bytes()) // Height
                    .chain(0u32.to_le_bytes())
                    .chain([0u8]) // Notes
                    .chain([0x01u8]) // Flags
                    .chain(format!("{:\0<22}", "IBM VGA").bytes()) // Font
                    .collect::<Vec<u8>>(),
            );

            return Ok(());
        }

        #[test]
        fn notes() -> Result<(), String> {
            let meta = read_raw(&mut File::open("res/test/comments.ans").map_err(|err| return err.to_string())?)?;
            assert!(meta.is_some());
            assert_eq!(
                meta.unwrap(),
                b"\x1ACOMNT"
                    .iter()
                    .cloned()
                    .chain(format!("{:<64}", "Lorem").bytes()) // Comment
                    .chain(format!("{:<64}", "ipsum").bytes()) // Comment
                    .chain(format!("{:<64}", "dolor").bytes()) // Comment
                    .chain(format!("{:<64}", "sit").bytes()) // Comment
                    .chain(format!("{:<64}", "amet").bytes()) // Comment
                    .chain(b"SAUCE00".iter().cloned())
                    .chain(format!("{:<35}", "TITLE").bytes()) // Title
                    .chain(format!("{:<20}", "AUTHOR").bytes()) // Author
                    .chain(format!("{:<20}", "GROUP").bytes()) // Group
                    .chain(b"19700101".iter().cloned()) // Date
                    .chain(416u32.to_le_bytes()) // Size
                    .chain([1u8, 1u8]) // Type
                    .chain(32u16.to_le_bytes()) // Width
                    .chain(8u16.to_le_bytes()) // Height
                    .chain(0u32.to_le_bytes())
                    .chain([5u8]) // Notes
                    .chain([0x01u8]) // Flags
                    .chain(format!("{:\0<22}", "IBM VGA").bytes()) // Font
                    .collect::<Vec<u8>>(),
            );

            return Ok(());
        }

        #[test]
        fn empty() -> Result<(), String> {
            let meta = read_raw(&mut File::open("res/test/empty.ans").map_err(|err| return err.to_string())?)?;
            assert!(meta.is_none());

            return Ok(());
        }

        #[test]
        fn no_data() -> Result<(), String> {
            let meta = read_raw(&mut File::open("res/test/no_data.ans").map_err(|err| return err.to_string())?)?;
            assert!(meta.is_some());
            assert_eq!(
                meta.unwrap(),
                b"\x1ASAUCE00"
                    .iter()
                    .cloned()
                    .chain(format!("{:<35}", "TITLE").bytes()) // Title
                    .chain(format!("{:<20}", "AUTHOR").bytes()) // Author
                    .chain(format!("{:<20}", "GROUP").bytes()) // Group
                    .chain(b"19700101".iter().cloned()) // Date
                    .chain(0u32.to_le_bytes()) // Size
                    .chain([1u8, 1u8]) // Type
                    .chain(32u16.to_le_bytes()) // Width
                    .chain(8u16.to_le_bytes()) // Height
                    .chain(0u32.to_le_bytes())
                    .chain([0u8]) // Notes
                    .chain([0x01u8]) // Flags
                    .chain(format!("{:\0<22}", "IBM VGA").bytes()) // Font
                    .collect::<Vec<u8>>(),
            );

            return Ok(());
        }

        #[test]
        fn one_hundred_twenty_eight_bytes() -> Result<(), String> {
            let meta = read_raw(&mut File::open("res/test/128_bytes.ans").map_err(|err| return err.to_string())?)?;
            assert!(meta.is_none());

            return Ok(());
        }
    }

    mod check {
        use super::*;

        mod meta {
            use super::*;

            #[test]
            fn none() -> Result<(), String> {
                return check(None);
            }

            #[test]
            fn some() -> Result<(), String> {
                return check(Some(&Meta::default()));
            }
        }

        mod date {
            use super::*;

            #[test]
            fn valid() -> Result<(), String> {
                return check_date(Some(&Meta { date: String::from("19700101"), ..Default::default() }));
            }

            #[test]
            fn invalid() {
                assert!(check_date(Some(&Meta { date: String::from("X"), ..Default::default() })).is_err());
            }

            #[test]
            fn illegal() {
                assert!(check_date(Some(&Meta { date: String::from("19700230"), ..Default::default() })).is_err());
            }
        }

        mod flags {
            use super::*;

            use pretty_assertions::assert_eq;

            #[test]
            fn b_0() {
                assert!(check_flags(Some(&Meta { flags: 0x00, ..Default::default() })).is_err());
            }

            #[test]
            fn ls_00() -> Result<(), String> {
                return check_flags(Some(&Meta { flags: 0x01, ..Default::default() }));
            }

            #[test]
            fn ls_01() -> Result<(), String> {
                return check_flags(Some(&Meta { flags: 0x03, ..Default::default() }));
            }

            #[test]
            fn ls_10() -> Result<(), String> {
                return check_flags(Some(&Meta { flags: 0x05, ..Default::default() }));
            }

            #[test]
            fn ls_11() {
                assert!(check_flags(Some(&Meta { flags: 0x07, ..Default::default() })).is_err());
            }

            #[test]
            fn ar_00() -> Result<(), String> {
                return check_flags(Some(&Meta { flags: 0x01, ..Default::default() }));
            }

            #[test]
            fn ar_01() -> Result<(), String> {
                return check_flags(Some(&Meta { flags: 0x09, ..Default::default() }));
            }

            #[test]
            fn ar_10() -> Result<(), String> {
                return check_flags(Some(&Meta { flags: 0x11, ..Default::default() }));
            }

            #[test]
            fn ar_11() {
                assert!(check_flags(Some(&Meta { flags: 0x19, ..Default::default() })).is_err());
            }

            #[test]
            fn invalid() {
                assert!(check_flags(Some(&Meta { flags: 0x21, ..Default::default() })).is_err());
            }

            #[test]
            fn ratio_1_00() {
                assert_eq!((Meta { flags: 0x11, ..Default::default() }).stretch(), 1.00);
            }

            #[test]
            fn ratio_1_20() {
                assert_eq!((Meta { flags: 0x03, ..Default::default() }).stretch(), 1.20);
            }

            #[test]
            fn ratio_1_35() {
                assert_eq!((Meta { flags: 0x01, ..Default::default() }).stretch(), 1.35);
            }

            #[test]
            fn font_size_8x16() {
                assert_eq!((Meta { flags: 0x03, ..Default::default() }).font_size(), (8, 16));
            }

            #[test]
            fn font_size_9x16() {
                assert_eq!((Meta { flags: 0x01, ..Default::default() }).font_size(), (9, 16));
            }
        }

        mod font {
            use super::*;

            use pretty_assertions::assert_eq;

            #[test]
            fn valid() -> Result<(), String> {
                return check_font(Some(&Meta { font: String::from("IBM VGA"), ..Default::default() }));
            }

            #[test]
            fn invalid() {
                assert!(check_font(Some(&Meta { font: String::from("X"), ..Default::default() })).is_err());
            }

            #[test]
            fn font_face_8x16() {
                assert_eq!(
                    (Meta { flags: 0x03, ..Default::default() }).font_face_otb().raw_face().data,
                    fonts::VGA_8X16.raw_face().data,
                );
            }

            #[test]
            fn font_face_9x16() {
                assert_eq!(
                    (Meta { flags: 0x01, ..Default::default() }).font_face_otb().raw_face().data,
                    fonts::VGA_9X16.raw_face().data,
                );
            }
        }

        mod notes {
            use super::*;

            #[test]
            fn empty() -> Result<(), String> {
                return check_notes(Some(&Meta { notes: vec![], ..Default::default() }));
            }

            #[test]
            fn not_empty() -> Result<(), String> {
                return check_notes(Some(&Meta { notes: vec![String::new()], ..Default::default() }));
            }

            #[test]
            fn too_many() {
                assert!(check_notes(Some(&Meta { notes: vec![String::new(); 256], ..Default::default() })).is_err());
            }
        }

        mod str {
            use super::*;

            use pretty_assertions::assert_eq;

            #[test]
            fn valid() -> Result<(), String> {
                return check_str(&String::from("string"), "name", 99);
            }

            #[test]
            fn valid_non_ascii() -> Result<(), String> {
                return check_str(&String::from("░"), "name", 99);
            }

            #[test]
            fn long() {
                let result = check_str(&String::from("string"), "name", 0);
                assert!(result.is_err());
                assert_eq!(result.unwrap_err(), "name is too long (expected <=0, got 6)");
            }

            #[test]
            fn control() {
                let result = check_str(&String::from("\0"), "name", 99);
                assert!(result.is_err());
                assert_eq!(result.unwrap_err(), "name contains illegal characters (0x00 is a control character)");
            }

            #[test]
            fn invalid() {
                let result = check_str(&String::from("🚫"), "name", 99);
                assert!(result.is_err());
                assert_eq!(
                    result.unwrap_err(),
                    "name contains illegal characters (🚫 (U+1F6AB) is not a valid CP437 character)",
                );
            }
        }

        mod r#type {
            use super::*;

            #[test]
            fn none() -> Result<(), String> {
                return check_type(Some(&Meta { r#type: (0, 0), ..Default::default() }));
            }

            #[test]
            fn ascii() -> Result<(), String> {
                return check_type(Some(&Meta { r#type: (1, 0), ..Default::default() }));
            }

            #[test]
            fn ansi() -> Result<(), String> {
                return check_type(Some(&Meta { r#type: (1, 1), ..Default::default() }));
            }

            #[test]
            fn bitmap() {
                assert!(check_type(Some(&Meta { r#type: (2, 0), ..Default::default() })).is_err());
            }

            #[test]
            fn vector() {
                assert!(check_type(Some(&Meta { r#type: (3, 0), ..Default::default() })).is_err());
            }

            #[test]
            fn audio() {
                assert!(check_type(Some(&Meta { r#type: (4, 0), ..Default::default() })).is_err());
            }

            #[test]
            fn binary_test() {
                assert!(check_type(Some(&Meta { r#type: (5, 0), ..Default::default() })).is_err());
            }

            #[test]
            fn xbin() {
                assert!(check_type(Some(&Meta { r#type: (6, 0), ..Default::default() })).is_err());
            }

            #[test]
            fn archive() {
                assert!(check_type(Some(&Meta { r#type: (7, 0), ..Default::default() })).is_err());
            }

            #[test]
            fn executable() {
                assert!(check_type(Some(&Meta { r#type: (8, 0), ..Default::default() })).is_err());
            }

            #[test]
            fn ansimation() {
                assert!(check_type(Some(&Meta { r#type: (1, 2), ..Default::default() })).is_err());
            }

            #[test]
            fn rip_script() {
                assert!(check_type(Some(&Meta { r#type: (1, 3), ..Default::default() })).is_err());
            }

            #[test]
            fn pcboard() {
                assert!(check_type(Some(&Meta { r#type: (1, 4), ..Default::default() })).is_err());
            }

            #[test]
            fn avatar() {
                assert!(check_type(Some(&Meta { r#type: (1, 5), ..Default::default() })).is_err());
            }

            #[test]
            fn html() {
                assert!(check_type(Some(&Meta { r#type: (1, 6), ..Default::default() })).is_err());
            }

            #[test]
            fn source() {
                assert!(check_type(Some(&Meta { r#type: (1, 7), ..Default::default() })).is_err());
            }

            #[test]
            fn tundra_draw() {
                assert!(check_type(Some(&Meta { r#type: (1, 8), ..Default::default() })).is_err());
            }
        }
    }
}
