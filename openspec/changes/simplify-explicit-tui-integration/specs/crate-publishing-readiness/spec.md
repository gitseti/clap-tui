## MODIFIED Requirements

### Requirement: Publishable crate metadata is complete
The `clap-tui` crate SHALL declare the metadata required for a public crates.io release, including `description`, `readme`, `repository`, `homepage`, `documentation`, `license`, `rust-version`, `keywords`, and `categories`. The `repository` and `homepage` values SHALL point at the canonical GitHub repository for this workspace and SHALL NOT use placeholders.

#### Scenario: Release metadata is present
- **WHEN** maintainers inspect the publishable manifest for `clap-tui`
- **THEN** they find the required crates.io packaging, discovery, and docs.rs metadata populated with real repository values instead of unpublished local context or placeholders

### Requirement: The published package is verified before release
The repository SHALL provide a repeatable verification step that confirms the published crate can be packaged for crates.io, and SHALL support a `cargo publish -p clap-tui --locked --dry-run` check for the library crate without a proc-macro prerequisite.

#### Scenario: Packaging verification succeeds
- **WHEN** maintainers or CI run the baseline release verification command
- **THEN** Cargo validates the package contents for the published crate without uploading it

#### Scenario: Library publish dry-run is attempted
- **WHEN** maintainers run the publish dry-run mode for `clap-tui`
- **THEN** `cargo publish -p clap-tui --locked --dry-run` succeeds without requiring a separate proc-macro release first

### Requirement: Release guidance is documented
The repository SHALL document the release prerequisites and the steps for preparing, validating, tagging, and publishing a new `clap-tui` version, including `CHANGELOG.md` maintenance, crates.io owner setup, the first manual releases, and the transition to trusted publishing.

#### Scenario: Maintainer follows the release guide
- **WHEN** a maintainer prepares a new release
- **THEN** they can complete the versioning and publishing flow by following repository documentation without guessing hidden manual steps

#### Scenario: Maintainer reviews release preconditions
- **WHEN** a maintainer checks the release guidance before tagging
- **THEN** the documentation explains the current verification and publishing prerequisites without referencing a separate proc-macro release dependency
