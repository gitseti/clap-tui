## Why

`clap-tui` has strong reducer, rendering, and controller tests, but it does not yet verify complete user flows that move through the real app loop with rendered frames, scripted key and mouse input, and final run outcomes. That leaves an important confidence gap between low-level correctness and the interactive behavior users actually experience.

Adding a first-class scripted TUI testing capability now is valuable because the crate already has the right architectural seams for it: a crate-local event model, an injectable `Runtime`, and frame snapshots that describe clickable layout. Capturing those seams in an explicit testing harness will improve regression coverage for interactive behavior without requiring flaky real-terminal automation as the primary strategy.

## What Changes

- Add a deterministic app-level testing harness that can drive `TuiApp` through scripted keyboard and mouse events while rendering into a test backend.
- Add a crate-internal, test-only observation seam needed for tests to inspect rendered output and frame layout after each draw so scripted flows can target UI elements by meaning rather than hard-coded coordinates.
- Add scenario-focused scripted app-flow tests that cover representative end-to-end TUI flows such as opening widgets, selecting values, entering text, and running a valid command.
- Document the intended testing pyramid for the crate so reducer tests, render tests, scripted app-flow tests, and any optional PTY smoke tests each have a clear role.

## Capabilities

### New Capabilities

- `scripted-tui-testing`: The crate supports deterministic, app-level scripted tests that drive the TUI through realistic event sequences and assert on rendered frames and final outcomes.

### Modified Capabilities

None.

## Impact

- Affected code will likely include `crates/clap-tui/src/app.rs`, `crates/clap-tui/src/runtime.rs`, `crates/clap-tui/src/frame_snapshot.rs`, and new Rust test support under the crate test surface.
- Observation support should remain crate-internal or test-only by default rather than becoming a new stable public extension surface.
- Test coverage will expand from reducer- and render-level assertions to scenario-level flows that validate the integration of rendering, input dispatch, state updates, and run behavior.
