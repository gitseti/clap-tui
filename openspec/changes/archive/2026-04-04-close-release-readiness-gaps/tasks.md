## 1. Package metadata and public release surface

- [x] 1.1 Update `crates/clap-tui/Cargo.toml` with the public crates.io metadata needed for release, including `readme`, repository/homepage or documentation links, keywords, and categories using the canonical public repository values.
- [x] 1.2 Verify the published package includes the referenced README and expected public files by checking the package file list for `clap-tui`.
- [x] 1.3 Rewrite the root `README.md` for external users so it leads with dependency setup, MSRV, public feature flags, terminal expectations, and guided example selection.

## 2. docs.rs and API documentation polish

- [x] 2.1 Add crate-level rustdoc in `crates/clap-tui/src/lib.rs` that gives docs.rs users a concise quick start and points them to the supported customization seams and examples.
- [x] 2.2 Correct public entry-point docs in `crates/clap-tui/src/app.rs` so cancellation and clap parsing behavior are described accurately.
- [x] 2.3 Update public configuration and runtime docs in `crates/clap-tui/src/config.rs`, `crates/clap-tui/src/runtime.rs`, and any related exported items so documented behavior matches current implementation and supported extension points.

## 3. Release-readiness verification

- [x] 3.1 Add or document a repeatable release-readiness verification flow covering manifest inspection, packaged README verification, and rustdoc validation for `clap-tui`.
- [x] 3.2 Run the readiness checks on the current branch state and fix any metadata or documentation issues they reveal.

## 4. Final review

- [x] 4.1 Confirm the README, crate-level docs, and item-level public docs use consistent terminology for the supported API surface and examples.
- [x] 4.2 Perform a final crates.io presentation review and verify the crate is ready for implementation follow-through and release preparation.
