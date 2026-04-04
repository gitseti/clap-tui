use clap::Parser;

#[derive(Parser)]
#[command(name = "tool")]
struct Cli;

#[clap_tui::main(config)]
fn main(_cli: Cli) -> Result<(), clap_tui::TuiError> {
    Ok(())
}
