//! Render a file as a PNG

use png::{
    text_metadata::ITXtChunk, BitDepth, ColorType, Compression, Encoder, PixelDimensions, Unit,
};
use std::{
    env::args,
    io::{stdout, BufWriter, IsTerminal},
};

use cp437_tools::{
    internal::{process, ExitCode, Input, Output},
    prelude::{Meta, CP437_TO_UTF8},
};

#[allow(dead_code)]
pub fn main() -> ExitCode {
    return run(args().collect());
}

#[inline]
pub fn run(args: Vec<String>) -> ExitCode {
    let exit_code = if args.len() < 2 {
        ExitCode::USAGE(String::from("Missing input file"))
    } else if args.len() > 3 {
        ExitCode::USAGE(String::from("Too many arguments"))
    } else if stdout().is_terminal() {
        ExitCode::USAGE(String::from("Refusing to write to terminal"))
    } else {
        process(&args[1], |i, o| {
            return draw(i, o, args.get(2).unwrap_or(&String::from("CLASSIC")));
        })
    };

    exit_code.print();
    return exit_code;
}

fn draw(input: &mut Input, output: &mut Output, scheme: &String) -> ExitCode {
    let meta = input.meta.clone().unwrap_or(Meta {
        size: input.size,
        ..Default::default()
    });

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
                        .ok_or_else(|| format!("Glyph for 0x{:02X} is missing", byte))?,
                    font_height as u16,
                )
                .ok_or_else(|| format!("Glyph bitmap for 0x{:02X} is missing", byte))?;

            for i in 0..(font_width * ar_x) {
                for j in 0..(font_height * ar_y) {
                    let offset = 3
                        * ((y * font_height * ar_y + j) * font_width * ar_x * width
                            + (x * font_width * ar_x + i));
                    let bitmap_offset = i / ar_x + j / ar_y * font_width;
                    canvas[offset..offset + 3].copy_from_slice(
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

    return write(output, &canvas, meta);
}

fn write(output: &mut Output, canvas: &[u8], meta: Meta) -> ExitCode {
    let mut encoder = Encoder::new(
        BufWriter::new(output),
        meta.width() as u32 * meta.font_width() as u32 * meta.aspect_ratio().0 as u32,
        meta.height() as u32 * meta.font_height() as u32 * meta.aspect_ratio().1 as u32,
    );
    encoder.set_color(ColorType::Rgb);
    encoder.set_depth(BitDepth::Eight);
    encoder.set_pixel_dims(Some(PixelDimensions {
        xppu: meta.aspect_ratio().0 as u32,
        yppu: meta.aspect_ratio().1 as u32,
        unit: Unit::Unspecified,
    }));
    encoder.set_compression(Compression::Best);
    encoder.validate_sequence(true);
    let mut writer = encoder.write_header().unwrap();
    writer.write_image_data(canvas).unwrap();
    if meta.title().is_some() {
        let mut title = ITXtChunk::new(String::from("Title"), &meta.title);
        title
            .compress_text()
            .map_err(|err| return ExitCode::ERROR(err.to_string()))?;
        writer.write_text_chunk(&title).unwrap();
    }
    if meta.author().is_some() {
        let mut author = ITXtChunk::new(String::from("Author"), &meta.author);
        author
            .compress_text()
            .map_err(|err| return ExitCode::ERROR(err.to_string()))?;
        writer.write_text_chunk(&author).unwrap();
    }
    if meta.group().is_some() {
        let mut group = ITXtChunk::new(String::from("Group"), &meta.group);
        group
            .compress_text()
            .map_err(|err| return ExitCode::ERROR(err.to_string()))?;
        writer.write_text_chunk(&group).unwrap();
    }
    if meta.date().is_some() {
        let mut date = ITXtChunk::new(String::from("Date"), &meta.date);
        date.compress_text()
            .map_err(|err| return ExitCode::ERROR(err.to_string()))?;
        writer.write_text_chunk(&date).unwrap();
    }

    for (i, note) in meta.notes().iter().enumerate() {
        let mut note = ITXtChunk::new(
            format!(
                "Notes[{:0width$}]",
                i,
                width = (meta.notes().len() as f32).log10().ceil() as usize
            ),
            note,
        );
        note.compress_text()
            .map_err(|err| return ExitCode::ERROR(err.to_string()))?;
        writer.write_text_chunk(&note).unwrap();
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
        assert_eq!(
            run(vec![String::from("cp437-to-png")]),
            ExitCode::USAGE(String::from("Missing input file"))
        );
    }

    #[test]
    fn too_many_args() {
        assert_eq!(
            run(vec![
                String::from("cp437-to-png"),
                String::from("a"),
                String::from("b"),
                String::from("c"),
            ]),
            ExitCode::USAGE(String::from("Too many arguments"))
        );
    }

    #[ignore]
    #[test]
    fn stdout() {
        assert_eq!(
            run(vec![String::from("cp437-to-png"), String::from("a")]),
            ExitCode::USAGE(String::from("Refusing to write to terminal"))
        );
    }

    #[test]
    fn simple() -> Result<(), String> {
        return test::file(
            |i, o| return draw(i, o, &String::from("CLASSIC")),
            "res/test/simple.ans",
            "res/test/simple.png",
        );
    }

    #[test]
    fn meta() -> Result<(), String> {
        return test::file(
            |i, o| return draw(i, o, &String::from("CLASSIC")),
            "res/test/meta.ans",
            "res/test/meta.png",
        );
    }

    #[test]
    fn notes() -> Result<(), String> {
        return test::file(
            |i, o| return draw(i, o, &String::from("CLASSIC")),
            "res/test/comments.ans",
            "res/test/comments.png",
        );
    }

    #[test]
    fn background() -> Result<(), String> {
        return test::file(
            |i, o| return draw(i, o, &String::from("CLASSIC")),
            "res/test/background.ans",
            "res/test/background.png",
        );
    }

    #[test]
    fn logo() -> Result<(), String> {
        return test::file(
            |i, o| return draw(i, o, &String::from("CLASSIC")),
            "res/logo/logo.ans",
            "res/logo/logo.png",
        );
    }

    #[test]
    fn banner() -> Result<(), String> {
        return test::file(
            |i, o| return draw(i, o, &String::from("CLASSIC")),
            "res/banner/banner.ans",
            "res/banner/banner.png",
        );
    }
}
