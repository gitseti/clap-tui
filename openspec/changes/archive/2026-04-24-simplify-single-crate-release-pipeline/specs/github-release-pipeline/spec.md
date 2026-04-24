## MODIFIED Requirements

### Requirement: Pull requests are gated by Rust verification
The repository SHALL run a GitHub Actions verification workflow for pull requests and normal branch pushes with a stable required job named `verify` that checks formatting, linting, tests, and package-surface verification for the published `clap-tui` crate. Maintainer documentation SHALL state that the `verify` job must be configured as a required status check in GitHub branch protection for the default branch. Release-tag pushes MAY be excluded from this separate workflow when the publish workflow reruns the same baseline verification contract.

#### Scenario: Pull request triggers verification
- **WHEN** a contributor opens or updates a pull request
- **THEN** GitHub Actions runs the `verify` job and reports pass or fail status before the change is considered release-ready

#### Scenario: Maintainers configure merge enforcement
- **WHEN** maintainers follow the repository setup instructions
- **THEN** they are told to mark the `verify` job as a required status check in GitHub branch protection

#### Scenario: Release tags avoid redundant CI duplication
- **WHEN** a maintainer pushes a `vX.Y.Z` release tag
- **THEN** the repository does not need a second standalone `verify` workflow run if the publish workflow reruns the same baseline verification before any publish attempt

## REMOVED Requirements

### Requirement: Proc-macro releases use a dedicated GitHub workflow
**Reason**: The repository no longer contains `clap-tui-macros`, so release automation no longer needs a proc-macro-specific tag workflow.
**Migration**: Treat `clap-tui` as the only published crate in GitHub Actions and remove any remaining references to proc-macro tag workflows.

## MODIFIED Requirements

### Requirement: Releases use a controlled GitHub workflow
The repository SHALL provide a GitHub-based release workflow at `.github/workflows/publish.yml` that triggers on pushed `v*` tags, validates that the tag version matches `crates/clap-tui/Cargo.toml`, and reruns the shared baseline repository verification contract before any publish attempt. The release process SHALL require maintainers to create release tags only from merged commits whose required `verify` check has already passed. When automated publishing is disabled, the workflow SHALL stop after successful verification and explain why publication did not run. When automated publishing is enabled, the workflow SHALL run a `cargo publish -p clap-tui --locked --dry-run` preflight and SHALL then publish `clap-tui` from that tagged revision without depending on any separate proc-macro release prerequisite.

#### Scenario: Tagged release starts publish flow
- **WHEN** maintainers push a `vX.Y.Z` tag for a reviewed commit
- **THEN** GitHub runs the publish workflow against that exact revision instead of publishing from an unreviewed working state

#### Scenario: Maintainers prepare a release tag
- **WHEN** maintainers follow the documented release process for `clap-tui`
- **THEN** they are instructed to create the `vX.Y.Z` tag only from a merged commit whose `verify` check has already passed

#### Scenario: Tag version does not match Cargo version
- **WHEN** the pushed `vX.Y.Z` tag does not match the crate version declared in `crates/clap-tui/Cargo.toml`
- **THEN** the publish workflow fails before attempting crates.io authentication or publication

#### Scenario: Publishing is disabled
- **WHEN** maintainers push a release tag while automated publishing is not enabled for the repository
- **THEN** the workflow reruns baseline verification, skips publication, and explains that it remained in verification-only mode

#### Scenario: Publishing runs without a proc-macro prerequisite
- **WHEN** automated publishing is enabled for a tagged `clap-tui` release
- **THEN** the workflow proceeds directly from baseline verification to the `clap-tui` publish preflight without checking for a separate proc-macro version on crates.io

#### Scenario: Publish dry-run gates publication
- **WHEN** automated publishing is enabled for a tagged `clap-tui` release
- **THEN** the workflow runs `cargo publish -p clap-tui --locked --dry-run` successfully before attempting the real crates.io publish

#### Scenario: GitHub Release pages stay non-authoritative
- **WHEN** maintainers create or edit a GitHub Release page for a tagged `clap-tui` release
- **THEN** that page does not become the authoritative trigger for crates.io publication, which remains the pushed `vX.Y.Z` tag
