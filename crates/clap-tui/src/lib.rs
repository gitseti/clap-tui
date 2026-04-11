#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::all, clippy::pedantic)]
#![deny(rust_2018_idioms)]
#![warn(rust_2024_compatibility)]

//! `clap-tui` turns a `clap` CLI into an interactive terminal UI while preserving the original
//! command-line interface.
//!
//! You can keep `clap` as the source of truth, collect input in the TUI, and then hand the
//! selected argv back to `clap` for typed parsing.
//!
//! This crate was heavily inspired by [Trogon](https://github.com/Textualize/trogon).
//! `clap-tui` is a community crate and is not an official `clap` project.
//!
//! # Quick Start
//!
//! Use [`TuiLauncher`] for most derive-based CLIs:
//!
//! ```no_run
//! use clap::Parser;
//! use clap_tui::TuiLauncher;
//!
//! #[derive(Debug, Parser)]
//! #[command(name = "tool")]
//! struct Cli {
//!     #[arg(long)]
//!     name: String,
//! }
//!
//! fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
//!     TuiLauncher::<Cli>::new().run(|cli| {
//!         println!("Hello, {}!", cli.name);
//!         Ok::<_, clap_tui::TuiError>(())
//!     })?;
//!     Ok(())
//! }
//! ```
//!
//! `TuiLauncher` adds a synthetic root `tui` subcommand, so users can launch the form
//! with `tool tui` while ordinary invocations still parse through `Cli`.
//! Call [`TuiLauncher::with_launcher_name`] to override that default subcommand name.
//!
//! # Choosing An Entry Point
//!
//! You probably want [`TuiLauncher`].
//!
//! - Use [`TuiLauncher`] if your app already uses `#[derive(clap::Parser)]` and you want to add
//!   a `tool tui` entry point while keeping the normal CLI behavior.
//! - Use [`crate::main`] if you want the same launcher behavior with less boilerplate.
//! - Use [`TypedTuiApp`] only when you want to launch the TUI directly, without the synthetic
//!   launcher subcommand.
//! - Use [`TuiApp`] when you are working directly with a hand-built [`clap::Command`].
//!
//! # Direct Typed TUI
//!
//! ```no_run
//! use clap::Parser;
//! use clap_tui::TuiApp;
//!
//! #[derive(Debug, Parser)]
//! #[command(name = "tool")]
//! struct Cli {
//!     #[arg(long)]
//!     name: String,
//! }
//!
//! fn main() -> Result<(), clap_tui::TuiError> {
//!     TuiApp::from_parser::<Cli>().run_with_parser(|cli| {
//!         println!("Hello, {}!", cli.name);
//!         Ok::<_, std::io::Error>(())
//!     })
//! }
//! ```
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
//! - `simple` for the smallest derive-based setup
//! - `showcase` for a compact CLI that demonstrates nested commands, dropdowns, and text input
//! - `subcommands` for `TuiLauncher` with nested command trees
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
mod launcher;
mod pipeline;
mod query;
mod runtime;
mod spec;
mod ui;
mod update;

/// TUI application entry points.
pub use app::{TuiApp, TypedTuiApp};
/// Convenience macro for the canonical typed launcher.
pub use clap_tui_macros::main;
/// Public configuration and theming types.
pub use config::{Keymap, LayoutConfig, Theme, ThemePreset, TuiConfig};
/// Error type returned by public `clap-tui` operations.
pub use error::TuiError;
/// Recommended typed launcher for derive-based CLIs.
pub use launcher::TuiLauncher;
/// Runtime customization surface for advanced integrations.
pub use runtime::{
    AppEvent, AppKeyCode, AppKeyEvent, AppKeyModifiers, AppMouseButton, AppMouseEvent,
    AppMouseEventKind, CrosstermRuntime, Runtime,
};
