## 1. Semantic theme and shared style vocabulary

- [x] 1.1 Extend `crates/clap-tui/src/config.rs` and built-in presets with semantic roles for focus, selected-but-unfocused state, success feedback, warning-like metadata, passive metadata, and layered surfaces.
- [x] 1.2 Refactor `crates/clap-tui/src/ui/styles.rs` so renderers can request shared styles for surface chrome, focused and invalid controls, sidebar row states, metadata badges, preview emphasis, and feedback chips without branching on raw colors locally.
- [x] 1.3 Add or update style-focused tests that lock in the semantic token contracts across supported theme presets.

## 2. Surface chrome and hierarchy

- [x] 2.1 Update `crates/clap-tui/src/ui/sidebar.rs` and related helpers so group labels, command rows, branch depth, and the active row follow a clearer and more consistent hierarchy.
- [x] 2.2 Update `crates/clap-tui/src/ui/preview.rs` and `crates/clap-tui/src/ui/layout.rs` so the preview reads as a primary result surface in both roomy and compact layouts.
- [x] 2.3 Align `crates/clap-tui/src/ui/footer.rs`, `crates/clap-tui/src/ui/dropdown.rs`, and `crates/clap-tui/src/ui/toast.rs` with the shared chrome vocabulary so footer chips, overlays, and feedback surfaces feel related to the rest of the UI.
- [x] 2.4 Add renderer tests covering sidebar emphasis, preview prominence, overlay layering, and feedback-surface severity styling.

## 3. Form hierarchy and control-family consistency

- [x] 3.1 Update `crates/clap-tui/src/ui/form.rs` so labels, values, help text, and metadata badges follow a consistent hierarchy throughout dense forms.
- [x] 3.2 Refine required, inherited, default, environment, and implicit-value treatments so resting invalid states stay calmer while metadata remains compact and scannable.
- [x] 3.3 Unify affordance patterns for text fields, toggles, counters, choice pickers, repeated values, and optional-value controls so each widget advertises its interaction model while still belonging to the same visual family.
- [x] 3.4 Add or update rendering tests for focused-versus-selected treatment, control affordance clarity, and compact metadata badge behavior.

## 4. Validation linkage and completion checks

- [x] 4.1 Use existing form ordering and frame-snapshot metadata to identify invalid fields in deterministic top-to-bottom order.
- [x] 4.2 Update footer or related validation presentation so summaries align visually and semantically with the invalid fields they represent.
- [x] 4.3 Add controller or update logic for highlighting or navigating to the next invalid field if required by the chosen interaction design.
- [x] 4.4 Add tests covering invalid-summary ordering, next-target behavior, and long-form error linkage across compact and roomy layouts.
