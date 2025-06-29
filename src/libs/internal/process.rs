use std::{
    cmp::min,
    fs::File,
    io::{self, stdout, BufReader, Read as _, Seek as _, Write},
    path::Path,
};

use crate::{
    internal::ExitCode,
    prelude::{
        meta::{self, Meta},
        ColourScheme,
    },
};

pub struct Input {
    real: File,
    pub size: u32,
    pub meta: Option<Meta>,
}

struct Colour {
    bg: [u8; 3],
    fg: [u8; 3],
    bright: bool,
}

impl Input {
    pub fn new<P: AsRef<Path>>(input: P) -> Result<Self, ExitCode> {
        let mut real = File::open(input)?;

        let meta = meta::read(&mut real)?;

        let size = match meta {
            Some(ref meta) => meta.size,
            None => u32::try_from(real.metadata()?.len())?,
        };

        return Ok(Self { real, size, meta });
    }

    pub fn read_by_chunks<'a, F: for<'b> FnMut(&'b [u8]) -> Result<(), ExitCode> + 'a>(
        &mut self,
        mut callback: F,
    ) -> Result<(), ExitCode> {
        self.real.rewind()?;

        let mut chunk = vec![0; 1 << 12]; // 4k chunks
        let mut reader = BufReader::with_capacity(chunk.len(), &self.real);

        let mut index = 0;
        while index < self.size {
            let count = u32::try_from(reader.read(&mut chunk)?)?;
            let count = min(count, self.size.saturating_sub(index));
            index += count;
            callback(&chunk[..count as usize])?;
        }

        return Ok(());
    }

    #[inline]
    pub fn read_by_bytes<'a, F: for<'b> FnMut(u8) -> Result<(), ExitCode> + 'a>(
        &mut self,
        mut callback: F,
    ) -> Result<(), ExitCode> {
        return self.read_by_chunks(|chunk| {
            for byte in chunk {
                callback(*byte)?;
            }

            return Ok(());
        });
    }

    #[inline]
    pub fn read_by_bytes_full<'a, F: for<'b> FnMut(u8, (u16, u16), [[u8; 3]; 2]) -> Result<(), ExitCode> + 'a>(
        &mut self,
        mut callback: F,
        scheme: &String,
    ) -> Result<(), ExitCode> {
        let meta = self.meta.clone().unwrap_or_else(|| {
            return Meta { size: self.size, ..Default::default() };
        });
        let colours = ColourScheme::get(scheme)?.colours();
        let mut colour = Colour { bg: colours[0], fg: colours[15], bright: false };
        let mut control: Vec<u8> = vec![];
        let (mut x, mut y) = (0, 0);

        return self.read_by_bytes(|byte| {
            if y >= meta.height() {
                return Ok(());
            }

            if !control.is_empty() {
                if byte == b'm' {
                    for mut num in control[2..].split(|r#char| return *r#char == b';') {
                        if num.is_empty() {
                            num = b"0";
                        }
                        let num = String::from_utf8(num.to_vec())?.parse::<usize>()?;
                        match num {
                            0 => {
                                colour.bg = colours[0];
                                colour.fg = colours[15];
                                colour.bright = false;
                            },
                            1 => {
                                colour.bright = true;
                            },
                            30..=37 => {
                                colour.fg = colours[num - 30 + (if colour.bright { 8 } else { 0 })];
                            },
                            39 => {
                                colour.fg = colours[15];
                            },
                            40..=47 => {
                                colour.bg = colours[num - 40];
                            },
                            49 => {
                                colour.bg = colours[0];
                            },
                            90..=97 => {
                                colour.fg = colours[num - 82];
                            },
                            100..=107 => {
                                colour.bg = colours[num - 92];
                            },
                            _ => {
                                eprintln!("\x1B[33mWARN: Unknown SGR param: {num}\x1B[0m");
                            },
                        }
                    }
                    control.clear();
                } else if byte == b't' {
                    let cmd = control[2..].split(|r#char| return *r#char == b';').collect::<Vec<&[u8]>>();
                    let r = String::from_utf8(cmd[1].to_vec())?.parse::<u8>()?;
                    let g = String::from_utf8(cmd[2].to_vec())?.parse::<u8>()?;
                    let b = String::from_utf8(cmd[3].to_vec())?.parse::<u8>()?;
                    match cmd[0] {
                        b"0" => {
                            colour.bg = [r, g, b];
                        },
                        b"1" => {
                            colour.fg = [r, g, b];
                        },
                        _ => {
                            eprintln!(
                                "\x1B[33mWARN: Invalid RGB target: {}\x1B[0m",
                                String::from_utf8(cmd[0].to_vec())?
                            );
                        },
                    }
                    control.clear();
                } else if byte == b'B' {
                    y += String::from_utf8(control[2..].to_vec())?.parse::<u16>()?;
                    control.clear();
                } else if byte == b'C' {
                    x = min(x + String::from_utf8(control[2..].to_vec())?.parse::<u16>()?, meta.width() - 1);
                    control.clear();
                } else if control.len() > 1 && (0x40..=0x7E).contains(&byte) {
                    eprintln!("\x1B[33mWARN: Invalid control sequence argument: 0x{byte:02X}\x1B[0m");
                    control.clear();
                } else {
                    control.push(byte);
                }
            } else if byte == 0x1B {
                control.push(byte);
            } else if byte == 0x0D {
                (x, y) = (0, y);
            } else if byte == 0x0A {
                (x, y) = (0, y + 1);
            } else {
                callback(byte, (x, y), [colour.bg, colour.fg])?;
                x += 1;
                if x >= meta.width() {
                    (x, y) = (0, y + 1);
                }
            }

            return Ok(());
        });
    }
}

pub struct Output {
    real: Box<dyn Write>,
}

impl Output {
    pub fn file<P: AsRef<Path>>(output: P) -> Result<Self, ExitCode> {
        let real = Box::new(File::create_new(output)?) as Box<dyn Write>;
        return Ok(Self { real });
    }

    pub fn stdout() -> Result<Self, ExitCode> {
        let real = Box::new(stdout()) as Box<dyn Write>;
        return Ok(Self { real });
    }

    pub fn write(&mut self, text: &[u8]) -> Result<(), ExitCode> {
        return Ok(self.write_all(text)?);
    }
}

impl Write for Output {
    fn write(&mut self, buf: &[u8]) -> Result<usize, io::Error> {
        return self.real.write(buf);
    }

    fn flush(&mut self) -> Result<(), io::Error> {
        return self.real.flush();
    }
}

#[inline]
pub fn process<F: for<'a> FnOnce(&'a mut Input, &'a mut Output) -> ExitCode>(input: &String, callback: F) -> ExitCode {
    return callback(&mut Input::new(input)?, &mut Output::stdout()?);
}
