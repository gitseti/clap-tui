## 1. Layout Boundary

- [x] 1.1 Add crate-level `layout::form` and wire it into the crate module tree without converting the existing `ui/layout.rs` screen-layout module.
- [x] 1.2 Move form field projection types and functions out of `query/form.rs` into `layout::form` while preserving existing behavior.
- [x] 1.3 Update `query/form.rs` to keep visible-argument, field ordering, section-heading, description-policy, and semantic form helpers while delegating responsive label/input/description projection math to the layout boundary.
- [x] 1.4 Update `frame_snapshot.rs`, navigation, update, and form rendering call sites to consume the explicit form layout projection.
- [x] 1.5 Preserve geometry tests that prove rendering, scrolling, hit testing, repeated fields, and snapshot population agree on the same projected field bounds.

## 2. Form Module Structure

- [x] 2.1 Convert `crates/clap-tui/src/ui/form.rs` into a `crates/clap-tui/src/ui/form/` module while preserving the existing `ui::form::populate_layout` and `ui::form::render_form` call paths.
- [x] 2.2 Create focused form UI modules for field orchestration, text rendering, repeated values, optional values, compact controls, and help rendering.
- [x] 2.3 Keep helper visibility narrow with `pub(super)` or private items unless an existing cross-module caller requires `pub(crate)`.

## 3. Field Render Model

- [x] 3.1 Add an internal `FieldRenderModel` that captures common per-field render state derived from `ScreenView`, `UiState`, `OrderedArg`, `FormFieldLayout`, and `FrameSnapshot`.
- [x] 3.2 Move common value, validation, required/editable, selected, default/source, placeholder, dropdown-open, block, and text-style decisions into the field model construction path.
- [x] 3.3 Use borrowed references or `Cow<'a, str>` for text-like render model values, allocating only for derived display strings that cannot be borrowed.
- [x] 3.4 Update the field render loop to dispatch widget renderers with `FieldRenderModel` plus drawing-specific geometry inputs.
- [x] 3.5 Add or preserve focused tests proving the model preserves existing validation, default/source, required, selected-state, and borrowed/owned display-value decisions.

## 4. Widget Extraction

- [x] 4.1 Move single text and textarea rendering helpers into the text module and consume `FieldRenderModel` for shared render state.
- [x] 4.2 Move compact toggle, choice, counter, and compact-control-line helpers into the compact module and consume `FieldRenderModel` for shared render state.
- [x] 4.3 Move optional-value visual-state derivation and rendering into the optional-value module and consume `FieldRenderModel` for shared render state.
- [x] 4.4 Move repeated-value rendering, row textarea rendering, add/remove controls, and visible-row clipping into the repeated module and consume `FieldRenderModel` for shared render state.
- [x] 4.5 Move field help text, section heading line, widget help hints, required prompts, and help overlay rendering into the help module without changing displayed text.

## 5. Test Re-Home And Regression Coverage

- [x] 5.1 Create a local form UI `test_support` submodule for shared builders, fixtures, and render helpers used by the extracted form tests.
- [x] 5.2 Re-home existing `ui/form` tests beside the extracted modules they validate while preserving behavioral assertions.
- [x] 5.3 Keep top-level form rendering regression tests for layout population, clipped rendering, help overlay, section headings, and mixed widget rendering.
- [x] 5.4 Ensure repeated-value, optional-value, compact-control, text-field, validation, inherited-field, and default/source styling tests still cover the same scenarios as before the split.

## 6. Verification

- [x] 6.1 Run `cargo fmt --check`.
- [x] 6.2 Run `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
- [x] 6.3 Run `cargo test --workspace --all-targets --all-features`.
- [x] 6.4 Confirm no public API, command parsing, argv serialization, input semantics, keybindings, mouse behavior, or visual grammar changes were introduced.
