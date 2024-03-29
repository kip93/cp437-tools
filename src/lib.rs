//! A small collection of tools to handle CP437 files.
//!
//! <div class="warning">
//!
//! This crate is primary written to supply CLI commands, not as a reusable
//! library.
//!
//! While I'll try and avoid making changes to the API, be warned that (at
//! least, at the moment) no guarantees are made about its stability.
//!
//! </div>
//!

#![deny(missing_docs)]

pub mod colour;
pub mod cp437;
pub mod fonts;
pub mod meta;

// Internal impl details
#[doc(hidden)]
pub mod exit;
#[doc(hidden)]
pub mod help;
#[doc(hidden)]
pub mod process;

pub use self::{colour::COLOURS, cp437::*, exit::*, meta::Meta, process::*};
