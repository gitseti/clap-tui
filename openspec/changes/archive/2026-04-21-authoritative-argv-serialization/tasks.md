## 1. Canonical argv foundation

- [x] 1.1 Change the serializer pipeline to produce canonical argv as `Vec<OsString>` plus a structured serialization result.
- [x] 1.2 Route validation, run, preview, and copy through the same canonical argv result.
- [x] 1.3 Remove any remaining code paths that treat rendered command text as an execution or validation input.

## 2. Rendering and canonical spelling

- [x] 2.1 Add shell rendering for preview and copy as a projection of canonical argv, starting with POSIX shell quoting.
- [x] 2.2 Define and document the Windows target-shell policy before treating Windows rendering as canonical UI behavior.
- [x] 2.3 Enforce canonical spelling: primary long names first, short only when no long exists, no aliases, no hidden aliases, no short clusters, and no attached short values unless parser shape requires it.
- [x] 2.4 Preserve explicit empty values in canonical argv and rendered output.

## 3. Parse-sensitive serialization

- [x] 3.1 Preserve required attachment, delimiter token shape, terminators, raw `--`, trailing capture, and subcommand boundaries.
- [x] 3.2 Respect hyphen-leading token safety using `allow_hyphen_values`, `allow_negative_numbers`, raw/trailing capture, and explicit `--` boundaries.
- [x] 3.3 Keep clap-derivable defaults, env values, conditional defaults, and default-missing values out of canonical argv unless explicitly emitted by the user.

## 4. Ambiguity and provenance

- [x] 4.1 Add serialization ambiguity diagnostics for occurrence grouping, ownership, hyphen-leading tokens, and positional/trailing shapes.
- [x] 4.2 Block validation, run, preview rendering, and copy rendering when serialization ambiguity is present.
- [x] 4.3 Add token provenance for value tokens, delimiter-joined tokens, terminators, raw `--`, and subcommand boundaries.
- [x] 4.4 Map serializer and validation diagnostics back to fields, occurrences where available, positional slots, and command/subcommand regions.
- [x] 4.5 Identify parser shapes that are fundamentally unsupported by the current TUI model and surface them distinctly from state-specific ambiguity.

## 5. Occurrence-aware state

- [x] 5.1 Audit arguments where flattened state loses grouping required for canonical serialization or diagnostics.
- [x] 5.2 Introduce occurrence-aware state for affected repeatable or variable-arity arguments, or surface clear ambiguity until occurrence-aware editing is implemented.
- [x] 5.3 Preserve delimiter and terminator semantics when editing occurrence-aware values.

## 6. Regression coverage

- [x] 6.1 Cover preview/copy rendering from the same canonical argv used for validation and run.
- [x] 6.2 Cover deterministic canonical spelling, including aliases, short flags, count flags, and short-cluster avoidance.
- [x] 6.3 Cover delimiter behavior, `require_equals`, raw `--`, terminators, subcommand boundaries, explicit empty values, and hyphen-leading tokens.
- [x] 6.4 Cover occurrence grouping ambiguity, ownership ambiguity, positional/trailing ambiguity, and serialization errors distinct from validation failures.
- [x] 6.5 Cover provenance mapping for value tokens, delimiter-joined tokens, structural tokens, fields, occurrences, positional slots, and command regions.
- [x] 6.6 Cover effective-value reporting from clap value sources without materializing derived values into canonical argv.
