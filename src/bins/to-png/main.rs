//! Render a file as a PNG.

use png::{text_metadata::ITXtChunk, BitDepth, ColorType, Compression, Encoder, PixelDimensions, Unit};
use std::{
    env::args,
    io::{stdout, BufWriter, IsTerminal as _},
};

use cp437_tools::{
    internal::{process, ExitCode, Input, Output},
    prelude::{Meta, CP437_TO_UTF8},
};

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
pub fn exec(args: &[String]) -> ExitCode {
    let exit_code = if args.len() < 2 {
        ExitCode::USAGE(String::from("Missing input file"))
    } else if args.len() > 3 {
        ExitCode::USAGE(String::from("Too many arguments"))
    } else if stdout().is_terminal() {
        ExitCode::USAGE(String::from("Refusing to write to terminal"))
    } else {
        process(&args[1], |i, o| {
            return run(i, o, args.get(2).unwrap_or(&String::from("CLASSIC")));
        })
    };

    exit_code.print();
    return exit_code;
}

#[allow(missing_docs, reason = "Just an entry point")]
#[allow(clippy::missing_docs_in_private_items, reason = "Just an entry point")]
pub fn run(input: &mut Input, output: &mut Output, scheme: &String) -> ExitCode {
    let meta = input.meta.clone().unwrap_or(Meta { size: input.size, ..Default::default() });

    let (width, height) = meta.dimensions();
    let (width, height) = (width as usize, height as usize);
    let (font_width, font_height) = meta.font_size();
    let (font_width, font_height) = (font_width as usize, font_height as usize);
    let (ar_x, ar_y) = meta.aspect_ratio();
    let (ar_x, ar_y) = (ar_x as usize, ar_y as usize);
    let font_face = meta.font_face_otb();
    let mut canvas = vec![0; 3 * width * height * font_width * font_height * ar_x * ar_y];
    input.read_by_bytes_full(
        |byte, (x, y), colour| {
            let (x, y) = (x as usize, y as usize);
            let bitmap = font_face
                .glyph_raster_image(
                    font_face
                        .glyph_index(CP437_TO_UTF8[byte as usize])
                        .ok_or_else(|| format!("Glyph for 0x{byte:02X} is missing"))?,
                    u16::try_from(font_height)?,
                )
                .ok_or_else(|| format!("Glyph bitmap for 0x{byte:02X} is missing"))?;

            for i in 0..(font_width * ar_x) {
                for j in 0..(font_height * ar_y) {
                    let offset =
                        3 * ((y * font_height * ar_y + j) * font_width * ar_x * width + (x * font_width * ar_x + i));
                    #[expect(clippy::integer_division, reason = "Intentional")]
                    let bitmap_offset = i / ar_x + j / ar_y * font_width;
                    canvas[offset..offset + 3].copy_from_slice(
                        #[expect(clippy::integer_division, reason = "Intentional")]
                        if (bitmap.data[bitmap_offset / 8] >> (7 - (bitmap_offset % 8))) & 1 == 0 {
                            &colour[0]
                        } else {
                            &colour[1]
                        },
                    );
                }
            }

            return Ok(());
        },
        scheme,
    )?;

    return write(output, &canvas, &meta);
}

/// Write image to disk, adding all available metadata.
fn write(output: &mut Output, canvas: &[u8], meta: &Meta) -> ExitCode {
    let mut encoder = Encoder::new(
        BufWriter::new(output),
        u32::from(meta.width()) * u32::from(meta.font_width()) * u32::from(meta.aspect_ratio().0),
        u32::from(meta.height()) * u32::from(meta.font_height()) * u32::from(meta.aspect_ratio().1),
    );
    encoder.set_color(ColorType::Rgb);
    encoder.set_depth(BitDepth::Eight);
    encoder.set_pixel_dims(Some(PixelDimensions {
        xppu: u32::from(meta.aspect_ratio().0),
        yppu: u32::from(meta.aspect_ratio().1),
        unit: Unit::Unspecified,
    }));
    encoder.set_compression(Compression::Best);
    encoder.validate_sequence(true);
    let mut writer = encoder.write_header()?;
    writer.write_image_data(canvas)?;
    if meta.title().is_some() {
        let mut title = ITXtChunk::new(String::from("Title"), &meta.title);
        title.compress_text().map_err(|err| return ExitCode::ERROR(err.to_string()))?;
        writer.write_text_chunk(&title)?;
    }
    if meta.author().is_some() {
        let mut author = ITXtChunk::new(String::from("Author"), &meta.author);
        author.compress_text().map_err(|err| return ExitCode::ERROR(err.to_string()))?;
        writer.write_text_chunk(&author)?;
    }
    if meta.group().is_some() {
        let mut group = ITXtChunk::new(String::from("Group"), &meta.group);
        group.compress_text().map_err(|err| return ExitCode::ERROR(err.to_string()))?;
        writer.write_text_chunk(&group)?;
    }
    if meta.date().is_some() {
        let mut date = ITXtChunk::new(String::from("Date"), &meta.date);
        date.compress_text().map_err(|err| return ExitCode::ERROR(err.to_string()))?;
        writer.write_text_chunk(&date)?;
    }

    for (i, note) in meta.notes().iter().enumerate() {
        #[expect(clippy::cast_possible_truncation, reason = "Range is [0,3]")]
        #[expect(clippy::cast_sign_loss, reason = "Range is [0,3]")]
        #[expect(clippy::cast_precision_loss, reason = "Range is [0,3]")]
        let mut note = ITXtChunk::new(
            format!("Notes[{:0width$}]", i, width = (meta.notes().len() as f32).log10().ceil() as usize),
            note,
        );
        note.compress_text().map_err(|err| return ExitCode::ERROR(err.to_string()))?;
        writer.write_text_chunk(&note)?;
    }

    return ExitCode::OK;
}

#[path = "."]
#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;

    #[path = "../../libs/internal/test_utils.rs"]
    mod test;

    #[test]
    fn no_input() {
        assert_eq!(exec(&[String::from("cp437-to-png")]), ExitCode::USAGE(String::from("Missing input file")));
    }

    #[test]
    fn too_many_args() {
        assert_eq!(
            exec(&[String::from("cp437-to-png"), String::from("a"), String::from("b"), String::from("c")]),
            ExitCode::USAGE(String::from("Too many arguments")),
        );
    }

    #[ignore]
    #[test]
    fn stdout() {
        assert_eq!(
            exec(&[String::from("cp437-to-png"), String::from("a")]),
            ExitCode::USAGE(String::from("Refusing to write to terminal")),
        );
    }

    #[test]
    fn simple() -> Result<(), String> {
        return test::file(
            |i, o| return run(i, o, &String::from("CLASSIC")),
            "res/test/simple.ans",
            "res/test/simple.png",
        );
    }

    #[test]
    fn meta() -> Result<(), String> {
        return test::file(|i, o| return run(i, o, &String::from("CLASSIC")), "res/test/meta.ans", "res/test/meta.png");
    }

    #[test]
    fn notes() -> Result<(), String> {
        return test::file(
            |i, o| return run(i, o, &String::from("CLASSIC")),
            "res/test/comments.ans",
            "res/test/comments.png",
        );
    }

    #[test]
    fn background() -> Result<(), String> {
        return test::file(
            |i, o| return run(i, o, &String::from("CLASSIC")),
            "res/test/background.ans",
            "res/test/background.png",
        );
    }

    #[test]
    fn logo() -> Result<(), String> {
        return test::file(|i, o| return run(i, o, &String::from("CLASSIC")), "res/logo/logo.ans", "res/logo/logo.png");
    }

    #[test]
    fn banner() -> Result<(), String> {
        return test::file(
            |i, o| return run(i, o, &String::from("CLASSIC")),
            "res/banner/banner.ans",
            "res/banner/banner.png",
        );
    }
}
