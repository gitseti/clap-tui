# Architecture Plan

## Purpose

Capture the current rendering/event architecture in `clap-tui`, assess the design, and outline an incremental refactor plan that preserves behavior while improving separation of concerns.

This document is intended to grow. Additional architecture ideas, refactors, and constraints can be appended here later.

## Current Design Summary

The app runs a single synchronous event/render loop in `crates/clap-tui/src/app.rs`.

Each iteration currently does the following:

1. Clears expired transient UI state such as toasts.
2. Normalizes state before rendering via `ui::prepare`.
3. Draws a full frame through ratatui.
4. Polls crossterm for a single event with a 200 ms timeout.
5. If an event arrives, dispatches it to keyboard or mouse handlers.
6. Mutates `AppState` in response to the event.
7. Renders again on the next loop iteration from the updated state.

The runtime abstraction in `crates/clap-tui/src/runtime.rs` is a useful seam. Terminal setup, event polling, event reading, and clipboard access are already abstracted behind `Runtime`.

## Main Architectural Property

The design is immediate-mode from a rendering perspective, but the renderer also writes interaction metadata back into shared state.

That is the key coupling:

- Render code computes layout rectangles for sidebar items, footer buttons, form inputs, tabs, dropdowns, and form viewport.
- Those rectangles are stored in `AppState.frame.layout`.
- Mouse handlers later read those rectangles for hit-testing and cursor placement.

This means the renderer is not just a pure view projection. It also produces controller-consumed state.

## Assessment

### What is working well

- The top-level loop is simple and deterministic.
- The execution model is easy to reason about and debug.
- Keyboard and mouse handling are split into separate controller modules.
- The `Runtime` trait is a strong boundary for terminal and OS integration.
- For the current feature set and data size, the approach is practical.

### Main design concerns

- `AppState` mixes durable domain state, transient UI state, layout snapshots, text editor widget state, scrolling, hover state, dropdown state, and notifications.
- Render and control are coupled through mutable shared layout state.
- Mouse behavior depends on geometry produced by the last render pass.
- Some navigation helpers also depend on last-frame layout information, not only durable model state.
- The loop redraws continuously even when idle, which is simple but not especially efficient.
- Per-frame view construction does avoidable cloning and recomputation.

### Overall judgment

This is a solid small-project design, but not yet a clean long-term architecture.

The render loop itself is not the problem. The main issue is the shared mutable contract between renderer and controller.

## Refactor Direction

Do not replace the event loop first.

Keep the current single-threaded loop and runtime abstraction. Refactor around state ownership and the handoff between render and interaction.

### Target state split

Split current state responsibilities into three categories:

1. Durable domain state
   - Command tree
   - Selected command path
   - Form values
   - Expanded nodes
   - Touched state

2. Transient interaction state
   - Focus
   - Active tab
   - Selected form field
   - Search query
   - Dropdown open/closed state
   - Scroll offsets
   - Hover state
   - Mouse selection state
   - Toast state
   - Textarea/editor state

3. Ephemeral frame snapshot
   - Sidebar row rectangles
   - Form viewport rectangle
   - Form input rectangles
   - Tab button rectangles
   - Footer button rectangles
   - Dropdown rectangle
   - Derived per-frame limits such as max scroll

## Recommended Incremental Plan

### Phase 1: Isolate frame-derived geometry

Introduce a dedicated `FrameSnapshot` type and stop treating layout data as a normal part of durable app state.

Goal:

- Rendering builds a snapshot for the current frame.
- Interaction code consumes that snapshot.
- `AppState` no longer owns general-purpose layout data directly.

Preferred shape:

- `render(...) -> FrameSnapshot`
- or `render(frame, ..., &mut FrameSnapshot)`

The important part is ownership and meaning, not the exact function signature.

### Phase 2: Separate durable state from interaction state

Split `AppState` into a smaller top-level composition such as:

- `DomainState`
- `UiState`
- `NotificationState`

Keep widget-specific and render-specific data out of domain-oriented structs.

The code already gestures in this direction, but the boundaries are still too porous.

### Phase 3: Remove controller dependence on render internals

Mouse and navigation logic should depend on a well-defined snapshot interface rather than on arbitrary mutable fields written by render code.

This should make it clearer which behavior depends on:

- command/form semantics
- interaction state
- actual screen geometry

### Phase 4: Reduce per-frame recomputation

After the boundaries are cleaner:

- trim unnecessary cloning in screen/view-model construction
- consider borrowing more immutable command data
- avoid rebuilding derived structures every frame where not needed

This is lower priority than untangling the architecture.

### Phase 5: Revisit idle redraw strategy

Only after the state boundaries are cleaner, evaluate whether the 200 ms polling/redraw cadence should remain as-is.

Potential options:

- keep the current behavior because simplicity is worth it
- redraw on input plus explicit timed UI events
- introduce a tick/event abstraction if transient UI needs grow

This should be treated as an optimization and clarity pass, not the first refactor.

## Non-Goals

- Replacing ratatui
- Introducing async for its own sake
- Rewriting the entire UI into a new pattern in one step
- Prematurely optimizing small rendering costs before separating responsibilities

## Practical Standard For Future Changes

When adding features, prefer this test:

"Is this state durable application state, transient interaction state, or a frame-local render artifact?"

If the answer is "frame-local render artifact", it should not quietly expand the responsibilities of the main app state.

## Follow-Up Issues Observed During Widget Refactor

The selective widget refactor after `dropdown.rs` exposed a few concrete architecture follow-ups that should be tracked explicitly.

### 1. Stop passing full `&mut AppState` into render code by default

Borrowed `ScreenView` worked, but only after narrowing many renderer inputs away from the full mutable app state.

Refinement to preserve:

- Renderers should take only the slices they actually need.
- Typical inputs should be things like `&UiState`, `&mut UiState`, `&FrameState`, `&mut FrameState`, `&DomainState`, and `&CommandPath`.
- Full `&mut AppState` should remain primarily a controller-level convenience, not the default rendering API.

Reason:

- Broad mutable renderer signatures make borrowed view models much harder to introduce.
- They hide true dependencies and create avoidable ownership friction.

### 2. Add a small borrowed render context type

After narrowing renderer signatures, call sites became more explicit but also more verbose.

Recommended follow-up:

- Introduce a lightweight borrowed context such as `RenderCtx<'a>` or `PanelCtx<'a>`.
- It should bundle read-only inputs commonly shared across renderers without reintroducing cloning or broad mutable access.

Likely contents:

- `&DomainState`
- `&UiState`
- `&FrameState`
- `&CommandPath`
- `&TuiConfig`

This should be a convenience wrapper, not a new catch-all mutable state object.

### 3. Make the panel/view/widget split an explicit pattern

The refactor worked best where the code was separated into three roles:

1. Panel glue
   - owns area splits
   - records hitboxes and frame snapshot data
   - decides when a widget is shown

2. View builder
   - computes labels, ordering, styles, hover flags, and similar render inputs

3. Widget
   - performs render-only drawing through `Widget` or `StatefulWidget`

This pattern should be treated as the default for compact UI pieces.

### 4. Decompose the form panel before attempting full form widgetization

The main form panel is still not a good widget candidate because it mixes too many responsibilities:

- content measurement
- field rendering
- form input rect capture
- dropdown anchor capture
- textarea/editor integration
- cursor placement
- scroll-bound handling

Recommended next step:

- split form layout/snapshot responsibilities from live editor integration
- only consider a larger form widget extraction after that split exists

### 5. Keep `FrameLayout` ownership strictly in panel/orchestration code

The current refactor reinforced a useful rule:

- widgets should never mutate layout snapshot state
- panel/orchestration code should remain the only place that writes `FrameLayout`

This is especially important for:

- sidebar row hitboxes
- form input and tab hitboxes
- footer button hitboxes
- dropdown placement

That rule should be stated explicitly to avoid accidental regression.

### 6. Mark small remaining renderers as optional later widget candidates

Not every renderer needs immediate conversion, but some are low-risk future candidates:

- `header`
- `toast`

These are much cheaper candidates than `sidebar` or the full form panel and can be converted later for consistency if it becomes useful.

## Open Extensions

This doc can be extended with:

- event model ideas
- clipboard/runtime abstraction changes
- text editing model cleanup
- view-model ownership cleanup
- testability improvements
- performance notes
