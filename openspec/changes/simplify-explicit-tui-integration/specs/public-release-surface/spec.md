## MODIFIED Requirements

### Requirement: README onboards external users
The public README SHALL describe `clap-tui` as an installable library for external users rather than as a repo-local development artifact. It SHALL explain how to add the crate as a dependency, state the supported Rust version, present the crate's value proposition early, disclose that the crate was heavily inspired by Trogon, state that it is not an official `clap` crate, include a minimal quick-start path, explain the explicit `Command::Tui` integration model, and surface examples or visuals directly enough that a new user can understand the main flow without leaving the page.

#### Scenario: New user opens the README from crates.io
- **WHEN** a user reads the top-level README without repository context
- **THEN** they can see how to depend on `clap-tui` in their own project
- **THEN** they can understand that the crate is community-built and inspired by Trogon
- **THEN** they can identify `Tui::<T>::run()` as the recommended explicit integration surface
- **THEN** they can see the normal `Command::Tui` dispatch pattern directly in the README

#### Scenario: User evaluates optional features
- **WHEN** a user reads the README before enabling optional functionality
- **THEN** they can see which public features exist and what enabling them changes

### Requirement: Crate-level docs provide a standalone quick start
The crate root rustdoc SHALL provide a concise quick-start path for docs.rs users, including the main value proposition, a brief project-status note that credits Trogon and states that `clap-tui` is not an official `clap` project, the recommended `Tui::<T>::run()` entry point, a short guide for explicit `Command::Tui` integration, and a pointer to a secondary customization path.

#### Scenario: User lands on docs.rs first
- **WHEN** a user opens the crate root documentation on docs.rs
- **THEN** they can see that `clap-tui` is inspired by Trogon and is not an official `clap` project
- **THEN** they can understand the recommended way to launch `clap-tui`
- **THEN** they can see the explicit dispatch pattern that keeps normal application routing outside the crate
- **THEN** they can discover where to look next for examples or supported customization
