#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::all, clippy::pedantic)]
#![deny(rust_2018_idioms)]
#![warn(rust_2024_compatibility)]

//! Auto-generate a TUI from a `clap` command definition.
//!
//! `clap-tui` gives a `clap` command an interactive terminal form without replacing the
//! ordinary CLI path. Applications can keep `clap` as the source of truth, collect input in
//! the TUI, and then hand the selected argv back to `clap` for typed parsing.
//!
//! # Quick Start
//!
//! Prefer [`ParserLauncher`] for derive-based CLIs:
//!
//! ```no_run
//! use clap::Parser;
//! use clap_tui::ParserLauncher;
//!
//! #[derive(Debug, Parser)]
//! #[command(name = "tool")]
//! struct Cli {
//!     #[arg(long)]
//!     name: String,
//! }
//!
//! fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
//!     ParserLauncher::<Cli>::new().run(|cli| {
//!         println!("Hello, {}!", cli.name);
//!         Ok::<_, clap_tui::TuiError>(())
//!     })?;
//!     Ok(())
//! }
//! ```
//!
//! That typed launcher adds a synthetic root `tui` subcommand, so users can launch the form
//! with `tool tui` while ordinary invocations still parse through `Cli`.
//!
//! # Supported Public Surface
//!
//! The stable public surface is intentionally small:
//! - [`ParserLauncher`] is the canonical typed launcher for derive-based CLIs.
//! - [`TuiApp`] is the primary untyped entry point for hand-built [`clap::Command`] values.
//! - [`ParserTuiApp`] is the schema-bound typed wrapper for direct TUI execution.
//! - [`TuiConfig`], [`Theme`], and related config types customize theme, layout, and startup.
//! - [`Runtime`] plus the exported runtime event types support advanced integration.
//!
//! Internal reducers, query helpers, frame snapshots, and clap-projection models are
//! implementation details and are not stable extension points.
//!
//! # Feature Flags And Runtime Expectations
//!
//! - The default `mouse` feature enables mouse capture and mouse-driven controls.
//! - The optional `tracing` feature enables internal tracing instrumentation.
//! - The default [`CrosstermRuntime`] expects an interactive terminal with raw mode and an
//!   alternate screen.
//!
//! # Examples
//!
//! The crate ships with three public examples:
//! - `simple` for the smallest derive-based setup
//! - `subcommands` for typed launch with nested command trees
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

/// Primary TUI application entry point.
pub use app::{ParserTuiApp, TuiApp};
/// Convenience macro for the canonical typed launcher.
pub use clap_tui_macros::main;
/// Public configuration and theming types.
pub use config::{Keymap, LayoutConfig, Theme, ThemePreset, TuiConfig};
/// Error type returned by public `clap-tui` operations.
pub use error::TuiError;
/// Canonical typed launcher for derive-based CLIs.
pub use launcher::ParserLauncher;
/// Runtime customization surface for advanced integrations.
pub use runtime::{
    AppEvent, AppKeyCode, AppKeyEvent, AppKeyModifiers, AppMouseButton, AppMouseEvent,
    AppMouseEventKind, CrosstermRuntime, Runtime,
};
