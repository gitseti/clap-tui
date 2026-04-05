# clap-tui release-readiness checks

Run this verification flow before preparing a public `clap-tui` release. It matches the
GitHub `verify` job and is the baseline gate for pull requests and release tags.

## 1. Run the repository verification script

```bash
./scripts/verify-release-readiness.sh
```

This script runs:

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace --all-targets --all-features`
- `cargo package -p clap-tui-macros --list`
- `cargo package -p clap-tui --list`

If you need to validate uncommitted local changes before opening a pull request, use:

```bash
./scripts/verify-release-readiness.sh --allow-dirty
```

The `--allow-dirty` flag is only for local iteration. CI and tag workflows intentionally use
the clean-tree default.

Once the referenced `clap-tui-macros` version already exists on crates.io, maintainers can also
run:

```bash
./scripts/verify-release-readiness.sh --publish-dry-run
```

This adds `cargo publish -p clap-tui --locked --dry-run` on top of the baseline checks.
When enabled, it first dry-runs `clap-tui-macros`, then `clap-tui`, to match the release order.

## 2. Configure GitHub branch protection

After `ci.yml` is enabled in GitHub Actions, configure the default branch protection rules
to require the `verify` status check before merges. The workflow job name is intentionally
stable so branch protection can target it directly.

## 3. Inspect package metadata and packaged files

```bash
cargo metadata --no-deps --format-version 1
cargo package -p clap-tui-macros --list
cargo package -p clap-tui --list
```

Confirm the `clap-tui` and `clap-tui-macros` package metadata include the expected public
description, README path, docs.rs link, keywords, categories, Rust version, and license.
Confirm the package contents include:

- `crates/clap-tui-macros/README.md`
- `README.md`
- the public library sources under `src/`
- the intended examples under `examples/`
- the expected tests under `tests/`

## 4. Validate rustdoc output

```bash
RUSTDOCFLAGS="-D warnings" cargo doc -p clap-tui --no-deps
```

This validates the crate-level docs and public item docs with rustdoc warnings promoted to
errors.

## 5. Dry-run the release tag path

The repository includes `.github/workflows/publish.yml`, which triggers on pushed `vX.Y.Z`
tags. In its current form it:

- validates that the tag version matches `crates/clap-tui/Cargo.toml` via `cargo run -q -p xtask -- check-tag-version`
- reruns `./scripts/verify-release-readiness.sh`
- runs `cargo publish -p clap-tui-macros --locked --dry-run`
- runs `./scripts/verify-release-readiness.sh --publish-dry-run` only when publishing is enabled
- publishes through GitHub OIDC when `CLAP_TUI_PUBLISH_MODE=trusted-publishing`
- falls back to `CRATES_IO_TOKEN` only when `CLAP_TUI_PUBLISH_MODE=token`
- otherwise stops after successful pre-publish verification without calling `cargo publish`

Cut release tags only from merged commits whose `verify` check is already green.

The workflow job uses the GitHub Actions `release` environment name. If you register trusted
publishing on crates.io, use:

- repository: `gitseti/clap-tui`
- workflow file: `publish.yml`
- environment: `release`

## 6. Proc-macro publishing blocks a real crates.io dry-run today

`clap-tui` depends on the local `clap-tui-macros` crate. Cargo will not let
`cargo publish -p clap-tui --locked --dry-run` succeed until the referenced
`clap-tui-macros` version already exists on crates.io, so the repository cannot honestly use a
green publish dry-run as a required check yet.

The current safe CI contract therefore stops at package-surface verification and version-tag
validation. Before enabling a real publish dry-run or automated `cargo publish`, maintainers
need a release plan that publishes `clap-tui-macros` first and then `clap-tui`.

When automated publishing is enabled for real, the expected sequence is:

1. publish `clap-tui-macros`
2. wait until the new version is visible through the crates.io index
3. publish `clap-tui`

## 7. Manual publish remains the release boundary for now

Actual crates.io publication is intentionally still manual in this repository. Before enabling
trusted publishing in GitHub Actions, maintainers should:

- decide how `clap-tui-macros` and `clap-tui` will be versioned and published together
- confirm the intended crates.io owners for `clap-tui`
- ensure `gitseti` has logged into crates.io and is recorded as an owner for both `clap-tui-macros` and `clap-tui`
- publish any new `clap-tui-macros` version required by the `clap-tui` release
- run `./scripts/verify-release-readiness.sh --publish-dry-run` after that proc-macro version is available on crates.io
- complete the first manual `clap-tui` crates.io release
- register `.github/workflows/publish.yml` as a trusted publisher for the crate

After those prerequisites are complete, the tag workflow can grow a real `cargo publish` step
without changing the verification contract above.

## 8. Enable automated publishing

The publish workflow is controlled by a repository variable named `CLAP_TUI_PUBLISH_MODE`.

- Leave it unset to keep the tag workflow in verification-only mode.
- Set it to `trusted-publishing` after crates.io trusted publishing is registered for
  `.github/workflows/publish.yml` and the `release` environment.
- Set it to `token` only when trusted publishing cannot yet be configured and the repository
  secret `CRATES_IO_TOKEN` is present as the fallback credential.

Trusted publishing is the default and preferred mode because it avoids long-lived credentials.

## 9. End-to-end release checklist

1. Confirm the canonical GitHub repository URL and intended crates.io owners are still correct.
   Intended owner: `gitseti`
2. Update `CHANGELOG.md` and bump the version in `crates/clap-tui/Cargo.toml`.
3. If `clap-tui` now depends on a new `clap-tui-macros` version, bump `crates/clap-tui-macros/Cargo.toml`, publish that crate manually first, and wait for it to appear on crates.io.
4. Run `./scripts/verify-release-readiness.sh --publish-dry-run` after the proc-macro prerequisite is available on crates.io.
5. Confirm the `verify` GitHub check is green on the merged commit you intend to release.
6. If using trusted publishing, ensure crates.io is configured for repository `gitseti/clap-tui`, workflow file `publish.yml`, and environment `release`, then set `CLAP_TUI_PUBLISH_MODE=trusted-publishing`.
7. If trusted publishing is unavailable, set `CLAP_TUI_PUBLISH_MODE=token` and confirm the `CRATES_IO_TOKEN` secret is configured.
8. Create and push the matching `vX.Y.Z` tag from the reviewed commit.
9. If automated publishing is enabled in the future, publish `clap-tui-macros`, wait for crates.io index visibility, and only then publish `clap-tui`.
10. Confirm the `publish` workflow either publishes the crate or stops in verification-only mode for the expected reason.
11. If a bad release escapes, yank the affected version on crates.io and cut a corrected patch release; if the proc-macro version caused the issue, evaluate whether both crates need coordinated follow-up releases.
