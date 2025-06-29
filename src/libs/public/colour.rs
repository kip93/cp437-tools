//! ANSI colour schemes.

use regex::Regex;
#[cfg(feature = "_gen")]
use strum_macros::EnumIter;

/// A collection of colour schemes.
///
/// Each entry is a list of 16 RGB values corresponding to the 4-bit colours
/// used by CP437 files.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "_gen", derive(EnumIter))]
pub enum ColourScheme {
    /// The classic scheme.
    ///
    /// ![CLASSIC scheme][scheme]
    #[cfg_attr(all(),
        doc = ::embed_doc_image::embed_image!("scheme", "res/schemes/CLASSIC.png"),
    )]
    CLASSIC,
    /// A modern looking scheme.
    ///
    /// ![MODERN scheme][scheme]
    #[cfg_attr(all(),
        doc = ::embed_doc_image::embed_image!("scheme", "res/schemes/MODERN.png"),
    )]
    MODERN,
    /// A [catppuccin](https://catppuccin.com/palette/)-based colour scheme.
    ///
    /// ![CATPPUCCIN scheme][scheme]
    #[cfg_attr(all(),
        doc = ::embed_doc_image::embed_image!("scheme", "res/schemes/CATPPUCCIN.png"),
    )]
    CATPPUCCIN,
    /// A [dracula](https://draculatheme.com/contribute#color-palette)-based colour scheme.
    ///
    /// ![DRACULA scheme][scheme]
    #[cfg_attr(all(),
        doc = ::embed_doc_image::embed_image!("scheme", "res/schemes/DRACULA.png"),
    )]
    DRACULA,
    /// A [rosé-pine](https://rosepinetheme.com/palette/)-based colour scheme.
    ///
    /// ![ROSEPINE scheme][scheme]
    #[cfg_attr(all(),
        doc = ::embed_doc_image::embed_image!("scheme", "res/schemes/ROSEPINE.png"),
    )]
    ROSEPINE,
    /// A configurable scheme.
    CUSTOM([[u8; 3]; 16]),
}

impl ColourScheme {
    /// Get the string representation of a scheme.
    #[must_use]
    pub fn name(&self) -> String {
        return match self {
            ColourScheme::CLASSIC => String::from("CLASSIC"),
            ColourScheme::MODERN => String::from("MODERN"),
            ColourScheme::CATPPUCCIN => String::from("CATPPUCCIN"),
            ColourScheme::DRACULA => String::from("DRACULA"),
            ColourScheme::ROSEPINE => String::from("ROSEPINE"),
            ColourScheme::CUSTOM(colours) => {
                let codes = colours
                    .iter()
                    .map(|colour| {
                        return format!("#{:02x}{:02x}{:02x}", colour[0], colour[1], colour[2]);
                    })
                    .fold(String::new(), |acc, x| return if acc.is_empty() { x } else { format!("{acc},{x}") });
                format!("CUSTOM({codes})")
            },
        };
    }

    /// Get a colour scheme from a string.
    ///
    /// # Errors
    ///
    /// Fails when the theme is invalid.
    ///
    #[expect(clippy::too_many_lines, reason = "Not much that can be done")]
    pub fn get(name: &String) -> Result<ColourScheme, String> {
        let uppercase_name = name.to_uppercase();
        return match uppercase_name.as_str() {
            "CLASSIC" => Ok(ColourScheme::CLASSIC),
            "MODERN" => Ok(ColourScheme::MODERN),
            "CATPPUCCIN" => Ok(ColourScheme::CATPPUCCIN),
            "DRACULA" => Ok(ColourScheme::DRACULA),
            "ROSEPINE" => Ok(ColourScheme::ROSEPINE),
            _ => {
                if uppercase_name.starts_with("CUSTOM(") {
                    if let Some(c) = Regex::new(r"^CUSTOM\(((?:#[0-9A-F]{6},){15}#[0-9A-F]{6})\)$")
                        .expect("Valid regex")
                        .captures(&uppercase_name)
                    {
                        let c = &c[1];

                        Ok(ColourScheme::CUSTOM([
                            // DARK
                            [
                                // BLACK
                                parse_hex(&c[1..3])?,
                                parse_hex(&c[3..5])?,
                                parse_hex(&c[5..7])?,
                            ],
                            [
                                // RED
                                parse_hex(&c[9..11])?,
                                parse_hex(&c[11..13])?,
                                parse_hex(&c[13..15])?,
                            ],
                            [
                                // GREEN
                                parse_hex(&c[17..19])?,
                                parse_hex(&c[19..21])?,
                                parse_hex(&c[21..23])?,
                            ],
                            [
                                // YELLOW
                                parse_hex(&c[25..27])?,
                                parse_hex(&c[27..29])?,
                                parse_hex(&c[29..31])?,
                            ],
                            [
                                // BLUE
                                parse_hex(&c[33..35])?,
                                parse_hex(&c[35..37])?,
                                parse_hex(&c[37..39])?,
                            ],
                            [
                                // MAGENTA
                                parse_hex(&c[41..43])?,
                                parse_hex(&c[43..45])?,
                                parse_hex(&c[45..47])?,
                            ],
                            [
                                // CYAN
                                parse_hex(&c[49..51])?,
                                parse_hex(&c[51..53])?,
                                parse_hex(&c[53..55])?,
                            ],
                            [
                                // WHITE
                                parse_hex(&c[57..59])?,
                                parse_hex(&c[59..61])?,
                                parse_hex(&c[61..63])?,
                            ],
                            // BRIGHT
                            [
                                // BLACK
                                parse_hex(&c[65..67])?,
                                parse_hex(&c[67..69])?,
                                parse_hex(&c[69..71])?,
                            ],
                            [
                                // RED
                                parse_hex(&c[73..75])?,
                                parse_hex(&c[75..77])?,
                                parse_hex(&c[77..79])?,
                            ],
                            [
                                // GREEN
                                parse_hex(&c[81..83])?,
                                parse_hex(&c[83..85])?,
                                parse_hex(&c[85..87])?,
                            ],
                            [
                                // YELLOW
                                parse_hex(&c[89..91])?,
                                parse_hex(&c[91..93])?,
                                parse_hex(&c[93..95])?,
                            ],
                            [
                                // BLUE
                                parse_hex(&c[97..99])?,
                                parse_hex(&c[99..101])?,
                                parse_hex(&c[101..103])?,
                            ],
                            [
                                // MAGENTA
                                parse_hex(&c[105..107])?,
                                parse_hex(&c[107..109])?,
                                parse_hex(&c[109..111])?,
                            ],
                            [
                                // CYAN
                                parse_hex(&c[113..115])?,
                                parse_hex(&c[115..117])?,
                                parse_hex(&c[117..119])?,
                            ],
                            [
                                // WHITE
                                parse_hex(&c[121..123])?,
                                parse_hex(&c[123..125])?,
                                parse_hex(&c[125..127])?,
                            ],
                        ]))
                    } else {
                        Err(format!("Unparseable colour scheme: {name}"))
                    }
                } else {
                    Err(format!("Unknown scheme: {name}"))
                }
            },
        };
    }

    /// Get this scheme's colours.
    #[must_use]
    pub fn colours(&self) -> [[u8; 3]; 16] {
        return match self {
            ColourScheme::CLASSIC => [
                // DARK
                [0x00, 0x00, 0x00], // BLACK
                [0xAB, 0x00, 0x00], // RED
                [0x00, 0xAB, 0x00], // GREEN
                [0xAB, 0x57, 0x00], // YELLOW
                [0x00, 0x00, 0xAB], // BLUE
                [0xAB, 0x00, 0xAB], // MAGENTA
                [0x00, 0xAB, 0xAB], // CYAN
                [0xAB, 0xAB, 0xAB], // WHITE
                // BRIGHT
                [0x57, 0x57, 0x57], // BLACK
                [0xFF, 0x57, 0x57], // RED
                [0x57, 0xFF, 0x57], // GREEN
                [0xFF, 0xFF, 0x57], // YELLOW
                [0x57, 0x57, 0xFF], // BLUE
                [0xFF, 0x57, 0xFF], // MAGENTA
                [0x57, 0xFF, 0xFF], // CYAN
                [0xFF, 0xFF, 0xFF], // WHITE
            ],
            ColourScheme::MODERN => [
                // DARK
                [0x0A, 0x0A, 0x0A], // BLACK
                [0x99, 0x4D, 0x4D], // RED
                [0x8C, 0x99, 0x4D], // GREEN
                [0xCC, 0x99, 0x66], // YELLOW
                [0x4D, 0x66, 0x99], // BLUE
                [0xB3, 0x59, 0x86], // MAGENTA
                [0x4D, 0x99, 0x99], // CYAN
                [0x99, 0x99, 0x99], // WHITE
                // BRIGHT
                [0x4D, 0x4D, 0x4D], // BLACK
                [0xCC, 0x7A, 0x7A], // RED
                [0xBE, 0xCC, 0x7A], // GREEN
                [0xFF, 0xCC, 0x99], // YELLOW
                [0x7A, 0x96, 0xCC], // BLUE
                [0xE6, 0x8A, 0xB8], // MAGENTA
                [0x7A, 0xCC, 0xCC], // CYAN
                [0xE6, 0xE6, 0xE6], // WHITE
            ],
            ColourScheme::CATPPUCCIN => [
                // DARK
                [0x23, 0x26, 0x34], // BLACK
                [0xDB, 0x63, 0x63], // RED
                [0x82, 0xBD, 0x64], // GREEN
                [0xD4, 0xAA, 0x68], // YELLOW
                [0x6C, 0x8A, 0xE6], // BLUE
                [0xE6, 0x93, 0xCD], // MAGENTA
                [0x4E, 0xB5, 0xAB], // CYAN
                [0xA5, 0xAD, 0xCE], // WHITE
                // BRIGHT
                [0x51, 0x57, 0x6D], // BLACK
                [0xE7, 0x82, 0x84], // RED
                [0xA6, 0xD1, 0x89], // GREEN
                [0xE5, 0xC8, 0x90], // YELLOW
                [0x8C, 0xAA, 0xEE], // BLUE
                [0xF4, 0xB8, 0xE4], // MAGENTA
                [0x81, 0xC8, 0xBE], // CYAN
                [0xC6, 0xD0, 0xF5], // WHITE
            ],
            ColourScheme::DRACULA => [
                // DARK
                [0x21, 0x22, 0x2C], // BLACK
                [0xFF, 0x55, 0x55], // RED
                [0x50, 0xFA, 0x7B], // GREEN
                [0xF1, 0xFA, 0x8C], // YELLOW
                [0xBD, 0x93, 0xF9], // BLUE
                [0xFF, 0x79, 0xC6], // MAGENTA
                [0x8B, 0xE9, 0xFD], // CYAN
                [0xF8, 0xF8, 0xF2], // WHITE
                // BRIGHT
                [0x62, 0x72, 0xA4], // BLACK
                [0xFF, 0x6E, 0x6E], // RED
                [0x69, 0xFF, 0x94], // GREEN
                [0xFF, 0xFF, 0xA5], // YELLOW
                [0xD6, 0xAC, 0xFF], // BLUE
                [0xFF, 0x92, 0xDF], // MAGENTA
                [0xA4, 0xFF, 0xFF], // CYAN
                [0xFF, 0xFF, 0xFF], // WHITE
            ],
            ColourScheme::ROSEPINE => [
                // DARK
                [0x19, 0x17, 0x24], // BLACK
                [0xB4, 0x52, 0x6E], // RED
                [0x8B, 0x95, 0x4D], // GREEN
                [0xC4, 0x96, 0x56], // YELLOW
                [0x31, 0x74, 0x8F], // BLUE
                [0x90, 0x7A, 0xA9], // MAGENTA
                [0x56, 0x94, 0x9F], // CYAN
                [0x90, 0x8C, 0xAA], // WHITE
                // BRIGHT
                [0x40, 0x3D, 0x52], // BLACK
                [0xEB, 0x6F, 0x92], // RED
                [0xB7, 0xC4, 0x6A], // GREEN
                [0xF6, 0xC1, 0x77], // YELLOW
                [0x3E, 0x8F, 0xB0], // BLUE
                [0xC4, 0xA7, 0xE7], // MAGENTA
                [0x9C, 0xCF, 0xD8], // CYAN
                [0xE0, 0xDE, 0xF4], // WHITE
            ],
            ColourScheme::CUSTOM(scheme) => *scheme,
        };
    }

    /// Get a single colour from this scheme.
    #[inline]
    #[must_use]
    pub fn colour(&self, index: u8) -> [u8; 3] {
        return self.colours()[index as usize];
    }
}

#[inline]
fn parse_hex(hex: &str) -> Result<u8, String> {
    return u8::from_str_radix(hex, 16).map_err(|err| return err.to_string());
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;
    use rand::{rng, Rng};

    #[test]
    fn classic() -> Result<(), String> {
        assert_eq!(ColourScheme::get(&String::from("ClAsSiC"))?, ColourScheme::CLASSIC);
        for i in 0..16 {
            assert_eq!(ColourScheme::CLASSIC.colours()[i], ColourScheme::CLASSIC.colour(i as u8));
        }

        return Ok(());
    }

    #[test]
    fn modern() -> Result<(), String> {
        assert_eq!(ColourScheme::get(&String::from("MoDeRn"))?, ColourScheme::MODERN);
        for i in 0..16 {
            assert_eq!(ColourScheme::MODERN.colours()[i], ColourScheme::MODERN.colour(i as u8));
        }

        return Ok(());
    }

    #[test]
    fn dracula() -> Result<(), String> {
        assert_eq!(ColourScheme::get(&String::from("DrAcUlA"))?, ColourScheme::DRACULA);
        for i in 0..16 {
            assert_eq!(ColourScheme::DRACULA.colours()[i], ColourScheme::DRACULA.colour(i as u8));
        }

        return Ok(());
    }

    #[test]
    fn custom() -> Result<(), String> {
        let colours = [
            [rng().random(), rng().random(), rng().random()],
            [rng().random(), rng().random(), rng().random()],
            [rng().random(), rng().random(), rng().random()],
            [rng().random(), rng().random(), rng().random()],
            [rng().random(), rng().random(), rng().random()],
            [rng().random(), rng().random(), rng().random()],
            [rng().random(), rng().random(), rng().random()],
            [rng().random(), rng().random(), rng().random()],
            [rng().random(), rng().random(), rng().random()],
            [rng().random(), rng().random(), rng().random()],
            [rng().random(), rng().random(), rng().random()],
            [rng().random(), rng().random(), rng().random()],
            [rng().random(), rng().random(), rng().random()],
            [rng().random(), rng().random(), rng().random()],
            [rng().random(), rng().random(), rng().random()],
            [rng().random(), rng().random(), rng().random()],
        ];
        let codes = colours
            .iter()
            .map(|colour| {
                return format!("#{:02x}{:02x}{:02x}", colour[0], colour[1], colour[2]);
            })
            .fold(String::new(), |acc, x| if &acc == "" { x } else { format!("{},{}", acc, x) });
        assert_eq!(ColourScheme::get(&format!("CuStOm({})", codes))?, ColourScheme::CUSTOM(colours));
        assert_eq!(ColourScheme::get(&format!("CuStOm({})", codes))?.name(), format!("CUSTOM({})", codes));
        assert_eq!(ColourScheme::CUSTOM(colours).colours(), colours);

        return Ok(());
    }

    #[test]
    fn custom_unparseable() {
        let result = ColourScheme::get(&String::from("CuStOm()"));
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Unparseable colour scheme: CuStOm()");
    }

    #[test]
    fn invalid() {
        let result = ColourScheme::get(&String::from("x"));
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Unknown scheme: x");
    }
}
