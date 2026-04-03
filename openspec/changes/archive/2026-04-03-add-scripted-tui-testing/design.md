## Context

`clap-tui` already has the ingredients for stronger app-level interaction tests:

- `TuiApp` runs through a single event loop that owns redraw, input dispatch, and run behavior
- `Runtime` is injectable, so tests can avoid `crossterm` and real terminal state
- `AppEvent` is already crate-local and deterministic
- `FrameSnapshot` records layout information that maps rendered UI back to meaningful hit targets
- existing tests already use `ratatui::backend::TestBackend` for render assertions and fake runtimes for event-loop assertions

What is missing is a reusable way to combine those seams into scenario-driven tests that observe rendered frames over time, interact with the app using semantic targets, and assert on the final outcome. The design should strengthen integration coverage without expanding the stable public surface more than necessary.

## Goals / Non-Goals

**Goals:**

- Add a deterministic, app-level testing harness that drives the real app loop with scripted events.
- Let scripted tests inspect both rendered output and layout metadata after each draw.
- Keep scripted tests resilient by targeting semantic UI elements through `FrameSnapshot` rather than hard-coded coordinates whenever possible.
- Cover representative end-to-end flows such as dropdown selection, text entry, focus changes, validation visibility, and successful Run behavior.
- Preserve the crate’s intentionally small public API and avoid turning test support into a supported extension surface.

**Non-Goals:**

- Making PTY-based terminal automation the primary testing strategy.
- Publishing internal frame snapshot or renderer details as stable public APIs for downstream users.
- Replacing existing reducer, controller, or pure render tests.
- Introducing async runtime dependencies or an external GUI automation stack.

## Decisions

### 1. Build the primary harness around the existing `Runtime` seam

The main testing strategy will use a scripted runtime implementation that feeds queued `AppEvent` values into the existing app loop while rendering into a test backend. This preserves realistic flow through `event_loop`, `handle_app_event`, reducers, effects, and redraw logic without depending on a real terminal session.

This is the closest fit to how Ratatui applications are typically tested: app logic handles synthetic events, and rendering is asserted through `TestBackend` rather than through PTY automation.

Alternatives considered:

- Spawn a real PTY and drive the example binary.
  Rejected as the primary strategy because it is slower, more brittle across platforms, and unnecessary for most app-behavior regressions.
- Keep only controller- and render-level tests.
  Rejected because that leaves gaps in redraw timing, event-loop integration, and effect handling across realistic flows.

### 2. Use a crate-internal test-only app-loop wrapper for frame observation

Scripted tests need to react to what the app rendered, not just to final results. The design will therefore add a narrow internal observation seam that captures the rendered backend state and the `FrameSnapshot` produced during each draw. Tests can then wait for a frame that satisfies a predicate and compute click targets from layout metadata.

The observation seam should be implemented as a crate-internal, test-only wrapper around the app loop in `app.rs` that records post-draw state for a scripted `TestBackend`-based runtime. This keeps the logic close to `event_loop`, avoids changes to the public `Runtime` trait, and avoids turning `TerminalSession` or `FrameSnapshot` into supported extension points. The stable public surface should remain centered on `TuiApp`, `TuiConfig`, and `Runtime`.

Alternatives considered:

- Make `FrameSnapshot` publicly stable.
  Rejected because the crate explicitly treats frame snapshots and query helpers as implementation details.
- Add a public observation hook to `Runtime` or `TerminalSession`.
  Rejected because it would widen the supported API solely for internal test ergonomics.
- Hard-code coordinates in scripted tests.
  Rejected because it would make tests brittle against layout adjustments and theme-independent spacing changes.
- Use only `CompletedFrame` buffer text without layout metadata.
  Rejected because text-only assertions are insufficient for robust mouse-driven tests.

### 3. Keep scripted flow tests in crate-internal test support

Because the most valuable testing information lives in internal state and layout types, the reusable harness should live under `#[cfg(test)]` crate test support rather than as a downstream-facing API. Scenario tests can then use internal helpers to:

- run a command through the actual event loop
- inspect the latest rendered frame text and snapshot
- enqueue keys, paste payloads, resize events, and mouse clicks
- assert on final argv or cancellation

This keeps the harness ergonomic for the crate without implying stability guarantees for external users.

Alternatives considered:

- Put all scripted tests in `tests/` integration crates.
  Rejected as the default because integration tests cannot easily rely on internal frame and layout types without widening the public API.
- Encode all flow expectations through public APIs only.
  Rejected because it would force an awkward public testing surface onto a library that deliberately keeps internals private.

### 4. Treat semantic interaction helpers as part of the harness contract

The test harness should provide helpers that operate on UI meaning, such as “click Run”, “open dropdown for selected field”, or “select footer button”, backed by snapshot-derived coordinates. Raw coordinate injection should remain available for edge cases, but semantic helpers will be the default because they produce clearer tests and survive non-behavioral layout refactors better.

Alternatives considered:

- Expose only raw event queues.
  Rejected because it would make every scenario repeat coordinate lookup and increase brittleness.
- Build a fully generic DSL before adding tests.
  Rejected because a small set of targeted helpers is enough to unlock immediate coverage.

### 5. Add one optional PTY smoke path only after the scripted harness exists

If the project later wants confidence in raw mode, alternate-screen handling, or terminal integration behavior beyond what the injected runtime covers, it may add a single PTY smoke test. That test is explicitly secondary to the deterministic harness and should verify only that a real terminal session can start, accept minimal input, and exit cleanly.

Alternatives considered:

- Plan full PTY scenario coverage alongside the main harness.
  Rejected because it would front-load the least deterministic part of the test story and slow the initial change down.

## Risks / Trade-offs

- [Internal observation hooks leak too much app structure] -> Keep the seam test-only or crate-internal, and expose only the minimal frame text and snapshot data needed for harness assertions.
- [Harness abstractions become more complex than the tests they support] -> Start with a small helper surface oriented around the first few scenario tests, and expand only when real duplication appears.
- [Scripted tests become sensitive to benign layout changes] -> Prefer snapshot-derived semantic target helpers and assert on behaviorally meaningful text rather than full-screen golden snapshots for every flow.
- [No PTY coverage misses terminal-session regressions] -> Reserve a single smoke test as a later additive step once the core scripted harness is established.

## Migration Plan

1. Add a crate-internal, test-only app-loop wrapper that records post-draw frame text and `FrameSnapshot` for a scripted `TestBackend` runtime.
2. Introduce a small set of semantic interaction helpers that map snapshot layout to mouse and keyboard events.
3. Add representative scenario tests covering a valid happy path, at least one validation-focused path, and at least one event-loop-specific path such as toast expiry or resize-driven redraw.
4. Document the testing pyramid so future changes know when to add reducer tests, render tests, scripted flow tests, or a PTY smoke test.

Rollback is straightforward because the harness can land incrementally and does not require replacing existing tests or changing runtime behavior for production users.

## Open Questions

- Which first scenarios provide the best regression value: a valid run flow, a validation-error flow, or a mouse-heavy dropdown flow?
- Should any documentation for downstream users mention the scripted harness, or should it remain purely contributor-facing?
