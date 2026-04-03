# clap feature remaining work

Extracted from `agent-docs/2026-03-15-2052-clap-feature-implementation-plan.md` after the foundation work landed.

Updated on March 29, 2026 after the integrated refactor-and-feature pass.

## Status

Completed from the original plan:

- Milestone 1: expand the extracted clap spec
- Milestone 2: replace single-value form storage with an invocation-oriented input-state model
- Milestone 3: make argv synthesis and clap validation authoritative
- follow-up validation adapter fix: footer/toast summaries now reflect the real clap validation failure instead of command help/about text
- remaining Milestone 4 UI work:
  - field validation errors are shown directly in the form
  - inherited/global args are clearer in the form
  - append-style option editing is no longer forced through one collapsed single-value path
- Milestone 5 foundations:
  - widget selection is metadata-driven
  - multi-select choice interaction is supported
  - count-style widget interaction is supported
  - optional-value flag presence is represented separately from explicit values
  - repeated-value fields now support row creation via `Enter` and render as explicit numbered rows when read-only
  - optional-value fields now preserve explicit values when re-activated and can be disabled from focused widget controls
- refactor follow-up tied to Milestones 5-7:
  - semantic and presentation traits are split
  - form geometry now builds through `FrameSnapshot`
  - source badges and display-order-aware field ordering are in place

Still open:

- remaining Milestone 4 polish around repeated/multi-value field UX
- remaining Milestone 5 widget depth
- Milestone 6
- remaining Milestone 7 help fidelity

## Remaining Milestone 4: Land the highest-value feature slices on top of the new model

Purpose: ship the biggest coverage gains first on top of the completed spec, input-state, argv, and clap-validation foundations.

Status: Mostly landed, with UX polish still open.

### Remaining work

- Smooth out repeated-value editing so it behaves like an explicit list editor rather than a text-area-first compatibility layer.
- Improve multi-value positional editing ergonomics.
- Improve multiple-default rendering in the form and preview surfaces.

### Notes

- Parser-backed validation is already in place.
- Footer and blocked-run validation summaries are already in place.
- Global args are already modeled in the invocation state; the remaining work is the end-to-end feature polish and form/UI clarity.
- Field errors are now rendered directly in the form.
- Inherited/global state is now surfaced with badges in the form.

### Exit criteria

- `Append`, multi-value enums, multiple defaults, real required/conflict feedback, and `global(true)` work end to end.

## Milestone 5: Extend widgets for richer actions

Purpose: stop forcing richer semantics through single-line text fields.

Status: Partially landed.

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

Implementation notes:

- widget mode selection is now derived from semantic metadata through `FieldWidget`
- multi-select choice interaction works through the dropdown seam
- count-style fields now have focused keyboard interactions
- optional-value flag presence is represented, but optional-value editing still leans on text compatibility paths
- repeated text values now have a first-pass row/list UX, but still need fuller per-row editing/reorder affordances to fully satisfy this milestone

## Milestone 6: Add syntax fidelity and command-level edge cases

Purpose: handle CLIs where exact token shape matters.

Status: Partially landed.

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

Implementation notes:

- `require_equals` serialization is supported
- delimiter-aware occurrence serialization/input splitting is partially supported
- the rest of this milestone remains open, especially command-level edge cases and stricter trailing parsing behavior

### Exit criteria

- `clap-tui` can represent CLIs whose parsing depends on token shape, trailing capture, or stricter command-level subcommand rules.

## Milestone 7: Improve help and value-source fidelity

Purpose: make the richer semantics understandable in the TUI.

Status: Partially landed.

### Work

- Show both short and long spellings where appropriate.
- Respect clap display order instead of alphabetizing by convenience.
- Group fields by help heading.
- Surface long help where it adds value.
- Show env/default source badges or placeholder metadata.
- Preserve alias and env information in help/preview surfaces where it affects user understanding.

Implementation notes:

- form ordering now respects clap display order
- env/default source badges are shown in the form
- help-heading grouping, richer long-help presentation, and broader alias/env visibility are still open

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

The next logical slice is the remaining Milestone 5 and 6 polish:

1. replace repeated-value compatibility text editing with an explicit row/list editor
2. finish syntax-fidelity coverage for delimiters, terminators, trailing var args, and command-level edge cases
3. deepen repeated-value editing with per-row reorder/remove affordances instead of relying on text-editor compatibility for the last mile
