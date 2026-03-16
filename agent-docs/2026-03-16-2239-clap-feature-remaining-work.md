# clap feature remaining work

Extracted from `agent-docs/2026-03-15-2052-clap-feature-implementation-plan.md` after the foundation work landed.

## Status

Completed from the original plan:

- Milestone 1: expand the extracted clap spec
- Milestone 2: replace single-value form storage with an invocation-oriented input-state model
- Milestone 3: make argv synthesis and clap validation authoritative
- follow-up validation adapter fix: footer/toast summaries now reflect the real clap validation failure instead of command help/about text

Still open:

- remaining Milestone 4 work
- Milestone 5
- Milestone 6
- Milestone 7

## Remaining Milestone 4: Land the highest-value feature slices on top of the new model

Purpose: ship the biggest coverage gains first on top of the completed spec, input-state, argv, and clap-validation foundations.

### Remaining work

- Complete `P0` end-to-end support for generalized repeated and multi-value args.
- Finish support for append-style options and positionals in the editing experience.
- Finish support for multi-value enums in the editing experience.
- Finish support for multiple default values in the editing experience and preview surfaces.
- Surface field error state directly in the form instead of relying only on footer/toast summaries.
- Make inherited/global state clearer in subcommand forms without duplicating storage.

### Notes

- Parser-backed validation is already in place.
- Footer and blocked-run validation summaries are already in place.
- Global args are already modeled in the invocation state; the remaining work is the end-to-end feature polish and form/UI clarity.

### Exit criteria

- `Append`, multi-value enums, multiple defaults, real required/conflict feedback, and `global(true)` work end to end.

## Milestone 5: Extend widgets for richer actions

Purpose: stop forcing richer semantics through single-line text fields.

### Work

- Add widget modes derived from the new spec/input model:
  - repeated text/list editor
  - multi-select choice editor
  - counter control for `ArgAction::Count`
  - optional-value flag editor for default-missing-value cases
- Keep widget selection driven by metadata, not by hardcoded `ArgKind`.
- Preserve keyboard-first flows and make mouse support additive.
- Avoid a broad UI rewrite; change only the widgets needed for newly supported semantics.

### Likely files

- `crates/clap-tui/src/ui/form.rs`
- `crates/clap-tui/src/form_editor.rs`
- `crates/clap-tui/src/controller/keyboard.rs`
- `crates/clap-tui/src/controller/mouse.rs`
- `crates/clap-tui/src/query/form.rs`

### Exit criteria

- Users can edit repeated, counted, and optional-value args without encoding values into newline conventions.

## Milestone 6: Add syntax fidelity and command-level edge cases

Purpose: handle CLIs where exact token shape matters.

### Work

- Implement serializer and UI support for:
  - `require_equals`
  - `value_delimiter`
  - `value_terminator`
  - `allow_hyphen_values`
  - `allow_negative_numbers`
  - `trailing_var_arg`
  - `last`
- Add command-level handling for:
  - required vs optional subcommand selection
  - argument/subcommand conflict rules
  - external subcommands
  - subcommand flags and aliases where useful in the UI

### Exit criteria

- `clap-tui` can represent CLIs whose parsing depends on token shape, trailing capture, or stricter command-level subcommand rules.

## Milestone 7: Improve help and value-source fidelity

Purpose: make the richer semantics understandable in the TUI.

### Work

- Show both short and long spellings where appropriate.
- Respect clap display order instead of alphabetizing by convenience.
- Group fields by help heading.
- Surface long help where it adds value.
- Show env/default source badges or placeholder metadata.
- Preserve alias and env information in help/preview surfaces where it affects user understanding.

### Exit criteria

- Complex CLIs remain understandable once the richer parser semantics are supported.

## Remaining testing strategy

Add tests in the same order as the remaining milestones.

### State, serializer, and validation tests

- repeated options vs repeated values in one occurrence
- multi-value enums
- count flags
- global inheritance into nested subcommands
- env/default omission behavior
- `--opt=value` output and delimiter-driven output
- trailing var args and hyphen-prefixed values
- preview argv plus `try_get_matches_from`
- conflicts, requires, required-unless, arg groups, required subcommands
- field-level error rendering once it lands in the form

### UI/controller tests

- repeated-value editing
- multi-select interactions
- counter increment/decrement flows
- inherited/global arg visibility
- validation styling in the form

## Recommended next slice

The next logical slice is the remaining Milestone 4 UI work:

1. surface field validation errors in the form
2. land repeated and multi-value editing affordances
3. close the gap between inherited/global storage and how it is displayed in subcommand forms
