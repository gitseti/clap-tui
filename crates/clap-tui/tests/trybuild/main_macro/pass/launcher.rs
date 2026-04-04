use clap::Parser;

#[derive(Parser)]
#[command(name = "tool")]
struct Cli;

#[clap_tui::main(launcher = "form")]
fn main(_cli: Cli) -> Result<(), clap_tui::TuiError> {
    Ok(())
}
