#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::all, clippy::pedantic)]
#![deny(rust_2018_idioms)]
#![warn(rust_2024_compatibility)]

//! Auto-generate a TUI from a `clap` command definition.

mod app;
mod config;
mod error;
mod input;
mod spec;
mod ui;

pub use app::TuiApp;
pub use config::{Keymap, LayoutConfig, Theme, ThemePreset, TuiConfig};
pub use error::TuiError;
