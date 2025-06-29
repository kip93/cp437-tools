//! Fonts used for rendering PNG images
//!
//! These fonts are free to use under the
//! [CC-BY-SA-4.0](https://creativecommons.org/licenses/by-sa/4.0) license.
//!
//! See <https://int10h.org/oldschool-pc-fonts>
//!

use lazy_static::lazy_static;
use rust_embed::RustEmbed;
use ttf_parser::Face;

#[derive(RustEmbed)]
#[folder = "$CARGO_MANIFEST_DIR/res/fonts"]
#[include = "*.otb"]
#[include = "*.woff"]
struct Fonts;

lazy_static! {
    /// IBM VGA 8x16 raw font.
    pub static ref VGA_8X16_OTB: Vec<u8> = Fonts::get("IBM VGA.8x16.otb").expect("File exists").data.into_owned();
    /// IBM VGA 9x16 raw font.
    pub static ref VGA_9X16_OTB: Vec<u8> = Fonts::get("IBM VGA.9x16.otb").expect("File exists").data.into_owned();
    /// IBM VGA 8x16 woff font.
    pub static ref VGA_8X16_WOFF: Vec<u8> = Fonts::get("IBM VGA.8x16.woff").expect("File exists").data.into_owned();
    /// IBM VGA 9x16 woff font.
    pub static ref VGA_9X16_WOFF: Vec<u8> = Fonts::get("IBM VGA.9x16.woff").expect("File exists").data.into_owned();

    /// IBM VGA 8x16 font.
    ///
    /// See [`ttf_parser::Face`](https://docs.rs/ttf-parser/latest/ttf_parser/struct.Face.html)
    ///
    pub static ref VGA_8X16: Face<'static> = Face::parse(&VGA_8X16_OTB, 0).expect("Valid font");

    /// IBM VGA 9x16 font.
    ///
    /// See [`ttf_parser::Face`](https://docs.rs/ttf-parser/latest/ttf_parser/struct.Face.html)
    ///
    pub static ref VGA_9X16: Face<'static> = Face::parse(&VGA_9X16_OTB, 0).expect("Valid font");
}
