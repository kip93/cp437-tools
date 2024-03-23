use lazy_static::lazy_static;
use rust_embed::RustEmbed;
use ttf_parser::Face;

#[derive(RustEmbed)]
#[folder = "$CARGO_MANIFEST_DIR/res/fonts"]
#[include = "*.otb"]
struct Fonts;

lazy_static! {
    static ref VGA_8X16_OTB: Vec<u8> = Fonts::get("IBM VGA.8x16.otb").unwrap().data.into_owned();
    static ref VGA_9X16_OTB: Vec<u8> = Fonts::get("IBM VGA.9x16.otb").unwrap().data.into_owned();
    pub static ref VGA_8X16: Face<'static> = Face::parse(&VGA_8X16_OTB, 0).unwrap();
    pub static ref VGA_9X16: Face<'static> = Face::parse(&VGA_9X16_OTB, 0).unwrap();
}
