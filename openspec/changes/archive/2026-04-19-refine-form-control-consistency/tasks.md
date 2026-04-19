## 1. Form Layout Model

- [x] 1.1 Update form metrics and frame-snapshot geometry to support stacked label-plus-metadata rows where badges render beneath the option name.
- [x] 1.2 Replace boxed section rails and caps with heading-plus-rule section framing and make section-boundary visibility stable under form clipping and scrolling.

## 2. Shared Control Rendering

- [x] 2.1 Refactor form rendering so flags, counters, dropdown-backed fields, and optional-value fields use the shared textarea-like control family while preserving their current click and keyboard semantics.
- [x] 2.2 Centralize value-tone selection so default-derived state stays muted and user-entered values remain in the primary text treatment across relevant widgets.
- [x] 2.3 Keep label, control, help, and metadata alignment coherent after the stacked-badge and shared-control changes.

## 3. Search Focus Feedback

- [x] 3.1 Update sidebar search rendering so a focused empty `Search commands` field clears its placeholder text and shows a visible cursor position.
- [x] 3.2 Preserve existing keyboard and pointer search behavior while making the focused search field render as an editable state.

## 4. Regression Coverage

- [x] 4.1 Update renderer and layout tests for lightweight section framing, clipped section-heading behavior, stacked metadata badges, and muted default-derived values.
- [x] 4.2 Add or refresh interaction tests proving unified controls keep their existing toggle, dropdown, stepper, and optional-value behavior and that focused search shows the new editable treatment.
