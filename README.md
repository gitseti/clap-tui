# clap-tui

Auto-generate a terminal UI from a `clap` command definition.

## Quick Start

```bash
cargo run -p clap-tui --example simple
```

```bash
cargo run -p clap-tui --example subcommands
```

## Library Usage

```rust
use clap::Parser;
use clap_tui::{Theme, ThemePreset, TuiApp, TuiConfig};

#[derive(Parser)]
struct Cli {
    #[arg(long)]
    name: String,
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let app = TuiApp::from_factory::<Cli>();
    let mut config = TuiConfig::default();
    config.theme = Theme::from_preset(ThemePreset::HighContrastDark);
    app.with_config(config).run_with_parser(|cli| {
        println!("Hello {}", cli.name);
        Ok::<_, std::io::Error>(())
    })?;
    Ok(())
}
```

### Supported extension points

The crate intentionally supports three public customization seams during the
ongoing internal refactor:
- custom runtimes with `TuiApp::with_runtime(...)`
- derive-based typed execution with `TuiApp::from_factory::<Cli>().run_with_parser(...)`
- theming and layout through `TuiConfig`
- initial command selection through `TuiConfig.start_command`

Internal modules and crate-private helper types are not stable extension points.

### Public API review outcome

The final refactor review kept the public API intentionally narrow and unchanged:
- `TuiApp` remains the main entry point
- `TuiConfig` and theme types remain the supported configuration surface
- `Runtime` plus the exported `AppEvent` / key / mouse types remain the advanced integration seam

No additional internal modules were promoted to public API, and no existing public seam
was narrowed because the internal cleanup did not reveal a concrete simplification worth a
breaking change.

### Current form capabilities

The current TUI supports:
- repeated options and repeated values
- count-style flags
- optional-value flags
- inherited global args across subcommands
- clap-backed validation summaries and field-level form feedback

### Theme presets

You can select a built-in theme preset via `TuiConfig`:

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
- `Tab` switch focus
- `Shift+Tab` cycle tabs
- `?` toggle Help tab
- `/` search in command tree
- `Ctrl+R` run
- `Ctrl+Enter` run when supported by the terminal
- `Ctrl+C` exit
- Type to edit the focused field
