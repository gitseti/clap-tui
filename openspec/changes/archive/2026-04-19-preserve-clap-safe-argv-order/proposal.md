## Why

The current serializer can reorder command-local options and positionals into an argv shape that clap parses differently from the user's form selections. This is now visible in `kitchen-sink serve`, where a required positional can be moved behind a variable-arity option and get consumed as another option value instead of the intended positional.

## What Changes

- Update argv synthesis so the emitted token order preserves the semantic argument assignment the user made in the TUI when clap reparses it.
- Fix parse-sensitive ordering for commands that combine positionals with variable-arity options, especially when an option could greedily consume a later positional.
- Align preview and parse/run serialization so parse-affecting materialized defaults do not create a hidden mismatch between what the user sees and what clap validates.
- Add regression coverage for mixed positional/option shapes that currently serialize into clap-invalid argv despite valid UI state.

## Capabilities

### New Capabilities
- None.

### Modified Capabilities
- `clap-argv-fidelity`: Tighten argv synthesis requirements so emitted token order remains clap-safe and preserves the user's intended semantic assignment across preview, validation, and run.

## Impact

- Affected code: `crates/clap-tui/src/argv_serializer.rs`, `crates/clap-tui/src/pipeline/argv.rs`, and serializer/validation tests.
- Affected behavior: Preview, validation, and run output for commands that mix positionals with parse-sensitive option arities or default-materialized values.
- No public API additions are expected; this is a correctness fix to existing clap-first serialization behavior.
