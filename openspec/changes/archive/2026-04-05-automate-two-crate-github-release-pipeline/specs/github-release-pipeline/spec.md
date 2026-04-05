## MODIFIED Requirements

### Requirement: Proc-macro releases use a dedicated GitHub workflow
The repository SHALL provide a GitHub-based release workflow at
`.github/workflows/publish-macros.yml` that triggers on pushed `clap-tui-macros-v*` tags,
validates that the tag version matches `crates/clap-tui-macros/Cargo.toml`, reruns pre-publish
verification, and publishes `clap-tui-macros` from that tagged revision when automated publishing
is enabled. When automated publishing is disabled, the workflow SHALL stop after verification and
explain why publication did not run.

#### Scenario: Tagged proc-macro release starts publish flow
- **WHEN** maintainers push a `clap-tui-macros-vX.Y.Z` tag for a reviewed commit
- **THEN** GitHub runs the proc-macro publish workflow against that exact revision

#### Scenario: Proc-macro tag version does not match Cargo version
- **WHEN** the pushed `clap-tui-macros-vX.Y.Z` tag does not match the crate version declared in
  `crates/clap-tui-macros/Cargo.toml`
- **THEN** the proc-macro publish workflow fails before attempting crates.io authentication or
  publication

#### Scenario: Proc-macro publishing is disabled
- **WHEN** maintainers push a proc-macro release tag while automated publishing is not enabled for
  the repository
- **THEN** the proc-macro workflow reruns verification, skips publication, and explains that it
  remained in verification-only mode

### Requirement: Releases use a controlled GitHub workflow
The repository SHALL provide a GitHub-based release workflow at `.github/workflows/publish.yml`
that triggers on pushed `v*` tags, validates that the tag version matches
`crates/clap-tui/Cargo.toml`, and reruns pre-publish verification. The release process SHALL
require maintainers to create release tags only from merged commits whose required `verify` check
has already passed. When automated publishing is disabled, the workflow SHALL stop after
verification and explain why publication did not run. When automated publishing is enabled, the
workflow SHALL determine whether the `clap-tui-macros` version referenced by the tagged
`clap-tui` release is already published, SHALL fail before authentication or publication when that
proc-macro prerequisite is missing, and SHALL only then dry-run and publish `clap-tui` from that
tagged revision.

#### Scenario: Tagged release starts publish flow
- **WHEN** maintainers push a `vX.Y.Z` tag for a reviewed commit
- **THEN** GitHub runs the publish workflow against that exact revision instead of publishing from
  an unreviewed working state

#### Scenario: Maintainers prepare a release tag
- **WHEN** maintainers follow the documented release process for `clap-tui`
- **THEN** they are instructed to create the `vX.Y.Z` tag only from a merged commit whose `verify`
  check has already passed

#### Scenario: Tag version does not match Cargo version
- **WHEN** the pushed `vX.Y.Z` tag does not match the crate version declared in
  `crates/clap-tui/Cargo.toml`
- **THEN** the publish workflow fails before attempting crates.io authentication or publication

#### Scenario: Publishing is disabled
- **WHEN** maintainers push a release tag while automated publishing is not enabled for the
  repository
- **THEN** the workflow reruns verification, skips publication, and explains that it remained in
  verification-only mode

#### Scenario: Referenced proc-macro version already exists
- **WHEN** automated publishing is enabled and the `clap-tui-macros` version referenced by
  `clap-tui` is already available on crates.io
- **THEN** the workflow proceeds to the main-crate publish preflight

#### Scenario: Referenced proc-macro version is missing
- **WHEN** automated publishing is enabled and the referenced `clap-tui-macros` version is not yet
  available on crates.io
- **THEN** the workflow fails with guidance to publish `clap-tui-macros` independently before
  retrying the `clap-tui` tag workflow
