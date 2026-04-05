## Why

`clap-tui` now has a small enough public surface for a first release, but the current docs do not present it that way. docs.rs users see type inventory before entry-point guidance, the examples are too indirect, and the typed direct-TUI path is still harder to understand than it should be.

This is worth fixing before `1.0` because documentation and naming are part of the release surface. A concise, usage-first story will make the crate easier to adopt and will reduce the risk of freezing a confusing public API shape.

## What Changes

- Rework the public README and crate-level rustdoc so they lead with the value proposition, a minimal quick start, and a short "which entry point should I use?" section.
- Make the public docs clearly disclose that the crate was heavily inspired by Trogon and that it is a community crate rather than an official `clap` project.
- Add one short inline example beyond the first quick-start snippet so docs.rs users can understand a second supported flow without leaving the page.
- Tighten item docs for the main public types and macro so they explain recommended usage in plain language instead of repeating architectural vocabulary.
- Rename the main derive-based launcher and the typed direct-TUI path to cleaner public names before `1.0`, and document those surfaces with clearer construction spelling.

## Capabilities

### New Capabilities

- `typed-direct-tui-entrypoint`: Defines the supported typed direct-TUI surface for derive-based CLIs and requires it to be clearly named and documented relative to `TuiLauncher`.

### Modified Capabilities

- `public-release-surface`: The README and crate-level docs become more usage-first, include entry-point selection guidance, and surface examples more directly.
- `public-api-doc-accuracy`: Item docs for the public API explain when to use each surface in concise, user-facing language.

## Impact

- Affected docs will include [`README.md`](/Users/tillseeberger/Projects/clap-tui/README.md), [`crates/clap-tui/src/lib.rs`](/Users/tillseeberger/Projects/clap-tui/crates/clap-tui/src/lib.rs), and the docs on the main exported types and macro.
- Public API impact is expected to be narrow: rename `ParserLauncher` to `TuiLauncher`, rename `ParserTuiApp` to `TypedTuiApp`, and prefer `TuiApp::from_parser::<T>()` as the public construction spelling for typed direct TUI execution.
- Examples, tests, and release-surface docs will need coordinated updates so the renamed API appears consistently everywhere.
