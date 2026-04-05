## MODIFIED Requirements

### Requirement: README onboards external users
The public README SHALL describe `clap-tui` as an installable library for external users rather than as a repo-local development artifact. It SHALL explain how to add the crate as a dependency, state the supported Rust version, present the crate's value proposition early, disclose that the crate was heavily inspired by Trogon, state that it is not an official `clap` crate, include a minimal quick-start path, explain how to choose between the supported entry points, and surface examples or visuals directly enough that a new user can understand the main flows without leaving the page.

#### Scenario: New user opens the README from crates.io
- **WHEN** a user reads the top-level README without repository context
- **THEN** they can see how to depend on `clap-tui` in their own project
- **THEN** they can understand that the crate is community-built and inspired by Trogon
- **THEN** they can identify the recommended entry point for a derive-based CLI
- **THEN** they can understand at least one secondary flow without opening example files

#### Scenario: User evaluates optional features
- **WHEN** a user reads the README before enabling optional functionality
- **THEN** they can see which public features exist and what enabling them changes

### Requirement: Crate-level docs provide a standalone quick start
The crate root rustdoc SHALL provide a concise quick-start path for docs.rs users, including the main value proposition, a brief project-status note that credits Trogon and states that `clap-tui` is not an official `clap` crate, the recommended entry point, a short guide for choosing between the supported entry points, and a pointer to a second example or supported customization path.

#### Scenario: User lands on docs.rs first
- **WHEN** a user opens the crate root documentation on docs.rs
- **THEN** they can see that `clap-tui` is inspired by Trogon and is not an official `clap` project
- **THEN** they can understand the recommended way to launch `clap-tui`
- **THEN** they can tell when to use the other supported entry points
- **THEN** they can discover where to look next for examples or customization
