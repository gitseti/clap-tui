## MODIFIED Requirements

### Requirement: Entry-point docs describe observable run semantics
The public documentation for `clap-tui` entry points SHALL describe behavior that callers can actually observe from the exported API. Documentation for the main entry points SHALL correctly describe how cancellation is surfaced, when clap parsing errors can occur, and when each entry point is the recommended choice.

#### Scenario: User reads `run` documentation
- **WHEN** a user reads the docs for `TuiApp::run` or `TypedTuiApp::run`
- **THEN** the docs describe argv selection and cancellation behavior accurately
- **THEN** they do not claim that clap parsing errors are returned from `run`

#### Scenario: User compares entry points
- **WHEN** a user reads the public docs for `TuiLauncher`, `TypedTuiApp`, and `TuiApp`
- **THEN** they can tell which surface is recommended for derive-based launchers
- **THEN** they can tell which surface is intended for direct TUI execution

### Requirement: Supported extension points are described consistently
Public documentation SHALL describe intentionally exported runtime and customization seams consistently across the README, crate docs, and item docs. Exported runtime event and integration types SHALL be described in concise user-facing language and SHALL not crowd out the primary entry-point guidance.

#### Scenario: User evaluates runtime customization
- **WHEN** a user reads the public docs for runtime-related exported types
- **THEN** the wording identifies them as advanced integration seams
- **THEN** it does not contradict the crate-level description of the public API surface
- **THEN** it uses terminology that is easier to understand than repeated architectural labels alone
