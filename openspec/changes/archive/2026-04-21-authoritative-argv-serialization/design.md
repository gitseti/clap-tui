## Context

`clap-tui` stores invocation data as structured state, while `clap` parses concrete argv tokens. A single argv path is necessary but not sufficient: the token sequence must also have deterministic spelling, safe shell rendering, parse-sensitive preservation, and a principled answer for states that cannot be serialized uniquely.

The authoritative object is the argv token sequence. Preview and copy are renderings of that sequence. Validation and execution must always use the token sequence directly, never a rendered string.

Serialization is derived from invocation state. The serializer must not reconstruct distinctions that are not represented in state, even if `clap` accepts multiple equivalent argv shapes.

---

## Goals / Non-Goals

**Goals:**
- Produce one canonical `Vec<OsString>` argv for validation, run, preview rendering, and copy rendering.
- Preserve `clap` parse semantics for every supported parser shape only when invocation state uniquely determines a parse-safe argv.
- Apply deterministic canonical spelling where spelling does not affect parsing.
- Detect serialization ambiguity instead of silently reordering or rewriting command state.
- Return provenance that maps value and structural tokens back to UI state and diagnostics, including delimiter joins, terminators, raw `--`, and subcommand or external-subcommand boundaries.
- Keep effective values and value-source display separate from serialization.

**Non-Goals:**
- Reconstruct the exact shell typing order a human originally used.
- Treat rendered preview/copy strings as execution inputs.
- Materialize tokens for values `clap` can derive internally, such as defaults, conditional defaults, default-missing values, or env fallbacks.
- Fully support parser shapes whose semantics exceed the TUI state model; such shapes may instead be classified as fundamentally unsupported or may yield runtime ambiguity for specific states.

---

## Decisions

### Use canonical argv as the authoritative object

Serialization returns canonical argv tokens plus provenance. Validation, run, preview, and copy all consume that same serialized result. Preview and copy render the tokens for a target shell; they do not mutate, reinterpret, or reserialize state.

**Alternatives considered:**
- Keep preview as an independent command string. Rejected because rendering is not the correctness boundary.
- Execute rendered shell text. Rejected because shell quoting is lossy and platform-specific.

---

### Keep rendering separate from serialization

Rendering applies shell-correct quoting for a target shell. POSIX rendering uses POSIX shell quoting. Windows rendering requires an explicit target shell policy before it can be treated as canonical UI behavior.

**Alternatives considered:**
- Store argv as already-rendered strings. Rejected because validation/run need tokens, not shell text.
- Use one cross-platform display string. Rejected because shell quoting rules differ by target shell.

---

### Distinguish parse-sensitive rules from canonical spelling

Parse-sensitive rules affect correctness and must preserve `clap` behavior: attachment, delimiters, terminators, raw `--`, ownership/ordering, subcommand and external-subcommand boundaries, hyphen-leading token safety, and explicit empty values.

Explicit empty authored values, such as `--opt=`, must be preserved and must not collapse into omission or default-derived behavior.

Canonical spelling rules make output deterministic without changing parsing:
- prefer primary long names
- use short names only when no long exists
- never emit aliases or hidden aliases
- never emit short clusters
- never attach short values unless required

Non-value repeated actions, such as counts or repeated booleans, serialize as repeated canonical flag occurrences rather than clustered shorthand.

Parse-sensitive rules operate within the structure defined by invocation state and must not introduce or remove structural distinctions not present in that state.

**Alternatives considered:**
- Normalize everything into one generic detached spelling. Rejected because parser shape can affect semantics.
- Preserve whichever spelling the user last interacted with. Rejected because the TUI owns structured state, not shell authorship history.

---

### Report ambiguity instead of inventing missing structure

Some invocation states do not contain enough information to produce one unique `clap`-correct argv.

Ambiguity occurs only when the current invocation state does not provide enough structure to produce a unique parse-safe argv without relying on implicit or ambiguous parser behavior.

**Representative cases include:**
- variable-length arguments whose ownership cannot be determined relative to later elements
- hyphen-leading tokens that cannot be safely emitted as values
- positional or trailing regions where boundaries are not represented in state

When no unique argv exists, serialization returns an ambiguity error. Validation, run, preview, and copy are blocked until the state becomes serializable.

**Alternatives considered:**
- Reorder tokens until `clap` accepts them. Rejected because that hides data loss and can change user intent.
- Pick any valid parse. Rejected because canonical argv must be deterministic and explainable.

---

### Occurrence handling follows invocation state

Occurrence structure is preserved only when it is represented in invocation state.

When invocation state models a multi-value field as one flattened logical occurrence, serialization emits exactly one canonical occurrence, using delimiter form only when supported by the parser definition.

Occurrence-aware state is only required when:
- the UI explicitly represents multiple occurrences, or
- correct serialization or diagnostics require distinctions that the current state does not represent

The serializer must:
- not invent additional occurrences
- not merge occurrences that are explicitly represented in state

Delimiter behavior applies within a represented occurrence and must not be used to collapse or synthesize occurrence structure.

**Alternatives considered:**
- Treat all repeatable arguments as permanently flattened. Rejected because some parser shapes and UI models require occurrence distinctions that flat values alone cannot express.
- Force delimiter joins for all repeatable values. Rejected because delimiter availability and occurrence grouping are parser-defined, not universal.

---

### Keep effective values outside serialization

Effective values are derived by parsing canonical argv with `clap` and using `value_source()`.

The UI may display defaults, env values, and conditional defaults, but those values do not change the canonical argv unless the user explicitly emits tokens.

**Alternatives considered:**
- Serialize effective values. Rejected because it conflates semantic derivation with user-authored argv.
- Hide effective values entirely. Rejected because source metadata remains useful for explaining the form.

---

## Risks / Trade-offs

- Ambiguity errors may appear for states that previously produced some argv → Prefer explicit errors over silently incorrect ownership.
- Occurrence-aware state expands the data model → Introduce it only where grouping affects correctness or diagnostics.
- Shell rendering adds platform complexity → Start with POSIX rendering and require an explicit Windows target-shell policy.
- Provenance adds serializer complexity → Accept this because diagnostics need to explain serialization failures and validation errors precisely.

---

## Migration Plan

1. Update serializer return types to carry canonical `Vec<OsString>`, provenance, and serialization diagnostics.
2. Separate shell rendering from serialization and route preview/copy through the renderer.
3. Enforce canonical spelling and parse-sensitive token-shape rules.
4. Add ambiguity and unsupported-shape detection before validation/run/copy.
5. Compute derived validation state, preview, copy, execution, and effective-value parsing only from a successful serialization result.
6. Report serialization failure distinctly from `clap` validation failure.
7. Keep effective-value reporting based on `clap` parsing of canonical argv.
8. Add regression coverage for rendering, ambiguity, provenance, and shape-sensitive cases.

---

## Open Questions

- Should occurrence-aware editing be introduced immediately for affected args or phased behind ambiguity errors?
- Which shell is the default rendering target on Windows?
- Which `clap` parser shapes are fundamentally unsupported by the current TUI model?
