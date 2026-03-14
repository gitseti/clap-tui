use clap::Parser;
use clap_tui::TuiApp;

#[derive(Debug, Parser)]
#[command(name = "simple", about = "Simple example")]
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

fn main() -> anyhow::Result<()> {
    let app = TuiApp::from_factory::<Cli>();
    app.run_with_parser::<Cli, _>(|cli| {
        if cli.verbose {
            println!("Verbose mode on");
        }
        for _ in 0..cli.count {
            println!("Hello, {}!", cli.name);
        }
        Ok(())
    })?;
    Ok(())
}
