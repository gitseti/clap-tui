## ADDED Requirements

### Requirement: Pull requests are gated by Rust verification
The repository SHALL run a GitHub Actions verification workflow for pushes and pull requests with a stable required job named `verify` that checks formatting, linting, tests, and package-surface verification for the crates intended to be published from this repository. Maintainer documentation SHALL state that the `verify` job must be configured as a required status check in GitHub branch protection for the default branch.

#### Scenario: Pull request triggers verification
- **WHEN** a contributor opens or updates a pull request
- **THEN** GitHub Actions runs the `verify` job and reports pass or fail status before the change is considered release-ready

#### Scenario: Maintainers configure merge enforcement
- **WHEN** maintainers follow the repository setup instructions
- **THEN** they are told to mark the `verify` job as a required status check in GitHub branch protection

### Requirement: Releases use a controlled GitHub workflow
The repository SHALL provide a GitHub-based release workflow at `.github/workflows/publish.yml` that triggers on pushed `v*` tags, validates that the tag version matches `crates/clap-tui/Cargo.toml`, and reruns pre-publish verification. The release process SHALL require maintainers to create release tags only from merged commits whose required `verify` check has already passed. After the documented proc-macro prerequisite and trusted publishing setup are complete, the workflow MAY grow into automated publication from that tagged revision.

#### Scenario: Tagged release starts publish flow
- **WHEN** maintainers push a `vX.Y.Z` tag for a reviewed commit
- **THEN** GitHub runs the publish workflow against that exact revision instead of publishing from an unreviewed working state

#### Scenario: Maintainers prepare a release tag
- **WHEN** maintainers follow the documented release process for `clap-tui`
- **THEN** they are instructed to create the `vX.Y.Z` tag only from a merged commit whose `verify` check has already passed and whose dependent `clap-tui-macros` version is already available on crates.io when required

#### Scenario: Tag version does not match Cargo version
- **WHEN** the pushed `vX.Y.Z` tag does not match the crate version declared in `crates/clap-tui/Cargo.toml`
- **THEN** the publish workflow fails before attempting crates.io authentication or publication

#### Scenario: Proc-macro prerequisite is not yet published
- **WHEN** maintainers have not yet published the referenced `clap-tui-macros` version
- **THEN** the documented release process keeps the GitHub tag workflow in verification-only mode instead of attempting `cargo publish`

#### Scenario: Proc-macro version is published but not yet visible in the index
- **WHEN** maintainers implement the future automated path that publishes `clap-tui-macros` before `clap-tui`
- **THEN** that automated path waits for the new proc-macro version to become resolvable from crates.io before attempting the `clap-tui` publish step

### Requirement: Publishing credentials minimize long-lived secrets
The release workflow SHALL use GitHub OIDC trusted publishing for crates.io authentication by default via `rust-lang/crates-io-auth-action@v1` and SHALL document `CRATES_IO_TOKEN` as the explicit fallback when trusted publishing cannot yet be configured.

#### Scenario: Repository authentication is configured
- **WHEN** maintainers enable the repository for automated publishing
- **THEN** the workflow uses the documented OIDC path by default and only falls back to `CRATES_IO_TOKEN` if the preferred integration cannot be used
