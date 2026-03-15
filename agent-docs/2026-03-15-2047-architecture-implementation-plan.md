# Architecture implementation plan

Based on `agent-docs/2026-03-15-2043-architecture-solid-review.md`.

## Goal

Address the main architectural issues called out in the review without destabilizing the crate:

1. remove widget-specific editor state from `UiState`
2. introduce crate-local runtime/event abstractions
3. split centralized interaction handling into smaller reducers
4. make layout computation a clearer phase separate from painting

The codebase already has good structure and tests. This plan assumes incremental refactors, not a rewrite.

## Non-goals

- no redesign of the visual UI
- no async runtime conversion
- no broad public API expansion beyond what is needed for cleaner abstractions
- no large behavioral changes to keyboard, mouse, dropdown, preview, or toast flows

## Guiding constraints

- keep every milestone shippable
- preserve current behavior with targeted tests
- prefer internal abstractions first, public API changes second
- keep `TuiApp` ergonomics simple for library users

## Milestone 1: Decouple editor state from widget state

Purpose: fix the strongest SRP and DIP issue first with limited blast radius.

### Work

- Introduce a widget-agnostic editor model, for example:
  - text buffer
  - cursor position
  - selection range
  - maybe scroll offset if needed
- Replace `EditorState` storage of `TextArea<'static>` with storage of the new editor model.
- Move `tui_textarea` adaptation into the UI/form rendering boundary.
- Keep `form_editor` focused on editing semantics instead of direct widget orchestration.

### Likely files

- `crates/clap-tui/src/input.rs`
- `crates/clap-tui/src/editor_state.rs`
- `crates/clap-tui/src/form_editor.rs`
- `crates/clap-tui/src/ui/form.rs`

### Expected outcome

- `UiState` remains responsible for transient editing state, but not for concrete widget instances.
- Editor behavior becomes easier to unit test without ratatui widget machinery.

### Verification

- extend editor-specific tests around default values, cursor movement, selection, and click placement
- run `cargo test -p clap-tui`

## Milestone 2: Introduce crate-local runtime events and session traits

Purpose: address the current partial DIP issue in `Runtime`.

### Work

- Define crate-local input events, for example:
  - `AppEvent`
  - `AppKeyEvent`
  - `AppMouseEvent`
- Translate crossterm events inside `CrosstermRuntime`.
- Narrow the runtime/session API so `app.rs` no longer matches on crossterm event types directly.
- Keep ratatui backend details inside runtime/session plumbing as much as possible.

### Likely files

- `crates/clap-tui/src/runtime.rs`
- `crates/clap-tui/src/app.rs`
- `crates/clap-tui/src/controller/keyboard.rs`
- `crates/clap-tui/src/controller/mouse.rs`

### Expected outcome

- the app loop depends on crate abstractions rather than crossterm event enums
- alternate runtimes become more realistic to implement

### Verification

- preserve existing app-loop tests by converting them to crate-local events where possible
- run `cargo test -p clap-tui`
- run `cargo clippy -p clap-tui --all-targets`

## Milestone 3: Split `update::apply_action` into smaller reducers

Purpose: improve OCP and keep interaction changes additive.

### Work

- Group actions by concern:
  - global commands
  - sidebar navigation
  - form editing
  - dropdown behavior
  - hover and mouse selection
- Replace one large reducer with smaller functions or modules.
- Keep `Effect` as the runtime-facing boundary unless that proves too limiting.
- Move navigation-specific mutations out of mixed global handlers where possible.

### Likely files

- `crates/clap-tui/src/update.rs`
- `crates/clap-tui/src/controller/navigation.rs`
- possibly new modules under `crates/clap-tui/src/update/`

### Expected outcome

- adding a new behavior touches fewer central files
- reducer logic becomes easier to test by concern

### Verification

- add reducer-level tests for each concern group
- re-run the full crate test suite

## Milestone 4: Separate layout computation from paint

Purpose: make render more one-way and reduce coordination leakage.

### Work

- Introduce an explicit layout/build phase that computes:
  - form input rects
  - tab hit boxes
  - dropdown geometry
  - scroll bounds
- Keep `FrameSnapshot` as the output of that phase.
- Let painting consume the layout model instead of constructing it inline while drawing.
- Avoid mutating editor or layout state during paint unless there is no practical alternative.

### Likely files

- `crates/clap-tui/src/ui/screen.rs`
- `crates/clap-tui/src/ui/form.rs`
- `crates/clap-tui/src/frame_snapshot.rs`
- possibly a new `crates/clap-tui/src/ui/layout.rs`

### Expected outcome

- render becomes easier to reason about as projection over precomputed view/layout data
- hit testing stays stable via `FrameSnapshot`, but its construction becomes more explicit

### Verification

- preserve geometry and hit-testing tests
- add tests around layout phase outputs where practical

## Milestone 5: Tighten public and internal API boundaries

Purpose: clean up the remaining boundary leaks after the structural refactors land.

### Work

- review whether `Runtime` should remain public exactly as-is after the new abstractions
- reduce broad `&mut AppState` usage where narrower borrows are now possible
- move any lingering UI-only helpers out of domain-oriented modules
- review naming so semantic models and widget helpers are clearly separated

### Likely files

- `crates/clap-tui/src/lib.rs`
- `crates/clap-tui/src/runtime.rs`
- `crates/clap-tui/src/input.rs`
- `crates/clap-tui/src/form_editor.rs`
- `crates/clap-tui/src/ui/*`

### Expected outcome

- cleaner internal dependency direction
- less accidental coupling in future feature work

## Recommended sequencing

1. Milestone 1 first because it is the most localized and removes the most obvious boundary leak.
2. Milestone 2 next because event abstraction affects the app loop and controller signatures.
3. Milestone 3 after that because reducer splitting is easier once event types are stable.
4. Milestone 4 once interaction boundaries are cleaner, so layout extraction does not fight moving targets.
5. Milestone 5 as a cleanup pass.

## Risk management

- Highest refactor risk: editor-state decoupling, because cursor and selection behavior are easy to regress.
- Highest API risk: runtime abstraction changes, because they may affect external embedders.
- Lowest risk: reducer splitting, if done after tests are strengthened.

To manage risk:

- land tests before each milestone where coverage is thin
- avoid combining public API changes with deep internal refactors in one patch
- keep each milestone small enough for easy rollback

## Suggested first implementation slice

Start with Milestone 1 and keep it narrow:

1. add a crate-local editor model beside the existing widget-backed implementation
2. migrate `EditorState` to store the new model
3. adapt `ui/form.rs` to build a temporary `TextArea` from that model for rendering
4. keep all external behavior identical

That slice gives the best architectural return for the least cross-cutting change.

## Validation checklist

- `cargo test -p clap-tui`
- `cargo clippy -p clap-tui --all-targets`
- manual smoke check with:
  - `cargo run -p clap-tui --example simple`
  - `cargo run -p clap-tui --example subcommands`
