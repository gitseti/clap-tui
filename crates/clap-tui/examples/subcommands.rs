use clap::{Parser, Subcommand, ValueEnum};
use clap_tui::TuiApp;

#[derive(Debug, Parser)]
#[command(name = "tool", about = "Subcommand example", version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Build {
        #[arg(short, long)]
        release: bool,

        #[arg(short, long, value_enum, default_value_t = Color::Blue)]
        color: Color,
    },
    Serve {
        #[arg(long, default_value = "127.0.0.1")]
        host: String,

        #[arg(long, default_value_t = 8080)]
        port: u16,
    },
}

#[derive(Debug, Copy, Clone, ValueEnum)]
enum Color {
    Red,
    Green,
    Blue,
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let app = TuiApp::from_factory::<Cli>();
    app.run_with_parser::<Cli, _, std::io::Error>(|cli| {
        println!("Selected: {:?}", cli.command);
        Ok(())
    })?;
    Ok(())
}
