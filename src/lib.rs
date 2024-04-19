//! <div align="center">
//!
//! ![logo][logo]
//!
//! **CP437 tools**
//!
//! A small collection of tools to handle CP437 files.
//!
//! </div>
//!
//! # Binaries
//!
//! ## Meta handling
//!
//! * **cp437-check-meta**
//!
//!   Reads a file's metadata and run some validations to see if there's any
//!   issues with it.
//!
//! * **cp437-read-meta**
//!
//!   Reads and prints a file's metadata, highlighting values to show potential
//!   errors, as well as showing the effective value when a real one is missing.
//!
//! * **cp437-remove-meta**
//!
//!   Takes a file and strips its metadata, piping the resulting file to stdout.
//!
//! * **cp437-set-meta**
//!
//!   Takes a file and modifies its metadata, piping the resulting file to
//!   stdout.
//!
//!   If the file has no metadata, it will add one filled with default values,
//!   and then proceed to add set the given field.
//!
//! ## Rendering
//!
//! * **cp437-to-png**
//!
//!   Renders the given file as a PNG image, piping the resulting file to
//!   stdout.
//!
//!   It will also embed the file's metadata, if available.
//!
//!   ![to-png][png]
//!
//! * **cp437-to-svg**
//!
//!   <div class="warning">
//!   NOTE: This binary is still a WIP. Redering seems to be kinda wonky.
//!   </div>
//!
//!   Renders the given file as an SVG image, piping the resulting file to
//!   stdout.
//!
//!   It will also embed the file's metadata, if available.
//!
//!   ![to-svg][svg]
//!
//! * **cp437-to-txt**
//!
//!   Takes the contents of the file and transpiles them to UTF-8 encoding,
//!   piping the resulting file to stdout.
//!
//!   ![to-txt][txt]
//!
//!
//! # Library
//!
//! <div class="warning">
//!
//! Note that this crate is primary written to supply the CLI commands shown
//! above, not as a reusable library. As such, by default it will pull
//! unnecessary dependencies.
//!
//! To avoid this, disable the default "binaries" feature.
//!
//! (see [cargo#1982](https://github.com/rust-lang/cargo/issues/1982) issue for
//! more details on why this workaround is even needed)
//!
//! </div>
//!
#![cfg_attr(all(),
  doc = ::embed_doc_image::embed_image!("logo", "res/logo/tiny.png"),
  doc = ::embed_doc_image::embed_image!("png", "res/screenshots/png.png"),
  doc = ::embed_doc_image::embed_image!("svg", "res/screenshots/svg.png"),
  doc = ::embed_doc_image::embed_image!("txt", "res/screenshots/txt.png"),
)]
#![deny(missing_docs)]
#![cfg_attr(feature = "binaries", feature(try_trait_v2))] // TODO https://github.com/rust-lang/rust/issues/84277

/// A list of things likely to be required by most dependents.
pub mod prelude {
    pub use super::{
        colour::*,
        cp437::*,
        meta::{self, Meta},
    };
}

#[path = "libs/public/mod.rs"]
mod public;
pub use self::public::*;

#[path = "libs/internal/mod.rs"]
pub mod internal;
