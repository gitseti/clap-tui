## Context

Repeated-value editing spans multiple modules today: row geometry is computed separately in the renderer and mouse hit-testing, keyboard routing decides whether repeated editors consume `Up` and `Down`, and the form snapshot clips fields before rendering. That split has allowed three related regressions to drift apart: middle rows do not reserve an external control gutter for the remove button, repeated-row navigation swallows boundary arrows instead of falling back to form traversal, and clipped repeated editors fall back to a merged paragraph rendering that no longer matches their interactive model.

The change is small in scope but cross-cutting in implementation because it touches render layout, pointer hit-testing, keyboard dispatch, and the form editor’s row-navigation contract.

## Goals / Non-Goals

**Goals:**
- Make repeated-row geometry consistent between rendering and click hit-testing.
- Preserve the current repeated-row model while letting first/last-row `Up` and `Down` continue normal form traversal.
- Keep repeated editors visually decomposed into row widgets even when the field is partially clipped by scrolling.
- Add focused tests that lock down layout, boundary traversal, and clipped rendering behavior.

**Non-Goals:**
- Redesign the repeated editor into a different widget model.
- Change how occurrences are stored, serialized, or validated.
- Introduce partial-row rendering beyond the existing clipped field behavior.

## Decisions

### 1. Base repeated-row gutter reservation on visible controls, not only on the add button

Both the renderer and mouse hit-testing already share the concept of a right-side control strip, but the textarea width only shrinks for rows that show `+`. The fix should treat any visible row control as requiring external gutter space, with lone remove buttons centered in that gutter and last-row remove/add controls sharing it. This keeps layout and click geometry aligned and matches the intended visual language of row editors plus external controls.

Alternative considered: keep the current full-width textarea for middle rows and special-case paint the remove button over the border. Rejected because it preserves the overlap bug and keeps render and click geometry harder to reason about.

### 2. Let repeated-row navigation report boundary escape instead of swallowing first/last-row arrows

The keyboard controller currently routes repeated-field `Up` and `Down` into widget input unconditionally, while row navigation returns `Ignored` at the edges. The design should make edge arrows fall through to ordinary form navigation rather than terminate inside the repeated editor. The simplest implementation is to preserve row-local movement when another repeated row exists and explicitly trigger previous/next form selection when the editor is already at the first or last row.

Alternative considered: stop routing plain `Up` and `Down` to repeated widgets and require modifier keys for row movement. Rejected because it would regress the existing in-editor navigation model.

### 3. Continue row-based rendering for clipped repeated editors

Repeated fields should keep rendering by visible row widgets whenever the clipped input area still has space for one or more repeated rows. Falling back to a plain paragraph makes the field appear as one merged rectangle and breaks the connection between rows and their controls. The renderer should therefore keep the repeated-row path for repeated widgets even when the input rect is height-clipped, using the visible row count derived from the clipped area.

Alternative considered: preserve the paragraph fallback for all truncated inputs, including repeated editors. Rejected because repeated editors are semantically multi-row widgets whose affordances disappear in that fallback.

## Risks / Trade-offs

- [Keyboard fallback could double-advance if both widget and form handlers react] -> Keep repeated-row edge handling explicit in one path and add tests for first-row `Up` and last-row `Down`.
- [Render and click geometry could drift again] -> Mirror any gutter-rule changes in both `ui/form.rs` and `update/form.rs` and cover them with mouse-hit tests.
- [Clipped repeated editors may still look odd when fewer than three terminal rows are visible] -> Preserve current clipping limits but require that any fully visible repeated row retains its own boundary and controls.

## Migration Plan

No data migration or rollout step is required. The change is contained to local TUI interaction behavior and can ship behind the normal test suite.

## Open Questions

None at proposal time. The intended UX is already clear from the current bug report.
