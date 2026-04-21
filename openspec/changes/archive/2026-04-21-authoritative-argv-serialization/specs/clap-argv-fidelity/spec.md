## REMOVED Requirements

### Requirement: Preview and run share one authoritative argv
**Reason**: Replaced by the canonical argv contract, which defines one token sequence plus shell rendering and ambiguity handling.
**Migration**: Use the `argv-serialization-boundary` capability and the updated canonical argv requirements.

## ADDED Requirements

### Requirement: Validation, run, preview, and copy share canonical argv
The system SHALL derive validation, run, preview, and copy behavior from one canonical argv token sequence. Preview and copy MUST be renderings of that sequence and MUST NOT define separate command behavior.

#### Scenario: Preview and copy render the validated argv
- **WHEN** the preview is rendered or copied for the current invocation state
- **THEN** it renders the same canonical argv that validation checks
- **AND** Run uses that same canonical argv

#### Scenario: Serialization ambiguity blocks command surfaces
- **WHEN** canonical serialization reports ambiguity
- **THEN** validation, run, preview, and copy are blocked
- **AND** the UI surfaces a serialization error distinct from a clap validation failure

## MODIFIED Requirements

### Requirement: Argv synthesis preserves clap-relevant token shape
The serializer SHALL preserve token-shape semantics that can change clap parsing, including grouped values per occurrence, `--opt=value`, delimiter-driven parsing, value terminators, trailing positional behavior, trailing delimiter controls, raw capture behavior, hyphen-leading token safety, explicit empty values, and parse-sensitive ordering between positionals, options, and subcommands. The canonical argv used for validation and run SHALL reparse under clap into the same semantic assignment represented by the interactive form state, or serialization SHALL report ambiguity.

#### Scenario: Require-equals option renders as attached value
- **WHEN** an option requires an equals sign
- **THEN** canonical argv serializes the option using `--option=value` form
- **AND** clap accepts the serialized token shape

#### Scenario: Delimited multi-value argument preserves clap parsing expectations
- **WHEN** an argument uses delimiter-driven parsing
- **THEN** canonical argv serializes the argument in a way that preserves the clap-configured delimiter behavior
- **AND** ambiguity is reported if flattened state cannot determine a unique occurrence grouping

#### Scenario: Raw or trailing capture keeps tokens available to the target argument
- **WHEN** a command uses raw capture or trailing positional semantics
- **THEN** canonical argv keeps the captured tokens attached to the intended argument
- **AND** subcommands or later arguments do not consume those tokens incorrectly

#### Scenario: Trailing values honor disabled delimiter splitting
- **WHEN** a command disables delimiter splitting for trailing values
- **THEN** canonical argv preserves those trailing values as unsplit tokens
- **AND** clap evaluates the same unsplit argv in validation and run

#### Scenario: Positional and greedy option ownership is unambiguous
- **WHEN** a command includes a positional and a variable-arity option whose value list could consume that positional
- **THEN** canonical argv preserves the intended semantic assignment when a unique clap-correct representation exists
- **AND** serialization reports ambiguity when no unique representation exists

### Requirement: Command-path edge cases remain parse-correct
The system SHALL support clap command-level parsing rules that materially affect interactive command construction, including required subcommands, argument and subcommand conflicts, external subcommands, missing-positional edge cases, and parse-boundary rules between subcommands and arguments. Validation and run SHALL evaluate those rules against canonical argv.

#### Scenario: Required subcommand remains invalid until selected
- **WHEN** a command requires a subcommand and none is selected
- **THEN** validation marks the invocation invalid
- **AND** the form and summary explain that a subcommand is required

#### Scenario: External subcommand can be entered and validated
- **WHEN** a command allows external subcommands
- **THEN** the user can provide an external subcommand and its trailing values through the TUI
- **AND** canonical argv preserves that external subcommand structure

#### Scenario: Parse boundary favors subcommand when configured
- **WHEN** a command uses parser settings that control whether a token is treated as an argument value or a subcommand
- **THEN** canonical argv respects the configured parse boundary
- **AND** the resulting argv is accepted or rejected by clap for the same reasons the user sees in the TUI

#### Scenario: Missing positional parsing follows command settings
- **WHEN** a command allows otherwise-missing positional inputs under a parser edge setting
- **THEN** validation and run follow that command-level behavior
- **AND** the TUI does not invent stricter local parsing rules than clap applies
