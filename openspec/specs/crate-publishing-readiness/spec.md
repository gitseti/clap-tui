## ADDED Requirements

### Requirement: Publishable crate metadata is complete
The `clap-tui` and `clap-tui-macros` crates SHALL declare the metadata required for public crates.io releases, including `description`, `readme`, `repository`, `homepage`, `documentation`, `license`, `rust-version`, `keywords`, and `categories`. The `repository` and `homepage` values SHALL point at the canonical GitHub repository for this workspace and SHALL NOT use placeholders.

#### Scenario: Release metadata is present
- **WHEN** maintainers inspect the publishable manifests for `clap-tui` and `clap-tui-macros`
- **THEN** they find the required crates.io packaging, discovery, and docs.rs metadata populated with real repository values instead of unpublished local context or placeholders

### Requirement: The published package is verified before release
The repository SHALL provide a repeatable verification step that confirms both published crates can be packaged for crates.io, and SHALL support a `cargo publish -p clap-tui --locked --dry-run` check for the library crate once the referenced `clap-tui-macros` version has already been published to crates.io.

#### Scenario: Packaging verification succeeds
- **WHEN** maintainers or CI run the baseline release verification command
- **THEN** Cargo validates the package contents for both published crates without uploading either crate

#### Scenario: Library publish dry-run is attempted after the proc-macro prerequisite is met
- **WHEN** maintainers run the publish dry-run mode after the referenced `clap-tui-macros` version is available on crates.io
- **THEN** `cargo publish -p clap-tui --locked --dry-run` succeeds without uploading the crate

### Requirement: Release guidance is documented
The repository SHALL document the release prerequisites and the steps for preparing, validating, tagging, and publishing a new `clap-tui` version, including `CHANGELOG.md` maintenance, the proc-macro prerequisite, the first manual releases, crates.io owner setup, and the transition to trusted publishing.

#### Scenario: Maintainer follows the release guide
- **WHEN** a maintainer prepares a new release
- **THEN** they can complete the versioning and publishing flow by following repository documentation without guessing hidden manual steps

#### Scenario: The proc-macro dependency changes
- **WHEN** maintainers prepare a release that updates the `clap-tui-macros` version required by `clap-tui`
- **THEN** the documentation instructs them to publish `clap-tui-macros` before relying on `clap-tui` publish dry-runs or release automation

### Requirement: Crates.io owner setup is a publishing prerequisite
The repository SHALL document crates.io owner configuration as a prerequisite for automated publishing and SHALL treat owner setup as required before the release workflow is considered ready to publish.

#### Scenario: Automated publishing prerequisites are reviewed
- **WHEN** a maintainer enables automated publishing for `clap-tui`
- **THEN** the documentation instructs them to confirm the intended crates.io owners before relying on the GitHub release workflow
