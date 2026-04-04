use clap::Parser;

#[derive(Debug, Parser)]
#[command(name = "simple", about = "Simple example", version = "0.1.0")]
struct Cli {
    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Optional name
    #[arg(short, long, default_value = "world")]
    name: String,

    /// Optional count
    #[arg(long, default_value_t = 1)]
    count: u32,
}

#[clap_tui::main]
fn main(cli: Cli) -> Result<(), clap_tui::TuiError> {
    if cli.verbose {
        println!("Verbose mode on");
    }
    for _ in 0..cli.count {
        println!("Hello, {}!", cli.name);
    }
    Ok(())
}
