## Why

`clap-tui` now has the core foundations for richer clap support, but the user-facing experience still falls short for many real-world CLIs. The remaining gaps are concentrated in how the TUI edits repeated and hybrid arguments, how faithfully it reconstructs argv for clap's parser, and how clearly it presents clap metadata such as defaults, environment sources, aliases, and help structure.

Finishing this work now is valuable because the crate already has the necessary architectural base: rich extracted metadata, invocation-oriented input state, and clap-backed validation. The next step is to turn that foundation into end-to-end feature coverage that makes complex clap commands practical to drive from the TUI.

## What Changes

- Add occurrence-aware editing for repeated values, multi-value enums, count flags, optional-value flags, and inherited global arguments.
- Preserve argv shapes that matter to clap parsing, including grouped values per occurrence, `--opt=value`, delimiter-driven forms, trailing positional semantics, parse-boundary rules, and external-subcommand edge cases relevant to a TUI.
- Expand the remaining extracted clap metadata needed for parser fidelity and source fidelity, including parser-edge command settings, default-missing behavior, and conditional-default metadata.
- Improve metadata and source fidelity in the TUI so forms and sidebars reflect clap display order, headings, aliases, long help, value names, and value sources such as defaults, environment variables, default-missing values, and conditional defaults.
- Keep clap as the source of truth for parser validation while making those validation results understandable directly in the form.

## Capabilities

### New Capabilities

- `clap-occurrence-editing`: Editing flows for repeated, counted, optional-value, and inherited-global arguments without newline-encoded compatibility hacks.
- `clap-argv-fidelity`: Preview and run argv generation that preserves clap-relevant occurrence, token-shape, and command-edge semantics.
- `clap-metadata-fidelity`: TUI presentation that reflects clap argument, subcommand, and value-source metadata closely enough for complex commands to remain understandable.

### Modified Capabilities

None.

## Impact

- Affected code will span `crates/clap-tui/src/spec.rs`, `crates/clap-tui/src/input.rs`, `crates/clap-tui/src/form_editor.rs`, `crates/clap-tui/src/argv_serializer.rs`, `crates/clap-tui/src/pipeline/`, `crates/clap-tui/src/query/form.rs`, and `crates/clap-tui/src/ui/`.
- This change relies on existing clap 4.x metadata and validation behavior rather than adding new runtime dependencies.
- The work is expected to increase test coverage around serializer fidelity, editing interactions, validation rendering, and parser edge cases.
