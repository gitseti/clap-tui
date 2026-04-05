## ADDED Requirements

### Requirement: Typed direct TUI execution has one clear public name
The crate SHALL expose a single clearly named typed direct-TUI surface for derive-based CLIs that want to run the TUI without the synthetic launcher. That surface SHALL be named `TypedTuiApp`.

#### Scenario: User looks for direct typed TUI execution
- **WHEN** a user reads the public API for derive-based direct TUI execution
- **THEN** they find `TypedTuiApp` as the named type for that flow
- **THEN** they do not have to infer that `TuiLauncher` and the direct-TUI path are different from context alone

### Requirement: Typed direct TUI execution has one primary documented spelling
The crate SHALL document one primary construction path for typed direct TUI execution. `TuiApp::from_parser::<T>()` SHALL be that primary documented spelling, and item docs SHALL describe `TypedTuiApp` as the type returned by that flow.

#### Scenario: User follows docs for direct typed TUI execution
- **WHEN** a user looks for an example of typed direct TUI execution
- **THEN** the docs show `TuiApp::from_parser::<T>()` as the recommended starting point
- **THEN** the same docs explain that the returned app is a `TypedTuiApp`

### Requirement: Typed direct TUI execution is positioned relative to the launcher flow
The public docs SHALL describe typed direct TUI execution as a secondary derive-based path for applications that want to launch the TUI directly rather than through the synthetic launcher. The docs SHALL continue to position `TuiLauncher` as the default derive-based choice.

#### Scenario: User chooses between derive-based entry points
- **WHEN** a user compares `TuiLauncher` and typed direct TUI execution
- **THEN** the docs describe `TuiLauncher` as the recommended launcher flow
- **THEN** the docs describe `TypedTuiApp` as the direct-run alternative
