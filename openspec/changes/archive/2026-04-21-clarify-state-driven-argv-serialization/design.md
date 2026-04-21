## Context

The canonical argv change established one authoritative `Vec<OsString>` boundary for validation, run, preview, and copy. This follow-up refines one part of that contract: occurrence preservation, delimiter joining, and ambiguity must be driven by invocation state, not by parser shape alone.

## Goals / Non-Goals

**Goals:**
- Clarify that canonical serialization preserves distinctions represented in invocation state.
- Clarify that distinctions not represented in invocation state are normalized according to canonical spelling and parser rules.
- Define delimiter-backed occurrence serialization precisely for flattened and occurrence-aware state.
- Make ambiguity state-relative.
- Define supported-shape boundaries and distinguish unsupported parser shapes from ambiguous current states.
- Define canonical ordering, token-preserving regions, hyphen-leading safety, authored emptiness, and provenance origin classes.
- Make serialization failure a first-class derived-state outcome before validation, effective-value parsing, preview, copy, or execution.

**Non-Goals:**
- Do not introduce a second argv path or any rendered-string execution model.
- Do not require occurrence-aware editing for every `Append + num_args(1..)` parser shape.
- Do not rewrite unchanged canonical spelling, rendering, effective-value, or provenance requirements.

## Decisions

### Definitions: supported, unsupported, and ambiguous

A parser shape is supported when the TUI state model can represent the distinctions needed for correct serialization. A parser shape is unsupported when those distinctions cannot be represented in principle by the current model. A state is ambiguous when the parser shape is supported, but the current invocation does not contain enough structure to derive one unique parse-safe argv.

### This replaces the broad flattened-state assumption

This replaces the previous assumption:

> Flattened state is insufficient for occurrence-sensitive args.

with:

> Flattened state is insufficient only when canonical serialization or diagnostics require distinctions that the invocation state does not represent.

The serializer reflects invocation state. Clap accepting multiple concrete argv shapes does not justify inventing occurrence structure that the state does not encode.

Alternatives considered:
- Treat every occurrence-sensitive parser shape as ambiguous when state is flat. Rejected because flattened controls such as multi-selects intentionally represent one logical occurrence.
- Infer occurrence boundaries from parser flexibility. Rejected because canonical argv must explain state, not choose any valid parse.

### State-driven serialization is the primary rule

Canonical serialization preserves distinctions represented in invocation state. Distinctions not represented in state are normalized according to canonical spelling and parser rules.

For example, when invocation state models a multi-value field as one flattened logical occurrence, serialization emits one canonical occurrence. An occurrence-aware repeated editor emits the represented occurrences. Both can be correct because they represent different invocation states.

Serialization is correct only when the derived argv is accepted by clap and uniquely justified by the invocation state plus parser definition. Clap accepting one of several possible argv shapes is not enough.

### Canonical serialization applies only to supported shapes

Canonical serialization is defined only for parser shapes and invocation states that the TUI state model can represent faithfully. For unsupported shapes, serialization returns a structured unsupported-shape diagnostic instead of approximating argv.

There are two failure classes:
- Fundamentally unsupported shape: the parser shape exceeds what the TUI model can represent in principle. The editor/form may need to degrade, refuse that shape, or expose a non-canonical fallback.
- Ambiguous current state: the parser shape is supported in general, but the current invocation state lacks structure needed to serialize uniquely. The user may fix this by adding structure or changing values.

### Delimiters operate inside represented occurrences

For delimiter-backed multi-value arguments:
- Values within a single represented occurrence may be joined using the declared delimiter.
- Distinct occurrences are preserved only when they are represented in invocation state.
- Flattened multi-value state serializes as a single occurrence.
- The serializer must not invent additional occurrences.
- The serializer must not merge distinct occurrences explicitly represented in state.

This keeps delimiter flattening correct for flattened UI models while preventing over-flattening when state actually encodes multiple occurrences.

### Ambiguity is state-relative

Occurrence ambiguity arises only when parser semantics make occurrence boundaries relevant and invocation state does not represent the boundaries required for canonical serialization or diagnostics.

Parser shape alone is not enough to report ambiguity. A flat model with a clear single-occurrence meaning is serializable as one occurrence.

### Parse-sensitive rules do not create structure

Attachment, delimiters, terminators, raw boundaries, ownership rules, subcommand boundaries, and hyphen-leading value safety operate within the structure defined by invocation state. They may choose token shape needed to preserve parsing, but they must not introduce or remove structural distinctions not present in that state.

A value that may be parsed by clap as an option, flag, or subcommand must not be emitted where ownership depends on ambiguous parser behavior unless the parser definition or token shape provides an unambiguous value boundary.

Serialization preserves authored emptiness when invocation state represents an explicit authored empty value. It must not rewrite explicit emptiness into omission, defaulting, or a semantically different token form.

Preservation refers to maintaining parse-correct token structure and boundaries required by clap, not preserving original shell spelling.

### Ordering is canonical unless state represents order

Serialization preserves all relative ordering explicitly represented in invocation state. Where invocation state does not encode relative order among independent elements, serialization applies a fixed canonical order derived from parser structure and field identity, provided that order is parse-safe; otherwise serialization reports ambiguity.

Positionals follow parser-defined positional order. Subcommand boundaries, raw boundaries, terminators, trailing regions, and external-subcommand tails are emitted at their represented parser boundaries. Options and independent repeated occurrences without authored cross-field ordering use the canonical field order rather than a guessed shell authoring order.

### Reconstruct modeled regions and preserve carried-through regions

This is a hybrid model. For regions fully modeled by invocation state, serialization reconstructs canonical tokens. For explicitly token-preserving regions that the TUI carries without full semantic structure, serialization preserves token content and boundaries verbatim and reports reduced provenance granularity where applicable.

If no token-preserving region exists for a parser shape, the serializer must not pretend to preserve it; unsupported or ambiguous diagnostics are preferred over partial reconstruction.

### Provenance records token origin

Provenance distinguishes token origins needed for diagnostics and debugging:
- structural token inserted by the serializer, such as `--`, a terminator, an option name, or a subcommand boundary
- value token authored through UI state
- delimiter-joined token synthesized from multiple UI values within one represented occurrence
- preserved token region carried verbatim
- canonical spelling substitution from a field identity

### Derived state is gated on serialization success

Validation, effective-value parsing, preview, copy, and execution consume the serialization result only after serialization succeeds. Serialization failure is a first-class derived-state outcome and must not be labeled as a clap validation error.

## Migration Plan

1. Classify serializer failures as unsupported-shape diagnostics or state-specific ambiguity diagnostics.
2. Apply state-driven delimiter and occurrence handling without inventing or merging represented occurrences.
3. Apply canonical ordering only where invocation state lacks relative order and the parser structure makes the order parse-safe.
4. Preserve explicitly token-carried regions verbatim; reconstruct fully modeled regions canonically.
5. Compute derived validation state, preview, copy, execution, and effective-value parsing only from a successful serialization result.
6. Report serialization failure distinctly from clap validation failure.

## Risks / Trade-offs

- [Flattened controls may hide occurrence distinctions a shell user could have typed] -> Treat this as intentional state normalization unless the UI state claims to preserve occurrences.
- [Occurrence-aware controls require stricter serializer behavior] -> Preserve represented occurrence boundaries and add regression tests that fail if they are merged.
- [Ambiguity diagnostics become more contextual] -> Tests should assert both parser shape and state shape before expecting ambiguity.
- [Unsupported-shape diagnostics may block shapes clap can parse] -> Prefer explicit unsupported diagnostics over argv approximations that the TUI cannot justify from state.
- [Canonical ordering can mask original shell authoring order] -> Treat shell authoring order as out of scope unless invocation state explicitly represents it.
