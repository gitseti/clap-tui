use clap::Parser;

#[derive(Parser)]
#[command(name = "tool")]
struct Cli {
    #[arg(long, default_value = "world")]
    name: String,
}

#[clap_tui::main]
fn main(cli: Cli) -> Result<(), clap_tui::TuiError> {
    let _ = cli.name;
    Ok(())
}
