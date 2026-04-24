## MODIFIED Requirements

### Requirement: Publishable crate metadata is complete
The `clap-tui` crate SHALL declare the metadata required for a public crates.io release, including `description`, `readme`, `repository`, `homepage`, `documentation`, `license`, `rust-version`, `keywords`, and `categories`. The `repository` and `homepage` values SHALL point at the canonical GitHub repository for this workspace and SHALL NOT use placeholders.

#### Scenario: Release metadata is present
- **WHEN** maintainers inspect the publishable manifest for `clap-tui`
- **THEN** they find the required crates.io packaging, discovery, and docs.rs metadata populated with real repository values instead of unpublished local context or placeholders

### Requirement: A shared baseline verification contract is repeatable
The repository SHALL provide a repeatable baseline verification entry point shared by local maintainers, the GitHub `verify` job, and the publish workflow's verification rerun. That baseline contract SHALL validate formatting, linting, tests, terminal-stack compatibility, and the packaged file surface for `clap-tui`. The repository MAY support a local dirty-worktree mode for the package-surface inspection, but the baseline CI contract SHALL remain clean-tree by default. The crates.io publish dry-run SHALL remain available as a distinct release-preflight step and SHALL NOT depend on any separate proc-macro publication.

#### Scenario: Local verification matches CI verification
- **WHEN** a maintainer runs the documented baseline verification command before opening or merging a release-related pull request
- **THEN** they execute the same baseline verification contract enforced by the GitHub `verify` job

#### Scenario: Baseline verification checks package and terminal invariants
- **WHEN** baseline verification runs
- **THEN** it includes the terminal-stack dependency check and `cargo package -p clap-tui --list` for the published crate

#### Scenario: Release preflight adds publish dry-run explicitly
- **WHEN** maintainers prepare a release and automated publishing is enabled or about to be enabled
- **THEN** the documented release flow adds `cargo publish -p clap-tui --locked --dry-run` as an explicit preflight on top of the baseline verification contract

### Requirement: Release guidance is documented
The repository SHALL document the release prerequisites and the steps for preparing, validating, tagging, and publishing a new `clap-tui` version in a structure that separates the routine release runbook from one-time publishing setup and from troubleshooting or rationale material. The documentation SHALL describe the single-crate `clap-tui` release flow, SHALL state that pushed `vX.Y.Z` tags are the authoritative publish trigger, SHALL describe verification-only mode when automated publishing is disabled, SHALL identify GitHub Release notes as the canonical human-facing release-notes artifact, SHALL describe `cargo release` only as an optional maintainer helper, and SHALL avoid referencing removed proc-macro prerequisites or nonexistent repository artifacts.

#### Scenario: Maintainer follows the routine release guide
- **WHEN** a maintainer prepares a normal `clap-tui` release after repository setup is already complete
- **THEN** they can follow a short happy-path runbook without digging through one-time publishing bootstrap details

#### Scenario: Maintainer enables publishing for the first time
- **WHEN** a maintainer prepares repository publishing infrastructure
- **THEN** they can find crates.io owner setup, trusted publishing registration, `CLAP_TUI_PUBLISH_MODE`, and token fallback guidance without mixing it into the routine release checklist

#### Scenario: Documentation matches real repository artifacts
- **WHEN** a maintainer follows the documented release instructions
- **THEN** the documentation refers only to active scripts, workflows, and release artifacts that actually exist in the repository

#### Scenario: Release notes do not require a changelog file
- **WHEN** a maintainer prepares notes for a new `clap-tui` release
- **THEN** the routine runbook directs them to prepare GitHub Release notes
- **THEN** the repository does not require a `CHANGELOG.md` file unless it intentionally adopts one later

#### Scenario: cargo-release is optional
- **WHEN** a maintainer follows the canonical happy-path release runbook
- **THEN** they can complete the release without using `cargo release`
- **THEN** separate setup or helper documentation may still describe `cargo release` as an optional tool supported by `release.toml`

### Requirement: Crates.io owner setup is a publishing prerequisite
The repository SHALL document crates.io owner configuration as a prerequisite for automated publishing and SHALL treat owner setup as required before the release workflow is considered ready to publish.

#### Scenario: Automated publishing prerequisites are reviewed
- **WHEN** a maintainer enables automated publishing for `clap-tui`
- **THEN** the documentation instructs them to confirm the intended crates.io owners before relying on the GitHub release workflow
