## ADDED Requirements

### Requirement: Canonical argv is the authoritative invocation
The system SHALL serialize structured invocation state into one canonical `Vec<OsString>` argv token sequence. Validation, run, preview rendering, and copy rendering MUST all derive from that same serialized result.

Serialization preserves all relative ordering explicitly represented in invocation state. Where invocation state does not encode relative order among independent elements, serialization applies a fixed canonical order derived from parser structure and field identity, provided that order is parse-safe; otherwise serialization reports ambiguity.

#### Scenario: One token sequence drives every surface
- **WHEN** the user edits invocation state in the TUI
- **THEN** validation parses the canonical argv with clap
- **AND** Run executes the same canonical argv
- **AND** preview and copy render that same canonical argv rather than building separate command strings

#### Scenario: Execution never depends on rendered shell text
- **WHEN** preview or clipboard output renders canonical argv for a shell
- **THEN** validation and run continue to use argv tokens directly
- **AND** the rendered string is not parsed back into argv for execution

### Requirement: Rendering is a projection of canonical argv
The system SHALL render preview and clipboard output from canonical argv for a target shell. Rendering MUST apply shell-correct quoting and escaping for the selected shell and MUST NOT mutate or reinterpret argv tokens.

#### Scenario: POSIX preview applies shell quoting
- **WHEN** canonical argv contains whitespace, quote characters, empty values, or shell metacharacters on a POSIX target
- **THEN** preview and copy render those tokens with POSIX-shell-correct quoting
- **AND** the underlying argv tokens remain unchanged

#### Scenario: Windows rendering requires explicit shell policy
- **WHEN** canonical argv is rendered on Windows
- **THEN** the renderer uses a documented explicit target shell policy
- **AND** the renderer does not claim one shell string is universally canonical for every Windows shell

### Requirement: Serializer preserves parse-sensitive token shape
The serializer SHALL preserve clap parser behavior whenever concrete argv structure affects parsing. Parse-sensitive shape includes attachment, delimiters, terminators, raw boundaries, ownership and ordering, subcommand and external-subcommand boundaries, hyphen-leading token safety, and explicit empty values.

Preservation refers to maintaining parse-correct token structure and boundaries required by clap, not preserving original shell spelling.

#### Scenario: Required attachment is honored
- **WHEN** an option requires attached `--opt=value` syntax
- **THEN** canonical argv emits that option in attached form
- **AND** clap receives the token shape required by the parser definition

#### Scenario: Delimited values stay in the declared token shape
- **WHEN** an argument declares `value_delimiter`
- **THEN** the serializer preserves the declared single-token delimiter shape where expanding values would change ownership or parse behavior
- **AND** clap receives a token stream that preserves the intended value assignment

#### Scenario: Structural boundaries are emitted when required
- **WHEN** an invocation depends on a terminator, raw `--`, trailing capture, subcommand boundary, or external-subcommand boundary
- **THEN** canonical argv emits the structural token or boundary in the parse-relevant position
- **AND** provenance records the structural token or boundary

#### Scenario: Explicit empty values are preserved
- **WHEN** the user explicitly authors an empty value such as `--opt=`
- **THEN** canonical argv preserves the empty value
- **AND** serialization does not collapse it into omission, a default, or a missing value

### Requirement: Serializer applies deterministic canonical spelling
The serializer SHALL use deterministic spelling where spelling does not change clap parsing. Canonical spelling MUST prefer the primary long name, use a short name only when no long name exists, never emit aliases or hidden aliases, never emit short clusters, and never attach short values unless required by parser shape. Non-value repeated actions, such as counts or repeated booleans, SHALL serialize as repeated canonical flag occurrences rather than clustered shorthand.

#### Scenario: Primary long name is preferred
- **WHEN** an argument has a primary long name plus aliases or a short name
- **THEN** canonical argv emits the primary long name
- **AND** aliases and hidden aliases are not emitted

#### Scenario: Non-value repeated actions use repeated canonical flags
- **WHEN** a count-style flag or repeated boolean has multiple occurrences
- **THEN** canonical argv emits repeated canonical flag occurrences
- **AND** it does not use clustered shorthand such as `-vv`

### Requirement: Serializer reports ambiguity when no unique argv exists
The serializer SHALL return a serialization ambiguity error when structured state cannot be represented as one unique clap-correct argv. Ambiguity errors MUST block validation, run, preview rendering, and copy rendering until the state becomes serializable.

#### Scenario: Occurrence grouping ambiguity is reported
- **WHEN** flattened state for an `Append + num_args(1..)` argument cannot distinguish repeated occurrences from grouped values
- **THEN** serialization reports occurrence grouping ambiguity
- **AND** it does not choose an arbitrary grouping

#### Scenario: Ownership ambiguity is reported
- **WHEN** a variable-length argument competes with later positionals, subcommands, or raw regions and no delimiter, terminator, or boundary resolves ownership uniquely
- **THEN** serialization reports ownership ambiguity
- **AND** it does not reorder tokens to invent a parse outcome

#### Scenario: Hyphen-leading ambiguity is reported
- **WHEN** a value begins with `-` and parser settings do not make it safe as a value through `allow_hyphen_values`, `allow_negative_numbers`, raw capture, trailing capture, or `--`
- **THEN** serialization reports hyphen-leading ambiguity
- **AND** it does not rely on context-sensitive interpretation that could parse as an option

#### Scenario: Positional or trailing ambiguity is reported
- **WHEN** variadic positionals, `last(true)`, trailing var args, or raw boundaries cannot be represented uniquely from current state
- **THEN** serialization reports positional or trailing ambiguity
- **AND** validation and run are blocked

### Requirement: Serializer returns provenance for tokens and diagnostics
The serializer SHALL return provenance mapping canonical argv tokens and structural tokens back to invocation state. Provenance MUST support mapping diagnostics to fields, occurrences when available, positional slots, and command or subcommand regions. Structural provenance includes delimiter-joined tokens, terminators, raw `--`, and subcommand or external-subcommand boundaries.

#### Scenario: Value and delimiter tokens carry provenance
- **WHEN** canonical argv includes ordinary value tokens or delimiter-joined tokens
- **THEN** provenance identifies the source field and occurrence when occurrence data is available
- **AND** diagnostics can point back to the relevant UI control

#### Scenario: Structural tokens carry provenance
- **WHEN** canonical argv includes a delimiter-joined token, terminator, `--`, subcommand boundary, or external-subcommand boundary
- **THEN** provenance identifies the parser structure that required the token
- **AND** diagnostics can distinguish structural serialization from user value input

### Requirement: Serializer does not materialize clap-derivable values into argv
The serializer SHALL NOT emit argv tokens solely to represent values clap can derive internally during parse, including default values, conditional defaults, default-missing values, and environment fallbacks.

#### Scenario: Default and environment values stay implicit
- **WHEN** clap can derive a default or environment-backed value and the user did not explicitly emit the argument
- **THEN** canonical argv omits tokens for that value
- **AND** clap remains responsible for deriving the effective value during parse

#### Scenario: Conditional and missing-value defaults stay implicit
- **WHEN** clap can derive a conditional default or default-missing value
- **THEN** canonical argv does not materialize extra tokens solely to mirror that effective value
- **AND** effective-value reporting remains separate from serialization
