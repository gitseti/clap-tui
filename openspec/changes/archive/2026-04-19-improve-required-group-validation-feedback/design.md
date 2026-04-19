## Context

The TUI already has a clear validation pipeline: clap parsing feeds `ValidationState`, and the form, footer, and navigation layers render or act on that derived state. The current defect is concentrated in validation adaptation for `ErrorKind::MissingRequiredArgument` when clap reports a required `ArgGroup` rather than an individually required argument.

Today the adapter rebuilds missing-required summaries from `ArgModel.required` and populates field errors from the same source. That works for individually required fields, but not for required groups such as `<--fast|--safe>`. In that case clap correctly rejects the command, but the adapter can produce an invalid state with no actionable summary and no field-level errors. Existing UI paths therefore have little or nothing to show even though they are already wired to consume `ValidationState`.

The change is cross-cutting enough to merit design because it touches parser adaptation, validation semantics, and visible correction UX. It also needs a clear scope boundary so a fix for actionable validation feedback does not accidentally grow into a broader metadata-model project.

## Goals / Non-Goals

**Goals:**

- Preserve clap-required group semantics when translating parser failures into `ValidationState`.
- Make missing required groups actionable through existing UI paths: inline errors, footer summary, and focus-first-invalid behavior.
- Define a stable UX contract for required group failures so tests can assert one intended outcome.
- Add targeted regression coverage at the adapter and UI layers.

**Non-Goals:**

- Introduce a new synthetic group widget or container in the form layout.
- Redesign the general validation UI beyond what is needed for required groups.
- Add resting-state "required group" affordances before validation is triggered.
- Perform a full metadata-model expansion unless later work explicitly targets pre-submit group affordances.

## Decisions

### Use validation adaptation as the primary fix point

The main functional defect sits in `pipeline/validation.rs`, where missing-required handling currently depends on per-arg `required` metadata. Fixing the adapter keeps the existing renderer, footer, and navigation behavior intact and reuses the current `ValidationState` contract rather than creating a special case downstream.

Alternative considered: add renderer-only fallback behavior for group failures. This was rejected because the renderer does not have reliable access to clap error context, and it would duplicate validation interpretation logic away from the validation pipeline.

### Treat required-group misses as field-linked errors on all member controls

For the minimal fix, required-group failures will populate field errors on each visible member control of the missing group and will focus the first visible member as the next correction target. This matches the current UI architecture, which reasons in terms of field ids rather than synthetic group containers.

Alternative considered: attach errors to a synthetic group-level field or container. This may become attractive in a future form model, but it would require new layout, hit-testing, and navigation concepts that are unnecessary for restoring actionable feedback now.

### Generate a stable summary for required-group misses

Missing required groups should produce a deterministic summary message such as `Choose one of: --fast, --safe` instead of relying on a local builder that only understands individually required args. The summary should be derived from clap-reported composite references or equivalent decoded group membership so it remains aligned with the same condition that produced the invalid state.

Alternative considered: preserve clap's raw diagnostic text verbatim. This was rejected for the minimal fix because the raw composite syntax (for example `<--fast|--safe>`) is parser-centric and less readable than a user-facing selection prompt.

### Keep resting-state grouped-required affordances out of scope

The current form affordance logic keys off per-arg `required` metadata. Grouped-required choices cannot participate in that logic without additional model/spec support for required-group metadata. That is a legitimate enhancement area, but it is separate from restoring actionable validation after clap reports a failure.

Alternative considered: extend the model in the same change. This was rejected to keep the first fix narrowly focused on broken validation feedback and to avoid mixing corrective work with a broader UI metadata project.

## Risks / Trade-offs

- [Composite ref parsing may be brittle if clap changes formatting] → Prefer decoding from clap context conservatively, add tests for observed forms such as `<--fast|--safe>`, and keep a readable summary fallback.
- [Marking every member invalid may feel visually heavier than a shared group message] → Document the UX choice explicitly and keep the summary text concise so the feedback still feels coordinated.
- [Some required groups may include fields not currently visible in the active form] → Limit field-linking to visible/effective members and focus the first visible invalid member while preserving summary feedback.
- [Future work may want synthetic group-level UI] → Keep the adapter changes localized so a later model enhancement can replace or refine the field mapping without undoing parser semantics work.

## Migration Plan

1. Update validation adaptation for missing required groups and add adapter-level tests.
2. Update rendered validation expectations and navigation tests to assert the required-group UX contract.
3. Verify that Run blocking, footer summaries, and correction focus all reflect the same derived validation state.
4. Land without migration or rollback requirements beyond reverting the code change if regressions appear.

## Open Questions

- Should the inline help text on each member field reuse the same shared message verbatim, or should the footer carry the full wording while fields use a shorter variant?
- How should mixed cases be summarized when a command has both individually missing required fields and one or more missing required groups?
