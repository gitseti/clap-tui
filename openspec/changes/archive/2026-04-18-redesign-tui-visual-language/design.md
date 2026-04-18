## Context

`clap-tui` already has the product structure needed for the target direction: a navigable command tree, a dense form workspace, a generated command preview, and compact footer actions. The current rendering is technically coherent, but it presents those surfaces with relatively even weight, similar panel chrome, and a restrained theme, so the interface reads as a capable default TUI rather than a deliberate showcase experience.

The redesign target is not a new information architecture. It is a stronger visual language: darker shell surfaces, more pronounced workflow hierarchy, a clearer command header, more expressive sidebar and preview treatments, lighter section framing, and a denser CLI-native control family. The codebase is well positioned for that work because theme tokens already live in `config.rs`, shared styling is centralized in `ui/styles.rs`, and the major surfaces are isolated in dedicated renderers.

The main constraint is that this remains a Ratatui application rendered inside ordinary terminals. We can redesign hierarchy, color roles, spacing rhythm, borders, titles, and widget affordances, but we cannot rely on blur, opacity, or emulator-specific glow effects to carry the result.

## Visual Reference

This change includes a non-normative visual reference asset at `openspec/changes/redesign-tui-visual-language/modern_terminal_cli_interface.png`.

Use this image to guide:
- overall surface hierarchy and contrast
- separated accent roles for control chrome versus command and result emphasis
- sidebar active-row emphasis
- visible branch-state and active-row affordances in navigation
- breadcrumb-style command header treatment
- lightweight section framing
- aligned label and control columns for dense forms
- compact single-line CLI-native control styling
- preview prominence and footer action balance

The image is a design reference, not a pixel-perfect acceptance target. The implementation should preserve terminal-native constraints and the repo's normative spec requirements.

## Goals / Non-Goals

**Goals:**
- Make the TUI feel like an intentional terminal product rather than a generic dark-panel application.
- Preserve the current sidebar -> form -> preview -> footer workflow and existing architecture.
- Create a stronger hierarchy between shell surfaces, active workflow surfaces, and passive chrome.
- Introduce a shared form-control grammar so different widget types feel related while still advertising their interaction model.
- Keep the redesigned hierarchy readable in compact layouts and across supported theme presets.

**Non-Goals:**
- Replacing the MVU/runtime structure or introducing a new navigation model.
- Adding permanent panes, tabs, or dashboard-style summary surfaces.
- Chasing pixel-perfect parity with screenshot-only effects that depend on terminal emulator rendering.
- Turning the redesign into a large public API overhaul beyond additive theme and style semantics.

## Decisions

### 1. Treat the redesign as a visual-system refresh, not a layout rewrite

The existing screen composition is already close to the target structure. The redesign should therefore preserve the current panel arrangement and focus on hierarchy, chrome, spacing, and component language. This keeps the change incremental and avoids destabilizing the interaction model just to achieve a different aesthetic.

Alternatives considered:

- Rebuild the layout around a new panel system.
  Rejected because the current composition already supports the desired workflow.
- Limit the work to a theme swap.
  Rejected because the gap is not only palette; it also includes control grammar, panel identity, and section framing.

### 2. Establish a stronger surface hierarchy with calmer shell chrome and louder workflow surfaces

The current TUI gives the sidebar, workspace, preview, and footer similar visual weight. The redesign should separate them into roles:

- shell surfaces: outer frame, empty background, passive panel edges
- navigation surface: sidebar and search
- editing surface: workspace header and form
- result surface: preview
- action/status surface: footer and toast

This requires expanding semantic theme roles and centralizing them in shared style helpers so renderers ask for intent instead of composing local color logic.

The reference image also suggests that one accent family should not do every job. The redesign should reserve one accent treatment for UI chrome and interactive controls, and a distinct accent treatment for command identity, breadcrumb emphasis, and generated invocation output. That separation is what lets the preview and command header feel like the semantic payoff instead of just another highlighted widget.

Alternatives considered:

- Increase contrast equally across all panels.
  Rejected because it would make the whole screen louder without creating a clearer focal path.
- Remove most chrome entirely.
  Rejected because the app still benefits from panel boundaries and titles in dense layouts.

### 3. Promote command orientation into a deliberate header system

The main workspace should expose the selected command path and description as a purposeful header region rather than relying mostly on a panel title. The selected path should read like a breadcrumb or command trail, and the command description should sit immediately below it. This creates the same “you are editing this command” confidence as the target screenshot without adding a new pane.

Section headers inside the form should follow the same approach: lightweight labels with divider rules, not extra bordered containers.

Alternatives considered:

- Keep orientation exclusively in panel titles.
  Rejected because titles alone are too subtle for the target visual direction.
- Add a dedicated context panel above the form.
  Rejected because it spends terminal height better used by the form itself.

### 4. Define one CLI-native control family with type-specific variants

The redesign should make all controls feel like members of one system while preserving their different interaction models. Text fields, dropdowns, counters, toggles, optional values, and repeated values should share:

- an aligned command-label column and control column in dense forms
- a compact, mostly single-line row rhythm
- consistent label/value/help ordering
- consistent container weight
- compact metadata badge placement
- predictable affordance placement

Within that shared family, each type should still advertise itself clearly:

- toggles read like switches
- counters read like steppers
- choice inputs read like pickers
- repeated values read like compact chips

Alternatives considered:

- Normalize every control into the same box treatment.
  Rejected because it hides interaction differences.
- Keep optimizing each widget independently.
  Rejected because that is what produces the current fragmented feel.

### 5. Treat the preview as the payoff surface and the footer as keycap-style utility chrome

The preview should visually read as the output of the user’s editing work, not as another passive panel. It should therefore carry stronger title treatment, more explicit command syntax emphasis, and a clearer copy affordance. The footer should become calmer utility chrome with strong primary and secondary action chips that read like compact keycaps, while low-priority hints stay visually subordinate.

Alternatives considered:

- Move more feedback into the footer.
  Rejected because the preview is the natural place for result emphasis.
- Merge preview and footer visually.
  Rejected because result and utility surfaces serve different roles.

### 6. Preserve compact mode by collapsing decorative weight before collapsing semantic identity

Compact layouts should still look like the redesigned product. The implementation should therefore reduce row cost and chrome density before removing the cues that make the screen feel intentional. In practice this means lighter headers, reduced preview height, and stricter footer truncation while preserving recognizable navigation, result, and action surfaces.

Alternatives considered:

- Let compact mode fall back to the old neutral styling.
  Rejected because it would make the redesign inconsistent across normal terminal sizes.
- Preserve all roomy chrome in compact mode.
  Rejected because it would crowd the form.

## Risks / Trade-offs

- [A more opinionated style could reduce perceived neutrality] → Keep the redesign driven by semantic hierarchy and test readability in all built-in presets.
- [Control-family unification could blur widget differences] → Preserve type-specific affordances within the shared structure.
- [The redesign could become screenshot chasing] → Anchor decisions to terminal-native constraints and spec language rather than emulator-only effects.
- [Compact mode could lose too much of the new character] → Define compact-specific requirements for header, preview, and footer identity.
- [Theme token growth could increase maintenance cost] → Add only semantic roles that are reused across multiple renderers.

## Migration Plan

1. Expand the proposal-era specs so the redesign is captured as a user-visible contract instead of a loose visual idea.
2. Extend theme semantics and shared style helpers to express the new hierarchy before touching individual renderers.
3. Rework shell surfaces: outer frame, sidebar, workspace header, preview, and footer.
4. Redesign form sections and widget families on top of the new shared style vocabulary.
5. Tune compact-layout behavior so the redesigned surfaces degrade cleanly in smaller terminals.
6. Refresh renderer and interaction tests across roomy and compact layouts, then iterate on visual polish within the locked contract.

Rollback remains straightforward because the work is localized to theme tokens, style helpers, and renderer modules.

## Open Questions

- Whether the redesign should ship as an evolution of the default theme or introduce a new built-in preset first.
- Whether rounded borders remain part of the final chrome vocabulary or should be reduced on some surfaces.
- Whether the workspace breadcrumb should stay in the panel title area or move fully into the header content region.
