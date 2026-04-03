## 1. Scripted Harness Foundations

- [x] 1.1 Add a crate-internal, test-only app-loop wrapper in `app.rs` that runs the real event loop with a deterministic scripted runtime and a `TestBackend` instead of `crossterm`.
- [x] 1.2 Implement the post-draw observation seam inside that wrapper so tests can read rendered text and the latest `FrameSnapshot` after each draw without widening the public API.
- [x] 1.3 Add low-level harness tests proving scripted flows can observe intermediate frames, cancellation, and successful run outcomes deterministically.

## 2. Semantic Interaction Helpers

- [x] 2.1 Add harness helpers that translate snapshot-derived layout into semantic interactions such as clicking footer buttons, opening dropdowns, and targeting rendered controls.
- [x] 2.2 Keep raw event injection available for edge cases while making semantic helpers the default path used by new scripted tests.
- [x] 2.3 Add focused tests proving the helpers derive interaction targets from current layout rather than from hard-coded coordinates.

## 3. Representative Scripted Scenarios

- [x] 3.1 Add a happy-path scripted scenario that mixes realistic interaction steps such as navigation, value editing, and Run, then asserts on the returned argv.
- [x] 3.2 Add an invalid-flow scripted scenario that attempts Run from an invalid state and asserts that run is blocked and visible validation feedback is rendered.
- [x] 3.3 Add at least one mouse-oriented scripted scenario that uses rendered layout to interact with a control such as a dropdown or footer action.
- [x] 3.4 Add at least one event-loop-focused scripted scenario that covers behavior such as toast expiry during interaction or resize-triggered redraw.

## 4. Testing Strategy Documentation

- [x] 4.1 Document the testing pyramid for `clap-tui`, including when to use reducer tests, render tests, scripted app-flow tests, and optional PTY smoke coverage.
- [x] 4.2 Update contributor-facing guidance or nearby test documentation so future behavior changes have an obvious place to add scripted TUI scenarios.
