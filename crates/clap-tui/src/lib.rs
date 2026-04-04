#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::all, clippy::pedantic)]
#![deny(rust_2018_idioms)]
#![warn(rust_2024_compatibility)]

//! Auto-generate a TUI from a `clap` command definition.
//!
//! # Intended Public Surface
//!
//! The crate intentionally keeps its stable public surface small:
//! - [`ParserLauncher`] is the canonical typed launcher for derive-based CLIs.
//! - [`TuiApp`] is the primary untyped entry point.
//! - [`ParserTuiApp`] is the schema-bound typed wrapper for direct TUI execution.
//! - [`TuiConfig`] and theme types customize look and layout.
//! - [`Runtime`] plus the crate-local event types support advanced runtime integration.
//!
//! Internal reducers, query helpers, frame snapshots, and clap-projection models are
//! implementation details and are not stable extension points.
//!
//! For derive-based root CLIs, prefer [`ParserLauncher`] or [`main`] so users get a
//! synthetic root `tui` command such as `tool tui`. In v1, the synthetic launcher only
//! attaches at the CLI root and is rejected when the root clap grammar already has a
//! conflicting or ambiguous `tui` path.

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
