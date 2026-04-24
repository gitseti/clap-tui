# clap-tui release-readiness checks

Run this verification flow before preparing a public `clap-tui` release. It matches the GitHub `verify` job and is the baseline gate for pull requests and release tags.

The repository also includes a root [release.toml](../release.toml) for `cargo-release` so maintainers can prepare version bumps and tags consistently without changing the GitHub-side publish gates.

## 1. Run the repository verification script

```bash
./scripts/verify-release-readiness.sh
```

This script runs:

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace --all-targets --all-features`
- `./scripts/check-terminal-stack.sh`
- `cargo package -p clap-tui --list`

If you need to validate uncommitted local changes before opening a pull request, use:

```bash
./scripts/verify-release-readiness.sh --allow-dirty
```

The `--allow-dirty` flag is only for local iteration. CI and tag workflows intentionally use the clean-tree default.

When you want the crates.io preflight too, run:

```bash
./scripts/verify-release-readiness.sh --publish-dry-run
```

This adds `cargo publish -p clap-tui --locked --dry-run` on top of the baseline checks.

## 2. Inspect package metadata and packaged files

```bash
cargo metadata --no-deps --format-version 1
cargo package -p clap-tui --list
```

Confirm the `clap-tui` package metadata includes the expected public description, README path, docs.rs link, keywords, categories, Rust version, and license. Confirm the package contents include:

- `README.md`
- the public library sources under `src/`
- the intended examples under `examples/`
- the expected tests under `tests/`

## 3. Validate rustdoc output

```bash
RUSTDOCFLAGS="-D warnings" cargo doc -p clap-tui --no-deps
```

This validates the crate-level docs and public item docs with rustdoc warnings promoted to errors.

## 4. Dry-run the release tag path

The repository includes one release workflow:

- `.github/workflows/publish.yml` triggers on pushed `vX.Y.Z` tags

The workflow:

- validates that the tag version matches `crates/clap-tui/Cargo.toml` via `cargo run -q -p xtask -- check-tag-version`
- reruns `./scripts/verify-release-readiness.sh`
- runs `./scripts/verify-release-readiness.sh --publish-dry-run` when publishing is enabled
- publishes `clap-tui` through GitHub OIDC when `CLAP_TUI_PUBLISH_MODE=trusted-publishing`
- falls back to `CRATES_IO_TOKEN` only when `CLAP_TUI_PUBLISH_MODE=token`
- otherwise stops after successful pre-publish verification without calling `cargo publish`

Cut release tags only from merged commits whose `verify` check is already green.

The workflow job uses the GitHub Actions `release` environment. If you register trusted publishing on crates.io, use:

- repository: `gitseti/clap-tui`
- workflow file: `publish.yml`
- environment: `release`

## 5. Automated publishing remains opt-in

Actual crates.io publication is still opt-in in this repository. Before enabling trusted publishing in GitHub Actions, maintainers should:

- confirm the intended crates.io owners for `clap-tui`
- ensure `gitseti` has logged into crates.io and is recorded as an owner for `clap-tui`
- run `./scripts/verify-release-readiness.sh --publish-dry-run`
- complete the first `clap-tui` crates.io release through the chosen credential path
- register `.github/workflows/publish.yml` as a trusted publisher for `clap-tui`

After those prerequisites are complete, the tag workflow can publish `clap-tui` without changing the verification contract above.

## 6. Enable automated publishing

The publish workflow is controlled by a repository variable named `CLAP_TUI_PUBLISH_MODE`.

- Leave it unset to keep the tag workflow in verification-only mode.
- Set it to `trusted-publishing` after crates.io trusted publishing is registered for `.github/workflows/publish.yml`, using the `release` environment.
- Set it to `token` only when trusted publishing cannot yet be configured and the repository secret `CRATES_IO_TOKEN` is present as the fallback credential.

Trusted publishing is the default and preferred mode because it avoids long-lived credentials.

## 7. Prepare releases with cargo-release

The root [release.toml](../release.toml) gives `cargo release` the repo-wide defaults that fit this workflow:

- only release from `main`
- skip local `cargo publish`, because GitHub Actions remains the publishing boundary
- skip the automatic post-release dev-version bump

If `cargo release` is not installed locally yet, install it with:

```bash
cargo install cargo-release
```

The `clap-tui` manifest uses the `v{{version}}` tag name, so `cargo release -p clap-tui` lines up with the GitHub workflow:

```bash
cargo release -p clap-tui --dry-run <level-or-version>
cargo release -p clap-tui <level-or-version>
```

Use `--dry-run` first to inspect the version bump, commit message, and tag name before creating the real release commit and tag.

## 8. End-to-end release checklist

1. Confirm the canonical GitHub repository URL and intended crates.io owner are still correct. Intended owner: `gitseti`.
2. Update `CHANGELOG.md` and bump the version in `crates/clap-tui/Cargo.toml`.
3. Confirm the `verify` GitHub check is green on the merged commit you intend to release.
4. If using trusted publishing, ensure crates.io is configured for repository `gitseti/clap-tui`, workflow file `publish.yml`, and environment `release`, then set `CLAP_TUI_PUBLISH_MODE=trusted-publishing`.
5. If trusted publishing is unavailable, set `CLAP_TUI_PUBLISH_MODE=token` and confirm the `CRATES_IO_TOKEN` secret is configured.
6. Run `./scripts/verify-release-readiness.sh --publish-dry-run`.
7. Create and push the matching `vX.Y.Z` tag from the reviewed commit.
8. Confirm the `publish` workflow either publishes `clap-tui` or stops in verification-only mode for the expected reason.
9. If a bad release escapes, yank the affected version on crates.io and cut a corrected patch release.
