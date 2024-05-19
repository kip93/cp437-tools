//! Render a file as a thumbnail

use image::{
    codecs::png::{CompressionType, FilterType, PngEncoder},
    ExtendedColorType, /*ImageBuffer,*/ ImageEncoder,
};
use std::{env::args, io::BufWriter};

use cp437_tools::{
    internal::{process_to_file, ExitCode, Input, Output},
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
    } else if args.len() < 3 {
        ExitCode::USAGE(String::from("Missing output file"))
    } else if args.len() < 4 {
        ExitCode::USAGE(String::from("Missing size"))
    } else if args.len() > 4 {
        ExitCode::USAGE(String::from("Too many arguments"))
    } else {
        process_to_file(&args[1], &args[2], |i, o| return draw(i, o, &args[3]))
    };

    exit_code.print();
    return exit_code;
}

fn draw(input: &mut Input, output: &mut Output, size: &String) -> ExitCode {
    let size = size.parse::<usize>()?;
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
    input.read_by_bytes_full(|byte, (x, y), colour| {
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
    })?;

    return write(output, &canvas, meta, size);
}

fn write(output: &mut Output, canvas: &[u8], meta: Meta, _size: usize) -> ExitCode {
    // TODO write resized image
    PngEncoder::new_with_quality(
        BufWriter::new(output),
        CompressionType::Best,
        FilterType::Adaptive,
    )
    .write_image(
        canvas,
        meta.width() as u32 * meta.font_width() as u32 * meta.aspect_ratio().0 as u32,
        meta.height() as u32 * meta.font_height() as u32 * meta.aspect_ratio().1 as u32,
        ExtendedColorType::Rgb8,
    )
    .map_err(|err| return err.to_string())?;
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
            run(vec![String::from("cp437-thumbnail")]),
            ExitCode::USAGE(String::from("Missing input file"))
        );
    }

    #[test]
    fn no_output() {
        assert_eq!(
            run(vec![String::from("cp437-thumbnail"), String::from("a"),]),
            ExitCode::USAGE(String::from("Missing output file"))
        );
    }

    #[test]
    fn missing_size() {
        assert_eq!(
            run(vec![
                String::from("cp437-thumbnail"),
                String::from("a"),
                String::from("b"),
            ]),
            ExitCode::USAGE(String::from("Missing size"))
        );
    }

    #[test]
    fn too_many_args() {
        assert_eq!(
            run(vec![
                String::from("cp437-thumbnail"),
                String::from("a"),
                String::from("b"),
                String::from("c"),
                String::from("d"),
            ]),
            ExitCode::USAGE(String::from("Too many arguments"))
        );
    }

    // #[test]
    // fn simple() -> Result<(), String> {
    //     return test::file(draw, "res/test/simple.ans", "res/test/simple.png");
    // }

    // #[test]
    // fn meta() -> Result<(), String> {
    //     return test::file(draw, "res/test/meta.ans", "res/test/meta.png");
    // }

    // #[test]
    // fn notes() -> Result<(), String> {
    //     return test::file(draw, "res/test/comments.ans", "res/test/comments.png");
    // }

    // #[test]
    // fn background() -> Result<(), String> {
    //     return test::file(draw, "res/test/background.ans", "res/test/background.png");
    // }

    // #[test]
    // fn logo() -> Result<(), String> {
    //     return test::file(draw, "res/logo/logo.ans", "res/logo/logo.png");
    // }
}
