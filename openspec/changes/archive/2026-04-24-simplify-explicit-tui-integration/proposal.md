## Why

`clap-tui` 0.1.0 currently centers its integration story on launcher interception, synthetic-versus-existing launcher modes, and macro convenience. That makes the public surface larger and harder to understand than the product direction now calls for.

0.1.0 should instead ship one small explicit integration model: the application defines a normal `tui` subcommand in its own clap tree, explicitly runs the TUI from that dispatch branch, receives a typed command value back, and then continues normal application dispatch itself.

## What Changes

- Add a small primary typed returning API, `Tui::<T>::run() -> Result<Option<T>, TuiError>`, for explicit integration from a normal clap dispatch branch.
- Preserve the returning contract for typed execution:
  - `Ok(Some(T))` for successful completion and reparse
  - `Ok(None)` for user cancellation before submission
  - `Err(TuiError::Clap(_))` for clap help, version, and parse-display flows after argv exists
  - other `TuiError` variants for runtime, rendering, or internal failures
- Make the primary documented 0.1.0 integration model: user-defined `tui` subcommand, `match` on `Command::Tui`, run the TUI for the whole CLI, then dispatch the returned typed command normally.
- Preserve the dependency cleanup goal by making `tui-textarea` backendless so `clap-tui` owns terminal/backend integration cleanly.
- **BREAKING** Remove launcher-centric 0.1.0 scope, including synthetic-versus-existing launcher positioning, launcher-specific docs, and launcher-specific expansion work.
- **BREAKING** Remove macro support from the 0.1.0 plan and release surface, and simplify workspace and publishing expectations accordingly.
- Keep Ratatui 0.30 migration and broad command-filtering APIs out of scope.

## Capabilities

### New Capabilities
- `terminal-stack-compatibility`: Ensure the default dependency graph uses one coherent terminal/backend stack by configuring `tui-textarea` in backendless mode.

### Modified Capabilities
- `typed-direct-tui-entrypoint`: Replace the `TypedTuiApp` / `run_parse` public story with `Tui::<T>::run()` as the canonical explicit integration surface.
- `synthetic-tui-entrypoint`: Remove synthetic launcher and macro-first requirements from the 0.1.0 public story.
- `public-api-doc-accuracy`: Update entry-point docs to describe `Tui::<T>::run()` semantics and explicit caller-managed dispatch.
- `public-release-surface`: Rework README and crate-level onboarding around the explicit `Command::Tui` integration model and the simplified 0.1.0 surface.
- `crate-publishing-readiness`: Update release-readiness expectations for a single-crate 0.1.0 public surface.
- `github-release-pipeline`: Remove the two-crate proc-macro publishing assumption from the 0.1.0 release workflow story.

## Impact

- Affected code includes the primary public API surface in [crates/clap-tui/src/lib.rs](/Users/tillseeberger/Projects/clap-tui/crates/clap-tui/src/lib.rs), direct-run entrypoints in [crates/clap-tui/src/app.rs](/Users/tillseeberger/Projects/clap-tui/crates/clap-tui/src/app.rs), launcher code in [crates/clap-tui/src/launcher.rs](/Users/tillseeberger/Projects/clap-tui/crates/clap-tui/src/launcher.rs), error types in [crates/clap-tui/src/error.rs](/Users/tillseeberger/Projects/clap-tui/crates/clap-tui/src/error.rs), package metadata in [crates/clap-tui/Cargo.toml](/Users/tillseeberger/Projects/clap-tui/crates/clap-tui/Cargo.toml), the workspace manifest in [Cargo.toml](/Users/tillseeberger/Projects/clap-tui/Cargo.toml), and public docs in [README.md](/Users/tillseeberger/Projects/clap-tui/README.md).
- The change also affects the existing proc-macro crate at [crates/clap-tui-macros](/Users/tillseeberger/Projects/clap-tui/crates/clap-tui-macros) and the related release/publishing assumptions currently captured in OpenSpec.
- Public API and release-surface impact is intentionally simplifying: fewer entrypoint concepts, fewer docs branches, and a smaller workspace/package story for 0.1.0.
