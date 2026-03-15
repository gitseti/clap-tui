#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::all, clippy::pedantic)]
#![deny(rust_2018_idioms)]
#![warn(rust_2024_compatibility)]

//! Auto-generate a TUI from a `clap` command definition.

mod app;
mod argv_serializer;
mod config;
mod controller;
mod editor_state;
mod error;
mod frame_snapshot;
mod form_editor;
mod input;
mod runtime;
mod spec;
mod ui;
mod update;
mod view;

pub use app::TuiApp;
pub use config::{Keymap, LayoutConfig, Theme, ThemePreset, TuiConfig};
pub use error::TuiError;
pub use runtime::{CrosstermRuntime, Runtime};
