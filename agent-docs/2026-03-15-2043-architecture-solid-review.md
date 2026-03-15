# clap-tui architecture, design, and SOLID review

Reviewed against the current `main` workspace state, with emphasis on module boundaries, dependency direction, and extension points.

## Overall assessment

The project has a solid small-library shape. The core interaction loop is easy to follow, the `spec` / `controller` / `update` / `ui` split is real rather than cosmetic, and the test coverage is unusually good for a TUI crate. There are no critical architectural defects in the current codebase.

The main weaknesses are boundary leaks rather than missing structure:

- UI state still owns concrete widget objects.
- The runtime abstraction still exposes crossterm and ratatui types.
- Interaction handling is centralized enough that adding features will require touching several places.

## Findings

1. Medium: `UiState` is still coupled to a concrete widget implementation, which weakens SRP and DIP.

`UiState` stores `EditorState` directly in the main application state (`crates/clap-tui/src/input.rs:69-82`), and `EditorState` stores `tui_textarea::TextArea<'static>` instances (`crates/clap-tui/src/editor_state.rs:7-28`). That means a rendering detail is part of the durable interaction state. The coupling continues in `form_editor`, where a single function both drives the widget and mutates domain values (`crates/clap-tui/src/form_editor.rs:34-61`), and in the renderer, which mutates the selected field’s textarea during paint (`crates/clap-tui/src/ui/form.rs:307-326`).

This is workable at the current size, but it makes the editor behavior harder to test independently, harder to swap out, and more invasive than it needs to be when new input widgets are introduced.

Recommendation: keep a widget-agnostic editor model in state, and adapt that model to `TextArea` only inside the UI layer.

2. Medium: the `Runtime` trait is only a partial abstraction, because higher layers still depend on crossterm and ratatui types.

`Runtime` exposes `Terminal<Self::Backend>` and `crossterm::event::Event` in its interface (`crates/clap-tui/src/runtime.rs:19-56`). `TuiApp` then matches on `crossterm::event::Event` directly in the event loop (`crates/clap-tui/src/app.rs:124-185`). This gives testability benefits, but it does not fully invert the dependency: alternative runtimes still have to look like crossterm plus ratatui.

From a SOLID perspective, this is the clearest DIP shortfall in the codebase. The app layer depends on concrete infrastructure concepts instead of crate-local abstractions.

Recommendation: define crate-local input events and a smaller session abstraction, then translate crossterm events inside the default runtime.

3. Medium: interaction behavior is extension-hostile because it is spread across several centralized dispatch points.

The action pipeline is understandable, but feature additions will keep concentrating change in a few files. New behavior typically requires edits in `controller/keyboard.rs` (`crates/clap-tui/src/controller/keyboard.rs:9-93`), sometimes `controller/mouse.rs` (`crates/clap-tui/src/controller/mouse.rs:8-82`), the `Action` enum, and the main `apply_action` reducer (`crates/clap-tui/src/update.rs:11-158`). Navigation logic is then split again into `controller/navigation.rs` (`crates/clap-tui/src/controller/navigation.rs:8-317`).

The code is still maintainable, but this is the area most likely to degrade as the interaction surface grows. Clippy already flags `apply_action` as too large, which matches the structural concern.

This weakens OCP more than the other SOLID principles: adding a new interaction mode is not hard, but it is rarely additive.

Recommendation: split action handling into smaller reducers by concern, for example sidebar, form editing, dropdowns, and global commands.

4. Medium-low: rendering still performs controller-facing layout/state work, so the render and interaction layers are not fully one-way.

`ui::screen::render` both constructs the screen view model and assembles the mutable `FrameSnapshot` used for hit testing (`crates/clap-tui/src/ui/screen.rs:47-148`). Inside `ui::form`, the render pass clears and repopulates form input rectangles, tab hit boxes, dropdown geometry, and also touches editor state (`crates/clap-tui/src/ui/form.rs:23-86`, `crates/clap-tui/src/ui/form.rs:188-326`). Mouse and navigation handling then depend on that geometry (`crates/clap-tui/src/controller/mouse.rs:31-80`, `crates/clap-tui/src/controller/navigation.rs:161-270`).

The good news is that `FrameSnapshot` is a strong design choice. The remaining issue is that the render pass is still doing too much coordination work instead of being a mostly pure projection step.

Recommendation: keep `FrameSnapshot` as the seam, but consider an explicit layout pass that computes interaction geometry before the widgets are painted.

## Strengths

- The top-level flow is clean: event mapping in `controller`, state mutation in `update`, drawing in `ui`, and clap projection in `spec` / `view`. For this crate size, that is a sensible architecture.
- `FrameSnapshot` is a strong design decision. It gives mouse handling a stable, testable geometry contract instead of forcing hit testing to inspect widgets directly.
- The `DomainState`, `UiState`, and `NotificationState` split is a good foundation even though the UI slice still carries too much widget detail.
- The project has good test coverage around view geometry, argv serialization, and interaction paths. That materially reduces architectural risk when refactoring.

## SOLID summary

- SRP: partially good. Module boundaries exist, but `UiState`, `form_editor`, and `ui::form` still carry mixed responsibilities.
- OCP: the weakest area. The action pipeline is centralized enough that new behaviors will usually require edits in multiple existing dispatch points.
- LSP: no material issues observed.
- ISP: acceptable overall, but `Runtime` is broader and more concrete than an ideal interface for the app layer.
- DIP: partially met. The runtime is abstracted, but the abstraction still leaks crossterm and ratatui; editor state also depends directly on a concrete widget type.

## Conclusion

This is a good, pragmatic architecture for an early library crate. The current structure is coherent and testable, and the codebase is not in a “needs redesign” state.

If the project grows, the highest-value refactors are:

1. move widget-specific editor state out of `UiState`
2. replace crossterm-facing runtime APIs with crate-local event/session abstractions
3. break `apply_action` and related interaction logic into smaller reducers

## Validation

- `cargo test -p clap-tui`
- `cargo clippy -p clap-tui --all-targets` (passes with warnings)
