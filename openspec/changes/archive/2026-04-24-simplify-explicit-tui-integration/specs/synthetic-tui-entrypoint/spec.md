## REMOVED Requirements

### Requirement: Derive-based CLIs have a canonical typed synthetic TUI launcher
**Reason**: 0.1.0 no longer centers on launcher interception or synthetic command injection. The primary integration model is an explicit application-owned dispatch branch that calls `Tui::<T>::run()`.
**Migration**: Define a normal `tui` subcommand in the application clap tree, match that variant during ordinary dispatch, and call `Tui::<T>::run()` explicitly.

### Requirement: `#[clap_tui::main]` is convenience syntax over the typed launcher
**Reason**: Macro support is out of scope for the simplified 0.1.0 release surface, and the proc-macro crate is no longer part of the core integration story.
**Migration**: Replace `#[clap_tui::main]` usage with an explicit clap parse plus a `Command::Tui` match arm that calls `Tui::<T>::run()`.

### Requirement: Help and parse diagnostics reflect the augmented launcher surface
**Reason**: `clap-tui` no longer augments the outer clap command surface in the primary 0.1.0 model. Outer help and parse behavior remain owned by the application's own clap configuration.
**Migration**: Treat outer help, completion, and parse diagnostics as ordinary clap behavior for the application's real command tree, including the user-defined `tui` subcommand.

### Requirement: TUI launch remains bound to the originating parser type
**Reason**: The typed binding remains important, but it now belongs to the explicit `Tui::<T>::run()` surface rather than to a synthetic launcher capability.
**Migration**: Use `Tui::<T>::run()` directly for typed TUI execution and read the typed-return contract from the direct entrypoint capability instead of the launcher capability.

### Requirement: Synthetic launcher attachment rejects conflicting or ambiguous roots
**Reason**: Removing synthetic launcher attachment from the 0.1.0 story eliminates the need for launcher-specific grammar conflict rules.
**Migration**: Define any `tui` entrypoint explicitly in the application's own clap schema and rely on normal clap validation for outer command conflicts.

### Requirement: The synthetic launcher is hidden from the rendered TUI command tree
**Reason**: Launcher-specific render hiding is no longer part of the 0.1.0 public story because launcher injection itself is out of scope.
**Migration**: If an application needs a different rendered command tree, it may shape the `clap::Command` it passes into the TUI layer without relying on a general launcher-hiding feature.
