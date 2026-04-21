## 1. Serializer Policy

- [x] 1.1 Audit delimiter-backed multi-value serialization for flattened state versus explicitly represented occurrences.
- [x] 1.2 Ensure flattened delimiter-backed fields emit one canonical occurrence and do not invent repeated occurrences.
- [x] 1.3 Ensure explicitly represented delimiter-backed occurrences are serialized independently and are not merged.
- [x] 1.4 Update occurrence ambiguity diagnostics so parser shape alone does not produce ambiguity without missing state distinctions.
- [x] 1.5 Classify fundamentally unsupported parser shapes separately from ambiguous current states.
- [x] 1.6 Enforce that emitted argv is uniquely justified by invocation state and parser definition, not merely accepted by clap.
- [x] 1.7 Define and apply canonical ordering for independent fields, parser boundaries, positionals, repeated occurrences, raw regions, and external-subcommand tails.

## 2. Parse-Sensitive Integration

- [x] 2.1 Verify attachment, delimiter joining, terminators, raw boundaries, ownership boundaries, and subcommand boundaries operate inside represented invocation state structure.
- [x] 2.2 Preserve the single authoritative `Vec<OsString>` result for validation, run, preview rendering, and copy rendering.
- [x] 2.3 Confirm default, env, conditional default, and default-missing effective values remain outside canonical argv unless user-authored.
- [x] 2.4 Preserve authored empty values without rewriting them to omission, defaulting, or a semantically different token form.
- [x] 2.5 Gate validation, effective-value parsing, preview, copy, and execution on successful serialization.
- [x] 2.6 Preserve carried-through token regions verbatim where such regions exist, or report ambiguity/unsupported shape when they cannot be represented.
- [x] 2.7 Expand provenance to distinguish structural tokens, authored value tokens, delimiter-joined synthesized tokens, preserved token regions, and canonical spelling substitutions.

## 3. Regression Coverage

- [x] 3.1 Add serializer tests for flattened delimiter-backed multi-value state emitting one joined occurrence.
- [x] 3.2 Add serializer tests for explicitly represented delimiter-backed repeated occurrences remaining separate.
- [x] 3.3 Add tests proving delimiter ownership fixes do not merge represented occurrences.
- [x] 3.4 Add ambiguity tests where occurrence boundaries are required but absent from invocation state.
- [x] 3.5 Add non-ambiguity tests where flattened state has a clear single-occurrence meaning despite parser support for repeated occurrences.
- [x] 3.6 Add tests for unsupported-shape diagnostics distinct from state-specific ambiguity diagnostics.
- [x] 3.7 Add ordering tests for independent options, positionals, subcommands, raw boundaries, terminators, repeated occurrences, and external-subcommand tails.
- [x] 3.8 Add hyphen-leading ownership tests for detached option values, variadic values, trailing regions, subcommand-adjacent values, and external-subcommand payloads.
- [x] 3.9 Add explicit empty-value tests for authored empty values versus omission and derived defaults.
- [x] 3.10 Add provenance tests for structural, authored, synthesized delimiter-joined, preserved-region, and canonical-spelling token origins.
- [x] 3.11 Run focused serializer/pipeline tests and the full relevant crate test command.
