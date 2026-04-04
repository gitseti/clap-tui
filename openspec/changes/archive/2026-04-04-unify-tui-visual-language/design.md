## Context

`clap-tui` already has the right screen structure for a strong terminal UI: a command tree, a dense form workspace, a generated preview, and compact footer feedback. The problem highlighted by the review is not layout or architecture first. It is that the visual language has drifted across modules: too many meanings share one accent color, similar interaction states are rendered differently by surface, and control families no longer feel like they come from one system.

The repository is also carrying an active proposal named `clarify-tui-visual-semantics` that captured part of this problem well, but its scope is now too narrow for the current review. This replacement change should supersede that proposal and fold its requirements into a broader pass that unifies semantic tokens, control grammar, surface chrome, and cross-surface state rules without changing the app's overall layout model.

The implementation is already well positioned for this. Theme tokens live in `crates/clap-tui/src/config.rs`, shared style helpers live in `crates/clap-tui/src/ui/styles.rs`, and the main renderers are isolated in `ui/sidebar.rs`, `ui/form.rs`, `ui/preview.rs`, `ui/footer.rs`, `ui/dropdown.rs`, and `ui/toast.rs`. That makes it practical to improve consistency incrementally and preserve the current public API shape.

## Goals / Non-Goals

**Goals:**
- Make the TUI feel like one coherent product across sidebar, workspace, preview, footer, dropdown, and toast surfaces.
- Define stable semantic roles for focus, selection, success, error, warning-like metadata, and passive information so unrelated states stop competing visually.
- Establish one interaction-state grammar for focused, selected, hovered, open, inherited, default, required, and invalid states.
- Unify surface chrome and control-family treatment while preserving the current dense layout and terminal-native character.
- Improve hierarchy and scanability without relying on color alone.
- Strengthen validation summaries so they visually and behaviorally connect back to invalid fields.

**Non-Goals:**
- Replacing the current panel layout, navigation model, or command-tree structure.
- Turning the UI into a sparse dashboard or introducing new permanent panes.
- Building a heavyweight component framework on top of Ratatui.
- Introducing breaking public API changes beyond additive theme-token expansion.

## Decisions

### 1. Expand theme tokens into semantic roles instead of styling directly from raw accent colors

The current `Theme` model is too color-oriented and not semantic enough for the number of states the UI needs to express. This change should add a small set of explicit semantic roles for focus, selected-but-unfocused state, success feedback, warning-like metadata, passive metadata, and layered surfaces.

This lets presets remain coherent while giving renderers clearer intent. It also makes visual reviews and tests more stable because they can assert semantic behavior rather than widget-local color choices.

Alternatives considered:

- Keep the existing token set and differentiate states only with bold or border usage.
  Rejected because the review problem is semantic overlap, not just insufficient emphasis.
- Add a large token matrix for every widget and substate.
  Rejected because the current UI only needs a tighter semantic vocabulary, not a full design-system explosion.

### 2. Treat focus, selection, hover, open, and invalidity as composable axes owned by `ui/styles.rs`

The current renderers mix state logic locally. A field border, a selected row fill, a preview hover border, and a dropdown highlight all express related concepts but are currently composed in different places. This change should move as much branching as possible into shared helpers so renderers request intent like "focused invalid field", "active unfocused sidebar row", "warning-like metadata badge", or "primary result surface".

This keeps visual logic centralized and lowers the chance of future drift when one renderer evolves.

Alternatives considered:

- Let each renderer continue to branch locally.
  Rejected because that is the direct cause of the current inconsistency.
- Flatten all states into one strongest treatment.
  Rejected because it would make dense forms louder and harder to scan.

### 3. Standardize chrome and surface layering without changing screen geometry

The review does not call for more panels. It calls for more consistent panel identity. The implementation should therefore preserve current geometry while standardizing how surfaces communicate depth and importance: outer shell, sidebar/workspace panels, preview surface, overlays, inline input containers, and footer/action surfaces should all draw from the same chrome vocabulary.

In practice this means defining consistent expectations for border weight, title treatment, background layering, and compact-mode fallbacks rather than letting each module improvise.

Alternatives considered:

- Increase chrome everywhere with more borders and labels.
  Rejected because it would make the TUI heavier without necessarily making it clearer.
- Leave chrome as-is and focus only on text styling.
  Rejected because the screenshots show that surface identity is part of the inconsistency.

### 4. Define a shared control-family grammar instead of making every widget look unique

The form should still support different widget types, but they should read as related members of the same family. Text entry, choice pickers, counters, toggles, repeated-value editors, and optional-value states should share rules for label/value/help ordering, container treatment, affordance placement, and metadata positioning.

This does not mean every control becomes visually identical. It means each control type should advertise its interaction model through a predictable pattern that still belongs to the same system.

Alternatives considered:

- Normalize all controls into one generic input appearance.
  Rejected because counters, toggles, and choice pickers benefit from distinct affordances.
- Keep each widget independently optimized.
  Rejected because the current result already feels fragmented.

### 5. Keep validation linkage lightweight and driven by existing ordered layout metadata

The review asks for clearer error linkage, but this does not require a new error panel or complex navigation model. Existing field ordering and frame-snapshot metadata are already the right place to derive deterministic invalid-field ordering, next-target highlighting, and visual correspondence between footer summaries and fields.

That keeps the controller logic small and avoids coupling the footer renderer to form traversal rules.

Alternatives considered:

- Add a new dedicated validation surface.
  Rejected because it would cost terminal space and duplicate existing feedback channels.
- Leave the footer summary as plain isolated text.
  Rejected because it does not solve the long-form correction problem.

## Risks / Trade-offs

- [More semantic tokens increase preset maintenance] → Keep the additions small, semantic, and shared across presets.
- [Consistency work becomes subjective and stalls] → Anchor design choices to spec language and screenshot-backed review findings, then lock them in with renderer tests.
- [The UI becomes too loud] → Prefer hierarchy through layered surfaces, label/value contrast, compact badges, and border/title emphasis before adding saturated fills.
- [Control unification erases useful distinctions] → Standardize structure and state grammar while preserving widget-specific affordances.
- [Validation linkage adds incidental complexity] → Reuse existing ordering metadata and stop at deterministic linkage plus minimal navigation/highlighting behavior.

## Migration Plan

1. Extend `Theme` and built-in presets with semantic roles needed for the unified visual language.
2. Refactor `ui/styles.rs` into the single source of truth for state composition, surface chrome, and shared badge/action styling.
3. Update sidebar, form, preview, footer, dropdown, and toast renderers to consume the shared style vocabulary while preserving geometry.
4. Add ordered invalid-field linkage through existing frame-snapshot and update/controller helpers.
5. Expand renderer and interaction tests across supported presets and compact versus roomy layouts.
6. Remove the superseded `clarify-tui-visual-semantics` change after the replacement proposal is in place so only one active visual-language proposal remains.

Rollback remains straightforward because the work stays localized to existing theme and UI modules.

## Open Questions

- Whether warning-like metadata should use one semantic family for inherited, default, env, and implicit states or multiple badge variants built on the same base token family.
- Whether preview keyboard focus needs its own explicit visual treatment beyond hover and titled prominence in the first pass.
- Whether the first implementation should include a dedicated jump-to-next-invalid action or stop at deterministic ordering and synchronized highlighting.
