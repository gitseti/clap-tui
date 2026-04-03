## ADDED Requirements

### Requirement: Preview and run share one authoritative argv
The system SHALL derive preview argv, validation argv, and run argv from the same invocation state and serializer path so that all three surfaces describe the same command invocation.

#### Scenario: Preview matches the argv used for validation and run
- **WHEN** the user edits inputs in the form
- **THEN** the preview pane shows the exact argv that clap validation checks
- **THEN** the Run action uses that same argv without a separate reconstruction path

### Requirement: Argv synthesis preserves clap-relevant token shape
The serializer SHALL preserve token-shape semantics that can change clap parsing, including grouped values per occurrence, `--opt=value`, delimiter-driven expansion, value terminators, trailing positional behavior, trailing delimiter controls, and raw capture behavior.

#### Scenario: Require-equals option renders as attached value
- **WHEN** an option requires an equals sign
- **THEN** preview and run serialize the option using `--option=value` form
- **THEN** clap validation accepts the serialized token shape

#### Scenario: Delimited multi-value argument preserves clap parsing expectations
- **WHEN** an argument uses delimiter-driven or terminator-driven parsing
- **THEN** preview and run serialize the argument in a way that preserves the clap-configured parsing behavior
- **THEN** validation is performed against that serialized argv

#### Scenario: Raw or trailing capture keeps tokens available to the target argument
- **WHEN** a command uses raw capture or trailing positional semantics
- **THEN** preview and run keep the captured tokens attached to the intended argument
- **THEN** subcommands or later arguments do not consume those tokens incorrectly

#### Scenario: Trailing values honor disabled delimiter splitting
- **WHEN** a command disables delimiter splitting for trailing values
- **THEN** preview and run preserve those trailing values as unsplit tokens
- **THEN** clap validation evaluates the same unsplit argv

### Requirement: Command-path edge cases remain parse-correct
The system SHALL support clap command-level parsing rules that materially affect interactive command construction, including required subcommands, argument and subcommand conflicts, external subcommands, missing-positional edge cases, and parse-boundary rules between subcommands and arguments.

#### Scenario: Required subcommand remains invalid until selected
- **WHEN** a command requires a subcommand and none is selected
- **THEN** validation marks the invocation invalid
- **THEN** the form and summary explain that a subcommand is required

#### Scenario: External subcommand can be entered and validated
- **WHEN** a command allows external subcommands
- **THEN** the user can provide an external subcommand and its trailing values through the TUI
- **THEN** preview and validation preserve that external subcommand structure

#### Scenario: Parse boundary favors subcommand when configured
- **WHEN** a command uses parser settings that control whether a token is treated as an argument value or a subcommand
- **THEN** preview and validation respect the configured parse boundary
- **THEN** the resulting argv is accepted or rejected by clap for the same reasons the user sees in the TUI

#### Scenario: Missing positional parsing follows command settings
- **WHEN** a command allows otherwise-missing positional inputs under a parser edge setting
- **THEN** preview and validation follow that command-level behavior
- **THEN** the TUI does not invent stricter local parsing rules than clap applies
