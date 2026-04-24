# clap-tui release runbook

Use this page for the routine `clap-tui` release happy path after repository publishing setup is already complete. For one-time setup, see [publishing-setup.md](publishing-setup.md). For troubleshooting and local simulation notes, see [release-troubleshooting.md](release-troubleshooting.md).

## 1. Run the baseline repository verification contract

```bash
./scripts/verify.sh
```

This is the same baseline verification enforced by the GitHub `verify` job:

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace --all-targets --all-features`
- `./scripts/check-terminal-stack.sh`
- `cargo package -p clap-tui --list`

If you need to validate local changes before committing them, use:

```bash
./scripts/verify.sh --allow-dirty
```

## 2. Run the explicit publish preflight when needed

If automated publishing is enabled or you want to rehearse the crates.io path locally, run:

```bash
cargo publish -p clap-tui --locked --dry-run
```

This publish preflight is intentionally separate from `./scripts/verify.sh` so the baseline verification contract stays the same in local use, CI, and the publish workflow.

## 3. Prepare the release tag and notes

1. Bump the version in `crates/clap-tui/Cargo.toml`.
2. Confirm the `verify` GitHub check is green on the merged commit you intend to release.
3. Check the tag locally before pushing it:

```bash
./scripts/check-tag-version.sh vX.Y.Z
```

4. Prepare GitHub Release notes for the tagged release. GitHub Release notes are the canonical human-facing release-notes artifact for this repository.
5. Create and push the matching `vX.Y.Z` tag from the reviewed commit.

## 4. Understand what the publish workflow does

`.github/workflows/publish.yml` is the authoritative publication path and triggers on pushed `vX.Y.Z` tags. It:

- validates the tag/version match via `./scripts/check-tag-version.sh`
- reruns `./scripts/verify.sh`
- runs `cargo publish -p clap-tui --locked --dry-run` only when publishing is enabled
- publishes `clap-tui` through GitHub OIDC when `CLAP_TUI_PUBLISH_MODE=trusted-publishing`
- falls back to `CRATES_IO_TOKEN` only when `CLAP_TUI_PUBLISH_MODE=token`
- otherwise stops after successful verification in verification-only mode

GitHub Release pages are optional and may contain the release notes, but they are not the trigger for crates.io publication. The pushed `vX.Y.Z` tag is the authoritative release trigger.

## 5. Optional extra inspection

If you want an extra manual check beyond the baseline contract, these commands are still useful before a public release:

```bash
cargo metadata --no-deps --format-version 1
cargo package -p clap-tui --list
RUSTDOCFLAGS="-D warnings" cargo doc -p clap-tui --no-deps
```

Confirm the package metadata, packaged files, and rustdoc output look correct for the version you are about to tag.
