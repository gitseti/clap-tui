## ADDED Requirements

### Requirement: Entry-point docs describe observable run semantics
The public documentation for `TuiApp` entry points SHALL describe behavior that callers can actually observe from the exported API. Documentation for `run`, `run_with_matches`, and `run_with_parser` SHALL correctly describe how cancellation is surfaced and when clap parsing errors can occur.

#### Scenario: User reads `run` documentation
- **WHEN** a user reads the docs for `TuiApp::run`
- **THEN** the docs describe argv selection and cancellation behavior accurately
- **THEN** they do not claim that clap parsing errors are returned from `run`

#### Scenario: User reads parser-based entry-point documentation
- **WHEN** a user reads the docs for `run_with_matches` or `run_with_parser`
- **THEN** the docs explain that clap parsing errors occur after the TUI returns argv and parsing is attempted

### Requirement: Documented configuration behavior matches implemented bounds
Public configuration docs SHALL describe the effective behavior of exported configuration fields, including any runtime bounds or clamping applied by the implementation.

#### Scenario: User configures sidebar width
- **WHEN** a user reads the docs for `LayoutConfig.sidebar_ratio`
- **THEN** the docs explain that the ratio is subject to layout bounds applied by the screen layout implementation

### Requirement: Supported extension points are described consistently
Public documentation SHALL describe intentionally exported runtime and customization seams consistently across README, crate docs, and item docs. Exported runtime event and integration types SHALL NOT be described as crate-private or crate-local implementation details when they are intended for library consumers.

#### Scenario: User evaluates runtime customization
- **WHEN** a user reads the public docs for runtime-related exported types
- **THEN** the wording identifies them as supported advanced integration seams
- **THEN** it does not contradict the crate-level description of the public API surface
