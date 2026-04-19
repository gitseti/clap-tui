## Context

The current `clap-tui` form pipeline already separates field classification, geometry, rendering, and interaction handling, but those layers still encode different visual assumptions. `query/form.rs` chooses `FieldWidget` variants, `frame_snapshot.rs` derives field and section geometry, `ui/form.rs` renders several unrelated control shapes, and `ui/sidebar.rs` draws the search field as static text rather than an editor-like surface. That split made it possible to add capability support quickly, but it now leaves the workspace with inconsistent controls, boxed section groups, cramped inline metadata badges, and a search field that does not visibly enter an editable state.

This change is still a refinement rather than an interaction-model reset. Existing behavior such as clicking a flag to toggle it, opening dropdowns for choice widgets, incrementing counters, preserving inherited-option behavior, and editing text through `tui-textarea` should remain intact. The work therefore needs to unify presentation and clipping behavior without flattening the semantic distinctions the reducer and renderer currently rely on.

## Goals / Non-Goals

**Goals:**
- Give text inputs, flags, counters, dropdown-backed fields, and optional-value fields a recognizably shared control family.
- Replace full boxed section framing with heading-plus-rule framing while preserving clear local versus inherited grouping.
- Stack compact metadata badges beneath the label so long option names remain readable.
- Render default-derived values as muted state and user-entered values as primary state across relevant widgets.
- Make focused empty search visibly editable by removing placeholder copy and showing a cursor position.
- Eliminate the clipped-section-header reentry artifact in long scrolled forms.

**Non-Goals:**
- Changing the underlying command model, field ownership model, or preview argv semantics.
- Replacing dropdown, toggle, counter, or optional-value behavior with free-form text editing.
- Reworking sidebar navigation, footer behavior, or preview layout beyond what is required for search focus and form consistency.
- Introducing a new external dependency or a second general-purpose editor state for the form.

## Decisions

### 1. Separate visual control family from interaction semantics

The renderer should stop equating compact one-line presentation with non-text behavior. Text-like chrome such as bordered textarea surfaces, muted placeholder/default tones, and aligned multi-line footprints should become reusable visual treatment that can wrap different widget semantics. Flags, counters, pickers, and optional-value fields will therefore keep their current click and keyboard reducers while adopting a shared control container and value presentation vocabulary.

Alternative considered:
- Collapse these widgets into literal text editors.
  Rejected because it would blur the behavior model and weaken affordance clarity for toggles, pickers, and steppers.

### 2. Promote stacked label metadata into the layout model

The current label row is effectively single-line, so badges compete directly with long option names. This change should make label layout explicit: the option name remains on the primary label row, while compact metadata badges render on a secondary label row beneath it when present. `FieldMetrics`, field-content geometry, hit-testing, and rendering should all consume the same stacked-label shape so badge placement does not become a paint-only hack.

Alternative considered:
- Keep badges inline and merely tighten chip width.
  Rejected because the crowding problem is structural and reappears with long names or multiple badges.

### 3. Replace per-field section boxing with section-run framing

Current section rails and bottom caps are derived per field, which makes the form read as a nested bordered panel and complicates clipping behavior. The new framing should treat a section as a run with one heading row and no closing cap: a heading label followed by a horizontal rule, then the indented section rows beneath it. Geometry should express whether a field belongs to a section run, but painting should no longer depend on left and right rail glyphs around every row.

Alternative considered:
- Remove section framing entirely and rely only on spacing.
  Rejected because inherited sections and other grouped runs still benefit from lightweight structural cues.

### 4. Centralize value-tone selection by effective source rather than widget branch

Muted default state and primary user state should not be decided independently for each widget path. The renderer should derive tone from effective source and touch state, then reuse that decision across compact controls, textareas, optional-value states, and repeated rows. This preserves the requested behavior that untouched default values stay muted while user-entered values remain promoted after focus changes.

Alternative considered:
- Special-case only single-line controls and leave textarea logic unchanged.
  Rejected because it would preserve inconsistent meaning between widget families.

### 5. Keep search state lightweight and add editor-like focused rendering

The search field already stores its content as a simple `String`, and that remains sufficient. Instead of introducing a dedicated editor subsystem, focused-empty search should render without placeholder text and place the terminal cursor at the start of the inner content area; focused non-empty search should continue to show the query with a visible cursor position at the end. This keeps search implementation simple while addressing the current discoverability gap.

Alternative considered:
- Replace the search field with a full `tui-textarea` editor.
  Rejected because the current search model is simpler than the form editors and does not justify separate persistent cursor state yet.

### 6. Model clipped section headings as visibility of section boundaries, not of individual field headings

The section-heading reentry bug comes from inferring headings while iterating field-by-field through a clipped viewport. The layout should instead suppress a heading once its own row is offscreen and only render a heading when the corresponding section boundary itself is visible. This prevents a heading from reappearing at the top edge merely because visible rows still belong to that section.

Alternative considered:
- Patch rendering to ignore headings when the top visible row is not the first row in the section.
  Rejected because that would duplicate visibility logic outside the layout model and make hit-testing and future maintenance harder.

## Risks / Trade-offs

- [Shared textarea-like chrome could make non-text widgets feel too text-like] → Keep explicit affordances for toggles, counters, and pickers inside the shared container.
- [Stacked label metadata increases vertical cost in dense forms] → Only add the second label row when badges are actually present and keep help placement aligned beneath the control.
- [Section-boundary changes could disturb clipping and hit-testing] → Update geometry, paint, and scripted/renderer tests together instead of relying on visual fixes alone.
- [Search cursor rendering without full editor state may feel limited] → Keep the change focused on click-to-edit clarity and revisit richer cursor movement only if follow-up feedback demands it.

## Migration Plan

1. Update the form-control, visual-state, interaction-feedback, and adaptive-layout specs to capture the new contract.
2. Rework field metrics and frame-snapshot geometry so stacked labels and heading-only sections are first-class layout concepts.
3. Update form rendering to use the shared control family and centralized value-tone logic while preserving existing widget semantics.
4. Update search rendering and pointer/focus behavior so focused empty search becomes visibly editable.
5. Refresh renderer, layout, and interaction tests for section framing, default-vs-user color treatment, search focus, and clipped-section behavior.

Rollback remains straightforward because the change is localized to form/sidebar rendering, snapshot geometry, and tests.

## Open Questions

- Whether the shared textarea-like control family should reuse the exact rounded border treatment from text inputs or introduce a slightly flatter variant for one-line non-text widgets.
- Whether muted default-derived rendering should apply identically to env-derived and conditional-default values or reserve slightly different badge/value combinations for those cases.
