## Why

`clap-tui` is close to a public crates.io release, but it still has a few last-mile gaps that make the package feel unfinished to outside users. The crate needs a stronger public release surface, clearer package metadata, and API docs that match real behavior so the first release feels trustworthy and easy to adopt.

## What Changes

- Complete the crate's public crates.io presentation by attaching a README to the published package, filling in discovery metadata, and rewriting the README for external users rather than repo-local development.
- Add crate-level docs and example guidance that show how to depend on `clap-tui`, what features and runtime expectations exist, and which examples to start with.
- Correct public API documentation so `run`, cancellation behavior, configuration semantics, and supported extension points are described accurately.
- Tighten the release-readiness bar with a documented validation pass that confirms the package contents and public docs are consistent before publish.

## Capabilities

### New Capabilities
- `public-release-surface`: Define the metadata, README content, docs.rs entry points, and package contents required for a polished public crates.io release.
- `public-api-doc-accuracy`: Define the accuracy requirements for public API docs and documented configuration behavior so published docs match runtime behavior.

### Modified Capabilities

None.

## Impact

- `crates/clap-tui/Cargo.toml` package metadata and packaged files
- Root `README.md` and crate-level rustdoc in `crates/clap-tui/src/lib.rs`
- Public API docs in `crates/clap-tui/src/app.rs`, `crates/clap-tui/src/config.rs`, `crates/clap-tui/src/runtime.rs`, and related exported types
- Local release-readiness verification commands and maintainer documentation
