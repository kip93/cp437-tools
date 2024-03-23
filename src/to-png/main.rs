use atty::{self, Stream::Stdout};
use png::{text_metadata::ITXtChunk, BitDepth, ColorType, Encoder, PixelDimensions, Unit};
use std::{
    cmp::min,
    env::args,
    fs::File,
    io::{BufWriter, Read, Seek, SeekFrom, Write},
    process::ExitCode,
};
use ttf_parser::Face;

use cp437_tools::{_process, colour::COLOURS, cp437::CP437, help, meta::Meta};

mod fonts;

struct XY {
    x: usize,
    y: usize,
}

#[allow(dead_code)]
pub fn main() -> ExitCode {
    return run(args().collect());
}

#[inline]
pub fn run(args: Vec<String>) -> ExitCode {
    if args.len() < 2 {
        eprintln!("\x1B[31mERROR: Missing input file\x1B[0m");
        help::print();
        return ExitCode::from(1);
    } else if args.len() > 3 {
        eprintln!("\x1B[31mERROR: Too many arguments\x1B[0m");
        help::print();
        return ExitCode::from(1);
    } else if args.len() == 2 && atty::is(Stdout) {
        eprintln!("\x1B[31mERROR: Refusing to write to terminal\x1B[0m");
        help::print();
        return ExitCode::from(1);
    }

    return _process(&args[1], &args.get(2).map(|x| return x.to_string()), draw);
}

fn draw(input: &mut File, output: &mut Box<dyn Write>, meta: Option<Meta>) -> Result<(), String> {
    let (bytes, size, font, ratio, face) = get_params(&meta);
    let bytes = match bytes {
        Some(b) => b,
        None => input.metadata().map_err(|x| return x.to_string())?.len() as u32,
    };

    let mut canvas = vec![0; size.x * size.y * 3 * font.x * font.y * ratio.x * ratio.y];
    let mut colour = (COLOURS[0], COLOURS[15]);
    let mut bright = false;
    let mut control: Vec<u8> = vec![];
    let mut index = 0;

    let mut chunk = vec![0; 1 << 12]; // 4k chunks
    input
        .seek(SeekFrom::Start(0))
        .map_err(|x| return x.to_string())?;
    for i in 0..(bytes as usize).div_ceil(chunk.len()) {
        let end = min(chunk.len(), bytes as usize - (i * chunk.len()));
        input
            .read_exact(&mut chunk[..end])
            .map_err(|x| return x.to_string())?;

        for item in chunk.iter().take(end) {
            insert(
                *item,
                &mut canvas,
                &size,
                &ratio,
                &font,
                &face,
                &mut index,
                &mut colour,
                &mut bright,
                &mut control,
            );
        }
    }

    return write(
        output,
        &canvas,
        XY {
            x: size.x * font.x,
            y: size.y * font.y,
        },
        ratio,
        meta,
    );
}

fn get_params<'a>(meta: &Option<Meta>) -> (Option<u32>, XY, XY, XY, &'a Face<'a>) {
    match meta {
        Some(ref m) => {
            return (
                Some(m.size),
                XY {
                    x: if m.width > 0 { m.width as usize } else { 80 },
                    y: if m.height > 0 { m.height as usize } else { 25 },
                },
                if m.flags & 0x02 == 0x02 {
                    XY { x: 8, y: 16 }
                } else {
                    XY { x: 9, y: 16 }
                },
                if m.flags & 0x18 == 0x10 {
                    XY { x: 1, y: 1 }
                } else if m.flags & 0x02 == 0x02 {
                    XY { x: 6, y: 5 }
                } else {
                    XY { x: 20, y: 27 }
                },
                if m.flags & 0x02 == 0x02 {
                    &fonts::VGA_8X16 as &Face
                } else {
                    &fonts::VGA_9X16 as &Face
                },
            )
        }
        None => {
            return (
                None,
                XY { x: 80, y: 25 },
                XY { x: 9, y: 16 },
                XY { x: 20, y: 27 },
                &fonts::VGA_9X16 as &Face,
            )
        }
    }
}

#[inline]
#[allow(clippy::too_many_arguments)]
fn insert<'a>(
    byte: u8,
    canvas: &mut Vec<u8>,
    size: &XY,
    ratio: &XY,
    font: &XY,
    face: &'a Face<'a>,
    index: &mut usize,
    colour: &mut ([u8; 3], [u8; 3]),
    bright: &mut bool,
    control: &mut Vec<u8>,
) {
    if *index >= size.x * size.y {
        return;
    }

    if !control.is_empty() {
        if byte == b'm' {
            for mut num in control[2..].split(|x| return *x == b';') {
                if num.is_empty() {
                    num = b"0";
                }
                let num = String::from_utf8_lossy(num).parse::<usize>().unwrap();
                match num {
                    0 => {
                        *colour = (COLOURS[0], COLOURS[15]);
                        *bright = false;
                    }
                    1 => {
                        *bright = true;
                    }
                    30..=37 => {
                        *colour = (colour.0, COLOURS[num - 30 + (if *bright { 8 } else { 0 })]);
                    }
                    39 => {
                        *colour = (colour.0, COLOURS[15]);
                    }
                    40..=47 => {
                        *colour = (COLOURS[num - 40], colour.1);
                    }
                    49 => {
                        *colour = (COLOURS[0], colour.1);
                    }
                    90..=97 => {
                        *colour = (colour.0, COLOURS[num - 82]);
                    }
                    100..=107 => {
                        *colour = (COLOURS[num - 92], colour.1);
                    }
                    _ => {
                        eprintln!("\x1B[33mWARN: Unknown SGR param: {}\x1B[0m", num);
                    }
                }
            }
            control.clear();
        } else if byte == b't' {
            let cmd = control[2..]
                .split(|x| return *x == b';')
                .collect::<Vec<&[u8]>>();
            let r = String::from_utf8_lossy(cmd[1]).parse::<u8>().unwrap();
            let g = String::from_utf8_lossy(cmd[2]).parse::<u8>().unwrap();
            let b = String::from_utf8_lossy(cmd[3]).parse::<u8>().unwrap();
            match cmd[0] {
                b"0" => {
                    *colour = ([r, g, b], colour.1);
                }
                b"1" => {
                    *colour = (colour.0, [r, g, b]);
                }
                _ => {
                    eprintln!(
                        "\x1B[33mWARN: Invalid RGB target: {}\x1B[0m",
                        String::from_utf8_lossy(cmd[0])
                    );
                }
            }
            control.clear();
        } else if control.len() > 1 && (0x40..=0x7E).contains(&byte) {
            eprintln!(
                "\x1B[33mWARN: Invalid control sequence argument: {}\x1B[0m",
                byte
            );
            control.clear();
        } else {
            control.push(byte);
        }
    } else if byte == 0x1B {
        control.push(byte);
    } else if byte == 0x0A {
        *index = ((*index - 1) / size.x + 1) * size.x;
    } else if byte == 0x0D {
        // Do nothing
    } else {
        let bitmap = face
            .glyph_raster_image(
                face.glyph_index(CP437[byte as usize])
                    .expect("Failed to fetch glyph"),
                font.y as u16,
            )
            .expect("Fail to fetch glyph bitmap");
        for x in 0..(font.x * ratio.x) {
            for y in 0..(font.y * ratio.y) {
                let offset = 3
                    * ((*index / size.x) * font.x * ratio.x * font.y * ratio.y * size.x
                        + (*index % size.x) * font.x * ratio.x
                        + x
                        + y * font.x * ratio.x * size.x);
                canvas[offset..offset + 3].copy_from_slice(
                    if (bitmap.data[(x / ratio.x + y / ratio.y * font.x) / 8]
                        >> (7 - ((x / ratio.x + y / ratio.y * font.x) % 8)))
                        & 1
                        == 0
                    {
                        &colour.0
                    } else {
                        &colour.1
                    },
                );
            }
        }
        *index += 1;
    }
}

fn write(
    file: &mut Box<dyn Write>,
    canvas: &[u8],
    size: XY,
    ratio: XY,
    meta: Option<Meta>,
) -> Result<(), String> {
    let mut encoder = Encoder::new(
        BufWriter::new(file),
        size.x as u32 * ratio.x as u32,
        size.y as u32 * ratio.y as u32,
    );
    encoder.set_color(ColorType::Rgb);
    encoder.set_depth(BitDepth::Eight);
    encoder.set_pixel_dims(Some(PixelDimensions {
        xppu: ratio.x as u32,
        yppu: ratio.y as u32,
        unit: Unit::Unspecified,
    }));
    encoder.validate_sequence(true);
    let mut writer = encoder.write_header().unwrap();
    writer.write_image_data(canvas).unwrap();
    if let Some(meta) = meta {
        if !meta.title.is_empty() {
            let mut title = ITXtChunk::new(String::from("Title"), meta.title);
            title.compress_text().map_err(|x| return x.to_string())?;
            writer.write_text_chunk(&title).unwrap();
        }
        if !meta.author.is_empty() {
            let mut author = ITXtChunk::new(String::from("Author"), meta.author);
            author.compress_text().map_err(|x| return x.to_string())?;
            writer.write_text_chunk(&author).unwrap();
        }
        if !meta.group.is_empty() {
            let mut group = ITXtChunk::new(String::from("Group"), meta.group);
            group.compress_text().map_err(|x| return x.to_string())?;
            writer.write_text_chunk(&group).unwrap();
        }
        if !meta.date.trim().is_empty() {
            let mut date = ITXtChunk::new(String::from("Date"), meta.date);
            date.compress_text().map_err(|x| return x.to_string())?;
            writer.write_text_chunk(&date).unwrap();
        }

        for (i, note) in meta.notes.iter().enumerate() {
            let mut note = ITXtChunk::new(
                format!(
                    "Notes[{:0width$}]",
                    i,
                    width = (meta.notes.len() as f32).log10().ceil() as usize
                ),
                note,
            );
            note.compress_text().map_err(|x| return x.to_string())?;
            writer.write_text_chunk(&note).unwrap();
        }
    }

    return Ok(());
}
