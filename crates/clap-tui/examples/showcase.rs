use clap::{Args, Parser, Subcommand, ValueEnum};
use clap_tui::Tui;
use std::path::PathBuf;

#[derive(Debug, Parser, PartialEq, Eq)]
#[command(
    name = "showcase",
    about = "A realistic clap-tui showcase CLI",
    version = "0.1.0"
)]
enum Command {
    /// Launch the interactive TUI
    Tui,
    /// Build the application bundle
    Build(BuildArgs),
    /// Run the local development server
    Serve(ServeArgs),
    /// Deploy a build to an environment
    Deploy(DeployArgs),
    /// Show or update persisted configuration
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
}

#[derive(Debug, Args, PartialEq, Eq)]
struct BuildArgs {
    /// Emit an optimized production bundle
    #[arg(short, long)]
    release: bool,

    /// Output format
    #[arg(long, value_enum, default_value_t = BuildFormat::Binary)]
    format: BuildFormat,

    /// Output directory
    #[arg(long, default_value = "dist")]
    output: String,

    /// Source directory to build
    #[arg(value_hint = clap::ValueHint::DirPath)]
    source: PathBuf,
}

#[derive(Debug, Args, PartialEq, Eq)]
struct ServeArgs {
    /// Host interface to bind
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// Port to listen on
    #[arg(long, default_value_t = 3000)]
    port: u16,

    /// Rebuild automatically when files change
    #[arg(short, long)]
    watch: bool,

    /// Optional config file
    #[arg(long, value_hint = clap::ValueHint::FilePath)]
    config: Option<PathBuf>,
}

#[derive(Debug, Args, PartialEq, Eq)]
struct DeployArgs {
    /// Deployment environment
    #[arg(long, value_enum)]
    environment: Environment,

    /// Deployment target
    #[arg(value_enum)]
    target: DeployTarget,

    /// Preview actions without applying changes
    #[arg(short = 'n', long)]
    dry_run: bool,

    /// Repeatable release tags
    #[arg(long = "tag", short = 't')]
    tags: Vec<String>,
}

#[derive(Debug, Subcommand, PartialEq, Eq)]
enum ConfigCommand {
    /// Print the effective configuration
    Show,
    /// Set a configuration value
    Set(ConfigSetArgs),
}

#[derive(Debug, Args, PartialEq, Eq)]
struct ConfigSetArgs {
    /// Config key to update
    key: String,

    /// Config value to store
    value: String,

    /// Where the setting should be written
    #[arg(long, value_enum, default_value_t = ConfigScope::Local)]
    scope: ConfigScope,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, ValueEnum)]
enum BuildFormat {
    Binary,
    Tarball,
    Docker,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, ValueEnum)]
enum Environment {
    Staging,
    Production,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, ValueEnum)]
enum DeployTarget {
    Web,
    Worker,
    Jobs,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, ValueEnum)]
enum ConfigScope {
    Local,
    Shared,
    Global,
}

fn dispatch(command: Command) {
    match command {
        Command::Tui => {}
        other => println!("{other:#?}"),
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
