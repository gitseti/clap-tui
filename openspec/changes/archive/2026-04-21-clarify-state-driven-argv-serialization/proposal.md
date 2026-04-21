## Why

The canonical argv work needs a narrower contract for occurrence preservation and delimiter handling. The previous wording can be read as requiring occurrence-aware state for every occurrence-sensitive parser shape, even when the TUI invocation state intentionally represents a flattened value set.

## What Changes

- Refine canonical serialization to be explicitly state-driven: preserve distinctions represented in invocation state, and normalize distinctions that are not represented.
- Replace the broad assumption that flattened state is inherently insufficient for occurrence-sensitive arguments.
- Clarify delimiter-backed multi-value serialization:
  - join values within one represented occurrence when delimiter shape is canonical or parse-protective
  - preserve distinct represented occurrences
  - serialize flattened multi-value state as a single occurrence
  - never invent occurrences and never merge occurrences that state explicitly represents
- Clarify that occurrence ambiguity is state-relative, not parser-shape-only.
- Clarify that parse-sensitive rules operate inside the structure provided by invocation state.
- Define the boundary between fundamentally unsupported parser shapes and state-specific ambiguity.
- Clarify canonical ordering, hyphen-leading value safety, authored emptiness, token-preserving regions, and provenance origin classes.
- Keep the single authoritative `Vec<OsString>` argv model unchanged.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `argv-serialization-boundary`: Clarify state-driven canonical serialization, delimiter occurrence policy, and state-relative ambiguity for the in-flight canonical argv contract.
- `clap-argv-fidelity`: Clarify that parse-correct argv synthesis preserves or normalizes occurrence structure according to invocation state.

## Impact

- Affected specs: `argv-serialization-boundary` and `clap-argv-fidelity`.
- Likely affected code: `crates/clap-tui/src/argv_serializer.rs`, derived-state serialization gating, occurrence-aware input state handling, and serializer regression tests around delimiter-backed `Append + num_args(1..)`.
- No new architectural concepts, execution paths, rendering targets, or dual argv models.

## Clarifications

A parser shape is supported when the TUI state model can represent all distinctions required for correct serialization. A state is ambiguous when the shape is supported but the current invocation lacks sufficient structure to derive a unique argv. Shapes that cannot be represented are treated as unsupported.

Serialization preserves all relative ordering explicitly represented in invocation state. Where invocation state does not encode relative order among independent elements, serialization applies a fixed canonical order derived from parser structure and field identity, provided that order is parse-safe; otherwise serialization reports ambiguity.

Preservation refers to maintaining parse-correct token structure and boundaries required by clap, not preserving original shell spelling.
