## Context

`clap-tui` currently serializes each command form by emitting non-positional arguments first and buffering positionals until the end of the command segment. That canonicalization keeps the implementation simple, but it is not parse-stable for clap when an earlier option remains open to consuming subsequent tokens as additional values. In `kitchen-sink serve`, the UI can represent `document_root = gdsag` and `feature = [gzip, brotli]`, yet serialization produces an argv where `gdsag` is reparsed as another `--feature` value.

The current pipeline also uses two serialization modes: preview hides untouched default-derived values, while validation and run materialize them. That makes parse-affecting defaults such as `--host 127.0.0.1` show up only in the clap-checked argv, which breaks the stricter invariant we want: preview, validation, and run must agree on the parse-relevant token sequence.

This change is limited to clap-first correctness. We are not trying to preserve the user's original typing order; we are trying to preserve the semantic assignment expressed by the form state when clap reparses the emitted argv.

## Goals / Non-Goals

**Goals:**
- Emit argv that clap reparses into the same argument assignment represented in the TUI state.
- Preserve parse-correct ordering when a positional would otherwise be consumed by a variable-arity option.
- Keep preview, validation, and run aligned on one parse-relevant argv shape so hidden materialized defaults cannot change parse behavior.
- Add regression coverage for mixed option/positional shapes that are currently valid in the form but invalid after serialization.

**Non-Goals:**
- Reconstruct the exact textual order a user might have typed on a shell command line.
- Redesign form ownership, command lineage, or non-parse-related preview presentation.
- Change clap metadata extraction or introduce new public configuration for ordering policy.

## Decisions

### Preserve semantic assignment as the serialization invariant

The serializer will optimize for clap-equivalent semantics, not a blanket "options first" or "positionals first" rule. A serialized argv is correct only if clap reparses it into the same argument ownership and values the form state currently expresses.

Alternatives considered:
- Always serialize positionals first. Rejected because it is too blunt and could break commands whose parse semantics depend on later boundaries such as trailing, raw, or `last` positionals.
- Keep the current options-first canonicalization. Rejected because it already produces clap-invalid argv for valid form states.

### Split command-local serialization into parse-sensitive segments

Within each command segment, serialization must account for positional indices and for options whose clap configuration leaves them able to consume subsequent tokens as additional values. The implementation can still build command-local segments, but if a positional value represented in form state would otherwise be reparsed as belonging to an earlier-emitted open-ended value-taking option, the positional must be emitted before that option unless clap syntax for that command provides an explicit boundary that preserves the same semantics. Reordering is only valid within the command segment that owns the arguments; it must not move tokens across subcommand boundaries.

Alternatives considered:
- Post-validate and retry different orderings until clap accepts one. Rejected because it is opaque, harder to reason about, and risks unstable output.
- Materialize explicit boundaries everywhere. Rejected because many cases have no valid boundary token for ordinary options and positionals.

### Use one parse-relevant argv shape across preview, validation, and run

The preview may still style untouched defaults differently, but preview, validation, and run must agree on the parse-relevant token sequence. If serialization needs to materialize a default to preserve clap behavior, preview must not hide that difference in a way that changes the implied parse.

Alternatives considered:
- Keep separate preview and parse serialization modes. Rejected because parse-affecting defaults can remain invisible until execution or validation failure.

## Risks / Trade-offs

- [More complex serializer logic] -> Keep the policy narrow and driven by clap parse hazards, with regression tests covering representative combinations.
- [Subtle regressions in existing command shapes] -> Extend serializer tests across multi-value options, subcommands, trailing args, and default-materialized values before changing behavior broadly.
- [Preview becomes more verbose when defaults are parse-relevant] -> Limit the requirement to parse-affecting tokens and keep non-parse presentation concerns out of the serializer contract.
