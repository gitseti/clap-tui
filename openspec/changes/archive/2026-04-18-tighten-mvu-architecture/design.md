## Context

The current TUI already follows a TEA-shaped flow:

- runtime events are translated into actions
- update logic mutates app state and emits effects
- the app loop interprets those effects
- rendering consumes state plus derived projections

That shape is a strength and should be preserved. The strongest pressure points are more local:

- controllers and reducers repeatedly recompute visible form and sidebar projections
- some interaction behavior depends on the boundary between stateful logic and `FrameSnapshot`-derived layout metadata
- preview, visible validation, and Run gating must remain aligned while those seams move
- the top-level action space in `update.rs` is flat and may still become harder to extend if selector extraction does not remove enough coupling
- `Effect` is explicit, but the artifacts previously left open whether this change might widen it into a broader command model

The goal is to make the existing MVU more disciplined without changing the crate's public shape or adopting a broader Elm runtime model. Because this is an internal-facing capability, its contract will be enforced primarily through selector tests, reducer tests, and scripted flow tests rather than through public API changes.

## Goals / Non-Goals

**Goals:**

- Keep the current single app loop and MVU architecture.
- Centralize shared interaction selectors so controller, reducer, and render logic stay aligned.
- Tighten `FrameSnapshot` back to a layout-only contract where domain semantics come from state and selectors.
- Keep side effects explicit at the app-loop boundary using the current single `Effect` model.
- Preserve preview argv, visible validation, and Run gating alignment throughout the refactor.
- Reduce cross-slice interaction coupling where the current flat action surface still causes churn.
- Land the refactor incrementally with tests that protect current interaction behavior.

**Non-Goals:**

- Rewriting the app around a new Elm runtime or dependency stack.
- Adding broad subscription infrastructure, async orchestration, or a new `Cmd/Sub` layer where the current TUI does not need it.
- Changing the public `Runtime`, `TuiApp`, or launcher surface as part of this refactor.
- Redesigning unrelated UI behavior or visual layout.
- Performing a broad standalone `form_editor` redesign in the same change.

## Decisions

### 1. Keep the current MVU shape and harden it instead of rewriting toward "more Elm"

The app already has the important Elm-style properties: a central loop, explicit state, explicit effects, and derived view data. The proposal therefore keeps the current shape and focuses on boundary clarity rather than replacing the architecture with a different conceptual model.

Alternatives considered:

- Full Elm-style rewrite with new `Msg`, `Cmd`, and subscription abstractions everywhere.
  Rejected because the current scale does not justify the cost or churn, and the existing loop already captures the valuable parts.
- Leave the current architecture untouched.
  Rejected because repeated projections and snapshot-boundary drift will become more expensive as interaction surface grows.

### 2. Prioritize shared selectors and `FrameSnapshot` discipline before message-shape changes

The strongest evidence in the current codebase is repeated projection logic and the risk of snapshot-derived semantics leaking outward. The first slice should therefore audit and classify those seams, add characterization coverage, and introduce shared selectors behind behavior-preserving APIs before changing message structure.

Alternatives considered:

- Start by changing the action enum shape first.
  Rejected because that would churn call sites before the higher-value duplication and snapshot-boundary issues are isolated.

### 3. Extract a shared selector layer for interaction context

Common projections such as visible sidebar rows, visible form items, the active form item, and invalid-field focus targets should be exposed through shared selector helpers. Controllers, reducers, and render view-model assembly should consume those helpers instead of open-coding similar logic in multiple places.

Alternatives considered:

- Leave projections embedded inside each controller/reducer helper.
  Rejected because behavior can drift when call sites update at different times.
- Push all selector logic into rendering only.
  Rejected because controllers and reducers also need these projections, not just the view layer.

### 4. Keep the current single `Effect` model and explicitly defer broader `Cmd/Sub` work

Run gating, clipboard writes, and similar operations should continue to cross the reducer boundary through the existing explicit `Effect` model interpreted by the app loop. This change intentionally does not introduce a broader `Cmd` or subscription framework; if future async or background work creates that need, it should be handled in a separate change with its own motivation.

Alternatives considered:

- Execute side effects directly in reducer and controller helpers.
  Rejected because it weakens testability and blurs the update boundary.
- Expand immediately to a broad `Vec<Cmd>` plus subscription model.
  Rejected because current needs are still modest and the added abstraction would be speculative.

### 5. Treat `FrameSnapshot` as layout-only and push semantics back into state/selectors

`FrameSnapshot` should remain a geometry and hit-testing artifact derived from the last render. Any logic that depends on selected command state, active argument semantics, or validation meaning should read from `AppState` or shared selectors rather than teaching `FrameSnapshot` more business context.

Alternatives considered:

- Continue allowing interaction helpers to grow ad hoc snapshot-based semantics.
  Rejected because it risks creating a second, view-derived state model that is harder to reason about.

### 6. Reassess the flat action surface only after selector extraction lands

If selector extraction and snapshot cleanup still leave the flat top-level action space as a real maintenance problem, the app can introduce scoped message families behind the existing dispatch entry point. That remains an implementation option, not the primary invariant this change is protecting.

Alternatives considered:

- Force nested message families as a requirement of the change.
  Rejected because message shape is a design choice; the stronger invariant is alignment between classification, update, and render through shared projections.
- Leave the flat action surface entirely unquestioned.
  Rejected because it may still be a real source of cross-slice churn after selectors land.

## Risks / Trade-offs

- [Shared selectors introduce extra indirection or allocations] -> Prefer lightweight borrowed selectors where possible and move only projections that are already duplicated.
- [Selector extraction creates hidden allocation or cloning costs] -> Keep selectors lightweight and measure hot redraw/navigation paths if they start owning data.
- [The refactor destabilizes mature interaction behavior] -> Preserve existing reducer tests and add scripted flow coverage for representative keyboard and mouse paths.
- [Scoped message refactor breaks dispatch semantics while types still compile] -> Add transition-phase tests that assert representative keyboard and mouse events still route to the same outcomes.
- [Effect abstraction grows prematurely] -> Keep the current single `Effect` shape for this change and defer broader command work.
- [Internal architectural requirements are hard to verify] -> Express each requirement through concrete characterization, selector-alignment, and effect-handling tests instead of relying on prose-only review.

## Audit Findings

- Visible form projection was being recomputed independently in `input.rs`, `controller/keyboard.rs`, `controller/navigation.rs`, `update/form.rs`, `update/sidebar.rs`, and `ui/screen.rs` via repeated `visible_args_for_path(...)` calls followed by local active-field lookup logic.
- Visible sidebar projection was being recomputed independently in `controller/navigation.rs`, `update/sidebar.rs`, and `ui/screen.rs` via repeated `tree_rows(...)` and `tree_items(...)` calls.
- `FrameSnapshot` already served well for layout- and hit-testing-oriented concerns such as sidebar row hit targets, footer button hits, dropdown geometry, scroll clamping, and cursor positioning inside rendered inputs.
- The domain-coupled snapshot usage was concentrated around invalid-field targeting and primary-invalid styling through `invalid_field_ids` / `first_invalid_field_id()`, which made render output a secondary source of business truth.
- The first implementation slice therefore extracts shared selectors for sidebar/form visibility and active/invalid form targeting, while removing the invalid-field business projection from `FrameSnapshot`.

## Migration Plan

1. Audit repeated projection logic and classify existing `FrameSnapshot` usage into layout-only versus domain-coupled call sites.
2. Add characterization tests for keyboard, mouse, dropdown, invalid-field focus, and preview/validation/run alignment before changing the architecture seams.
3. Introduce shared selectors behind existing APIs and switch controller/reducer/render call sites to them with no intended behavior change.
4. Refactor domain-coupled snapshot call sites to read business context from app state or selectors while keeping snapshot helpers layout-focused.
5. Reassess whether the flat action surface is still a meaningful source of coupling; only then consider scoped message families behind the existing dispatch path.
6. Preserve the current single `Effect` model throughout this change.

Rollback remains straightforward because each slice is internal, incremental, and testable.

## Post-Selector Reassessment

After moving the duplicated projection logic behind shared selectors, the remaining flat `Action` surface is still understandable and stable for the current interaction set. This change therefore keeps the existing top-level dispatch path and defers any scoped message-family experiment to a follow-up change if future interaction growth makes that coupling painful again.

## Open Questions

- Should the shared selector layer live under `query/` or under a new dedicated module name such as `selectors/`?
- Should selector results be lightweight borrowed views, or are there cases where owned snapshots are worth the cost for clarity?
- After selector extraction lands, is the flat action surface still causing enough cross-slice churn to justify scoped message families in the same change?
