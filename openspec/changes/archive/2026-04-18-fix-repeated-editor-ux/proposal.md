## Why

Repeated-value editors in the TUI currently have a few interaction mismatches that make multi-line editing feel unreliable: middle rows draw their remove control inside the textarea footprint, arrow-key navigation gets trapped at the first or last repeated row, and partially clipped repeated editors can collapse into a merged-looking rectangle. These issues make repeated argument editing harder precisely in the longer forms where users most need the UI to stay predictable.

## What Changes

- Adjust repeated-row control layout so non-terminal rows reserve a right-side control gutter and render the lone remove button outside the textarea, centered within that gutter.
- Let `Up` and `Down` escape from the first and last repeated rows to the previous or next visible form field instead of trapping focus inside the repeated editor.
- Preserve repeated-row rendering and control hit targets when the field is partially clipped by the form viewport so rows do not visually merge into a single rectangle.
- Add focused tests for repeated-row layout, boundary navigation, and clipped repeated-editor rendering.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `clap-occurrence-editing`: repeated occurrence editors need consistent row controls and boundary navigation that moves between repeated rows when possible and falls through to adjacent form fields at the editor edges.
- `interaction-feedback-clarity`: repeated-value inputs need to keep distinct row boundaries, visible external controls, and correct interaction affordances even when the field is partially offscreen.

## Impact

- Affected code: `crates/clap-tui/src/ui/form.rs`, `crates/clap-tui/src/update/form.rs`, `crates/clap-tui/src/controller/keyboard.rs`, `crates/clap-tui/src/controller/navigation.rs`, and `crates/clap-tui/src/form_editor.rs`
- Affected tests: repeated-editor render tests, form interaction tests, and keyboard navigation coverage
- No API or dependency changes
