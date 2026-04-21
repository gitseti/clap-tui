## MODIFIED Requirements

### Requirement: Argv synthesis preserves clap-relevant token shape
The serializer SHALL preserve token-shape semantics that can change clap parsing, including grouped values per represented occurrence, `--opt=value`, delimiter-driven expansion, value terminators, trailing positional behavior, trailing delimiter controls, raw capture behavior, and parse-sensitive ordering between positionals and variable-arity options. The emitted argv SHALL reparse under clap into the same semantic assignment represented by the interactive form state.

Canonical serialization SHALL be driven by invocation state. It MUST preserve occurrence boundaries that invocation state explicitly represents. It MUST normalize distinctions not represented in invocation state according to canonical spelling and parser rules. Flattened state is insufficient only when canonical serialization or diagnostics require distinctions that the invocation state does not represent.

Serialization is correct only when the emitted argv is accepted by clap and uniquely justified by invocation state and parser definition. Parser shapes the TUI cannot represent faithfully MUST produce unsupported-shape diagnostics rather than approximate argv. A state-specific ambiguity MUST be used only when the parser shape is supported but the current invocation lacks enough structure to derive one unique parse-safe argv.

A parser shape is supported when the TUI state model can represent the distinctions required for correct serialization. A state is ambiguous when the shape is supported but the current invocation state lacks enough structure to derive a unique parse-safe argv. Shapes whose required distinctions cannot be represented are unsupported.

Serialization preserves all relative ordering explicitly represented in invocation state. Where invocation state does not encode relative order among independent elements, serialization applies a fixed canonical order derived from parser structure and field identity, provided that order is parse-safe; otherwise serialization reports ambiguity.

Preservation refers to maintaining parse-correct token structure and boundaries required by clap, not preserving original shell spelling.

#### Scenario: Require-equals option renders as attached value
- **WHEN** an option requires an equals sign
- **THEN** preview and run serialize the option using `--option=value` form
- **THEN** clap validation accepts the serialized token shape

#### Scenario: Delimited multi-value argument preserves represented occurrence shape
- **WHEN** an argument uses delimiter-driven or terminator-driven parsing
- **THEN** preview and run serialize the argument in a way that preserves the clap-configured parsing behavior within the occurrence structure represented by invocation state
- **THEN** validation is performed against that serialized argv

#### Scenario: Flattened delimiter input emits a single occurrence
- **WHEN** invocation state models a delimiter-backed multi-value field as one flattened logical occurrence, such as a multi-select
- **THEN** preview and run emit one canonical occurrence for that field
- **THEN** the serializer does not invent repeated occurrences only because clap would accept them

#### Scenario: Explicit delimiter occurrences are not merged
- **WHEN** invocation state explicitly represents multiple occurrences for a delimiter-backed multi-value argument
- **THEN** preview and run preserve those occurrence boundaries
- **THEN** delimiter joining is applied separately within each represented occurrence

#### Scenario: Raw or trailing capture keeps tokens available to the target argument
- **WHEN** a command uses raw capture or trailing positional semantics
- **THEN** preview and run keep the captured tokens attached to the intended argument
- **THEN** subcommands or later arguments do not consume those tokens incorrectly

#### Scenario: Trailing values honor disabled delimiter splitting
- **WHEN** a command disables delimiter splitting for trailing values
- **THEN** preview and run preserve those trailing values as unsplit tokens
- **THEN** clap validation evaluates the same unsplit argv

#### Scenario: Positional stays ahead of a greedy option when state requires it
- **WHEN** invocation state represents a positional and a variable-arity option whose value list could otherwise consume that positional
- **THEN** the serializer emits argv in a form that preserves the represented semantic assignment under clap parsing when such a form exists
- **THEN** serialization reports ambiguity when parser semantics require ownership distinctions that invocation state does not represent

#### Scenario: Unsupported parser shape is not approximated
- **WHEN** a clap parser shape cannot be faithfully represented by the TUI invocation state model
- **THEN** argv synthesis reports an unsupported-shape diagnostic
- **THEN** preview, run, and validation do not proceed with an approximate argv

#### Scenario: Canonical ordering is parse-safe
- **WHEN** invocation state does not represent authoring order among independent fields
- **THEN** argv synthesis emits a fixed canonical order derived from parser structure and field identity
- **THEN** it reports ambiguity instead of reordering across parser boundaries where ownership would not be uniquely justified

#### Scenario: Hyphen-leading value ownership is explicit
- **WHEN** a value could be parsed as an option, flag, or subcommand
- **THEN** argv synthesis emits it only when parser settings or token boundaries make value ownership unambiguous
- **THEN** otherwise serialization reports a hyphen-leading ambiguity
