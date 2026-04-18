## 1. Semantic Styling Foundation

- [x] 1.1 Expand `TuiConfig::Theme` with the semantic roles needed for shell surfaces, active workflow surfaces, separate control-versus-result accent families, and redesigned action chrome.
- [x] 1.2 Refactor `crates/clap-tui/src/ui/styles.rs` so surface, border, selection, badge, and action styling are composed from shared intent-based helpers.
- [x] 1.3 Add or update style-focused tests that lock the redesigned semantic hierarchy across supported theme presets.

## 2. Surface Hierarchy And Context

- [x] 2.1 Redesign the outer shell, sidebar, and workspace chrome so passive panel boundaries calm down while active navigation and editing surfaces become more intentional.
- [x] 2.2 Rework the workspace header to present the selected command path and description as a deliberate context area rather than relying only on panel titles.
- [x] 2.3 Redesign sidebar hierarchy with explicit branch-state and active-row affordances that remain legible when focus is elsewhere.
- [x] 2.4 Restyle the preview and footer so the preview reads as the payoff surface and footer actions read as compact utility keycaps with clearer priority.

## 3. Form Control Visual Grammar

- [x] 3.1 Redesign form section framing to use lightweight headings and divider rules for local and inherited option groups.
- [x] 3.2 Introduce an aligned label column and control column that preserve a compact mostly single-line row rhythm in dense forms.
- [x] 3.3 Update text, choice, counter, toggle, optional-value, and repeated-value controls to share one CLI-native control family with type-specific affordances.
- [x] 3.4 Restyle metadata badges, empty states, and helper text so inherited and required states remain explicit while staying secondary to the editable value.

## 4. Compact Layout And Verification

- [x] 4.1 Tune compact layout behavior so the redesigned sidebar, workspace header, preview, and footer keep their identity while yielding space to the form and preserving dense row scanability.
- [x] 4.2 Update dropdown, toast, and other transient surfaces so their chrome matches the redesigned visual system in roomy and compact layouts.
- [x] 4.3 Refresh renderer snapshots and interaction coverage for the redesigned hierarchy, with examples that exercise sidebar navigation, dense forms, compact mode, and preview emphasis.
