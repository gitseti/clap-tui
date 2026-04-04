## 1. Launcher foundations

- [x] 1.1 Add a canonical typed launcher API in `crates/clap-tui` for derive-based root parser types and wire it into `crates/clap-tui/src/lib.rs`
- [x] 1.2 Add launcher support code for synthetic subcommand insertion, conflict checks, launch detection, and TUI-facing command filtering
- [x] 1.3 Extend `crates/clap-tui/src/error.rs` with explicit launcher setup failures for root conflicts and ambiguous synthetic attachment

## 2. Command-surface and TUI behavior

- [x] 2.1 Implement root launcher eligibility checks that reject existing `tui` subcommands or aliases and clap host grammars that would already accept or ambiguously consume `tool tui`
- [x] 2.2 Implement augmented-command handling so help, version output, and parse diagnostics come from the command surface that includes the synthetic `tui` launcher
- [x] 2.3 Update `crates/clap-tui/src/spec.rs` to exclude hidden subcommands from the TUI command tree and related command-derived views

## 3. Typed and macro launch flows

- [x] 3.1 Route the synthetic `tool tui` path through `TuiApp` in the canonical typed launcher and parse successful TUI output back into the same root parser type before calling the user handler
- [x] 3.2 Preserve ordinary non-TUI execution through the root typed parser and return success without invoking the user handler when the TUI is cancelled
- [x] 3.3 Add a new `crates/clap-tui-macros` workspace member and implement `#[clap_tui::main]` as a thin wrapper over the canonical typed launcher, including optional `config = path::to::fn`

## 4. Verification and documentation

- [x] 4.1 Add runtime behavior tests for the typed launcher core, including launcher conflicts, augmented help and diagnostics, successful typed TUI launch, cancellation, and non-TUI fallthrough
- [x] 4.2 Add `trybuild` coverage for valid macro usage, invalid signatures, and malformed or unsupported macro arguments
- [x] 4.3 Document the typed launcher as the canonical API and `#[clap_tui::main]` as convenience syntax, including `tool tui` behavior and v1 scope, in `README.md` and any relevant crate-level public docs
