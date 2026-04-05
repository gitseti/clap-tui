## Why

`clap-tui` already has working library code and basic usage documentation, but it is not yet set up for a low-risk public release on crates.io. The workspace metadata is incomplete for discovery, there is no repository automation verifying packaging and release quality, and the project lacks a documented release path that accounts for the dependent `clap-tui-macros` proc-macro crate, real GitHub repository identity, and crates.io ownership configuration.

## What Changes

- Make the workspace, `clap-tui`, and `clap-tui-macros` explicitly publish-ready for crates.io, including complete package metadata with a canonical GitHub repository URL, packaging validation, and release-facing documentation.
- Add a GitHub Actions verification workflow with a stable required check contract for pushes and pull requests before release-related changes can merge.
- Add a release workflow triggered by pushed `vX.Y.Z` tags and a documented release process that starts with manual publication of any new `clap-tui-macros` version, then the first manual `clap-tui` crates.io publish, and then supports GitHub-based automated publishing for `clap-tui`.
- Define the concrete security and maintenance choices for publishing, including changelog expectations, crates.io owner setup, trusted publishing via GitHub OIDC, and the explicit `CRATES_IO_TOKEN` fallback.

## Capabilities

### New Capabilities
- `crate-publishing-readiness`: Ensure the crate contains the exact metadata, packaged files, verification steps, and release documentation needed for a crates.io release without placeholders.
- `github-release-pipeline`: Provide GitHub CI and release automation with stable PR checks and a tag-driven publish path from GitHub.

### Modified Capabilities
- None.

## Impact

- Cargo manifests in the workspace root, `crates/clap-tui`, and `crates/clap-tui-macros`
- Repository documentation such as `README.md`, `CHANGELOG.md`, and maintainer release instructions
- GitHub Actions workflows and related release configuration
- Release operations for versioning, `vX.Y.Z` tagging, crates.io owner setup, `clap-tui-macros` prerequisite publication, and crates.io publishing
