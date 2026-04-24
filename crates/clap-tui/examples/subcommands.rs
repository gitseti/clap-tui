use clap::{Parser, ValueEnum};
use clap_tui::Tui;

#[derive(Debug, Parser, PartialEq, Eq)]
#[command(name = "tool", about = "Subcommand example", version = "0.1.0")]
enum Command {
    /// Launch the interactive TUI
    Tui,
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, ValueEnum)]
enum Color {
    Red,
    Green,
    Blue,
}

fn dispatch(command: Command) {
    match command {
        Command::Tui => {}
        other => println!("Selected: {other:?}"),
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
