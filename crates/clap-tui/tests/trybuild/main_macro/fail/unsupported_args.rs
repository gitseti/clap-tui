use clap::Parser;

#[derive(Parser)]
#[command(name = "tool")]
struct Cli;

#[clap_tui::main(theme = theme_config)]
fn main(_cli: Cli) -> Result<(), clap_tui::TuiError> {
    Ok(())
}

fn theme_config() -> clap_tui::TuiConfig {
    clap_tui::TuiConfig::default()
}
