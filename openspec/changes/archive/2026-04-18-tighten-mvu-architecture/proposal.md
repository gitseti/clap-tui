## Why

`clap-tui` already has a strong MVU spine, but the internal architecture is starting to flatten in ways that will make future changes harder: repeated selector logic, mixed effect boundaries, and a risk that render snapshots become a second source of truth. The code does not need an Elm-style rewrite; it needs tighter discipline around the MVU it already has.

This is worth doing now because the crate is still small enough to refactor incrementally. Tightening the boundaries before adding more screens, effects, or interaction modes should preserve the current strengths without paying the cost of a broader architectural reset.

## What Changes

- Keep the current MVU/event-loop architecture and explicitly avoid a broad Elm rewrite or new state-management dependency.
- Make the first implementation slice behavior-preserving and internal-only, with no intended UI or public API changes.
- Extract shared selectors for visible tree rows, visible form rows, active field context, and invalid-field targeting so controller, reducer, and render logic stop recomputing equivalent projections independently.
- Tighten the `FrameSnapshot` boundary so it stays a layout and hit-testing artifact rather than growing into a secondary domain model.
- Keep run, clipboard, and similar non-stateful work in the current single `Effect` model at the app-loop boundary; this change does not introduce a broader `Cmd` or subscription layer.
- Reduce cross-slice interaction coupling at the controller/update boundary, considering scoped message families only if selector extraction still leaves the flat action surface as a proven maintenance problem.
- Add explicit characterization and regression coverage at the selector, reducer, and scripted-flow layers so the refactor can land incrementally with confidence.

## Capabilities

### New Capabilities
- `mvu-architecture-discipline`: Internal architecture rules for shared selectors, explicit single-effect handling, behavior-preserving refactors, and layout-only frame snapshots.

### Modified Capabilities
- None.

## Impact

- Affected code will include `crates/clap-tui/src/app.rs`, `crates/clap-tui/src/update.rs`, `crates/clap-tui/src/controller/`, `crates/clap-tui/src/input.rs`, `crates/clap-tui/src/query/` or a new selector module, `crates/clap-tui/src/frame_snapshot.rs`, and related tests.
- The public API should remain stable; this proposal is aimed at internal architecture rather than new end-user features.
- Test coverage will need to expand around selector behavior, preview/validation/run alignment, pointer/layout boundaries, dispatch regression, and end-to-end interaction flows that exercise the real app loop.
