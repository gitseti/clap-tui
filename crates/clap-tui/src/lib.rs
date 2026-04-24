#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::all, clippy::pedantic)]
#![deny(rust_2018_idioms)]
#![warn(rust_2024_compatibility)]

//! `clap-tui` turns a `clap` CLI into an interactive terminal UI while preserving the original
//! command-line interface.
//!
//! You can keep `clap` as the source of truth, collect input in the TUI, and then hand the
//! selected command value back to your normal application dispatch.
//!
//! This crate was heavily inspired by [Trogon](https://github.com/Textualize/trogon).
//! `clap-tui` is a community crate and is not an official `clap` project.
//!
//! # Quick Start
//!
//! The recommended integration model is to define a normal `tui` subcommand in your own CLI and
//! run [`Tui`] from that dispatch branch:
//!
//! ```no_run
//! use clap::Parser;
//! use clap_tui::Tui;
//!
//! #[derive(Debug, Parser, PartialEq, Eq)]
//! #[command(name = "tool")]
//! enum Command {
//!     Tui,
//!     Hello {
//!         #[arg(long, default_value = "world")]
//!         name: String,
//!     },
//! }
//!
//! fn dispatch(command: Command) {
//!     match command {
//!         Command::Tui => {}
//!         Command::Hello { name } => println!("Hello, {name}!"),
//!     }
//! }
//!
//! fn main() -> Result<(), clap_tui::TuiError> {
//!     match Command::parse() {
//!         Command::Tui => {
//!             if let Some(command) = Tui::<Command>::new().run()? {
//!                 dispatch(command);
//!             }
//!         }
//!         command => dispatch(command),
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! # Choosing An Entry Point
//!
//! You probably want [`Tui`].
//!
//! - Use [`Tui::<T>::run()`][Tui::run] when you want typed results from a derive-based parser.
//! - Use [`TuiApp`] when you are working directly with a hand-built [`clap::Command`].
//!
//! # Typed Outcomes
//!
//! ```no_run
//! use clap::Parser;
//! use clap_tui::Tui;
//!
//! #[derive(Debug, Parser, PartialEq, Eq)]
//! #[command(name = "tool")]
//! struct Cli {
//!     #[arg(long)]
//!     name: String,
//! }
//!
//! fn main() -> Result<(), clap_tui::TuiError> {
//!     if let Some(cli) = Tui::<Cli>::new().run()? {
//!         println!("Hello, {}!", cli.name);
//!     }
//!     Ok(())
//! }
//! ```
//!
//! [`Tui::run`] returns:
//! - `Ok(Some(T))` when the user submits a valid command
//! - `Ok(None)` when the user cancels before submission
//! - `Err(TuiError::Clap(_))` for clap help, version, and parse-display flows
//! - another [`TuiError`] variant for runtime, terminal, or internal failures
//!
//! # Feature Flags And Runtime Expectations
//!
//! - The default `mouse` feature enables mouse capture and mouse-driven controls.
//! - The default [`CrosstermRuntime`] expects an interactive terminal with raw mode and an
//!   alternate screen.
//!
//! # Customization
//!
//! - [`TuiConfig`] controls theme, layout, key bindings, and initial command selection.
//! - [`Theme`] and [`ThemePreset`] help you start from a built-in look and adjust from there.
//! - [`Runtime`] plus the exported runtime event types support advanced integration.
//!
//! # Examples
//!
//! The crate ships with four public examples:
//! - `simple` for the smallest explicit `Command::Tui` setup
//! - `showcase` for a compact CLI that demonstrates nested commands, dropdowns, and text input
//! - `subcommands` for explicit typed dispatch with nested command trees
//! - `kitchen_sink` for the untyped [`TuiApp`] surface and broader `clap` coverage

mod app;
mod argv_serializer;
mod config;
mod controller;
mod editor_state;
mod error;
mod form_editor;
mod frame_snapshot;
mod input;
mod pipeline;
mod query;
mod repeated_field;
mod runtime;
mod spec;
mod ui;
mod update;

/// TUI application entry points.
pub use app::{Tui, TuiApp};
/// Public configuration and theming types.
pub use config::{Keymap, LayoutConfig, Theme, ThemePreset, TuiConfig};
/// Error type returned by public `clap-tui` operations.
pub use error::TuiError;
/// Runtime customization surface for advanced integrations.
pub use runtime::{
    AppEvent, AppKeyCode, AppKeyEvent, AppKeyModifiers, AppMouseButton, AppMouseEvent,
    AppMouseEventKind, CrosstermRuntime, Runtime,
};
