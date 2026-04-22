## Why

The current TUI can manufacture missing-required validation errors from static clap metadata after canonical argv has already been accepted by clap. That contradicts the crate's authoritative argv and clap-driven validation model, and it shows up on subcommand paths where ancestor-owned fields remain visible but no longer have the same effective semantics.

## What Changes

- Add a path-sensitive derived field-semantics layer for form presentation.
- Keep canonical argv serialization and clap validation as the only authorities for command validity.
- Stop converting declared `ArgModel.required` metadata into validation errors after clap validation succeeds.
- Drive required badges, required placeholders, label sizing, missing-required visual treatments, field activity, editability, and conflict presentation from derived field semantics instead of raw `ArgModel.required`.
- Distinguish field visibility, activity, conflict state, required presentation, and editability as separate semantic dimensions.
- Preserve user-authored ancestor input even when it conflicts with a selected subcommand; let clap report the actual conflict instead of silently dropping state.

## Capabilities

### New Capabilities
- `path-sensitive-field-semantics`: Defines how visible form fields derive current-path UI semantics such as required presentation, activity, editability, and conflict state from selected command path, ownership, parser rules, and validation projection.

### Modified Capabilities
- `derived-state-lifecycle`: Derived state must include and cache the path-sensitive field semantics used by rendering, focus, layout, and navigation.
- `clap-metadata-fidelity`: Validation feedback must remain sourced from serialization diagnostics or clap validation projection; raw clap metadata may guide presentation but must not create validation failures after clap success.
- `interaction-feedback-clarity`: Required indicators, placeholders, missing styling, and inherited-field affordances must follow effective field semantics for the selected command path.

## Impact

- Affected code: `crates/clap-tui/src/pipeline`, `crates/clap-tui/src/input.rs`, `crates/clap-tui/src/query/form.rs`, `crates/clap-tui/src/query/selectors.rs`, `crates/clap-tui/src/ui/form.rs`, `crates/clap-tui/src/ui/screen.rs`, `crates/clap-tui/src/frame_snapshot.rs`, and validation/navigation tests.
- No public API break is intended.
- No new dependency is expected.
- The primary behavioral change is that clap-accepted canonical argv cannot be marked invalid by static required metadata.
