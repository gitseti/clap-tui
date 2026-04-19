## Why

The current form workspace still mixes several visual grammars: text inputs use textarea-style controls, while toggles, counters, and choice pickers render as compact one-line widgets, section groups still draw a boxed frame, metadata badges compete with labels for horizontal space, and default-derived values do not consistently read as secondary state. These gaps make the TUI feel less deliberate than the current behavior model deserves, and a few clipping and focus details now visibly break the intended polish.

## What Changes

- Refine the form control family so flags, counters, dropdown-backed options, optional-value flags, and similar fields adopt the same textarea-like visual treatment as text inputs while preserving their current interaction semantics.
- Replace boxed section framing with lightweight section labels followed by a horizontal rule, and remove the bottom section cap so sections read as grouped rows rather than nested panels.
- Adjust section layout and clipping behavior so a section heading does not reappear at the top of the viewport after it has scrolled offscreen while rows from that section remain visible.
- Move compact metadata badges such as `Default` beneath the field label so long option names and status markers no longer fight for the same horizontal line.
- Render default-derived values in a muted treatment across relevant controls, while preserving primary input color for user-entered values even after focus moves away.
- Improve the search field focus treatment so clicking into the empty `Search commands` field removes the placeholder copy and shows a visible cursor position for immediate typing.

## Capabilities

### New Capabilities
- `form-control-visual-grammar`: Defines the shared textarea-like control family, lightweight section framing, and stacked label-plus-metadata layout for dense command forms.

### Modified Capabilities
- `visual-state-semantics`: Refine how default-derived values, user-entered values, labels, and metadata badges are visually prioritized.
- `interaction-feedback-clarity`: Clarify how focused search, textarea-like non-text widgets, and preserved widget semantics are communicated to the user.
- `adaptive-terminal-layout`: Tighten clipped section-heading behavior so scrolling long sections preserves a stable hierarchy without header reentry artifacts.

## Impact

- Affected code will center on `crates/clap-tui/src/query/form.rs`, `crates/clap-tui/src/frame_snapshot.rs`, `crates/clap-tui/src/ui/form.rs`, `crates/clap-tui/src/ui/sidebar.rs`, `crates/clap-tui/src/controller/navigation.rs`, `crates/clap-tui/src/controller/keyboard.rs`, and `crates/clap-tui/src/update/form.rs`.
- Existing renderer and interaction tests around section framing, default-value rendering, clipped inputs, and search focus will need updates and new coverage.
- No public Rust API changes are intended; this is a TUI behavior and presentation refinement within the existing interaction model.
