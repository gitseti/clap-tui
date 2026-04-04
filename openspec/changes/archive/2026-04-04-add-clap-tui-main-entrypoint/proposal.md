## Why

`clap-tui` can already render and run typed `clap` applications, but library users still have to wire the TUI entrypoint manually through `TuiApp::from_factory(...)` and a follow-up execution call. That makes the happy path more verbose than comparable tools and leaves no built-in typed launcher API for derive-based CLIs to expose an ergonomic `tool tui` command.

Adding a first-class typed launcher now is valuable because the crate already has the core runtime and command-model seams needed to support it. Making that launcher a supported part of the library will improve ergonomics, give users a discoverable CLI entrypoint, and provide a safer typed execution path than the current unbound parser helper, while still allowing a macro convenience layer on top.

## What Changes

- Add a typed launcher API for derive-based `clap` root parser types that exposes a synthetic root `tui` subcommand such as `tool tui` as the canonical library entrypoint.
- Add a macro-based convenience entrypoint, `#[clap_tui::main]`, that wraps the typed launcher instead of being the fundamental implementation surface.
- Add library support helpers that validate whether the synthetic launcher can be attached safely, inject the synthetic subcommand into the clap command surface, detect TUI launches, and hide the synthetic launcher from the TUI command tree.
- Introduce a schema-bound typed launch flow so TUI execution parses back into the same root parser type that produced the TUI.
- Reject ambiguous or conflicting launcher setups, including real `tui` subcommands or aliases and clap grammars that would already treat `tui` as ordinary input.
- Document the new entrypoint, its supported signature, and its v1 scope and limitations.

## Capabilities

### New Capabilities

- `synthetic-tui-entrypoint`: The crate supports a canonical typed launcher API for derive-based CLIs plus a `#[clap_tui::main]` convenience entrypoint that add a synthetic root `tui` launcher while preserving consistent clap help, parsing, and TUI behavior.

### Modified Capabilities

None.

## Impact

- Affected code will likely include the workspace manifest, `crates/clap-tui/src/lib.rs`, `crates/clap-tui/src/app.rs`, `crates/clap-tui/src/spec.rs`, `crates/clap-tui/src/error.rs`, and a new proc-macro crate plus launcher support helpers.
- The public API surface will gain a canonical typed launcher API and a `#[clap_tui::main]` convenience entrypoint that share the same schema-bound launcher core.
- Test coverage will need to expand around launcher conflicts, augmented help output, non-TUI fallthrough behavior, and hiding the synthetic launcher from rendered TUI navigation.
