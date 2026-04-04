## Why

The current TUI can show ancestor-owned options in the generated preview while hiding them from the active form panel, which makes the invocation feel inconsistent and leaves users unsure where those flags came from or how to edit them. This is especially confusing for inherited global options because the current "Inherited" treatment does not clearly explain ownership or the effect of editing from a descendant command.

## What Changes

- Show invocation-relevant inherited options in the active form panel when a descendant command is selected.
- Distinguish local options from ancestor-owned options using explicit ownership cues rather than a badge alone.
- Explain the effect of editing inherited options from a descendant command, including which command owns the setting and whether the edit changes shared owner-scoped state.
- Keep the selected command's local options visually primary while still making inherited invocation-affecting options inspectable and editable from the same workspace.

## Capabilities

### New Capabilities
- None.

### Modified Capabilities
- `interaction-feedback-clarity`: inherited options shown from ancestor commands need clearer visibility, ownership, and edit-scope feedback in the active form panel.

## Impact

- Affected specs: `openspec/specs/interaction-feedback-clarity/spec.md`
- Affected code: form argument querying, form layout/rendering, inherited field copy, and tests around descendant command forms and preview consistency
- No new external dependencies or API changes
