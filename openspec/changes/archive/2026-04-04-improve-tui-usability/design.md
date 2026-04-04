## Context

The UI review exposed a set of related usability issues in `clap-tui`'s core interaction loops rather than in isolated widgets. The current sidebar can select commands that are no longer visible, the screen layout spends a fixed amount of height on chrome even when the terminal is short, and nested command context is easy to lose once the user leaves the command tree. Several controls also suggest the wrong interaction model: counters look like dropdowns, multi-select relies on undocumented keyboard behavior, and the preview's copy interaction is invisible unless the user discovers it accidentally.

These issues cut across rendering, layout geometry, UI state, controller logic, and reducer behavior. The design needs to improve usability without replacing the app architecture, adding dependencies, or disturbing the crate's current public API surface.

## Goals / Non-Goals

**Goals:**

- Keep the active sidebar command visible during keyboard and pointer navigation.
- Preserve more usable form space on narrow or short terminals through adaptive layout behavior.
- Make nested command context and preview affordances easier to understand from the main workspace.
- Align focus, dropdown, counter, preview, footer, and toast behavior with their visible affordances.
- Keep the work incremental, testable, and localized to current UI/controller/update modules.

**Non-Goals:**

- Replacing the overall `ratatui` rendering structure.
- Introducing new theming or layout dependencies.
- Redesigning the entire visual style system beyond the states needed to clarify priority and severity.
- Expanding the app to support new tabs or a wholly different navigation model.

## Decisions

### 1. Add explicit sidebar scroll state and keep selection visibility centralized

The app should track a sidebar scroll offset in UI state and use it consistently for rendering, hit testing, and pointer behavior. Keyboard navigation, search result changes, expand/collapse actions, and sidebar wheel input should all call a shared helper that ensures the selected item remains within the visible window.

This avoids the current split where navigation operates over the full tree while rendering only shows the first visible slice. It also keeps the fix incremental: the tree query model can remain unchanged, while the frame snapshot and sidebar renderer become windowed views over the full row list.

Alternatives considered:

- Recompute a filtered tree that only contains the visible rows.
  Rejected because it complicates navigation semantics and makes hit testing dependent on layout-only transformations.
- Auto-jump focus back to the top visible row when selection leaves the window.
  Rejected because it hides the real problem instead of preserving the user's intended selection.

### 2. Introduce compact layout behavior instead of reserving fixed chrome unconditionally

The screen layout should switch between roomy and compact modes based on terminal dimensions. Compact mode should activate whenever the terminal is below a defined layout budget of 20 rows tall or 80 columns wide. The layout logic should expose this as an explicit `LayoutMode` so compact behavior is decided once and then reused consistently by the header, preview, footer, and overlays. In compact mode, decorative or secondary chrome must collapse first: headers should use only the rows they actually need, the preview should downgrade to a one-line presentation with a minimal bordered treatment, and footer content should use priority-aware truncation rather than competing in one flat row.

This approach preserves the current layout structure while making constrained terminals usable. It also gives the change a clear rule: the form is the primary workspace, so auxiliary surfaces must yield space when height or width is tight.

The same layout work should also address perceived density balance. When a form contains only a handful of visible fields, the footer and sidebar should not feel more visually crowded than the main workspace, so spacing and truncation need to be driven by content priority rather than fixed decoration. Footer priority should be deterministic: validation summary first, then primary action, then secondary action, then search or focus hints, then the rest.

Transient overlays must follow the same compact rules. Dropdowns, help overlays, and toasts should clamp to the available viewport and avoid rendering outside the visible terminal frame in compact mode.

Alternatives considered:

- Keep the fixed layout and only add more scrolling.
  Rejected because it preserves the underlying density problem and makes editing feel claustrophobic on normal 24-row terminals.
- Remove the preview entirely in small layouts.
  Rejected because users still need command visibility; the problem is the preview's cost, not its existence.

### 3. Surface command orientation directly in the main workspace

The workspace title should expose the selected command path rather than only the leaf command name. The header should become a content-driven surface that can show a breadcrumb or full command path plus command description without reserving empty lines when the description is missing. The preview should also expose its purpose explicitly with a title or inline copy hint, preserve click-to-copy, and add a keyboard-accessible copy action. The default keyboard copy path should be `Ctrl+Y`, with the hint rendered directly on the preview in roomy and compact layouts. The sidebar tree should use stronger depth cues so nested branches are easier to parse quickly.

This keeps users oriented even when the sidebar is filtered, scrolled, or unfocused, and it makes the preview feel like a deliberate tool rather than a hidden interaction target.

Alternatives considered:

- Keep orientation solely in the sidebar.
  Rejected because focus often moves to the form, and the sidebar can be filtered, collapsed, or off to the side on smaller terminals.
- Put all context into the footer.
  Rejected because the footer is already a high-pressure area with actions and status messaging.

### 4. Align interaction affordances with actual behavior

Interaction-heavy widgets should advertise the behaviors they actually support. Search should participate in the normal focus cycle in a fixed order of Sidebar -> Search -> Form with `BackTab` reversing that order, implemented with explicit next-focus and previous-focus helpers rather than a sidebar or form toggle. Multi-select dropdowns should explain that `Space` toggles and `Enter` confirms or finishes, counters should render stepper-oriented affordances instead of dropdown chevrons, required fields should use instructional empty states instead of passive “nothing here” messaging, inherited fields should explain that editing creates a local override for the selected command path, and dropdown dismissal should preserve the current retarget-on-click behavior for outside interactions.

This decision favors learnability over preserving every current interaction quirk. The goal is not to add more controls, but to reduce the number of hidden rules a user has to infer from trial and error. Because the current reducer and pointer flow already appear to retarget most outside clicks correctly, the dropdown portion of this work should focus on auditing edge cases and locking the current contract in with tests rather than assuming a large behavior rewrite.

Alternatives considered:

- Keep the current behavior and only document it in README.
  Rejected because these are interaction-level mismatches that need to be discoverable in the UI itself.
- Standardize every choice widget on the same `Enter` behavior without regard to multi-select expectations.
  Rejected because multi-select still benefits from explicit toggle semantics; the real issue is discoverability.

### 5. Strengthen status hierarchy through explicit state-specific styling

Primary actions, passive hints, validation summaries, success toasts, and error toasts should each use distinct visual treatments. Validation summaries must no longer share the same subtle treatment as low-priority hints, selected dropdown rows must keep high-contrast text even when the value is a default, required widgets should read as needing action rather than as neutral placeholders, inherited badges should be clearly secondary to editable values, and error toasts should use error-oriented styling instead of the neutral border color.

This preserves the current theme system while making it expressive enough for severity and priority. The change should reuse theme tokens where possible and add only the minimal new styling branches needed to distinguish states clearly. Priority and severity must remain legible through more than color alone, using combinations of border treatment, labels, emphasis, and placement.

Alternatives considered:

- Solve hierarchy purely through wording and placement.
  Rejected because the review findings are partly about states that already exist but look too similar.
- Add a large set of new theme tokens up front.
  Rejected because it would widen the scope unnecessarily; the immediate need is better branching over the existing theme surface.

## Risks / Trade-offs

- [Sidebar scroll state drifts from selection state] -> Centralize selection-visibility updates in controller helpers and cover keyboard, mouse, search, and expand/collapse transitions with tests.
- [Compact layout introduces hard-to-predict breakpoints] -> Keep the mode rules simple, deterministic, and snapshot-tested across representative terminal sizes.
- [Retargeted dropdown clicks differ subtly across surfaces] -> Audit existing left-click retarget behavior first, then add targeted fixes only where tests expose gaps across form, sidebar, footer, preview, and search interactions.
- [Inherited-value messaging adds noise to dense forms] -> Keep the source treatment concise and use explanatory text only on selection or when a field is inherited and editable.
- [More distinct styling increases theme branching complexity] -> Reuse existing theme colors where possible and isolate state decisions inside `ui/styles.rs`.

## Migration Plan

1. Add sidebar scroll state, visible-window rendering, and sidebar visibility helpers.
2. Introduce a shared layout mode for header, preview, footer, and overlays plus width-aware footer/status behavior.
3. Update workspace context surfaces, sidebar hierarchy cues, and preview affordances.
4. Align interaction semantics for explicit focus cycling, preview copy, multi-select hints, inherited overrides, counters, and dropdown-retarget audits.
5. Tighten feedback styling for primary actions, validation, inherited indicators, dropdown selection, and toasts, then add or update tests for each slice.

Rollback remains straightforward because each slice is local to existing UI/controller/update modules and can be reverted independently.

## Resolved Assumptions

- Compact preview uses a single content row with a minimal bordered treatment instead of becoming unbordered.
- Preview copy uses `Ctrl+Y` as the default keyboard path and advertises that binding in the rendered preview.
- Dropdown outside-click work starts as an audit-and-tests pass because the current implementation already appears to retarget most actionable surfaces.
