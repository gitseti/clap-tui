## 1. Validation Boundary

- [x] 1.1 Add a regression test proving clap-accepted canonical argv cannot become invalid from static `ArgModel.required` metadata.
- [x] 1.2 Remove or constrain the post-clap-success missing-required pass in `pipeline::validation` so `try_get_matches_from()` success remains valid.
- [x] 1.3 Keep missing-required field errors for clap-reported missing arguments and required groups.
- [x] 1.4 Verify serialization diagnostics still produce validation summaries and field errors when canonical argv cannot be built.

## 2. Field Semantics Model

- [x] 2.1 Add a `pipeline::field_semantics` module with stable field instance identity, visibility, activity, conflict, required presentation, editability, owner path, and reason fields.
- [x] 2.2 Extend `DerivedState` to include field semantics derived for the selected command path.
- [x] 2.3 Derive effective required presentation for local and inherited fields, including selected paths with `subcommand_negates_reqs`.
- [x] 2.4 Derive potential and actual conflict state for ancestor args affected by `args_conflicts_with_subcommands`.
- [x] 2.5 Ensure field semantics lookup is keyed by projected field instance identity such as `(owner_path, arg_id)` rather than raw `arg_id` alone.
- [x] 2.6 Add unit tests for local required fields, inherited neutral fields, subcommand-negated required fields, hidden field exclusion, disabled state preservation, potential path conflicts, and actual validation conflicts.

## 3. UI Projection Migration

- [x] 3.1 Pass field semantics through `ScreenView` and form layout/population paths.
- [x] 3.2 Replace label required markers and label width calculations that use raw `ArgModel.required`.
- [x] 3.3 Replace required placeholder wording and empty-state styling that use raw `ArgModel.required`.
- [x] 3.4 Update form rendering to show inactive, disabled, inherited, potential-conflict, and actual-conflict states from field semantics.
- [x] 3.5 Update focus, hit-testing, and invalid-field navigation to consume shared field semantics where field visibility, actionability, or invalidity matters.
- [x] 3.6 Add input/update guards for non-editable fields if disabled semantics are implemented as non-interactive.

## 4. End-to-End Regression Coverage

- [x] 4.1 Add a render test for parent required arg plus selected child subcommand under `subcommand_negates_reqs(true)` showing no required badge, placeholder, or missing error.
- [x] 4.2 Add a validation/render test for an untouched ancestor arg under `args_conflicts_with_subcommands(true)` showing neutral or disabled presentation without conflict validation.
- [x] 4.3 Add a validation/render test for a user-authored ancestor arg under `args_conflicts_with_subcommands(true)` preserving argv and surfacing clap's conflict.
- [x] 4.4 Add regression tests proving required local child args and required groups still produce inline field errors and footer summaries.
- [x] 4.5 Run formatting, clippy, and the crate test suite for the affected workspace.
