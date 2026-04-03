## Why

`clap-tui` is in good mechanical shape, but the review exposed a small set of structural and behavioral issues that weaken the library’s reliability as it grows. The most important gaps are that some runtime behavior is incomplete, some state reads depend on ambient process state, derived validation work sits on the render hot path, and one public execution API can drift away from the schema the TUI rendered.

Addressing these issues now is valuable because they sit at the crate’s main seams: input handling, state projection, validation, and public execution APIs. Fixing them together will make the library more deterministic, more scalable, and safer to integrate before more features build on the current architecture.

## What Changes

- Materialize environment-backed defaults into owned app state at command initialization time so effective reads are stable and deterministic after startup.
- Support paste as first-class interactive input for search and text-editing flows, and make toast expiration independent of idle periods.
- Move argv derivation and clap-backed validation out of the unconditional render path so redraw cost does not scale with parser complexity.
- Introduce a schema-bound parser execution path and remove or constrain the current footgun where `run_with_parser` can parse with a different clap schema than the one used to build the TUI.
- Tighten the surrounding design so runtime behavior, derived state, and execution flow remain aligned with the rendered command model.

## Capabilities

### New Capabilities

- `deterministic-input-state`: Effective form state is fully determined by initialized app state rather than by re-reading environment variables during later queries.
- `interactive-runtime-integrity`: Runtime-level interactive behavior preserves supported input events and expires transient UI feedback correctly under continuous activity.
- `derived-state-lifecycle`: Preview argv and validation are derived once per relevant state change and reused by rendering and Run instead of being recomputed on every redraw.
- `schema-bound-parser-execution`: Parser-backed execution APIs remain bound to the clap schema that generated the TUI.

### Modified Capabilities

- None.

## Impact

- Affected code will include `crates/clap-tui/src/app.rs`, `crates/clap-tui/src/input.rs`, `crates/clap-tui/src/pipeline/`, `crates/clap-tui/src/ui/screen.rs`, and the public runtime and app APIs.
- The public API surface around parser-backed execution may gain a new preferred path and may deprecate or constrain the current unbound helper.
- Test coverage will need to expand around startup default materialization, paste handling, toast timing, derived-state recomputation, and parser-execution guarantees.
