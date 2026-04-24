## REMOVED Requirements

### Requirement: Proc-macro releases use a dedicated GitHub workflow
**Reason**: The proc-macro crate is no longer part of the intended 0.1.0 workspace and release surface, so a dedicated proc-macro publish workflow is no longer required.
**Migration**: Remove the proc-macro-specific publish workflow and treat `clap-tui` as the only published crate in the 0.1.0 release pipeline.

## MODIFIED Requirements

### Requirement: Releases use a controlled GitHub workflow
The repository SHALL provide a GitHub-based release workflow at `.github/workflows/publish.yml` that triggers on pushed `v*` tags, validates that the tag version matches `crates/clap-tui/Cargo.toml`, and reruns pre-publish verification. The release process SHALL require maintainers to create release tags only from merged commits whose required `verify` check has already passed. When automated publishing is disabled, the workflow SHALL stop after verification and explain why publication did not run. When automated publishing is enabled, the workflow SHALL dry-run and publish `clap-tui` from that tagged revision without depending on a separate proc-macro release prerequisite.

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
- **THEN** the workflow reruns verification, skips publication, and explains that it remained in verification-only mode

#### Scenario: Publishing runs without a proc-macro prerequisite
- **WHEN** automated publishing is enabled for a tagged `clap-tui` release
- **THEN** the workflow proceeds directly from verification to the `clap-tui` publish preflight without checking for a separate proc-macro version on crates.io
