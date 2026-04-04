use clap::Parser;

#[derive(Parser)]
#[command(name = "tool")]
struct Cli;

fn tui_config() -> clap_tui::TuiConfig {
    let mut config = clap_tui::TuiConfig::default();
    config.start_command = Some("tool".to_string());
    config
}

#[clap_tui::main(config = tui_config)]
fn main(_cli: Cli) -> Result<(), clap_tui::TuiError> {
    Ok(())
}
