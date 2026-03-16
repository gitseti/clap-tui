# clap feature implementation plan

Based on `agent-docs/2026-03-15-1335-clap-feature-overview.md`.

## Current baseline

As of March 15, 2026, the crate is back to a green baseline:

- `cargo test -p clap-tui` passes
- the event loop, controller split, and `FrameSnapshot` work are already in place
- the event loop architecture is serviceable, but the current domain model is still too command-local and lossy for the target clap coverage
- the main gaps are therefore in clap modeling, invocation-state modeling, argv synthesis, validation, and form widgets

The main implementation seams for this work are:

- `crates/clap-tui/src/spec.rs`
  - currently flattens clap args into a small `ArgSpec`
- `crates/clap-tui/src/input.rs`
  - still stores one `ArgValue` per arg, which cannot represent append/count/global/source-aware behavior
- `crates/clap-tui/src/argv_serializer.rs`
  - currently treats multi-value input as newline splitting and repeats `--opt value` pairs
- `crates/clap-tui/src/pipeline/argv.rs`
  - currently builds preview argv from the selected command's local form state
- `crates/clap-tui/src/ui/form.rs`
  - renders a small set of widgets derived from `InputPresentation`
- `crates/clap-tui/src/app.rs`
  - only asks clap to validate at final run time

## Goal

Expand `clap-tui` from a small subset of clap semantics to a model that can drive real-world CLIs without duplicating clap’s parser logic in the UI.

The implementation order should be:

1. enrich the extracted spec
2. replace the single-value form state with a full invocation-state model
3. make argv building and clap validation authoritative
4. add UI affordances that are justified by the richer model

## Non-goals

- no event-loop rewrite
- no visual redesign while the feature model is changing
- no custom reimplementation of clap validation rules when `try_get_matches_from` can be the source of truth
- no attempt to land every clap feature in one patch

## Guiding rules

- keep milestones shippable
- preserve current behavior for already-supported flags, options, positionals, and enums
- prefer adapter layers over flag-day rewrites
- test argv shape and clap acceptance together for every new feature slice
- treat Milestones 1-3 as architectural preconditions for feature completeness, not as optional cleanup

## Architectural precondition

The current event loop, controller/update split, and rendering phases are good enough
to keep. The blocking architectural issue is deeper in the domain boundary:

- `CommandSpec` currently drops too much clap meaning
- form state is keyed per selected command and cannot model a full invocation across
  root command plus selected subcommand path
- argv preview and run paths are built from the selected command's local state rather
  than from the complete invocation state

That means the plan must explicitly reshape the core model before landing feature work.
Milestones 1-3 are the architecture change that makes Milestones 4-7 realistic.

## Milestone 1: Expand the extracted clap spec

Purpose: stop treating `ArgKind` plus `is_multi` as the primary truth and make the
spec rich enough to drive an invocation-oriented state model.

### Work

- Reshape `ArgSpec` in `crates/clap-tui/src/spec.rs` to carry normalized metadata for:
  - identifiers: `id`, `short`, `long`, visible aliases, display label
  - placement: option vs positional, positional index, `global`, `last`, `trailing_var_arg`
  - action: set, append, count, bool-like, optional-value variants
  - cardinality: min values, max values, unbounded
  - syntax: `require_equals`, `value_delimiter`, `value_terminator`, `allow_hyphen_values`, `allow_negative_numbers`
  - value metadata: possible values, value names, value hint
  - defaults and environment: default values, env var, hidden-default/env policy
  - display metadata: help, long help, help heading, display order
- Extend `CommandSpec` with command-level parser rules that affect validity or selection:
  - `subcommand_required`
  - `arg_required_else_help`
  - `args_conflicts_with_subcommands`
  - `subcommand_negates_reqs`
  - external-subcommand support
- Keep enough command-path and inheritance metadata to answer:
  - which args are defined on each command in the selected path
  - which args are inherited by descendants via `global(true)`
  - which command owns a value for storage and serialization purposes
- Keep compatibility helpers for the current UI so the crate still renders before later milestones land.

### Likely files

- `crates/clap-tui/src/spec.rs`
- `crates/clap-tui/src/query/form.rs`
- `crates/clap-tui/src/ui/form.rs`

### Exit criteria

- `ArgSpec` can represent `Append`, `Count`, multi-value enums, globals, multiple defaults, and syntax-affecting metadata without lossy flattening.
- Existing supported flows still work through compatibility helpers.

## Milestone 2: Replace single-value form storage with an input-state model

Purpose: make the in-memory form state capable of representing clap semantics across
the full selected command path before changing widgets.

### Work

- Replace `ArgValue` in `crates/clap-tui/src/input.rs` with an arg input model that can represent:
  - boolean presence
  - count occurrences
  - zero-or-more string values
  - repeated occurrences vs multiple values in one occurrence
  - selection state for one-or-many choices
  - source metadata: user, default, env
- Introduce an invocation-state model that spans the selected command path from root to
  leaf rather than only the currently selected command.
- Split command-local values from inherited global values.
- Store values by owning command, then derive an effective state for the selected command.
- Keep `touched` semantics, but tie omission logic to source plus edit intent rather than to a single touched bit alone.
- Add mutation helpers on `DomainState` for:
  - set/replace values
  - append/remove/reorder values
  - increment/decrement counters
  - toggle optional-value flags
  - read effective values for the selected command
  - resolve the full invocation state used by preview, validation, and run

### Likely files

- `crates/clap-tui/src/input.rs`
- `crates/clap-tui/src/form_editor.rs`
- `crates/clap-tui/src/query/form.rs`

### Exit criteria

- Form state can represent every `P0` feature from the overview without relying on newline-encoded text blobs.
- The selected command can resolve an effective form state that includes inherited globals.
- Preview/run logic no longer depends on `current_form()` from the selected command alone.

## Milestone 3: Make argv synthesis and validation authoritative

Purpose: centralize correctness at the clap boundary instead of in ad hoc UI checks.

### Work

- Rewrite `crates/clap-tui/src/argv_serializer.rs` around the richer spec and the
  full invocation-state model.
- Preserve distinctions that matter to clap:
  - repeated occurrences
  - grouped values in one occurrence
  - `--opt=value`
  - delimiter-driven expansion
  - positional ordering
  - trailing positional behavior
- Build argv from root command through the selected command path, merging inherited
  globals and command-local values in ownership order.
- Add a validator module that rebuilds argv and runs `Command::try_get_matches_from`.
- Refresh validation after edits that can affect parser state and always before `Run`.
- Replace local missing-required checks with clap-backed validation results.
- Store validation output in a small UI-facing model:
  - overall valid/invalid state
  - field-linked errors when they can be inferred
  - summary text for footer or preview panes

### Likely files

- `crates/clap-tui/src/argv_serializer.rs`
- `crates/clap-tui/src/pipeline/argv.rs`
- `crates/clap-tui/src/pipeline/validation.rs`
- `crates/clap-tui/src/app.rs`
- new validation module under `crates/clap-tui/src/`

### Exit criteria

- Preview argv matches the real run path.
- Required, conflicts, requires, groups, and parser failures come from clap validation, not from hand-maintained UI rules.
- Validation runs against the same full invocation argv that `Run` uses.

## Milestone 4: Land the highest-value feature slices on top of the new model

Purpose: ship the biggest coverage gains first once the foundations are ready.

### Work

- Implement `P0` support from the overview in this order:
  1. generalized repeated and multi-value args
  2. parser-backed required/conflict validation
  3. global args across subcommands
- For repeated values:
  - support append-style options and positionals
  - support multi-value enums
  - support multiple default values
- For validation:
  - surface field error state in the form
  - show a summary before run when clap rejects the preview argv
- For globals:
  - merge inherited values into preview argv
  - show inherited/global state clearly in subcommand forms without duplicating storage

### Likely files

- `crates/clap-tui/src/input.rs`
- `crates/clap-tui/src/argv_serializer.rs`
- `crates/clap-tui/src/pipeline/argv.rs`
- `crates/clap-tui/src/ui/form.rs`
- `crates/clap-tui/src/app.rs`

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

## Testing strategy

Add tests in the same order as the milestones.

### Spec extraction tests

- synthetic clap commands covering `Append`, `Count`, globals, multi-defaults, delimiters, `require_equals`, trailing positionals, and command-level flags
- assertions on extracted `ArgSpec` / `CommandSpec`, not just on rendered labels

### State and serializer tests

- repeated options vs repeated values in one occurrence
- multi-value enums
- count flags
- global inheritance into nested subcommands
- env/default omission behavior
- `--opt=value` output and delimiter-driven output
- trailing var args and hyphen-prefixed values

### Validation tests

- preview argv plus `try_get_matches_from`
- conflicts, requires, required-unless, arg groups, required subcommands
- field-level error mapping where possible, and summary-only fallback where not

### UI/controller tests

- repeated-value editing
- multi-select interactions
- counter increment/decrement flows
- inherited/global arg visibility
- validation styling and disabled/warn-on-run behavior

## Risk management

- Highest correctness risk: re-encoding clap rules in local helpers. Avoid this by routing validation through clap early.
- Highest migration risk: changing `ArgValue` without a compatibility layer. Land adapters first, then remove them.
- Highest UX risk: adding richer widgets before the data model settles. Keep widget work after the serializer and validator are stable.

## Recommended first slice

Start with a narrow vertical slice for repeated values:

1. expand `ArgSpec` with explicit action and cardinality metadata
2. replace `ArgValue` with a collection-capable input model
3. update `argv_serializer.rs` to serialize append-style options and multi-value positionals correctly
4. add clap-backed tests that assert both argv shape and accepted `ArgMatches`

That slice unlocks the largest `P0` gap with the least UI churn and establishes the model needed for the rest of the plan.
