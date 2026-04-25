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
//! It reduces trial-and-error for complex CLIs by making commands, flags, and values easier to
//! explore before execution. It also improves discoverability without changing the CLI behavior
//! your existing scripts and docs already rely on.
//!
//! This crate was heavily inspired by [Trogon](https://github.com/Textualize/trogon).
//! `clap-tui` is a community crate and is not an official `clap` project.
//!
//! ![clap-tui hero screenshot](https://raw.githubusercontent.com/gitseti/clap-tui/main/docs/assets/hero.png)
//!
//! # Quick Start
//!
//! Add a `Tui` subcommand and delegate to [`Tui::run`].
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
//!             if let Some(command) = Tui::<Command>::new().hide_entrypoint("tui")?.run()? {
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
//! Use [`Tui`] (recommended).
//!
//! - Use [`Tui::<T>::run()`][Tui::run] when you want typed results from a derive-based parser.
//! - Use [`TuiApp`] when you are working directly with a hand-built [`clap::Command`] or need a
//!   lower-level integration surface.
//!
//! # Typed Outcomes
//!
//! You can also use [`Tui`] with a single struct instead of an enum:
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
//! - `Some(T)` when the user submits a valid command
//! - `None` when the user cancels before submission
//! - `Err(TuiError)` for runtime failures or clap-driven flows such as help and version handling
//!
//! See [`TuiError`] for the detailed error taxonomy.
//!
//! # Feature Flags
//!
//! - The default `mouse` feature enables mouse capture and mouse-driven controls.
//!
//! # Runtime Expectations
//!
//! - The default [`CrosstermRuntime`] requires a terminal supporting raw mode and an alternate
//!   screen.
//!
//! # Customization
//!
//! - [`TuiConfig`] controls theme, layout, key bindings, and initial command selection.
//! - [`Theme`] and [`ThemePreset`] help you start from a built-in look and adjust from there.
//! - [`Runtime`] plus the exported runtime event types support custom event loops or embedding into
//!   existing runtimes.
//!
//! # Examples
//!
//! The crate ships with four public examples:
//! - `simple` for minimal `Command::Tui` setup
//! - `showcase` for a realistic, compact command tree
//! - `subcommands` for typed dispatch across command trees
//! - `clap_features` for the full [`TuiApp`] compatibility fixture

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
