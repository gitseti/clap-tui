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
use clap_tui::{ParserLauncher, Theme, ThemePreset, TuiConfig};

#[derive(Parser)]
#[command(name = "tool")]
struct Cli {
    #[arg(long)]
    name: String,
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut config = TuiConfig::default();
    config.theme = Theme::from_preset(ThemePreset::HighContrastDark);

    ParserLauncher::<Cli>::new().with_config(config).run(|cli| {
        println!("Hello {}", cli.name);
        Ok::<_, std::io::Error>(())
    })?;
    Ok(())
}
```

This is the canonical derive-based entrypoint. It augments the root clap surface with a
synthetic `tui` subcommand, so users can launch the TUI with `tool tui` while ordinary
invocations still parse through `Cli`.

### Macro convenience

```rust
use clap::Parser;

#[derive(Parser)]
#[command(name = "tool")]
struct Cli {
    #[arg(long)]
    name: String,
}

#[clap_tui::main]
fn main(cli: Cli) -> Result<(), clap_tui::TuiError> {
    println!("Hello {}", cli.name);
    Ok(())
}
```

`#[clap_tui::main]` is convenience syntax over `ParserLauncher`; it does not define
different runtime behavior. You can also provide `#[clap_tui::main(config = path::to::fn)]`
to compute a `TuiConfig` before the synthetic `tool tui` launch path starts the TUI.

### Supported extension points

The crate intentionally supports three public customization seams during the
ongoing internal refactor:
- custom runtimes with `TuiApp::with_runtime(...)`
- derive-based main entrypoints with `ParserLauncher::<Cli>::run(...)`
- direct derive-based TUI execution with `TuiApp::from_factory::<Cli>().run_with_parser(...)`
- theming and layout through `TuiConfig`
- initial command selection through `TuiConfig.start_command`

Internal modules and crate-private helper types are not stable extension points.

### Synthetic `tui` scope

The v1 synthetic launcher is intentionally narrow:
- it attaches only at the CLI root, producing `tool tui`
- it appears in ordinary clap help and parse diagnostics
- it is hidden from the rendered TUI command tree itself
- it is rejected when the root command already defines a conflicting `tui` path or uses
  ambiguous host grammar such as external subcommands or trailing raw capture

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

## Testing Strategy

`clap-tui` uses a small testing pyramid so behavior changes can land at the
lowest layer that still protects the regression:

- reducer and controller tests for pure state transitions and command semantics
- render tests for layout, styling, and visible validation feedback
- scripted app-flow tests for real event-loop behavior, rendered frames, mouse hits,
  and final run or cancel outcomes
- optional PTY smoke coverage only for terminal integration concerns such as raw mode
  or alternate-screen startup and teardown

When interactive behavior depends on redraw timing, focus changes, mouse layout, or
the integration between rendering and input dispatch, prefer adding a scripted
scenario in `crates/clap-tui/src/app/scripted_tests.rs`. Those tests use the
crate-internal scripted harness in `crates/clap-tui/src/app/scripted.rs` and
should prefer semantic helpers such as footer clicks and dropdown targeting over
hard-coded coordinates. Keep raw event injection for the rare edge case where a
semantic helper would hide the intent of the test.

## Controls
- `Tab` switch focus
- `Shift+Tab` cycle tabs
- `?` toggle Help tab
- `/` search in command tree
- `Ctrl+R` run
- `Ctrl+Enter` run when supported by the terminal
- `Ctrl+C` exit
- Type to edit the focused field
