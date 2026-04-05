## Why

The repository now has a safe tag-triggered release workflow, but it still treats the
`clap-tui-macros` dependency mostly as process documentation rather than a checked prerequisite.
That leaves the main crate publish path more ambiguous than it needs to be when a release depends
on a proc-macro version that may or may not already exist on crates.io.

## What Changes

- Extend the GitHub release workflow so it computes the tagged `clap-tui` release plan, including
  the referenced `clap-tui-macros` version.
- Add an explicit crates.io prerequisite check so `publish.yml` only attempts the `clap-tui`
  publish after the referenced proc-macro version is already available.
- Add a separate GitHub release workflow for `clap-tui-macros` that publishes from
  `clap-tui-macros-vX.Y.Z` tags.
- Add a lightweight `cargo-release` repo configuration so maintainers can prepare version bumps and
  tags in a way that matches the GitHub workflows.
- Keep trusted publishing as the default credential path while preserving the documented
  `CRATES_IO_TOKEN` fallback.
- Update release documentation so the two crates are automated independently: a dedicated macro
  workflow for `clap-tui-macros`, and a main-crate workflow that enforces the proc-macro
  prerequisite before publishing `clap-tui`.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `github-release-pipeline`: The release workflow will change from verification-first automation to
  coordinated but independent publish workflows: one for `clap-tui-macros`, and one for `clap-tui`
  that enforces the `clap-tui-macros` crates.io prerequisite.

## Impact

- `.github/workflows/publish.yml`
- `.github/workflows/publish-macros.yml`
- `release.toml`
- `scripts/verify-release-readiness.sh`
- `docs/release-readiness.md`
- `openspec/specs/github-release-pipeline/spec.md`
- `xtask`
