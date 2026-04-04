## Why

`clap-tui` already has a strong terminal-native layout, but the UI review shows the library still feels like several adjacent mini-systems instead of one cohesive product. Too many states share the same accent treatment, similar interaction states are rendered differently across surfaces, and the sidebar, form, preview, footer, dropdown, and toast each express hierarchy with slightly different rules.

This is the right moment to tighten that visual language because the existing renderer split is already good enough to support a focused consistency pass without redesigning the product. A merged change should replace the narrower `clarify-tui-visual-semantics` proposal with one that covers semantic tokens, shared state grammar, consistent surface chrome, and a more unified control vocabulary across the whole TUI.

## What Changes

- Replace the superseded `clarify-tui-visual-semantics` change with a single proposal for a unified TUI visual language.
- Define stable semantic theme roles for focus, selection, success, error, warning-like metadata, passive copy, and layered surfaces so one accent family no longer carries unrelated meanings.
- Centralize renderer intent in shared style helpers so similar states are composed once and reused across sidebar, form, preview, footer, dropdown, and toast surfaces.
- Establish a consistent interaction-state grammar for focused, selected, hovered, open, inherited, default, required, and invalid states.
- Unify surface chrome and spacing rhythm so sidebar, workspace, preview, footer, overlays, and inline controls feel like one system while preserving the current layout.
- Refine form hierarchy and control affordances so labels, values, metadata badges, help text, toggles, choice pickers, counters, and optional-value states remain dense but more scannable.
- Strengthen preview prominence, sidebar hierarchy, and validation-summary linkage so the app's primary workflow is easier to parse at a glance.

## Capabilities

### New Capabilities
- `visual-state-semantics`: Establishes the semantic theme roles, shared visual hierarchy, and surface layering rules that make the TUI feel coherent across dense screens.

### Modified Capabilities
- `command-context-orientation`: Strengthen sidebar hierarchy and preview prominence so command context and generated output read as part of the same visual system.
- `interaction-feedback-clarity`: Expand the requirements around control affordances, field-state semantics, validation linkage, and feedback severity so interaction patterns stay consistent across widgets and surfaces.

## Impact

- Affected code will center on `crates/clap-tui/src/config.rs`, `crates/clap-tui/src/ui/styles.rs`, `crates/clap-tui/src/ui/sidebar.rs`, `crates/clap-tui/src/ui/form.rs`, `crates/clap-tui/src/ui/preview.rs`, `crates/clap-tui/src/ui/footer.rs`, `crates/clap-tui/src/ui/dropdown.rs`, and `crates/clap-tui/src/ui/toast.rs`, with small supporting updates in layout, frame snapshot, controller, or update helpers where state linkage is needed.
- No public API break is expected, but theme tokens and renderer-facing style semantics will likely expand.
- Snapshot, renderer, and interaction tests will need to cover semantic token usage, focused-versus-selected treatment, control affordance clarity, preview hierarchy, sidebar row emphasis, and ordered invalid-field linkage.
