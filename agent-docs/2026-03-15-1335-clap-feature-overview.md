# clap-tui clap feature support overview

## Context

This overview is based on the current source layout, mainly:

- `crates/clap-tui/src/spec.rs`
- `crates/clap-tui/src/input.rs`
- `crates/clap-tui/src/view/argv.rs`
- `crates/clap-tui/src/ui/form.rs`

Important constraint: the current worktree does not compile on March 15, 2026 because the UI/controller code is mid-refactor. The notes below are therefore based on source inspection, not on a runnable TUI session.

## What the current model supports

Today the extracted `CommandSpec` / `ArgSpec` model is intentionally small:

- command name, about, rendered help, nested subcommands
- one display name per arg (`--long` or `-s`, otherwise positional id)
- one `required` bit
- one default value
- one list of `possible_values`
- positional index
- one `is_multi` bit
- one `value_hint`
- coarse arg kind: `Flag`, `Option`, `Positional`, `Enum`

That is enough for:

- basic bool flags (`SetTrue` / `SetFalse`)
- single-value options and positionals
- single-choice enums
- plain text multi-value entry by newline splitting
- nested subcommand selection
- default-value prefill / omission when untouched

## Main gaps

### 1. Generalized repeated and multi-value args

This is the biggest feature gap.

Missing support:

- `ArgAction::Append`
- multi-value enums
- value counts / ranges from `get_num_args()`
- repeated occurrences vs multiple values in one occurrence
- `ArgAction::Count` (`-vvv`)
- multiple default values

Why it matters:

- the current `ArgValue` enum can only hold `Bool`, one `Text`, or one enum index
- `is_multi` is only a boolean derived from `max_values() > 1`
- `build_argv` assumes repeated `--opt value` pairs for multi options and newline splitting for text fields

What likely needs to change:

- replace `ArgKind` as the primary model axis
- model arg behavior separately:
  - position: positional vs option
  - action: set / append / count / bool
  - cardinality: min / max / unbounded
  - input style: free text / single choice / multi choice
- replace `ArgValue` with a collection-based representation, for example `Vec<String>` plus action metadata

Priority: `P0`

### 2. Required-state UX and parser-backed validation

Required support is incomplete rather than absent.

Current state:

- the spec already stores `required`
- the form renderer already tries to append `*` to required labels
- there is a helper to compute missing required args in `view/argv.rs`

Still missing:

- inline missing-state highlighting
- a summary of unmet requirements before run
- conditional requirements
- parser-backed validation errors while editing

Bigger clap features that fall into this bucket:

- `requires*`
- `required_unless*`
- `required_if_eq*`
- `conflicts_with*`
- `exclusive`
- `overrides_with*`
- `ArgGroup` one-of / at-least-one rules

Recommendation:

- do not try to manually reimplement all clap validation rules in custom state logic
- instead, rebuild argv and run `Command::try_get_matches_from` as the source of truth after edits and before run
- use the resulting clap error to drive:
  - field error styling
  - footer / toast summaries
  - disabling or warning on run

Priority: `P0`

### 3. Global args shared across subcommands

Missing support:

- `Arg::global(true)`

Why it matters:

- current inputs are keyed by command path
- a global flag or option should remain active when the user moves between subcommands
- preview argv should merge inherited args with command-local args

What likely needs to change:

- distinguish global inputs from command-local inputs
- merge effective inputs when building argv and when running validation
- show inherited values in subcommand forms without duplicating storage

Priority: `P0`

### 4. Rich value syntax and positional parsing semantics

Missing or flattened support:

- `value_delimiter`
- `value_terminator`
- `require_equals`
- multiple `value_names`
- `allow_hyphen_values`
- `allow_negative_numbers`
- `trailing_var_arg`
- `last`

Why it matters:

- current serialization always emits space-separated tokens
- current multi-value editing only understands newline splitting
- some clap CLIs depend on exact token shape, not just the final value set

What likely needs to change:

- add syntax metadata to `ArgSpec`
- teach argv building about exact token emission rules
- surface syntax hints in the UI, especially for path-like and trailing args

Priority: `P1`

### 5. Flag hybrids and richer action types

Missing support:

- `ArgAction::Count`
- flag/option hybrids with `default_missing_value`
- any action where presence alone is meaningful but values may also be accepted

Why it matters:

- today every non-bool arg is treated like a text or enum value editor
- clap supports arguments whose behavior changes based on occurrence count or whether a value was attached

What likely needs to change:

- separate `action` from widget kind
- add dedicated widgets for:
  - counters
  - optional-value flags
  - repeated toggles / incrementers

Priority: `P1`

### 6. Command-level parsing rules

Missing support:

- `subcommand_required`
- `arg_required_else_help`
- `args_conflicts_with_subcommands`
- `subcommand_negates_reqs`
- external subcommands
- subcommand flags (`get_short_flag`, `get_long_flag`)

Why it matters:

- current sidebar treats subcommands as a simple navigable tree
- clap command selection can itself be constrained or aliased
- external subcommands need an escape hatch that the current tree model cannot represent

What likely needs to change:

- extend `CommandSpec` with command-level parser flags
- let the UI represent "no subcommand selected" as either valid or invalid depending on command settings
- add a raw external-subcommand entry path for commands that allow unknown subcommands

Priority: `P1`

### 7. Better display metadata and help fidelity

Missing support:

- both short and long names shown together
- visible aliases
- display order
- help headings
- long help
- env display
- subcommand long about

Why it matters:

- current labels choose a single display spelling and discard the rest
- complex CLIs become much harder to understand when aliases, headings, and long descriptions disappear

What likely needs to change:

- store identifiers as structured metadata instead of one rendered name
- preserve clap display order instead of sorting options alphabetically
- add grouped sections in the form, for example by help heading

Priority: `P2`

### 8. Environment and default-source awareness

Missing or reduced support:

- env-backed args
- hidden env / hidden default policies
- multiple default values
- distinction between default source and user-entered source

Why it matters:

- current state stores only one default string
- the UI cannot explain why a field is prefilled
- env-backed defaults are common in real CLIs

What likely needs to change:

- store value source metadata: user / default / env
- render source badges or placeholder text
- keep omission rules tied to source, not just "touched vs untouched"

Priority: `P2`

### 9. Value parser feedback beyond enumerated values

Missing support:

- inline feedback from custom value parsers
- richer possible-value descriptions
- parser-specific error text before final submit

Why it matters:

- `possible_values` covers only part of clap's validation story
- many CLIs use typed parsers without enumerated values

Recommendation:

- rely on clap validation for correctness
- only add custom UI affordances when the metadata is directly accessible

Priority: `P2`

## Recommended implementation order

### Phase 1: fix the data model first

Before adding features, reshape the extracted spec.

Suggested `ArgSpec` directions:

- identifiers:
  - `id`
  - `long`
  - `short`
  - visible aliases
- placement:
  - positional index
  - option vs positional
  - `last`
  - `trailing_var_arg`
  - `global`
- action:
  - set / append / count / bool-like
- cardinality:
  - min values
  - max values / unbounded
- value metadata:
  - possible values
  - value names
  - delimiter / terminator
  - require equals
  - value hint
- display metadata:
  - help
  - long help
  - help heading
  - display order
- defaults / sources:
  - default values
  - env var

Key design rule:

- model "where the arg appears" separately from "how values are entered"
- model "how many values / occurrences are legal" separately from "which widget we render"

### Phase 2: make argv building and validation authoritative

After the spec is richer:

- rebuild argv from the richer occurrence model
- run clap validation from the previewed argv during editing
- surface validation results in the form instead of duplicating clap rules manually

This phase unlocks:

- real required indicators
- conflicts / requires feedback
- parser error previews
- confidence that serialization matches actual clap semantics

### Phase 3: improve widgets only where the model now justifies it

Once the model and validation are correct, add targeted UI features:

- chips or list editors for repeated values
- multi-select dropdowns for repeated enums
- count steppers for `ArgAction::Count`
- inherited/global arg badges
- grouped sections by help heading
- source badges for env/default values

## Suggested first backlog

If you want a pragmatic order, I would implement these first:

1. Generalized multi-value / repeated arg model
2. Parser-backed validation and missing-required UX
3. Global args across subcommands
4. Rich token-shape support (`require_equals`, delimiter, trailing positionals)
5. Count flags and optional-value flags
6. Command-level subcommand rules

## Testing notes for later implementation

When these features are implemented, the tests should focus on argv shape and clap acceptance, not just local state.

Add golden-style cases for:

- repeated options vs repeated values
- multi-value enums
- count flags
- globals inherited into nested subcommands
- conflicts / requires / groups
- required subcommands
- `--opt=value` cases
- trailing var args and hyphen-prefixed values
- env/default omission rules

The safest pattern is:

1. build state
2. generate argv
3. validate it with clap
4. assert both argv shape and resulting `ArgMatches`
