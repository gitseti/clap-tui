## Why

`clap-tui` maintains structured invocation state, but `clap` ultimately parses an argv token stream. The current serializer must be tightened from “one argv path” into a canonical argv contract that preserves parse semantics, renders safely for humans, and reports ambiguity when state cannot be represented uniquely.

## What Changes

- Define one authoritative canonical argv token sequence as `Vec<OsString>`.
- Use that same token sequence for validation, run, preview rendering, and copy rendering.
- Treat preview and clipboard output as shell-specific renderings of canonical argv, never as separate command definitions.
- Preserve clap parse-sensitive token shape for attachment, delimiters, terminators, raw boundaries, ordering/ownership, subcommand and external-subcommand boundaries, hyphen-leading values, and explicit empty values.
- Add deterministic canonical spelling rules, including primary long names, no aliases, no hidden aliases, no short clusters, and no attached short values unless required.
- Add serialization ambiguity handling so unsupported invocation states surface a serialization error instead of being silently reordered or rewritten.
- Add provenance from argv tokens and structural tokens back to form fields, occurrences, positional slots, command regions, delimiter joins, terminators, raw `--`, and subcommand or external-subcommand boundaries.
- Keep effective-value reporting separate: clap derives defaults/env/conditional values from the canonical argv and the UI may display them without changing serialization.

## Capabilities

### New Capabilities
- `argv-serialization-boundary`: Defines canonical argv serialization, rendering, provenance, and ambiguity handling from structured invocation state.

### Modified Capabilities
- `clap-argv-fidelity`: Reframe argv fidelity around canonical argv tokens shared by preview, copy, validation, and run.
- `clap-metadata-fidelity`: Clarify that value-source metadata and effective values remain UI/diagnostic concerns and do not alter canonical serialization.

## Impact

- Affected specs: new `argv-serialization-boundary`, plus updates to `openspec/specs/clap-argv-fidelity/spec.md` and `openspec/specs/clap-metadata-fidelity/spec.md`.
- Likely affected code: `crates/clap-tui/src/argv_serializer.rs`, `crates/clap-tui/src/pipeline/argv.rs`, `crates/clap-tui/src/pipeline/mod.rs`, preview/copy renderers, validation adapters, and diagnostics/provenance plumbing.
- Tests need coverage for canonical spelling, shell rendering, delimiter/attachment/boundary behavior, ambiguity errors, explicit empty values, hyphen-leading value safety, provenance mapping, and continued effective-value reporting.
