## ADDED Requirements

### Requirement: Effective input state is deterministic after initialization
The TUI SHALL resolve environment-backed and default-backed input state during command initialization and MUST NOT re-read process environment variables while projecting effective form state for rendering, validation, or interaction.

#### Scenario: Environment-backed default is materialized at startup
- **WHEN** a command argument declares an environment-backed default and the app initializes that command path
- **THEN** the resolved value is stored in owned input state
- **THEN** later effective-state reads use the stored value without consulting the process environment again

#### Scenario: Effective reads remain stable after environment mutation
- **WHEN** the app has already initialized input state for a command path and the process environment changes afterward
- **THEN** the rendered form, preview argv, and validation output remain unchanged until the app creates a new session or reinitializes state explicitly
