# clap-tui

`clap-tui` turns a `clap` CLI into an interactive terminal UI while preserving the original command-line interface. You can keep `clap` as the source of truth, collect input in a terminal form, and still hand the final argv back to `clap` for typed parsing.

This crate was heavily inspired by [Trogon](https://github.com/Textualize/trogon). `clap-tui` is a community crate and is not an official `clap` project.

![clap-tui hero screenshot](docs/assets/hero.png)

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
use clap_tui::TuiLauncher;

#[derive(Debug, Parser)]
#[command(name = "tool")]
struct Cli {
    #[arg(long)]
    name: String,
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    TuiLauncher::<Cli>::new().run(|cli| {
        println!("Hello, {}!", cli.name);
        Ok::<_, clap_tui::TuiError>(())
    })?;

    Ok(())
}
```

`TuiLauncher` is the recommended entry point for derive-based CLIs. It augments the root command with a synthetic `tui` subcommand, so users can launch the form with `tool tui` while ordinary invocations still parse through `Cli`.

If you want a different launch path, call `.with_launcher_name("form")` or use
`#[clap_tui::main(launcher = "form")]`.

## Choose an entrypoint

You probably want `TuiLauncher`.

- Use `TuiLauncher::<Cli>::run(...)` if your app already uses `#[derive(Parser)]` and you want to add a `tool tui` entrypoint while keeping the normal CLI behavior. This is the default choice.
- Use `#[clap_tui::main]` if you want the same `TuiLauncher` behavior with less boilerplate.
- Use `TuiApp::from_parser::<Cli>().run_with_parser(...)` only if you want to open the TUI directly, without adding a synthetic `tui` subcommand to your CLI.
- Use `TuiApp::from_command(...)` if you are building a `clap::Command` by hand and want the untyped API.
- Use `TuiLauncher::<Cli>::with_launcher_name(...)` only when you want a launch path other than `tool tui`.

## Direct typed TUI

```rust
use clap::Parser;
use clap_tui::TuiApp;

#[derive(Debug, Parser)]
#[command(name = "tool")]
struct Cli {
    #[arg(long)]
    name: String,
}

fn main() -> Result<(), clap_tui::TuiError> {
    TuiApp::from_parser::<Cli>().run_with_parser(|cli| {
        println!("Hello, {}!", cli.name);
        Ok::<_, std::io::Error>(())
    })
}
```

## Supported customization seams

- `TuiConfig` controls theme, layout, key bindings, and initial command selection.
- `TuiApp::with_runtime(...)` and the exported `Runtime` event types support advanced runtime integration.
- `TuiConfig.start_command` lets you preselect a command path such as `build::release`.

Internal reducers, projections, render helpers, and other support modules are not stable extension points.

## Feature flags

- `mouse` is enabled by default and turns on mouse capture plus mouse-driven controls.

## Terminal expectations

- `clap-tui` is designed for interactive terminals that support raw mode and an alternate screen.
- The default `CrosstermRuntime` restores the terminal before returning, including when the user cancels.
- Mouse interactions require the default `mouse` feature.
- Validation stays inside the TUI flow: invalid forms show inline feedback instead of returning partially parsed values.

## Example guide

- `simple` shows the smallest derive-based setup with `#[clap_tui::main]`.
- `showcase` is the best starting point for demos and screenshots: it shows nested commands, dropdowns, text input, shared global fields, and a readable preview.
- `subcommands` shows the `TuiLauncher` flow for a CLI with nested subcommands.
- `kitchen_sink` demonstrates the untyped `TuiApp::from_command(...)` path and a wider range of `clap` surface area.

```bash
cargo run -p clap-tui --example simple -- tui
cargo run -p clap-tui --example showcase -- tui
cargo run -p clap-tui --example subcommands -- tui
cargo run -p clap-tui --example kitchen_sink
```

## Release verification

Maintainers can run `./scripts/verify-release-readiness.sh` for the same formatting,
linting, test, and package-surface checks that the GitHub `verify` workflow enforces.
See `docs/release-readiness.md` for the branch-protection setup, tag dry-run flow,
the proc-macro publishing prerequisite, and the current manual-publish boundary.

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
