# Implementation Plan

## Status Update

Current branch status after implementation work in this turn:

- Milestone 0: completed
- Milestone 1: completed
- Milestone 2: completed
- Milestone 3: completed
- Milestone 4: completed
- Milestone 5: completed
- Milestone 6: completed
- Milestone 7: completed
- Milestone 8: completed

### Implemented so far

- Added controller, geometry, and app-loop-adjacent regression tests for sidebar, form, dropdown, mouse hit-testing, cancel/run flows, and clipboard toast behavior.
- Extracted frame-derived geometry into `crates/clap-tui/src/frame_snapshot.rs` and removed frame ownership from `AppState`.
- Changed render to produce a `FrameSnapshot` and moved snapshot ownership to the event-loop boundary in `crates/clap-tui/src/app.rs`.
- Updated keyboard, mouse, and navigation flows to consume `&FrameSnapshot` explicitly.
- Moved durable form-value and touched-state helpers onto `DomainState`.
- Moved focus/tab/scroll helpers onto `UiState`.
- Moved toast lifecycle helpers onto `NotificationState`.
- Updated call sites to use `state.domain`, `state.ui`, and `state.notifications` directly where those ownership boundaries are now explicit.
- Added `crates/clap-tui/src/update.rs` and routed input through an explicit controller `-> Action -> update -> Effect -> app runtime effect` path.
- Converted keyboard and mouse controllers into read-only translators over `&AppState`.
- Kept runtime side effects in `app.rs` via explicit `Effect` handling for run, exit, and clipboard flows.
- Removed `ui::prepare` from the runtime path and moved normalization into `update::normalize_state(...)`.
- Added `crates/clap-tui/src/editor_state.rs` so `UiState` no longer exposes `tui_textarea::TextArea` directly.
- Reduced a small render-path clone hotspot by borrowing `selected_path` and `search_query` directly in `ui::screen::render`.
- Changed the event loop to redraw on demand, using a timer only for toast expiration.

## Purpose

Turn the architecture direction in `agent-docs/2026-03-15-1448-architecture-plan.md` into a concrete, incremental execution plan for the current `clap-tui` codebase.

This plan keeps the existing single-threaded event loop and focuses on reducing the coupling between render, controller, and state ownership.

## Current Implementation Anchors

These files are the main seams for the refactor:

- `crates/clap-tui/src/app.rs`
  - owns the event loop
  - renders via `ui::render`
  - dispatches to keyboard and mouse controllers
  - applies update effects and redraw policy
- `crates/clap-tui/src/input.rs`
  - defines `AppState`, `DomainState`, `UiState`, `NotificationState`
  - now enforces state ownership more explicitly than the original version
- `crates/clap-tui/src/ui/screen.rs`
  - builds screen view data
  - computes layout
  - returns frame-derived geometry via `FrameSnapshot`
- `crates/clap-tui/src/ui/form.rs`
  - computes form geometry and scrollbar limits
  - writes form input rects, tabs, dropdown rect, and scroll max into the frame snapshot
- `crates/clap-tui/src/ui/dropdown.rs`
  - depends on snapshot geometry
- `crates/clap-tui/src/controller/mouse.rs`
  - translates terminal mouse events into actions using snapshot queries
- `crates/clap-tui/src/controller/navigation.rs`
  - still hosts some update-time navigation behavior, now against snapshot queries
- `crates/clap-tui/src/controller/keyboard.rs`
  - translates terminal key events into actions
- `crates/clap-tui/src/form_editor.rs`
  - uses the local editor abstraction instead of exposing raw `TextArea` through `UiState`
- `crates/clap-tui/src/view/form.rs`
  - contains both read-only form selectors and geometry / hit-testing helpers

## Refactor Constraints

- Keep behavior stable for keyboard, mouse, dropdown, scrolling, preview, and toast flows.
- Do not replace the event loop with async or a framework runtime.
- Preserve the `Runtime` abstraction in `crates/clap-tui/src/runtime.rs`.
- Keep changes shippable in small slices with tests after each milestone.
- Avoid broad rewrites of styling or widget appearance while architectural work is in progress.

## Milestone 0: Stabilize Baseline With Focused Tests

Status: Completed

Before moving types around, add tests that protect the current behavior most likely to regress.

### Work

- Add controller-oriented tests for:
  - sidebar selection and expand/collapse
  - form selection movement
  - dropdown open/close and scroll behavior
  - mouse hit-testing against footer, tabs, form inputs, and dropdowns
- Add event-loop adjacent tests where practical around:
  - `Ctrl+C` cancel flow
  - `Ctrl+Enter` run flow
  - clipboard toast behavior
- Expand existing geometry tests in `ui/dropdown.rs` and `view/form.rs` where useful.

### Likely files

- `crates/clap-tui/src/controller/mouse.rs`
- `crates/clap-tui/src/controller/navigation.rs`
- `crates/clap-tui/src/controller/keyboard.rs`
- `crates/clap-tui/src/ui/dropdown.rs`
- `crates/clap-tui/src/view/form.rs`
- possible new crate-local test helpers under `crates/clap-tui/src/`

### Exit criteria

- Current interaction behavior is covered well enough to support internal refactors.
- Tests clearly distinguish durable state behavior from geometry-dependent behavior.

## Milestone 1: Extract `FrameSnapshot` From `AppState`

Status: Completed

This is the highest-value first move because it breaks the implicit contract where render mutates shared state and controllers later read it as normal app state.

### Work

- Introduce a dedicated snapshot module, for example:
  - `crates/clap-tui/src/frame_snapshot.rs`
- Move frame-derived types out of `input.rs`:
  - `FrameLayout`
  - `FrameState`
  - `SidebarItemLayout`
  - `TabButtonLayout`
  - `FooterButtonLayout`
- Rename the extracted top-level render artifact to something explicit such as `FrameSnapshot`.
- Change render to produce a snapshot instead of writing layout data into `AppState`.
- Store only the latest snapshot at the loop boundary in `app.rs`.

### Target shape

- `ui::render(...) -> FrameSnapshot`
- or `ui::render(frame, ..., &mut FrameSnapshot)`
- `app.rs` owns `last_frame_snapshot`
- controllers receive `&FrameSnapshot` when they need geometry

### Likely files

- `crates/clap-tui/src/app.rs`
- `crates/clap-tui/src/input.rs`
- `crates/clap-tui/src/ui/mod.rs`
- `crates/clap-tui/src/ui/screen.rs`
- `crates/clap-tui/src/ui/form.rs`
- `crates/clap-tui/src/ui/footer.rs`
- `crates/clap-tui/src/ui/sidebar.rs`
- `crates/clap-tui/src/ui/dropdown.rs`
- `crates/clap-tui/src/controller/mouse.rs`
- `crates/clap-tui/src/controller/navigation.rs`

### Notes

- The snapshot is still allowed to persist between frames so the next input cycle can consume it.
- The important change is ownership: snapshot data is frame-derived, not part of durable application meaning.

### Exit criteria

- `AppState` no longer owns general-purpose render geometry.
- Render code can be understood as “draw plus snapshot production”.
- Mouse and geometry-aware navigation compile against `&FrameSnapshot`, not `state.frame`.

## Milestone 2: Enforce State Ownership Boundaries

Status: Completed

The structural split already exists in `input.rs`, but the APIs still let most code take broad `&mut AppState` access.

### Work

- Reduce direct `&mut AppState` usage where narrower borrows are sufficient.
- Move durable-state helpers onto `DomainState` when they truly belong there.
- Keep transient interaction helpers on `UiState` or small crate-local helper modules.
- Move notification helpers behind `NotificationState` or narrow wrapper functions where appropriate.
- Stop coupling scroll-limit storage to app state if it is purely frame-derived.

### Concrete changes

- Revisit methods in `impl AppState` and split them by owner:
  - durable form values and touched state stay with domain-oriented state
  - focus, active tab, dropdown state, hover state, selection, and search stay with UI state
  - toast management stays with notification state
- Replace broad helper signatures that only need specific pieces of state.
- Keep one small composition root if desired, but make ownership obvious in the API surface.

### Likely files

- `crates/clap-tui/src/input.rs`
- `crates/clap-tui/src/app.rs`
- `crates/clap-tui/src/controller/navigation.rs`
- `crates/clap-tui/src/controller/keyboard.rs`
- `crates/clap-tui/src/controller/mouse.rs`
- `crates/clap-tui/src/form_editor.rs`

### Exit criteria

- Common controller and render paths no longer require blanket mutable access to the full app state.
- Durable state mutations and transient UI mutations are visibly separated in function signatures.

## Milestone 3: Remove Controller Dependence on Render Internals

Status: Completed

After `FrameSnapshot` exists, the next step is to define a stable geometry interface rather than exposing raw arbitrary fields everywhere.

### Work

- Add focused query helpers on `FrameSnapshot`:
  - hit-test footer target
  - hit-test tab
  - find sidebar item by point
  - resolve form input rect by arg id
  - compute dropdown containment
- Move geometry-specific helper logic out of controllers into snapshot/query code.
- Keep `view/form.rs` for semantic selectors and shared field measurement, but stop using it as a mixed semantic plus controller utility bag where possible.

### Concrete changes

- Replace controller code like `state.frame.layout.form_inputs.get(arg_id)` with snapshot queries.
- Replace direct rect iteration in `mouse.rs` with snapshot helper methods.
- Keep navigation logic focused on semantics:
  - selected command
  - selected arg
  - expanded nodes
  - dropdown state
  - scroll offsets

### Likely files

- `crates/clap-tui/src/controller/mouse.rs`
- `crates/clap-tui/src/controller/navigation.rs`
- `crates/clap-tui/src/view/form.rs`
- new snapshot/query module if separated from raw snapshot structs

### Exit criteria

- Controllers depend on a narrow geometry API instead of on internal render data layout.
- Geometry rules are centralized and testable without pulling in unrelated controller logic.

## Milestone 4: Introduce an Explicit `Event -> Action -> Update -> Render` Flow

Status: Completed

Only do this after the snapshot boundary is stable. Otherwise the update layer will inherit the same coupling.

### Work

- Expand controller responsibility to translation only:
  - terminal event in
  - app action out
- Add an update layer that applies actions to state and may return side effects.
- Keep side effects simple and explicit at first.

### Suggested action families

- command tree:
  - `SelectCommand`
  - `ToggleExpand`
- focus and tabs:
  - `SetFocus`
  - `CycleTabs`
  - `ToggleHelp`
- form:
  - `MoveFormSelection`
  - `ToggleFlag`
  - `SetText`
  - `OpenDropdown`
  - `CloseDropdown`
  - `SetChoice`
  - `ScrollForm`
  - `ScrollDropdown`
- session:
  - `Run`
  - `Exit`
  - `CopyPreview`
  - `ShowToast`

### Suggested side-effect boundary

- update returns something like:
  - `None`
  - `Run(argv)`
  - `CopyToClipboard(text)`
  - `Exit`
- `app.rs` remains responsible for executing runtime effects and feeding resulting toast actions back into state.

### Likely files

- `crates/clap-tui/src/app.rs`
- `crates/clap-tui/src/controller/keyboard.rs`
- `crates/clap-tui/src/controller/mouse.rs`
- `crates/clap-tui/src/controller/mod.rs`
- possible new `crates/clap-tui/src/update.rs`

### Exit criteria

- controllers mostly stop mutating state directly
- meaningful state transitions happen through one explicit update path
- invariants currently split across controllers and `ui::prepare` begin moving into update logic

## Milestone 5: Shrink or Eliminate `ui::prepare`

Status: Completed

Once update logic owns invariants, `ui::prepare` should either become very small or disappear.

### Work

- Audit everything currently done in `ui::prepare`:
  - defaults initialization
  - tab visibility normalization
  - selected-arg normalization
- Decide which parts are:
  - update-time invariants
  - read-only selectors
  - true pre-render derived state
- Move normalization into the update path where possible.

### Likely files

- `crates/clap-tui/src/ui/screen.rs`
- `crates/clap-tui/src/app.rs`
- `crates/clap-tui/src/update.rs`
- `crates/clap-tui/src/input.rs`

### Exit criteria

- render consumes already-valid state
- app invariants are no longer tied to “this function happens to run before draw”

## Milestone 6: Hide Widget Implementation Details Behind Local Abstractions

Status: Completed

The architecture plan calls out `UiState.textareas` as an undesirable boundary leak.

### Work

- Replace raw `HashMap<String, HashMap<String, TextArea<'static>>>` exposure with a crate-local editor abstraction.
- Keep `tui_textarea` as an implementation detail of a small editor-state module.
- Move editor-specific cursor and selection behavior out of general UI state definitions.

### Suggested direction

- introduce a module such as `editor_state.rs`
- expose intent-level operations:
  - ensure editor for field
  - apply key
  - set cursor from click
  - read display text
  - clear selection

### Likely files

- `crates/clap-tui/src/input.rs`
- `crates/clap-tui/src/form_editor.rs`
- possible new `crates/clap-tui/src/editor_state.rs`
- `crates/clap-tui/src/ui/form.rs`
- `crates/clap-tui/src/controller/mouse.rs`
- `crates/clap-tui/src/controller/keyboard.rs`

### Exit criteria

- `UiState` no longer exposes concrete widget crate types as an architectural boundary.
- swapping text-editor implementation becomes a localized change.

## Milestone 7: Reduce Per-Frame Recomputation

Status: Completed

Do this only after the state and update boundaries are clean. Optimizing the current structure first would lock in the wrong design.

### Work

- Reduce repeated cloning of `current_command()`.
- Revisit `ScreenView::build` allocations, especially:
  - tree item building
  - preview argv construction
  - active arg collections
- Separate stable semantic selectors from geometry-dependent helpers.
- Cache or borrow only where it remains simple and measurable.

### Likely files

- `crates/clap-tui/src/ui/screen.rs`
- `crates/clap-tui/src/view/command_tree.rs`
- `crates/clap-tui/src/view/form.rs`
- `crates/clap-tui/src/view/argv.rs`
- `crates/clap-tui/src/app.rs`

### Exit criteria

- obvious cloning and rebuild hotspots are reduced without obscuring the architecture
- selector code has a clearer home than it does today

## Milestone 8: Revisit Idle Redraw Policy

Status: Completed

Only reconsider redraw timing after the update and snapshot flow is explicit.

### Work

- Measure whether the current `poll_event(Duration::from_millis(200))` plus redraw-on-loop is still justified.
- Consider redraw-on-event plus redraw-on-timer only for expiring toasts or cursor-related needs.
- Keep terminal behavior simple unless measured need proves otherwise.

### Likely files

- `crates/clap-tui/src/app.rs`
- `crates/clap-tui/src/runtime.rs`

### Exit criteria

- redraw behavior is a conscious policy choice rather than a side effect of the original loop design

## Recommended Delivery Slices

To keep the work reviewable, implement in this order:

1. Baseline tests for geometry-sensitive behavior.
2. `FrameSnapshot` extraction with no behavior changes.
3. Controller migration to snapshot queries.
4. Narrower state ownership and signatures.
5. Explicit update layer with action types.
6. `ui::prepare` reduction.
7. Editor abstraction cleanup.
8. Performance and redraw follow-ups.

## Risks To Watch

- Mouse behavior regressions during snapshot extraction.
- Scroll and dropdown bugs caused by moving geometry ownership.
- State normalization regressions if `ui::prepare` logic is relocated too early.
- Over-engineering the update layer before the real action vocabulary is clear.
- Borrow-checker pressure if the refactor tries to split too many ownership concerns at once.

## Definition Of Done

The architecture plan should be considered implemented when all of the following are true:

- render produces a frame snapshot instead of mutating shared layout state in `AppState`
- controllers consume semantic state plus a narrow snapshot interface
- durable domain state, transient UI state, and notifications are enforced by APIs, not only by struct names
- meaningful interactions flow through explicit actions and a central update path
- widget implementation details are no longer exposed as the main application-state boundary
- geometry-sensitive behavior remains covered by tests
