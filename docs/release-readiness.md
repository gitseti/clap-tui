# clap-tui release-readiness checks

Run this focused verification pass before preparing a public `clap-tui` release. It is meant
to confirm the public package surface and rustdoc presentation, not to replace the broader test
and publishing workflow.

## 1. Inspect package metadata

```bash
cargo metadata --no-deps --format-version 1
```

Confirm the `clap-tui` package metadata includes the expected public description, README path,
docs.rs link, keywords, categories, Rust version, and license.

## 2. Verify the packaged file list

```bash
cargo package -p clap-tui --list
```

Confirm the package includes:

- `README.md`
- the public library sources under `src/`
- the intended examples under `examples/`
- the expected tests under `tests/`

If you are validating uncommitted local changes on a branch, rerun the same command with
`--allow-dirty` so Cargo packages the current working tree rather than the last commit.

## 3. Validate rustdoc output

```bash
RUSTDOCFLAGS="-D warnings" cargo doc -p clap-tui --no-deps
```

This validates the crate-level docs and public item docs with rustdoc warnings promoted to
errors.

## 4. Final presentation review

Before publishing, quickly compare the README, crate-level docs, and the public item docs for:

- consistent terminology around `ParserLauncher`, `TuiApp`, `Runtime`, and `TuiConfig`
- accurate cancellation semantics for `run`, `run_with_matches`, and `run_with_parser`
- example guidance that still matches the shipped examples
