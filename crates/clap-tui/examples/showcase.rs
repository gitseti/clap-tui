use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(
    name = "showcase",
    about = "Compact showcase for nested commands, dropdowns, and text input",
    version = "0.1.0"
)]
struct Cli {
    /// Shared deployment profile
    #[arg(long, value_enum, default_value_t = Profile::Preview, global = true)]
    profile: Profile,

    /// Team or project name
    #[arg(long, default_value = "checkout", global = true)]
    team: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Deploy an application target
    Deploy {
        #[command(subcommand)]
        target: DeployTarget,
    },
    /// Inspect logs for a running target
    Logs {
        #[command(subcommand)]
        target: LogsTarget,
    },
}

#[derive(Debug, Subcommand)]
enum DeployTarget {
    /// Deploy the web application
    Web(ReleaseOptions),
    /// Deploy the background worker
    Worker(ReleaseOptions),
}

#[derive(Debug, Subcommand)]
enum LogsTarget {
    /// Inspect web logs
    Web(LogOptions),
    /// Inspect worker logs
    Worker(LogOptions),
}

#[derive(Debug, Args)]
struct ReleaseOptions {
    /// Deployment environment
    #[arg(long, value_enum, default_value_t = Environment::Staging)]
    environment: Environment,

    /// Cloud region
    #[arg(long, value_enum, default_value_t = Region::EuCentral1)]
    region: Region,

    /// Image tag to deploy
    #[arg(long, default_value = "2026.04.1")]
    image_tag: String,

    /// Number of replicas
    #[arg(long, default_value_t = 2)]
    replicas: u16,

    /// Roll out as a canary release first
    #[arg(long)]
    canary: bool,
}

#[derive(Debug, Args)]
struct LogOptions {
    /// Environment to inspect
    #[arg(long, value_enum, default_value_t = Environment::Staging)]
    environment: Environment,

    /// Cloud region
    #[arg(long, value_enum, default_value_t = Region::EuCentral1)]
    region: Region,

    /// Time window such as 15m or 1h
    #[arg(long, default_value = "15m")]
    since: String,

    /// Keep streaming new log lines
    #[arg(long)]
    follow: bool,
}

#[derive(Debug, Copy, Clone, ValueEnum)]
enum Profile {
    Preview,
    Team,
    Production,
}

#[derive(Debug, Copy, Clone, ValueEnum)]
enum Environment {
    Dev,
    Staging,
    Production,
}

#[derive(Debug, Copy, Clone, ValueEnum)]
enum Region {
    UsEast1,
    EuCentral1,
    ApSouth1,
}

#[clap_tui::main]
fn main(cli: Cli) -> Result<(), clap_tui::TuiError> {
    println!("{cli:#?}");
    Ok(())
}
