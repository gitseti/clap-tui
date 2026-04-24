use clap::Parser;
use clap_tui::Tui;

#[derive(Debug, Parser, PartialEq, Eq)]
#[command(name = "simple", about = "Simple example", version = "0.1.0")]
enum Command {
    /// Launch the interactive TUI
    Tui,
    /// Print a greeting
    Hello {
        /// Enable verbose output
        #[arg(short, long)]
        verbose: bool,

        /// Optional name
        #[arg(short, long, default_value = "world")]
        name: String,

        /// Optional count
        #[arg(long, default_value_t = 1)]
        count: u32,
    },
}

fn dispatch(command: Command) {
    match command {
        Command::Tui => {}
        Command::Hello {
            verbose,
            name,
            count,
        } => {
            if verbose {
                println!("Verbose mode on");
            }
            for _ in 0..count {
                println!("Hello, {name}!");
            }
        }
    }
}

fn main() -> Result<(), clap_tui::TuiError> {
    match Command::parse() {
        Command::Tui => {
            if let Some(command) = Tui::<Command>::new().hide_entrypoint("tui")?.run()? {
                dispatch(command);
            }
        }
        command => dispatch(command),
    }

    Ok(())
}
