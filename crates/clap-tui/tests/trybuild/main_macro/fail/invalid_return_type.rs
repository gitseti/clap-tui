use clap::Parser;

#[derive(Parser)]
#[command(name = "tool")]
struct Cli;

#[clap_tui::main]
fn main(_cli: Cli) {
}
