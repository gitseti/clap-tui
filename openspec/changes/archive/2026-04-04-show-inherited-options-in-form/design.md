## Context

The current preview is assembled from the full selected command lineage, so root and ancestor options can appear in the generated invocation even when the active form panel only shows the leaf command's local controls. The TUI already marks some fields as inherited, but that treatment is too subtle and, in at least some cases, suggests an edit model that is not aligned with the actual owner-scoped storage used by the app.

Inherited options therefore have two UX problems today:
- discoverability: users can see an option in the preview without seeing a corresponding control in the active form
- ownership clarity: an "Inherited" chip alone does not explain where the option comes from or what happens if the user edits it from a descendant command

## Goals / Non-Goals

**Goals:**
- Make all invocation-relevant inherited options visible and editable from the active descendant command form.
- Preserve a clear distinction between local command options and ancestor-owned options.
- Explain inherited option ownership and edit effect in language that matches the actual state model.
- Keep the selected command's own fields visually primary.

**Non-Goals:**
- Redesign the preview tokenization or shell-copy format.
- Introduce a new per-descendant override storage model for global options.
- Add new tabs or a separate inspector surface for inherited values.

## Decisions

### Show inherited options inline, but group them by owner section

The form should remain the single editing surface for the current invocation, so inherited options that affect the selected command belong in the same workspace. However, mixing them silently into the same flat list would obscure ownership. The chosen approach is:
- show local options in the primary section
- add secondary sections such as `Inherited from kitchen-sink` or `Inherited from workflow`
- keep inherited fields editable within those sections

This is better than a badge-only approach because the section heading communicates provenance before the user even focuses a field, while the field-level treatment can remain lightweight.

Alternatives considered:
- Badge only: too easy to miss and does not explain ownership at list-scan time.
- Separate tab or inspector: adds navigation overhead and makes the preview/form mismatch harder to reconcile.
- Fully merging inherited and local fields in one order: simplest implementation, but weakest clarity.

### Describe ownership and effect explicitly in helper copy

Selected inherited fields should explain both:
- which command owns the option
- what editing does in the current model

The copy should prefer truthful scope language such as `Defined on kitchen-sink. Editing here updates that shared option for commands in this lineage.` rather than implying a descendant-local override when the state is actually stored on the owner command.

Alternatives considered:
- Reuse the existing `Inherited` chip and current helper text: insufficiently explicit and potentially misleading.
- Encode all semantics into badges alone: too dense and hard to scan.

### Keep inherited fields visually secondary, not hidden

Inherited sections should use lighter headings and secondary framing so that local command-specific inputs remain the main focus. Within those sections, the current lightweight chips can still help, but they become supporting cues rather than the primary explanation mechanism.

Alternatives considered:
- Equal visual weight for local and inherited sections: makes it harder to tell what belongs to the selected command itself.
- Read-only inherited fields with "jump to owner" affordance: would preserve clarity, but would make the current form less capable than the preview implies.

## Risks / Trade-offs

- [Long forms become taller] → Keep local fields first and use compact owner headings so inherited visibility does not drown the leaf command.
- [Users may still confuse "global" with "local override"] → Use owner-path copy consistently in headings, badges, and selected-field help.
- [Implementation has to reconcile query order and owner grouping] → Centralize the form query output around explicit field provenance metadata rather than layering grouping in the renderer alone.
- [Future override support could change the semantics] → Phrase requirements in terms of truthful ownership/effect messaging so the copy can evolve with the state model.
