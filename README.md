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

fn main() -> anyhow::Result<()> {
    let app = TuiApp::from_factory::<Cli>();
    let mut config = TuiConfig::default();
    config.theme = Theme::from_preset(ThemePreset::HighContrastDark);
    app.with_config(config).run_with_parser::<Cli, _>(|cli| {
        println!("Hello {}", cli.name);
        Ok(())
    })?;
    Ok(())
}
```

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
- `Ctrl+Enter` run
- `Ctrl+C` exit
- Type to edit the focused field
