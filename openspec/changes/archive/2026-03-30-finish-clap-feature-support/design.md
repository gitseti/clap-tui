## Context

`clap-tui` already completed the architectural work that was previously blocking broader clap support: the extracted spec carries rich metadata, form state is invocation-oriented rather than command-local, preview argv is built across the selected command path, and validation is driven by `Command::try_get_matches_from`.

The remaining gaps are concentrated in the compatibility layer that still projects the richer model back into simplified UI conventions. In particular, multi-value and repeated inputs are still flattened for editing, the serializer still emits a simplified token shape for many cases, some remaining clap parser and source metadata are not yet extracted into the local model, and presentation code ignores much of the metadata that the spec now extracts.

This design therefore treats the current foundation as stable and focuses on replacing the remaining lossy adapters with occurrence-aware editing, higher-fidelity argv synthesis, and better UI projection of clap metadata.

## Goals / Non-Goals

**Goals:**

- Preserve the current architecture and build on the existing spec, input-state, argv, and validation foundations.
- Eliminate newline-encoded and single-value compatibility flows for clap features that require richer editing semantics.
- Make preview argv and validation reflect clap-sensitive token shape closely enough for real-world CLIs.
- Expand the extracted spec only where the remaining clap surface still depends on missing metadata.
- Surface validation, display metadata, subcommand metadata, and value sources in the form so complex commands remain understandable.
- Keep the work shippable in small slices with tests after each slice.

**Non-Goals:**

- Rewriting the event loop, rendering pipeline, or controller architecture.
- Supporting every clap builder knob as a first-class TUI feature in one pass.
- Reimplementing clap parser rules in custom UI logic when clap already provides authoritative validation.
- Changing runtime dependencies unless a later slice shows that the current stack cannot support the required editing interactions.

## Decisions

### 1. Keep the current foundation and retire compatibility paths incrementally

The project should not perform another model rewrite. The codebase already has the right ownership boundaries for spec extraction, invocation state, and clap-backed validation. The remaining work should replace compatibility-oriented projections in targeted areas rather than reopening core architecture.

Alternatives considered:

- Rewriting the form model again around a new widget abstraction.
  Rejected because the current model is already expressive enough and another rewrite would mostly recreate existing capabilities.
- Leaving the compatibility layer in place and adding special cases.
  Rejected because it would keep newline encoding and flattened occurrences as permanent debt.

### 2. Make occurrence-aware input the editing truth

Repeated occurrences and multiple values in one occurrence must remain distinguishable through editing, storage, serialization, and preview. Editing widgets should operate directly on occurrence-aware state rather than converting through newline-delimited text except as a short-lived compatibility fallback for unsupported cases.

This applies to:

- append-style options and positionals
- multi-value enums
- count flags
- optional-value flags with default-missing semantics
- inherited global values shown from their owning command

Alternatives considered:

- Keeping a single text editor and encoding richer inputs in text conventions.
  Rejected because it loses occurrence shape and makes count or hybrid flags awkward.

### 3. Centralize clap-sensitive token rules in the serializer layer

All token-shape behavior that can change clap parsing should live in argv synthesis, driven by extracted metadata and occurrence-aware input state. Validation should continue to use the exact same argv that preview and Run use.

This serializer-focused slice begins by extending extracted metadata where the current spec still lacks parser-relevant state. It must cover:

- grouped values in one occurrence versus repeated occurrences
- `--opt=value`
- delimiter and terminator behavior, including `dont_delimit_trailing_values`
- positional ordering and trailing capture
- parser-affecting command options such as required subcommands, argument/subcommand conflicts, external subcommands, `allow_missing_positional`, and `subcommand_precedence_over_arg`
- clap convenience shorthands that change parse shape, such as `raw(true)`
- default-missing and conditional-default metadata needed to explain source behavior accurately

Alternatives considered:

- Handling token-shape differences in per-widget code.
  Rejected because parsing fidelity belongs at the argv boundary, not in the UI.

### 4. Treat metadata fidelity as a UI projection problem, not a parser problem

The spec already extracts much of the metadata needed for understandable forms. The remaining work should extend extraction where necessary and then use that metadata directly instead of continuing to render only the simplest labels and orderings.

The first presentation improvements should use existing metadata for:

- `display_label` instead of only `display_name`
- clap `display_order`
- help headings
- long help
- visible aliases where they improve discoverability
- value names and choice-level metadata where they improve editor clarity
- subcommand ordering, labels, aliases, and headings in the sidebar or command tree
- value source badges and placeholders for env, defaults, default-missing values, and conditional defaults

Alternatives considered:

- Leaving help and metadata fidelity to the raw help overlay only.
  Rejected because the form itself is the primary interaction surface.

### 5. Prioritize the clap surface that matters to a TUI

The change should focus on clap features that affect editability, parser correctness, or user comprehension in an interactive form. Some clap options can remain lower priority because clap-backed validation already preserves correctness even if the TUI does not expose special affordances for them.

High priority:

- append, count, optional-value flags, grouped occurrences
- `require_equals`, delimiters, trailing capture, raw capture
- external subcommands and command/subcommand parse boundaries
- defaults, env, conditional defaults, help structure

Lower priority unless a concrete need appears:

- inference-oriented parser conveniences
- error-tolerant parser modes
- niche command behaviors that do not change the interactive editing model

## Risks / Trade-offs

- [Serializer fidelity touches many edge cases] -> Add focused tests that assert both argv shape and clap acceptance for every new slice.
- [Occurrence-aware widgets can sprawl into a UI rewrite] -> Keep widgets narrow and metadata-driven; do not redesign unrelated controls.
- [Conditional defaults may be harder to surface than plain defaults] -> Start with explicit extraction work and define a graceful fallback when clap exposes only partial detail.
- [External subcommands can introduce awkward UX] -> Start with a minimal raw-entry path and improve it only if the initial interaction proves insufficient.

## Migration Plan

This change can land incrementally with no user-facing migration step. Each slice should preserve existing supported behavior while replacing one compatibility path at a time:

1. render existing field-level validation in the form
2. introduce occurrence-aware editors for repeated and hybrid inputs
3. expand extracted metadata and upgrade argv synthesis for clap-sensitive token shape
4. project display, subcommand, and source metadata into the form, sidebar, and preview

Rollback is straightforward because each slice can be reverted independently if a regression appears.

## Open Questions

- How much structured support should external subcommands receive beyond a minimal raw token entry path?
- If conditional-default metadata is only partially extractable from clap, what fallback explanation is acceptable in the UI without overstating certainty?
- Should multi-value free-text editing use a list/chip metaphor, a row editor, or a hybrid text-plus-list interaction for keyboard-first workflows?
