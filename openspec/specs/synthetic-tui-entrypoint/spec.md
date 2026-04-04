# synthetic-tui-entrypoint Specification

## Purpose
Define the canonical derive-based launcher flow that exposes a synthetic root `tui`
entrypoint while preserving clap-correct help, parsing, and TUI behavior.
## Requirements
### Requirement: Derive-based CLIs have a canonical typed synthetic TUI launcher
The crate SHALL provide a canonical typed launcher API for derive-based root parser types that adds a synthetic root `tui` launcher without requiring users to define a corresponding parser variant. That typed launcher SHALL be the primary library surface for this flow and SHALL support supplying `TuiConfig` before TUI launch.

#### Scenario: Typed launcher exposes a synthetic `tui` command
- **WHEN** a derive-based root CLI uses the canonical typed launcher API
- **THEN** the compiled application accepts `tool tui` as a valid launch path for the TUI
- **THEN** the user does not need to add a real `Tui` subcommand to their parser type

#### Scenario: Typed launcher can configure TUI launch
- **WHEN** a user configures the canonical typed launcher with `TuiConfig`
- **THEN** the launcher uses that configuration before starting the TUI
- **THEN** the TUI session uses that configuration for the synthetic launch path

### Requirement: `#[clap_tui::main]` is convenience syntax over the typed launcher
The crate SHALL provide `#[clap_tui::main]` as additive convenience syntax over the canonical typed launcher rather than as a separate launcher implementation. The macro SHALL support an optional `config = path::to::fn` argument that supplies a `TuiConfig` before TUI launch.

#### Scenario: Macro delegates to the typed launcher
- **WHEN** a derive-based root CLI uses `#[clap_tui::main]` with the supported function signature
- **THEN** the generated wrapper uses the canonical typed launcher behavior for synthetic launch, non-TUI fallthrough, and typed parsing
- **THEN** the macro path does not define different runtime semantics than the typed API

#### Scenario: Macro can configure TUI launch
- **WHEN** a user applies `#[clap_tui::main(config = path::to::fn)]`
- **THEN** the generated launcher calls that function to obtain `TuiConfig` before starting the TUI
- **THEN** the TUI session uses that configuration for the synthetic launch path

### Requirement: Help and parse diagnostics reflect the augmented launcher surface
The synthetic launcher SHALL participate in the authoritative clap command surface used by the canonical typed launcher for help, version output, and parse diagnostics. Users SHALL see the synthetic `tui` launcher in ordinary CLI help, and clap failures related to the augmented command surface SHALL be reported from that same augmented surface.

#### Scenario: Root help includes the synthetic launcher
- **WHEN** a user runs the root CLI help for an application using `#[clap_tui::main]`
- **THEN** the displayed clap help includes the synthetic root `tui` subcommand
- **THEN** the help text reflects the command surface users can actually invoke

#### Scenario: Parse failure uses augmented command semantics
- **WHEN** a user provides argv that is invalid against the augmented command surface
- **THEN** the application reports the clap failure using the augmented command semantics that include the synthetic launcher
- **THEN** diagnostics do not fall back to a plain typed parser surface that omits `tui`

### Requirement: TUI launch remains bound to the originating parser type
The synthetic launcher SHALL execute the TUI against the same root parser type that generated the clap command definition. When the TUI completes successfully, the returned argv SHALL be parsed into that same root parser type before the user handler runs. When the TUI is cancelled, the launcher SHALL return successfully without invoking the user handler.

#### Scenario: Successful TUI launch parses back into the root type
- **WHEN** a user launches `tool tui` through the typed launcher, completes the TUI flow, and runs a valid command
- **THEN** the returned argv is parsed into the same root parser type used by `#[clap_tui::main]`
- **THEN** the generated launcher calls the user handler with that typed value

#### Scenario: Cancelled TUI does not call the user handler
- **WHEN** a user launches `tool tui` through the typed launcher and exits the TUI without running
- **THEN** the launcher returns success rather than surfacing cancellation as an application error
- **THEN** the user handler is not invoked

#### Scenario: Ordinary CLI invocation still uses the typed parser path
- **WHEN** a user invokes the application without the synthetic `tui` command
- **THEN** the launcher parses argv through the root typed parser for normal execution
- **THEN** the user handler receives the same typed value shape it would receive without the macro

### Requirement: Synthetic launcher attachment rejects conflicting or ambiguous roots
The launcher SHALL reject root parser configurations where a synthetic `tui` subcommand would conflict with existing clap grammar or create ambiguous behavior. This includes existing real `tui` subcommands or aliases and host grammars that already accept `tool tui` as ordinary input or otherwise make the synthetic launcher ambiguous.

#### Scenario: Existing `tui` subcommand or alias is rejected
- **WHEN** the root parser already defines a real `tui` subcommand or visible alias
- **THEN** the synthetic launcher setup fails rather than shadowing the existing command surface
- **THEN** the failure explains that the root launcher conflicts with an existing `tui` path

#### Scenario: Ambiguous host grammar is rejected
- **WHEN** the root parser uses host grammar such as external subcommands or raw trailing capture that would make `tool tui` ambiguous
- **THEN** the synthetic launcher setup fails rather than attaching an ambiguous synthetic command
- **THEN** the user is not given a launcher whose parse behavior depends on hidden precedence rules

### Requirement: The synthetic launcher is hidden from the rendered TUI command tree
The synthetic root `tui` launcher SHALL be visible in ordinary clap help but SHALL NOT appear as a selectable command inside the rendered TUI itself. Command-model and rendering behavior SHALL treat hidden subcommands as non-renderable when building the TUI command tree and related command-derived views.

#### Scenario: TUI command tree omits the synthetic launcher
- **WHEN** the TUI starts through the synthetic launcher path
- **THEN** the rendered command tree does not display `tui` as a selectable command
- **THEN** the user interacts only with the real application command structure

#### Scenario: Hidden subcommands are not surfaced in derived TUI views
- **WHEN** the command model is built from a clap command that contains hidden subcommands
- **THEN** those hidden subcommands are excluded from the TUI-visible command tree and related command-derived views
- **THEN** synthetic launcher hiding uses the same hidden-command behavior rather than a one-off renderer exception
