use clap::Parser;
use clap_tui::Tui;

#[derive(Debug, Parser)]
#[command(name = "hello")]
enum Command {
    Tui,
    Hello,
}

fn main() -> Result<(), clap_tui::TuiError> {
    match Command::parse() {
        Command::Tui => {
            if let Some(invocation) = Tui::<Command>::new()
                .hide_entrypoint("tui")?
                .run_with_argv()?
            {
                println!("Running argv: {:?}", invocation.argv);

                if let Command::Hello = invocation.command {
                    println!("Hello, world!");
                }
            }
        }
        Command::Hello => println!("Hello, world!"),
    }

    Ok(())
}
