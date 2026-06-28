## Why

The typed `Tui::run()` API returns the parsed clap value but discards the canonical argv that produced it, forcing callers that need invocation logging or inspection to drop to the untyped API and repeat the typed parse themselves.

## What Changes

- Add a public `TuiSubmission<T>` containing the parsed command and canonical `Vec<OsString>` argv.
- Add `Tui::run_with_argv()` as an opt-in typed API returning the richer submission.
- Keep `Tui::run()` as the primary typed shortcut with unchanged behavior and signature.
- Document that returned argv includes the executable token and is not shell-rendered command text.
- Bump the crate version to 0.2.0 for the new public API release.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `typed-direct-tui-entrypoint`: Add an opt-in typed submission API while preserving the primary `Tui::run()` contract.
- `argv-serialization-boundary`: Expose the same canonical argv used for validation and typed reparsing without introducing a second serialization path.

## Impact

The public `clap-tui` API gains one exported result type and one additive method, and the crate version advances to 0.2.0. The typed runner implementation, package metadata, crate documentation, README, tests, and two existing specifications are affected. No dependency, runtime, cancellation, rendering, or error behavior changes.
