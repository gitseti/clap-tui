## ADDED Requirements

### Requirement: Canonical serialization has an explicit support boundary
Canonical serialization SHALL be defined only for parser shapes and invocation states that the TUI state model can represent faithfully. For unsupported shapes, serialization MUST return a structured unsupported-shape diagnostic rather than approximating argv.

Serialization correctness requires the derived argv to be accepted by clap and uniquely justified by invocation state and parser definition. The serializer MUST NOT treat "one of several clap-accepted parses" as sufficient.

A parser shape is supported when the TUI state model can represent the distinctions needed for correct serialization. A parser shape is unsupported when those distinctions cannot be represented in principle by the current model. A state is ambiguous when the parser shape is supported, but the current invocation does not contain enough structure to derive one unique parse-safe argv.

#### Scenario: Unsupported parser shape is diagnosed
- **WHEN** a parser shape exceeds what the TUI state model can represent faithfully
- **THEN** serialization returns a structured unsupported-shape diagnostic
- **AND** validation, run, preview, and copy are blocked by that serialization result

#### Scenario: Clap acceptance alone is insufficient
- **WHEN** clap would accept multiple argv shapes for the same partial state
- **THEN** the serializer emits argv only when one shape is uniquely justified by invocation state and parser definition
- **AND** otherwise reports ambiguity or unsupported shape instead of choosing an arbitrary accepted parse

### Requirement: Canonical serialization is state-driven
Canonical serialization SHALL preserve distinctions represented in invocation state. Distinctions not represented in invocation state MUST be normalized according to canonical spelling and parser rules. This refines canonical serialization without changing the single authoritative argv model.

This replaces the previous broad assumption that "flattened state is insufficient for occurrence-sensitive args" with: flattened state is insufficient only when canonical serialization or diagnostics require distinctions that the invocation state does not represent.

Serialization preserves all relative ordering explicitly represented in invocation state. Where invocation state does not encode relative order among independent elements, serialization applies a fixed canonical order derived from parser structure and field identity, provided that order is parse-safe; otherwise serialization reports ambiguity.

#### Scenario: Represented distinctions are preserved
- **WHEN** invocation state represents distinct occurrences, explicit empty values, structural boundaries, or authored presence
- **THEN** canonical argv preserves those represented distinctions
- **AND** validation, run, preview, and copy derive from that same argv result

#### Scenario: Unrepresented distinctions are normalized
- **WHEN** clap accepts multiple concrete argv shapes but invocation state represents only one flattened value set
- **THEN** canonical argv emits the normalized spelling for that state
- **AND** the serializer does not invent occurrence structure solely because clap could parse it

### Requirement: Delimiter-backed occurrences follow invocation state
For delimiter-backed multi-value arguments, the serializer SHALL apply delimiter joining within the occurrence structure represented by invocation state. Values within a single represented occurrence MAY be joined using the declared delimiter. Distinct occurrences MUST only be preserved when they are represented in invocation state.

If the state is modeled as a flattened multi-value field, such as a multi-select, serialization SHALL emit a single occurrence. The serializer MUST NOT invent additional occurrences. The serializer MUST NOT merge distinct occurrences that are explicitly represented in state.

#### Scenario: Flattened delimiter state emits one occurrence
- **WHEN** invocation state models a delimiter-backed multi-value field as one flattened logical occurrence
- **THEN** canonical argv emits one occurrence containing the field's values
- **AND** values in that occurrence may be joined using the declared delimiter

#### Scenario: Explicit delimiter occurrences remain distinct
- **WHEN** invocation state explicitly represents multiple occurrences for a delimiter-backed argument
- **THEN** canonical argv preserves those occurrence boundaries
- **AND** delimiter joining is applied independently within each represented occurrence

### Requirement: Occurrence ambiguity is state-relative
Occurrence ambiguity SHALL be reported only when parser semantics make occurrence boundaries relevant and invocation state does not represent the boundaries required for canonical serialization or diagnostics. Parser shape alone MUST NOT cause occurrence ambiguity.

This refines failure classification: a fundamentally unsupported shape exceeds what the TUI can represent in principle; an ambiguous current state belongs to a supported shape but lacks enough represented structure for this invocation.

#### Scenario: Parser flexibility alone is not ambiguous
- **WHEN** a parser accepts repeated occurrences but invocation state represents a flattened single occurrence
- **THEN** serialization emits the canonical single occurrence
- **AND** it does not report occurrence ambiguity solely because repeated argv shapes are possible

#### Scenario: Missing required occurrence distinctions are ambiguous
- **WHEN** parser semantics require occurrence boundaries for correct token shape or diagnostics
- **AND** invocation state does not represent those boundaries
- **THEN** serialization reports occurrence ambiguity
- **AND** validation, run, preview, and copy are blocked by the serialization diagnostic

#### Scenario: Unsupported shape is distinct from ambiguous state
- **WHEN** the parser shape cannot be represented by the TUI model in principle
- **THEN** serialization reports unsupported shape
- **AND** it does not report a user-fixable occurrence ambiguity for that condition

### Requirement: Parse-sensitive rules preserve state structure
Parse-sensitive rules, including attachment, delimiters, terminators, raw boundaries, ownership, subcommand boundaries, external-subcommand boundaries, and hyphen-leading value safety, SHALL operate within the structure defined by invocation state. They MUST NOT introduce or remove structural distinctions that are not present in that state.

Occurrence-aware state is required only for arguments where grouping is explicitly represented or needed for correct parsing or diagnostics. Flattened models MAY emit a single occurrence.

Preservation refers to maintaining parse-correct token structure and boundaries required by clap, not preserving original shell spelling.

#### Scenario: Token shape does not invent occurrences
- **WHEN** parse-sensitive token shape requires attached values, delimiter joining, a terminator, or a raw boundary
- **THEN** the serializer emits the required token shape inside the represented state structure
- **AND** it does not add or remove represented occurrences

#### Scenario: Flattened models remain serializable when distinctions are unnecessary
- **WHEN** a flattened model contains enough information to produce canonical argv and diagnostics
- **THEN** the serializer emits a normalized single-occurrence argv shape
- **AND** occurrence-aware state is not required for that argument

### Requirement: Canonical ordering follows represented state or parser structure
Serialization SHALL preserve all relative ordering explicitly represented in invocation state. Where invocation state does not represent relative order among independent elements, serialization MUST apply a fixed canonical order derived from parser structure and field identity, provided that order is parse-safe. If no parse-safe canonical order is uniquely justified, serialization MUST report ambiguity.

Positionals MUST follow parser-defined positional order. Subcommand boundaries, raw boundaries, terminators, trailing regions, and external-subcommand tails MUST be emitted at their represented parser boundaries. Options and independent repeated occurrences without authored cross-field ordering MUST use canonical field order rather than guessed shell authoring order.

#### Scenario: Independent options use canonical field order
- **WHEN** invocation state contains independent options without represented relative authoring order
- **THEN** canonical argv emits them in fixed canonical field order
- **AND** the serializer does not infer shell typing order

#### Scenario: Parser boundaries constrain ordering
- **WHEN** invocation state includes positionals, subcommands, raw boundaries, terminators, trailing values, or external-subcommand tails
- **THEN** canonical argv places those tokens at parser-defined or represented boundaries
- **AND** options are not moved across those boundaries unless the move is uniquely parse-safe and justified by state

### Requirement: Hyphen-leading values require unambiguous ownership
A value that may be parsed by clap as an option, flag, or subcommand MUST NOT be emitted in a position where ownership depends on ambiguous parser behavior, unless the parser definition or token shape provides an unambiguous value boundary.

This applies to detached option values, variadic values, trailing positional regions, values near subcommand boundaries, and external-subcommand payloads.

#### Scenario: Unsafe hyphen-leading value is blocked
- **WHEN** a user-authored value begins with `-`
- **AND** parser settings or token boundaries do not make it unambiguously owned by the intended argument
- **THEN** serialization reports a hyphen-leading ambiguity diagnostic
- **AND** validation, run, preview, and copy are blocked

#### Scenario: Hyphen-leading value is emitted when ownership is explicit
- **WHEN** parser settings or emitted token shape provide an unambiguous value boundary for a hyphen-leading value
- **THEN** canonical argv may include that value
- **AND** clap parses it as the represented value rather than as an option, flag, or subcommand

### Requirement: Authored empty values remain explicit
Serialization SHALL preserve authored emptiness when invocation state represents an explicit authored empty value. The serializer MUST NOT rewrite explicit emptiness into omission, defaulting, or a semantically different token form.

#### Scenario: Explicit empty value is not omitted
- **WHEN** invocation state represents a user-authored empty value
- **THEN** canonical argv preserves the empty value in a parse-correct token shape
- **AND** the serializer does not collapse it into an omitted argument or derived default

### Requirement: Token preservation is explicit
For regions fully modeled by invocation state, serialization SHALL reconstruct canonical tokens. For explicitly token-preserving regions that the TUI carries without full semantic structure, serialization SHALL preserve token content and boundaries verbatim and report reduced provenance granularity where applicable.

If a parser region is neither fully represented nor intentionally preserved as a token region, the serializer MUST report ambiguity or unsupported shape rather than partially reconstructing it.

#### Scenario: Modeled region is reconstructed
- **WHEN** invocation state semantically represents a field, occurrence, structural boundary, or command region
- **THEN** canonical argv reconstructs that region from state using canonical spelling and parser rules

#### Scenario: Carried-through region is preserved
- **WHEN** the TUI intentionally carries a region as raw tokens without semantic field modeling
- **THEN** canonical argv preserves those tokens verbatim
- **AND** provenance indicates reduced granularity for the preserved region

### Requirement: Provenance records canonical token origin
Provenance SHALL distinguish token origins needed for diagnostics and debugging, including structural tokens inserted by the serializer, value tokens authored through UI state, delimiter-joined tokens synthesized from multiple UI values within one represented occurrence, preserved token regions carried verbatim, and canonical spelling substitutions from field identity.

#### Scenario: Synthesized and structural tokens are distinguishable
- **WHEN** canonical argv includes option names, terminators, raw boundaries, subcommand boundaries, or delimiter-joined values
- **THEN** provenance identifies whether each token is structural, synthesized from values, or a canonical spelling substitution
- **AND** diagnostics can explain the token's relationship to invocation state

#### Scenario: Preserved regions have reduced provenance
- **WHEN** canonical argv includes a token-preserved region
- **THEN** provenance marks the region as preserved
- **AND** diagnostics do not claim field-level precision that the state does not contain

### Requirement: Serialization gates downstream derived state
Derived validation state, effective-value parsing, preview rendering, copy rendering, and execution SHALL consume the serialization result only after serialization succeeds. Serialization failure SHALL be a first-class derived-state outcome, distinct from clap validation failure.

#### Scenario: Serialization failure blocks downstream surfaces
- **WHEN** serialization returns ambiguity or unsupported-shape diagnostics
- **THEN** validation and effective-value parsing are skipped
- **AND** preview, copy, and execution are blocked by the serialization result

#### Scenario: Clap validation runs only after serialization succeeds
- **WHEN** serialization succeeds and produces canonical argv
- **THEN** clap validation parses that argv
- **AND** any resulting clap errors are reported separately from serialization diagnostics
