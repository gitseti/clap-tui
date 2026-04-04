use clap::Parser;

#[derive(Parser)]
#[command(name = "tool")]
struct Cli;

#[clap_tui::main]
fn main() -> Result<(), clap_tui::TuiError> {
    let _ = Cli::parse();
    Ok(())
}
