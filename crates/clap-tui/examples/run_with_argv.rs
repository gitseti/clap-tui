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
            if let Some(submission) = Tui::<Command>::new()
                .hide_entrypoint("tui")?
                .run_with_argv()?
            {
                println!("Running argv: {:?}", submission.argv);

                if let Command::Hello = submission.command {
                    println!("Hello, world!");
                }
            }
        }
        Command::Hello => println!("Hello, world!"),
    }

    Ok(())
}
