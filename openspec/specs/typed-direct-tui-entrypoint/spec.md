## MODIFIED Requirements

### Requirement: Typed direct TUI execution has one clear public name
The crate SHALL expose a single clearly named typed direct-TUI surface for derive-based CLIs that want to run the TUI explicitly. That surface SHALL be named `Tui`.

#### Scenario: User looks for direct typed TUI execution
- **WHEN** a user reads the public API for derive-based direct TUI execution
- **THEN** they find `Tui` as the named type for that flow
- **THEN** they do not have to infer that framework-like names such as `TypedTuiApp` are the intended 0.1.0 surface

### Requirement: Typed direct TUI execution has one primary documented spelling
The crate SHALL document one primary construction path for typed direct TUI execution. `Tui::<T>::run()` SHALL be that primary documented spelling, and public documentation SHALL not position `TypedTuiApp` or `run_parse` as the primary 0.1.0 spelling.

#### Scenario: User follows docs for direct typed TUI execution
- **WHEN** a user looks for an example of typed direct TUI execution
- **THEN** the docs show `Tui::<T>::run()` as the recommended starting point
- **THEN** the same docs explain that this is the canonical explicit integration surface for 0.1.0

### Requirement: Typed direct TUI execution is positioned relative to the launcher flow
The public docs SHALL describe typed direct TUI execution as the primary derive-based integration model for 0.1.0 and SHALL not position synthetic launcher interception as the recommended path.

#### Scenario: User chooses between derive-based entry points
- **WHEN** a user compares the documented derive-based entry points
- **THEN** the docs describe `Tui::<T>::run()` as the recommended explicit integration path
- **THEN** they do not present `TuiLauncher` as the default 0.1.0 story

## ADDED Requirements

### Requirement: Typed direct TUI execution returns explicit typed outcomes
`Tui::<T>::run()` SHALL return `Result<Option<T>, TuiError>` and SHALL never print automatically or call `std::process::exit`.

#### Scenario: Successful submission returns a typed value
- **WHEN** a caller runs `Tui::<T>::run()` and the user completes the TUI flow successfully
- **THEN** the call returns `Ok(Some(parsed_value))`
- **THEN** the parsed value uses the requested clap parser type `T`

#### Scenario: Cancellation returns `None`
- **WHEN** a caller runs `Tui::<T>::run()` and the user exits the TUI without submitting
- **THEN** the call returns `Ok(None)`
- **THEN** that `None` result means cancellation only

#### Scenario: Clap display and parse flows are returned
- **WHEN** canonical argv exists and reparsing through clap triggers help, version, or parse-display behavior
- **THEN** `Tui::<T>::run()` returns `Err(TuiError::Clap(_))`
- **THEN** the caller remains responsible for deciding whether to print or exit

#### Scenario: Runtime failures remain non-clap errors
- **WHEN** terminal setup, rendering, or internal TUI execution fails before a typed value can be produced
- **THEN** `Tui::<T>::run()` returns a non-clap `TuiError`
- **THEN** the failure is not normalized into `Ok(None)`

### Requirement: Typed direct TUI execution supports explicit subcommand integration
The crate SHALL support explicit integration from an ordinary user-defined clap subcommand without altering the outer clap command surface, help output, routing, or completion behavior beyond the fact that the application itself added that subcommand.

#### Scenario: Application dispatches through a normal `tui` subcommand
- **WHEN** an application defines `Command::Tui` as a normal clap subcommand and calls `Tui::<Command>::run()` from that match arm
- **THEN** the TUI runs for the whole CLI command tree
- **THEN** successful submission returns a parsed `Command`
- **THEN** normal application dispatch remains outside `clap-tui`
