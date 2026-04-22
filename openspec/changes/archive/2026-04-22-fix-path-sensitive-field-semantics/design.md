## Context

`clap-tui` already has the right validation boundary: `pipeline::derive()` serializes one canonical argv and validates that argv with the original clap `Command`. The bug class comes from a separate UI path that still treats `ArgModel.required` as current-path truth. In particular, `missing_required_arg_models()` can add missing-required field errors after `try_get_matches_from()` has succeeded.

The form model also projects ancestor-owned fields into descendant command views. That is useful, but ancestor metadata is not always effective on a selected descendant path. Command-level clap rules such as `subcommand_negates_reqs(true)` and `args_conflicts_with_subcommands(true)` can change whether an ancestor field is required, neutral, or conflicting for the current path.

## Goals / Non-Goals

**Goals:**
- Keep canonical argv as the single authoritative invocation model.
- Keep serialization diagnostics and clap validation as the only sources of invalidity.
- Add a path-sensitive field semantics layer for UI presentation.
- Make required badges, required placeholders, missing styling, label sizing, field activity, editability, and conflict presentation use derived field semantics instead of raw `ArgModel.required`.
- Preserve user-authored state, even when it conflicts with selected subcommands, so clap can report the real conflict.
- Use one shared semantics projection for rendering, layout, focus, hit-testing, and invalid-field navigation.

**Non-Goals:**
- Do not add a parallel validator for clap rules.
- Do not change canonical argv serialization or provenance beyond what field semantics consumers need.
- Do not silently discard user-authored ancestor field values to make a descendant path appear valid.
- Do not rely on hiding inherited fields from descendant forms as the primary fix for path-sensitive semantics.

## Decisions

### Add `FieldSemantics` to derived state

Extend `DerivedState` with semantics keyed by a stable field instance identity. The key should include at least `(owner_path, arg_id)` or an equivalent `FieldInstanceId` that matches the visible form projection. Raw `arg_id` alone is too weak unless implementation proves arg ids are globally unique across every projected field instance.

```rust
struct FieldInstanceId {
    owner_path: CommandPath,
    arg_id: String,
}

enum FieldVisibility {
    Visible,
    Hidden,
}

enum FieldActivity {
    Active,
    NeutralInherited,
    Disabled,
}

enum FieldConflictState {
    None,
    PotentialPathConflict,
    ActualValidationConflict,
}

struct FieldSemantics {
    id: FieldInstanceId,
    arg_id: String,
    owner_path: CommandPath,
    visibility: FieldVisibility,
    activity: FieldActivity,
    conflict: FieldConflictState,
    required_badge: bool,
    can_edit: bool,
    reason: Option<String>,
}
```

Rationale: visibility, activity, conflict, required presentation, and editability are separate axes. A field can be inherited but editable, inherited and neutral, path-conflicting but not currently invalid, or actually invalid because clap rejected the canonical argv. The identity is for the projected field instance, not only for the arg definition, because ancestor-owned fields can appear in descendant views.

Alternative considered: a single `FieldRelevance` enum. That is simpler, but it mixes ownership, editability, and conflict state and would be brittle as more clap path rules are represented in UI.

### Define field-state axes operationally

`Hidden` fields do not reserve layout space, are not focusable, are excluded from hit-testing, and are not correction targets for invalid-field navigation. `Visible` fields can participate in those projections subject to activity and editability.

`Disabled` means the current UI should not accept edits for that field. `PotentialPathConflict` means the field is visible in the current projection but authoring or retaining a value may conflict with the selected path. `ActualValidationConflict` means serialization diagnostics or clap validation projection already reported a real field-linked problem.

Disabling a field affects editability and presentation only; it does not clear authored state. Existing authored values remain in invocation state and continue to serialize unless the user explicitly removes them through an allowed interaction.

### Clap success is final for validation validity

If serialization succeeds and `Command::try_get_matches_from(canonical_argv)` succeeds, `ValidationState` must remain valid. Static metadata may influence `FieldSemantics`, but it must not add global summaries or field errors.

Rationale: this preserves the crate's existing authoritative argv contract and eliminates the architectural contradiction directly.

Alternative considered: keep `missing_required_arg_models()` as a secondary validation pass with more path rules. That still risks disagreeing with clap and is not acceptable as a validity source.

### Field semantics are presentation state, not validation

The semantics layer can mark an empty ancestor field as neutral, disabled, or potentially path-conflicting, and can suppress required badges when a selected subcommand negates ancestor requirements. It must not change `ValidationState.is_valid`.

Actual validation conflicts come from serialization diagnostics or clap validation projection. For example, an ancestor arg under `args_conflicts_with_subcommands(true)` can be neutral when untouched, but if the user authored a value it remains serialized and clap reports the conflict.

Field-level error styling may only come from serialization diagnostics or clap-validation projection. Field semantics may suppress or reshape presentation, but it must not create validation errors independently.

### Use semantics throughout the form pipeline

The semantics layer should be consumed by:
- `ScreenView` so UI modules get semantics with validation and effective values.
- form layout and label measurement so required markers do not reserve width when not effective.
- form rendering for badges, placeholders, field styles, inherited/disabled copy, and edit affordances.
- selector/navigation paths for first invalid field and focus decisions where semantics affects whether a field is actionable.
- update/input guards if disabled fields become non-editable.

Rationale: `visible_input_args_for_path()` and `effective_args_for_path()` influence more than drawing. A badge-only fix would leave focus order, invalid navigation, and placeholders inconsistent.

### Preserve declared metadata

`ArgModel.required` should remain as declared clap metadata extracted from `arg.is_required_set()`. It is useful as an input to semantics, but it is not authoritative for current-path UI semantics.

## Risks / Trade-offs

- [Risk] Field semantics may drift from clap if it attempts to model too many clap rules. -> Mitigation: semantics only affects presentation; invalidity stays clap-authored.
- [Risk] Disabled inherited fields could surprise users who expect to edit every visible field. -> Mitigation: keep the state explicit with reason text, and preserve user-authored values when they already exist.
- [Risk] Error localization from clap contexts remains best-effort. -> Mitigation: keep footer summaries clap-authored, localize inline errors only when the validation adapter can identify fields, and avoid inventing missing errors from metadata.
- [Risk] Touching layout, rendering, selectors, and update code is cross-cutting. -> Mitigation: add the semantics model first, migrate static required usages one surface at a time, and protect the invariant with pipeline tests.

## Migration Plan

1. Introduce the field semantics model and derive it inside `pipeline::derive()`.
2. Remove or constrain the post-clap-success missing-required pass so clap success cannot become invalid locally.
3. Replace static `arg.required` UI usages with `FieldSemantics.required_badge`.
4. Route form layout, rendering, focus, hit-testing, and invalid-field navigation through the shared semantics projection where relevant.
5. Add regression tests for subcommand-negated requirements and ancestor/subcommand conflicts.

Rollback is straightforward: the change is internal and can be reverted to the prior metadata-driven UI behavior. No data migration is required.

## Open Questions

- Should disabled ancestor fields be fully non-interactive, or selectable with an explanation and a path-change prompt?
- Should neutral inherited sections remain expanded by default when every field in the section is inactive?
- Can the current clap version expose richer structured arg ids for error localization than the existing context and diagnostic parsing path uses?
