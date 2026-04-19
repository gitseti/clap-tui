## MODIFIED Requirements

### Requirement: Preview and run share one authoritative argv
The system SHALL derive preview argv, validation argv, and run argv from the same invocation state and serializer path so that all three surfaces describe the same command invocation. When clap parsing depends on materialized defaults or parse-sensitive ordering, the preview SHALL reflect that same parse-relevant token sequence rather than hiding a different argv shape.

#### Scenario: Preview matches the argv used for validation and run
- **WHEN** the user edits inputs in the form
- **THEN** the preview pane shows the exact argv that clap validation checks
- **THEN** the Run action uses that same argv without a separate reconstruction path

#### Scenario: Parse-affecting defaults stay visible across surfaces
- **WHEN** validation or run materializes a default-derived token that affects clap parsing
- **THEN** the preview reflects the same token in the same parse-relevant position
- **THEN** the user does not encounter a hidden parse difference between preview and execution

### Requirement: Argv synthesis preserves clap-relevant token shape
The serializer SHALL preserve token-shape semantics that can change clap parsing, including grouped values per occurrence, `--opt=value`, delimiter-driven expansion, value terminators, trailing positional behavior, trailing delimiter controls, raw capture behavior, and parse-sensitive ordering between positionals and variable-arity options. The emitted argv SHALL reparse under clap into the same semantic assignment represented by the interactive form state.

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

#### Scenario: Positional stays ahead of a greedy option that would consume it
- **WHEN** a command includes a positional and a variable-arity option whose value list would otherwise consume that positional
- **THEN** the serializer emits argv in an order that preserves the positional's semantic assignment under clap parsing
- **THEN** clap does not reinterpret that positional as another value for the option
