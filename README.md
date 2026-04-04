# clap-tui

`clap-tui` turns a `clap` command definition into an interactive terminal UI without replacing your existing CLI flow. You can keep `clap` as the source of truth, collect input in a terminal form, and still hand the final argv back to `clap` for typed parsing.

## Add `clap-tui` to your project

```toml
[dependencies]
clap = { version = "4.5", features = ["derive", "env"] }
clap-tui = "0.1.0"
```

Minimum supported Rust version: `1.85`.

## Quick start

```rust
use clap::Parser;
use clap_tui::{ParserLauncher, Theme, ThemePreset, TuiConfig};

#[derive(Debug, Parser)]
#[command(name = "tool")]
struct Cli {
    #[arg(long)]
    name: String,
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut config = TuiConfig::default();
    config.theme = Theme::from_preset(ThemePreset::HighContrastDark);

    ParserLauncher::<Cli>::new()
        .with_config(config)
        .run(|cli| {
            println!("Hello, {}!", cli.name);
            Ok::<_, clap_tui::TuiError>(())
        })?;

    Ok(())
}
```

`ParserLauncher` is the canonical typed entrypoint for derive-based CLIs. It augments the root command with a synthetic `tui` subcommand, so users can launch the form with `tool tui` while ordinary invocations still parse through `Cli`.

## Choose an entrypoint

- `ParserLauncher::<Cli>::run(...)` is the canonical typed launcher for derive-based CLIs.
- `#[clap_tui::main]` is convenience syntax over `ParserLauncher` with the same runtime behavior.
- `TuiApp::from_command(...)` is the untyped entrypoint for hand-built `clap::Command` values.
- `TuiApp::from_factory::<Cli>().run_with_parser(...)` runs the TUI directly and reparses the selected argv into the bound parser type.

## Supported customization seams

- `TuiConfig` controls theme, layout, key bindings, and initial command selection.
- `TuiApp::with_runtime(...)` and the exported `Runtime` event types support advanced runtime integration.
- `TuiConfig.start_command` lets you preselect a command path such as `build::release`.

Internal reducers, projections, render helpers, and other support modules are not stable extension points.

## Feature flags

- `mouse` is enabled by default and turns on mouse capture plus mouse-driven controls.
- `tracing` enables internal tracing instrumentation for applications that want to hook the crate into an existing tracing setup.

## Terminal expectations

- `clap-tui` is designed for interactive terminals that support raw mode and an alternate screen.
- The default `CrosstermRuntime` restores the terminal before returning, including when the user cancels.
- Mouse interactions require the default `mouse` feature.
- Validation stays inside the TUI flow: invalid forms show inline feedback instead of returning partially parsed values.

## Example guide

- `simple` shows the smallest derive-based setup with `#[clap_tui::main]`.
- `subcommands` shows the typed launcher flow for a CLI with nested subcommands.
- `kitchen_sink` demonstrates the untyped `TuiApp::from_command(...)` path and a wider range of `clap` surface area.

```bash
cargo run -p clap-tui --example simple -- tui
cargo run -p clap-tui --example subcommands -- tui
cargo run -p clap-tui --example kitchen_sink
```

## Synthetic `tui` scope

The v1 synthetic launcher is intentionally narrow:

- it attaches only at the CLI root, producing paths such as `tool tui`
- it appears in ordinary clap help and parse diagnostics
- it is hidden from the rendered TUI command tree itself
- it is rejected when the root command already defines a conflicting `tui` path or uses ambiguous host grammar such as external subcommands or trailing raw capture

## Theme presets

```rust
use clap_tui::{Theme, ThemePreset, TuiConfig};

let mut config = TuiConfig::default();
config.theme = Theme::from_preset(ThemePreset::Light);
```

Available presets:

- `ThemePreset::CalmDark` (default)
- `ThemePreset::HighContrastDark`
- `ThemePreset::Light`

## Controls

- `Tab` switches focus
- `Shift+Tab` cycles tabs
- `?` toggles the Help tab
- `/` opens command search
- `Ctrl+R` runs the current selection
- `Ctrl+Enter` runs when supported by the terminal
- `Ctrl+C` exits without running
- typing edits the focused field
