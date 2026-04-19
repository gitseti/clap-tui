## Why

Required `clap::ArgGroup` failures currently fall through the validation adaptation path without producing actionable `ValidationState` feedback. That leaves commands invalid while existing inline error, footer summary, and focus-navigation UI paths often have nothing specific to render or target.

## What Changes

- Update validation adaptation so missing required groups reported by clap populate `ValidationState.summary` and field-linked validation feedback instead of being treated as generic invalid state.
- Define the required-group UX contract for inline validation, summary wording, and focus behavior so adapter and UI changes target one consistent outcome.
- Add regression coverage for required-group validation at both the adapter and rendered UI layers.
- Keep pre-submit grouped-required affordances out of the minimal fix, but document model/spec metadata as a follow-up enhancement area.

## Capabilities

### New Capabilities

### Modified Capabilities

- `interaction-feedback-clarity`: Validation feedback for required groups must remain actionable in summaries, inline errors, and correction navigation.
- `clap-metadata-fidelity`: Validation adaptation must preserve clap-required group semantics when translating parser failures into TUI-visible validation state.

## Impact

- Affected code: `crates/clap-tui/src/pipeline/validation.rs`, form rendering, footer summary presentation, and invalid-field navigation helpers.
- Affected tests: pipeline validation tests plus rendered/form interaction tests covering required group failures.
- No public API changes are expected; the change is internal to validation adaptation and TUI behavior.
