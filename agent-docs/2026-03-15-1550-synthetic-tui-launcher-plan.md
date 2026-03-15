# Synthetic TUI Entrypoint Plan

## Summary

Add a macro-driven way to expose exactly one synthetic `tui` subcommand on derive-based `clap` CLIs.

This plan reflects the current repo state:

- the workspace currently contains only `crates/clap-tui`
- command/spec generation already skips hidden args, but not hidden subcommands

Target behavior:

- integration happens in one place: `#[clap_tui::main(...)]`
- only one synthetic `tui` entrypoint is supported
- by default, that entrypoint is attached at the CLI root, yielding `tool tui`
- v1 supports root attachment only
- `tui` is always the last command segment
- TUI config is provided directly on the macro via a config function path
- synthetic `tui` is visible in normal CLI help
- synthetic `tui` is hidden from the TUI command tree itself
- user code does not need a `Tui` enum variant or an `unreachable!` match arm

## User-Facing API

### Macro

Expose a single entrypoint macro:

- `#[clap_tui::main]`

Supported forms:

- `#[clap_tui::main]`
- `#[clap_tui::main(config = path::to::fn)]`

### Default behavior

```rust
#[clap_tui::main]
fn main(cli: Cli) -> anyhow::Result<()> {
    match cli.command {
        Commands::Build => println!("build"),
        Commands::Serve => println!("serve"),
    }
    Ok(())
}
```

This adds:

- `tool tui`

Defaults:

- `config` omitted => `TuiConfig::default()`

## Why `main` Still Owns It

The macro must own startup because the synthetic `tui` command does not exist in the typed `clap` parser.

The wrapper must:

- read raw argv
- augment the `clap::Command` with the synthetic `tui` subcommand
- parse against that augmented command
- preserve the augmented command as the authoritative clap help/version/error surface
- detect whether the invocation is the TUI entrypoint
- either start the TUI or fall back to the normal user handler

The wrapper should call clap directly and let clap terminate directly for help/version/usage output instead of routing parse failures through `TuiError`.

That logic must run before ordinary `Cli::parse()`.

## Config Hook

Config callback shape:

```rust
fn config() -> clap_tui::TuiConfig
```

Behavior:

- if `config` is omitted on the macro, use `TuiConfig::default()`
- if `config` is present, call it before starting the TUI
- in v1, the synthetic launcher does not override `config.start_command`

## Macro Contract

Support this function shape:

- synchronous free function
- exactly one parameter: the root parser type
- return type `Result<(), E>`
- `E: From<clap_tui::TuiError>`

Supported macro arguments:

- optional `config = path::to::fn`

Validation at macro expansion time:

- invalid function signature => compile error
- malformed macro arguments => compile error
- unsupported macro arguments => compile error

## Entrypoint Semantics

### Path rules

The synthetic launcher is attached at the CLI root in v1.

Rules:

- the root command must accept an unambiguous subcommand slot for synthetic insertion
- attaching is rejected when the unmodified root clap grammar already accepts `tool tui` as ordinary input
- existing real root `tui` subcommands and aliases are rejected
- external subcommands and trailing/raw positional capture that can consume arbitrary tail tokens are rejected
- `tui` is always synthetic and always terminal
- required positionals alone are not sufficient to reject insertion

## Runtime Flow

The generated wrapper around `main` should:

1. collect `std::env::args_os()`
2. build the root `clap::Command` from the typed parser
3. validate that the root command can accept the synthetic launcher:
   - fail if the root already defines a real `tui` subcommand or alias
   - fail if a probe parse of `tool tui` against the unmodified command succeeds as ordinary input
   - fail if the root uses external subcommands or trailing/raw positional capture that would make `tui` ambiguous
4. clone and augment the command by inserting a synthetic root `tui` subcommand
5. detect whether argv resolved to the synthetic root `tui` command
6. parse argv against the augmented command first and treat that parse as the authoritative clap result surface
7. if that augmented parse returns clap help/version or any other clap error, terminate through clap from the augmented command surface so usage and diagnostics include the synthetic `tui` entrypoint
8. if argv resolves to the synthetic `tui` launch:
   - compute `TuiConfig` from the macro-level config callback or default
   - hide the synthetic `tui` command in the command passed to `TuiApp`
   - run `TuiApp::from_command(render_command).with_config(config).run()`
   - if the TUI is cancelled, return `Ok(())`
   - otherwise parse the returned argv into `Cli` and let clap terminate directly on parse failure
   - call the user handler directly with the parsed `Cli`
9. otherwise:
   - parse the original argv into `Cli` with the typed parser instead of reconstructing `Cli` from augmented matches
   - let clap terminate directly if that typed parse fails unexpectedly
   - call the user handler directly

This keeps both launch paths consistent:

- clap help/version/diagnostics always come from the augmented command
- user handler errors remain the user’s own `E`
- only TUI runtime/setup failures are converted via `E: From<clap_tui::TuiError>`
- non-TUI execution may reparse argv with the typed parser after augmented-command validation succeeds

## Library Changes

### Workspace/package changes

Add a proc-macro crate as a new workspace member:

- `crates/clap-tui-macros`

Update the root `Cargo.toml` workspace members to include it.

Re-export the macro from `crates/clap-tui/src/lib.rs`.

### Macro support module

Add a doc-hidden support module in `clap-tui` exposing helpers for:

- validating whether the root command can safely accept the synthetic launcher without shadowing existing clap grammar
- injecting the synthetic root `tui` subcommand
- detecting whether `ArgMatches` ended on the synthetic `tui` path
- hiding the synthetic `tui` path from the command passed to `TuiApp`

### Hide `tui` inside the TUI

Update command/spec generation so hidden subcommands are excluded.

Required change:

- in `crates/clap-tui/src/spec.rs`, skip subcommands where `is_hide_set()` is true

This keeps the synthetic `tui` launcher out of:

- the rendered command tree
- the form/help views derived from the command model

### Public additions

Add:

- `#[clap_tui::main]`

Macro implementation notes:

- the proc-macro should resolve the runtime crate path robustly when `clap-tui` is renamed in `Cargo.toml`
- expansion should call the user handler directly rather than reusing `TuiApp::run_with_parser`
- expansion should reparse non-TUI execution with the typed parser rather than assuming `Cli` can be rebuilt from augmented `ArgMatches`

### Error additions

Extend `TuiError` with explicit variants for:

- conflict with an existing real `tui` subcommand or alias
- invalid root launcher attachment when the unmodified grammar would treat `tui` as ordinary input
- launcher setup failures if needed by the support helpers

## Tests

### Macro tests

Use `trybuild` for:

- valid no-argument `#[clap_tui::main]`
- valid `config = path::to::fn`
- invalid `main` signature
- malformed macro arguments
- unsupported macro arguments

### Runtime helper tests

Test helper behavior without starting a real terminal:

- bare `#[clap_tui::main]` generates synthetic `tool tui`
- real existing `tui` conflict fails
- existing `tui` alias conflict fails
- root launcher placement under raw-argv/external-subcommand hosts fails before normal parsing
- root launcher placement fails when a probe parse shows `tool tui` is already accepted as ordinary input
- required-positionals-only hosts are allowed when clap still leaves room for an unambiguous root subcommand
- non-synthetic clap failures are reported from the augmented command surface, not the plain typed parser
- the TUI launch path returns user handler errors as the original `E`, not as `TuiError::Runner`

### Help tests

Verify:

- `tool --help` includes synthetic `tui` when attached at root
- help/version and ordinary clap diagnostics are served from the augmented command path, not the plain typed parser path

### TUI filtering tests

Verify:

- the synthetic `tui` subcommand is hidden from the rendered TUI command tree
- normal subcommands remain visible

## Implementation Order

1. add `crates/clap-tui-macros` to the workspace
2. add command-tree helpers for root-launcher eligibility checks, synthetic insertion, launch detection, and hidden-launcher filtering
3. update `crates/clap-tui/src/spec.rs` to skip hidden subcommands
4. implement macro expansion for `config`, including robust crate-path resolution when the dependency is renamed
5. make non-TUI execution reparse argv with the typed parser after augmented-command validation succeeds
6. ensure the wrapper uses augmented-command clap results directly for help, version, and diagnostics
7. re-export the macro from `crates/clap-tui/src/lib.rs`
8. add `trybuild` tests
9. add runtime helper, root ambiguity, alias, and TUI filtering tests
10. document the single-macro workflow in `README.md`

## Assumptions and Defaults

- first version supports derive-based `clap` CLIs only
- only one synthetic `tui` entrypoint is supported
- synthetic launcher name is always exactly `tui`
- `tui` must be terminal in the command path
- omitted `config` means `TuiConfig::default()`
- the macro path is synchronous only in the first version
- v1 supports root launcher attachment only
- synthetic `tui` is visible in normal CLI help but hidden inside the TUI
- this feature adds a synthetic launcher path only; it does not otherwise change the existing runtime API
