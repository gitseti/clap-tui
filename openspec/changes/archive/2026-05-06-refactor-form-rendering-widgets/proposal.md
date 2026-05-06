## Why

`ui/form.rs` has grown into a boundary-crossing module that derives field display state, chooses visual styles, dispatches widget rendering, renders helper text, and hosts a large test suite. The current shape makes form rendering changes harder to review because widget-specific drawing code is coupled to validation, effective-value, selection, and layout concerns.

## What Changes

- Split form rendering into focused widget-oriented modules while preserving the existing public API and user-visible form behavior.
- Move shared form field geometry into an explicit layout boundary so responsive/adaptive field projection is no longer owned by `query/form.rs` or widget rendering code.
- Introduce an internal field render model that centralizes derived per-field state such as selected status, effective value display, validation text, default/source styling, required/editability state, block styling inputs, and widget kind.
- Move repeated-text, optional-value, compact controls, text-field rendering, and help rendering behind small module boundaries that consume the shared render model.
- Keep shared geometry and hit-testing behavior aligned with the existing `FrameSnapshot` and form layout projection; this change is a refactor, not a layout redesign.
- Do not change command parsing, argv serialization, input semantics, visual grammar, keybindings, mouse behavior, or public crate APIs.

## Capabilities

### New Capabilities
- None.

### Modified Capabilities
- `mvu-architecture-discipline`: Tighten the layout/rendering boundary so form layout projection is owned by an explicit layout mechanism and form widgets consume shared render models instead of recomputing geometry, domain, validation, and effective-value context inside widget-specific drawing code.

## Impact

- Affected modules include `crates/clap-tui/src/ui/form.rs`, the existing form layout projection helpers, and new sibling modules under the form UI/layout areas.
- Existing form rendering tests may move into more focused module-level test sections, but their behavioral assertions should remain equivalent.
- No dependency, public API, command parser, serialization, or runtime architecture changes are intended.
