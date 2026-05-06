## Context

The form UI is currently implemented mostly in `crates/clap-tui/src/ui/form.rs`. That module owns the top-level form render entrypoints, field iteration, display-value derivation, validation/error styling, default/source styling, selected-state styling, text editor rendering, repeated-value rendering, optional-value rendering, compact controls, field help, help overlay rendering, and a large embedded test suite.

Recent layout work has already pushed geometry toward shared projections in `query/form.rs` and `frame_snapshot.rs`. That geometry is conceptually layout, but its current home under `query` makes ownership harder to read: selectors, field ordering, responsive field projection, scrolling bounds, and hit-testing helpers sit together. Rendering has not yet received a clear boundary either: widget-specific render functions still receive loose bundles of arguments and sometimes participate in deriving state that applies to every field. The refactor should preserve behavior while making layout projection explicit and making per-widget rendering a consumer of one shared field render model.

## Goals / Non-Goals

**Goals:**
- Keep `populate_layout` and `render_form` as the stable entrypoints used by the rest of the UI.
- Establish an explicit form layout module/mechanism that owns responsive field projection and produces geometry for snapshots, scrolling, hit testing, and rendering.
- Split the production code in `ui/form.rs` into focused modules for field orchestration, text fields, repeated values, optional values, compact controls, and help rendering.
- Add an internal `FieldRenderModel` that centralizes per-field derived state before widget dispatch.
- Reduce long argument lists in widget render functions by passing named render inputs.
- Preserve current behavior for rendering, layout, scrolling, hit testing, validation display, default/source styling, focus, cursor placement, repeated-row clipping, optional values, and help overlay behavior.
- Keep tests close to the behavior they verify, either in focused submodules or as moved equivalents of current tests.

**Non-Goals:**
- No visual redesign or new form layout mode.
- No public API, parser, argv serialization, input-state, keybinding, or mouse interaction changes.
- No new external dependencies.
- No broad rewrite of `FrameSnapshot`, `update/form.rs`, or controller modules beyond import/call-site adjustments required by the layout/rendering boundary split.

## Decisions

### Put form field projection behind an explicit layout boundary

Responsive/adaptive field projection should live in a layout module rather than in `query/form.rs` or widget rendering code. The layout boundary should answer "where does this field go?" by producing rects/projections for labels, inputs, descriptions, field bounds, input offsets, and mode-specific geometry. It should not render widgets and should not know about `ratatui::Frame`.

The intended dependency direction is:

```text
selectors/query: which commands and fields are visible?
layout/form: where do visible fields go?
frame_snapshot: which layout rects were visible this frame?
ui/form widgets: draw inside those rects
controllers/update: use snapshot/layout geometry for interaction
```

Use a crate-level `layout::form` module for form field projection. This keeps the responsive field layout boundary structurally separate from `ui` rendering modules and avoids converting the existing `ui/layout.rs` screen-layout module into a nested directory as part of this refactor. The crate-level layout module may use geometry primitives such as `ratatui::layout::Rect`, but it must not depend on `ratatui::Frame`, widgets, styles, or form rendering modules.

`query/form.rs` should retain form ordering, visible-argument helpers, section-heading and semantic form queries, plus wrappers that need to answer form questions. It should not be the long-term owner of label/input/description projection math.

Alternative considered: leave projection in `query/form.rs` because it is already shared. Rejected because the name hides the architectural role and encourages future "query" helpers to accumulate layout responsibilities.

Alternative considered: place form layout under `ui::layout::form`. Rejected for this change because it increases module churn around the existing `ui/layout.rs` file and makes it easier for future layout code to drift toward widget/rendering dependencies.

### Introduce a shared field render model before widget rendering

The render loop should derive a `FieldRenderModel` for each visible `FormFieldLayout` and `OrderedArg` pair. The model should hold the facts every widget needs, such as:

- argument reference and widget kind
- current display value and selected values where needed
- selected/focused state
- validation error and primary-invalid state
- effective value/source/default information
- required and editability state
- text style, block style inputs, placeholder/default-state flags, and open dropdown state
- optional references to current input/effective value for widgets with richer visual states

Widget modules should consume this model plus drawing-specific inputs such as `Frame`, `TuiConfig`, `UiState`, `Rect`, and `input_clip_top`. The model should not own layout projection; it should consume layout results from the explicit form layout boundary and the visible rects already stored in `FrameSnapshot`.

Text-like values in the model should use borrowed references or `Cow<'a, str>` instead of eagerly owned `String` values when the value can be borrowed from domain or derived state. The render loop runs frequently, so the model should allocate only when it must construct derived display strings such as joined multi-choice values, placeholder details, or formatted helper text.

Alternative considered: split files first and keep current argument lists. That would reduce file size but preserve the same hidden coupling and parameter-swap risk. The model-first approach creates a real boundary.

Alternative considered: store owned `String` display values throughout the model for implementation simplicity. Rejected because it would allocate per visible field on each render frame even when most values can be borrowed.

### Organize modules by rendering responsibility

Use a nested `ui/form/` module structure with `mod.rs` retaining the public module entrypoints. Initial module boundaries should be:

- `fields.rs`: field iteration, `FieldRenderModel` construction, widget dispatch, label rendering, field description placement
- `text.rs`: selected and unselected textarea/text-field rendering plus cursor placement helpers
- `repeated.rs`: repeated-value field rendering, repeated-row textareas, add/remove controls, visible-row clipping
- `optional_value.rs`: optional-value visual-state derivation and rendering
- `compact.rs`: toggles, choice/counter compact controls, compact control line construction
- `help.rs`: field help text, section heading line, help overlay

These boundaries are intentionally coarse. Very small helpers can stay with their nearest consumer instead of becoming their own modules.

Alternative considered: one module per current helper function or widget atom. That would create more files without improving ownership.

### Preserve layout and domain ownership

`FieldRenderModel` should derive render-facing state from `ScreenView`, `UiState`, `OrderedArg`, `FormFieldLayout`, and `FrameSnapshot`, but it must not become a second domain model. Domain truth remains in `AppState`, `ScreenView`, shared selectors, and derived state. Geometry truth remains in `FrameSnapshot` and form layout projection.

This keeps the refactor aligned with the existing MVU discipline: rendering consumes view models and snapshots; reducers and controllers continue to own behavior.

Alternative considered: build a larger form view model that also owns geometry and interaction metadata. Rejected because it risks duplicating `FrameSnapshot` and blurring the render/interaction boundary.

### Move tests only as needed to protect behavior

The safest implementation path is to move or re-home tests around the extracted modules while keeping their assertions equivalent. Tests that verify top-level form rendering may stay near `mod.rs` or `fields.rs`; tests for repeated rows, optional values, compact controls, and help text may move next to those modules.

The refactor should prefer preserving current test names unless a rename clarifies the new module responsibility. New tests are only required where extraction creates a new contract that existing tests do not cover, such as model construction for default/source/validation state.

Shared form UI test builders and fixtures should live in a local `test_support` submodule under the form UI test area. This keeps common setup available to the extracted module tests without scattering builders across the new `ui/form/` directory or promoting them to broader crate-level test APIs.

## Risks / Trade-offs

- [Layout module conversion creates churn] -> Use crate-level `layout::form` and leave `ui/layout.rs` focused on screen layout for this change.
- [Render model becomes too large] -> Keep it render-facing and per-field; move widget-only derived values into widget-specific helper structs only if they are not shared.
- [Borrowed render values complicate lifetimes] -> Use `Cow<'a, str>` at the model boundary where plain borrowing becomes awkward, and allocate only for genuinely derived strings.
- [Behavior changes during file movement] -> Move code in thin slices and run the existing form/UI test suite after each meaningful step.
- [Module split exposes private helper friction] -> Prefer `pub(super)` and small local helper modules over broad `pub(crate)` expansion.
- [Tests become harder to navigate] -> Group tests by module responsibility and keep shared fixtures/builders in a local `test_support` submodule.
- [Duplicate style decisions remain after extraction] -> Centralize common style/default/placeholder decisions in `FieldRenderModel` construction before widget dispatch.

## Migration Plan

1. Move form field projection types and functions from `query/form.rs` into crate-level `layout::form` while preserving behavior and call paths.
2. Update `query/form.rs`, `frame_snapshot.rs`, navigation, update, and form rendering call sites to consume the layout module.
3. Convert `ui/form.rs` into a `ui/form/` module while keeping `populate_layout` and `render_form` available at the same module path.
4. Add `FieldRenderModel` construction inside the field-rendering orchestration layer without changing widget behavior.
5. Move text-field rendering helpers into `text.rs` and update call sites.
6. Move compact toggle/choice/counter helpers into `compact.rs`.
7. Move optional-value state and rendering into `optional_value.rs`.
8. Move repeated-value rendering and controls into `repeated.rs`.
9. Move field-help and help-overlay helpers into `help.rs`.
10. Re-home focused tests with the modules they validate and keep top-level regression tests for full-form rendering.
11. Run formatting, Clippy, and the affected test suite.

Rollback is a source-only revert because the change is internal and does not alter persisted data, public APIs, command parsing, or runtime effects.

## Acceptance Criteria

- `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes.
- Existing form rendering, snapshot, navigation, update, and pipeline tests pass.
- Form field geometry projection lives in crate-level `layout::form` rather than widget rendering or broad form query code.
- `render_form` and `populate_layout` remain callable through the existing `ui::form` module path.
- Form rendering behavior is unchanged for text, repeated text, optional values, toggles, choices, counters, validation errors, source/default styling, help overlay, and clipping scenarios covered by current tests.
- Widget render functions no longer need to recompute common per-field validation/effective-value/selection/default state that is available from the shared render model.
- `FieldRenderModel` borrows render values or uses `Cow<'a, str>` where possible, allocating only for derived strings that cannot be borrowed.

## Open Questions

- None.
