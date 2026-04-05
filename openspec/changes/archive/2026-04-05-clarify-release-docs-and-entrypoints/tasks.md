## 1. Rename and reposition the typed direct-TUI surface

- [x] 1.1 Rename `ParserTuiApp` to `TypedTuiApp` in the public crate surface and update any affected type docs or re-exports
- [x] 1.2 Rename `ParserLauncher` to `TuiLauncher` in the public crate surface and update the macro-facing docs that describe the canonical launcher
- [x] 1.3 Rename `TuiApp::from_factory::<T>()` to `TuiApp::from_parser::<T>()` and update docs and examples to use that spelling
- [x] 1.4 Adjust tests and example references that mention the old launcher or typed direct-TUI names

## 2. Rewrite the landing documentation

- [x] 2.1 Rewrite [`README.md`](/Users/tillseeberger/Projects/clap-tui/README.md) into a usage-first structure with a minimal quick start, explicit entry-point guidance, and a short note crediting Trogon while clarifying that `clap-tui` is not an official `clap` crate
- [x] 2.2 Rewrite crate-level rustdoc in [`crates/clap-tui/src/lib.rs`](/Users/tillseeberger/Projects/clap-tui/crates/clap-tui/src/lib.rs) to mirror the same narrative, include the provenance/status note, and add one short second example
- [x] 2.3 Add or link a visual reference for the TUI if a maintained screenshot asset is available

## 3. Tighten item docs and verify the release surface

- [x] 3.1 Update docs on `TuiLauncher`, `TypedTuiApp`, `TuiApp`, `TuiConfig`, runtime types, and `#[clap_tui::main]` to explain recommended usage in plain language
- [x] 3.2 Remove repetitive architectural phrasing where simpler user-facing wording is sufficient while preserving accurate run semantics
- [x] 3.3 Run the release-readiness documentation checks and confirm the final public docs present one consistent entry-point story
