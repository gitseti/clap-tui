## Why

`clap-tui` already has the right product structure: a sidebar for command navigation, a dense form workspace, a generated preview, and compact footer actions. The current rendering is usable, but it reads like a balanced default theme rather than an intentional, high-character terminal product, so the main workflow does not land with the same clarity or visual confidence as the target redesign direction.

This is the right time to address that gap because the renderer split is already mature enough to support a visual-language redesign without replacing the app architecture. We can keep the existing layout model and interaction flow while redefining the chrome, hierarchy, control grammar, and semantic styling that make the interface feel more like a polished showcase.

## What Changes

- Redesign the TUI visual language around a more opinionated terminal aesthetic with darker shell surfaces, brighter accent hierarchy, and stronger separation between passive chrome and active workflow surfaces.
- Rework the main workspace context so the selected command path and description read like a deliberate header area rather than incidental panel text.
- Strengthen sidebar hierarchy with a more prominent active row, clearer branch depth, and a calmer treatment for passive groups and empty space.
- Redesign form sections and controls so text input, toggles, counters, choice pickers, multi-value chips, inherited badges, and helper text feel like one CLI-native component family instead of generic boxed fields.
- Elevate the generated command preview into a more prominent result surface with clearer payoff, stronger syntax emphasis, and tighter relationship to the current form state.
- Refine footer action styling so primary actions, secondary actions, validation state, and low-priority hints read with clearer priority and less visual competition.
- Preserve compact-layout behavior while making sure the redesigned chrome still degrades cleanly on smaller terminals.

## Capabilities

### New Capabilities
- `form-control-visual-grammar`: Defines the shared visual grammar for dense command-form controls, section framing, chip treatments, and type-specific affordances.

### Modified Capabilities
- `visual-state-semantics`: Tighten semantic color and surface roles so focus, selection, metadata, feedback, and adjacent surfaces map to a more opinionated and stable hierarchy.
- `command-context-orientation`: Strengthen workspace header context, sidebar legibility, and preview prominence so the selected command and generated invocation stay visually central.
- `interaction-feedback-clarity`: Update control affordance and feedback requirements so redesigned widgets remain self-explanatory, especially for counters, toggles, multi-value fields, inherited states, and footer status.
- `adaptive-terminal-layout`: Preserve the redesigned hierarchy and result-surface identity in compact layouts without letting decorative chrome crowd out the form.

## Impact

- Affected code will center on `crates/clap-tui/src/config.rs`, `crates/clap-tui/src/ui/styles.rs`, `crates/clap-tui/src/ui/screen.rs`, `crates/clap-tui/src/ui/layout.rs`, `crates/clap-tui/src/ui/sidebar.rs`, `crates/clap-tui/src/ui/header.rs`, `crates/clap-tui/src/ui/form.rs`, `crates/clap-tui/src/ui/preview.rs`, `crates/clap-tui/src/ui/footer.rs`, `crates/clap-tui/src/ui/dropdown.rs`, and `crates/clap-tui/src/ui/toast.rs`.
- No architecture rewrite or public interaction-model reset is intended; the change is primarily a redesign of theme semantics, shared styles, control presentation, and panel hierarchy.
- Theme tokens, style helpers, snapshot coverage, and renderer tests will expand to lock the new visual direction into the existing TUI surfaces and layout modes.
