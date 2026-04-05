# clap-tui release-readiness checks

Run this verification flow before preparing a public `clap-tui` release. It matches the
GitHub `verify` job and is the baseline gate for pull requests and release tags.

The repository also includes a root [release.toml](../release.toml)
for `cargo-release` so maintainers can prepare version bumps and tags consistently without changing
the GitHub-side publish gates.

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
When enabled, it first dry-runs `clap-tui-macros`, then `clap-tui`, so the local preflight matches
the companion-crate dependency relationship.

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

## 5. Dry-run the release tag paths

The repository includes two release workflows:

- `.github/workflows/publish-macros.yml` triggers on pushed `clap-tui-macros-vX.Y.Z` tags
- `.github/workflows/publish.yml` triggers on pushed `vX.Y.Z` tags

The proc-macro workflow:

- validates that the tag version matches `crates/clap-tui-macros/Cargo.toml` via `cargo run -q -p xtask -- check-macro-tag-version`
- reruns `./scripts/verify-release-readiness.sh`
- runs `cargo publish -p clap-tui-macros --locked --dry-run`
- publishes `clap-tui-macros` through GitHub OIDC when `CLAP_TUI_PUBLISH_MODE=trusted-publishing`
- falls back to `CRATES_IO_TOKEN` only when `CLAP_TUI_PUBLISH_MODE=token`
- otherwise stops after successful pre-publish verification without calling `cargo publish`

The main crate workflow:

- validates that the tag version matches `crates/clap-tui/Cargo.toml` via `cargo run -q -p xtask -- check-tag-version`
- reruns `./scripts/verify-release-readiness.sh`
- runs `cargo publish -p clap-tui-macros --locked --dry-run`
- computes the tagged release plan, including the exact `clap-tui-macros` version referenced by `clap-tui`
- when publishing is enabled, requires that referenced `clap-tui-macros` version to already exist on crates.io
- runs `./scripts/verify-release-readiness.sh --publish-dry-run` only when publishing is enabled
- publishes `clap-tui` through GitHub OIDC when `CLAP_TUI_PUBLISH_MODE=trusted-publishing`
- falls back to `CRATES_IO_TOKEN` only when `CLAP_TUI_PUBLISH_MODE=token`
- otherwise stops after successful pre-publish verification without calling `cargo publish`

Cut release tags only from merged commits whose `verify` check is already green.

Both workflow jobs use the GitHub Actions `release` environment name. If you register trusted
publishing on crates.io, use:

- repository: `gitseti/clap-tui`
- workflow file: `publish.yml` for the `clap-tui` crate
- workflow file: `publish-macros.yml` for the `clap-tui-macros` crate
- environment: `release`

Local validation note:

- `act -l` successfully discovers `ci.yml`, `publish.yml`, and `publish-macros.yml`
- a full local `act` run on the current Colima setup still fails during container startup with a
  Docker socket mount error

## 6. Proc-macro publication is an external release prerequisite

`clap-tui` depends on `clap-tui-macros` with an exact version requirement, for example
`=0.1.0`. Cargo will not let `cargo publish -p clap-tui --locked --dry-run` succeed until that
exact proc-macro version already exists on crates.io.

The current release model keeps `clap-tui-macros` and `clap-tui` as independently automated
releases. `publish-macros.yml` handles proc-macro publication from `clap-tui-macros-vX.Y.Z` tags.
`publish.yml` does not try to publish both crates in one run. Instead, when publishing is enabled,
it checks that the exact referenced proc-macro version already exists and fails early with guidance
if it does not.

The expected release sequence is:

1. publish `clap-tui-macros` independently when the `clap-tui` release needs a new proc-macro version
2. confirm the exact referenced proc-macro version is visible on crates.io
3. run or enable the `clap-tui` tag workflow

## 7. Automated publishing remains opt-in

Actual crates.io publication is still opt-in in this repository. Before enabling trusted publishing
in GitHub Actions, maintainers should:

- confirm the intended crates.io owners for `clap-tui`
- ensure `gitseti` has logged into crates.io and is recorded as an owner for both `clap-tui-macros` and `clap-tui`
- publish any new `clap-tui-macros` version required by the exact dependency in `clap-tui`
- run `./scripts/verify-release-readiness.sh --publish-dry-run` after that proc-macro version is available on crates.io
- complete the first `clap-tui` crates.io release through the chosen credential path
- register `.github/workflows/publish-macros.yml` as a trusted publisher for `clap-tui-macros`
- register `.github/workflows/publish.yml` as a trusted publisher for `clap-tui`

After those prerequisites are complete, the tag workflow can publish `clap-tui` without changing
the verification contract above.

## 8. Enable automated publishing

The publish workflows are controlled by a repository variable named `CLAP_TUI_PUBLISH_MODE`.

- Leave it unset to keep the tag workflow in verification-only mode.
- Set it to `trusted-publishing` after crates.io trusted publishing is registered for
  both `.github/workflows/publish.yml` and `.github/workflows/publish-macros.yml`, using the
  `release` environment for each crate.
- Set it to `token` only when trusted publishing cannot yet be configured and the repository
  secret `CRATES_IO_TOKEN` is present as the fallback credential.

Trusted publishing is the default and preferred mode because it avoids long-lived credentials.

## 9. Prepare releases with cargo-release

The root [release.toml](../release.toml) gives `cargo release`
the repo-wide defaults that fit this workflow:

- only release from `main`
- update dependent crate versions with `dependent-version = "fix"`
- skip local `cargo publish`, because GitHub Actions remains the publishing boundary
- skip the automatic post-release dev-version bump

If `cargo release` is not installed locally yet, install it with:

```bash
cargo install cargo-release
```

Per-crate tag names are configured in the manifests so `cargo release -p ...` lines up with the
GitHub workflows:

- [clap-tui Cargo.toml](../crates/clap-tui/Cargo.toml) uses `v{{version}}`
- [clap-tui-macros Cargo.toml](../crates/clap-tui-macros/Cargo.toml) uses `clap-tui-macros-v{{version}}`

Typical maintainer prep commands:

```bash
cargo release -p clap-tui-macros --dry-run <level-or-version>
cargo release -p clap-tui-macros <level-or-version>

cargo release -p clap-tui --dry-run <level-or-version>
cargo release -p clap-tui <level-or-version>
```

Use `--dry-run` first to inspect the version bump, dependent-version updates, commit message, and
tag name before creating the real release commit and tag.

## 10. End-to-end release checklist

1. Confirm the canonical GitHub repository URL and intended crates.io owners are still correct.
   Intended owner: `gitseti`
2. Update `CHANGELOG.md` and bump the version in `crates/clap-tui/Cargo.toml`.
3. If `clap-tui` now depends on a new exact `clap-tui-macros` version, bump `crates/clap-tui-macros/Cargo.toml`, update the exact dependency in `crates/clap-tui/Cargo.toml`, and confirm the `verify` GitHub check is green on the merged commit you intend to release.
4. If using trusted publishing, ensure crates.io is configured for repository `gitseti/clap-tui`, workflow file `publish-macros.yml` for `clap-tui-macros`, workflow file `publish.yml` for `clap-tui`, and environment `release`, then set `CLAP_TUI_PUBLISH_MODE=trusted-publishing`.
5. If trusted publishing is unavailable, set `CLAP_TUI_PUBLISH_MODE=token` and confirm the `CRATES_IO_TOKEN` secret is configured.
6. When a new proc-macro release is needed, create and push the matching `clap-tui-macros-vX.Y.Z` tag from the reviewed commit.
7. Confirm the `publish-macros` workflow either publishes `clap-tui-macros` or stops in verification-only mode for the expected reason.
8. Run `./scripts/verify-release-readiness.sh --publish-dry-run` after the proc-macro prerequisite is available on crates.io.
9. Create and push the matching `vX.Y.Z` tag from the reviewed commit.
10. Confirm the `publish` workflow either publishes `clap-tui`, fails early because the exact proc-macro prerequisite is missing, or stops in verification-only mode for the expected reason.
11. If a bad release escapes, yank the affected version on crates.io and cut a corrected patch release; if the proc-macro version caused the issue, evaluate whether both crates need coordinated follow-up releases.
