use clap::{Arg, ArgAction, ArgGroup, Command, ValueHint, value_parser};
use clap_tui::TuiApp;

fn build_command() -> Command {
    Command::new("kitchen-sink")
        .version("0.1.0")
        .about("Track clap feature coverage in the TUI generator")
        .long_about(
            "A broad clap command graph used to track which clap features are \
             already supported by clap-tui and which ones are still missing.",
        )
        .next_line_help(true)
        .arg_required_else_help(true)
        .subcommand_negates_reqs(true)
        .allow_external_subcommands(true)
        .external_subcommand_value_parser(value_parser!(String))
        .group(ArgGroup::new("mode").args(["fast", "safe"]).required(true))
        // Global args.
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Increase logging verbosity")
                .long_help("Repeat to increase verbosity, for example `-v`, `-vv`, or `-vvv`.")
                .help_heading("Global")
                .display_order(1)
                .action(ArgAction::Count)
                .global(true),
        )
        .arg(
            Arg::new("quiet")
                .long("quiet")
                .help("Silence normal output")
                .help_heading("Global")
                .display_order(2)
                .action(ArgAction::SetTrue)
                .conflicts_with("verbose")
                .global(true),
        )
        .arg(
            Arg::new("color")
                .long("color")
                .visible_alias("colour")
                .help("Control color output")
                .long_help(
                    "Optional-value flag. `--color` uses the default missing value, while \
                     `--color=never` sets an explicit value.",
                )
                .help_heading("Global")
                .display_order(3)
                .num_args(0..=1)
                .require_equals(true)
                .default_value("auto")
                .default_missing_value("always")
                .value_parser(["auto", "always", "never"])
                .global(true),
        )
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .aliases(["conf"])
                .visible_alias("cfg")
                .help("Path to the main config file")
                .help_heading("Global")
                .display_order(4)
                .value_name("FILE")
                .value_hint(ValueHint::FilePath)
                .global(true),
        )
        .arg(
            Arg::new("profile")
                .long("profile")
                .visible_aliases(["environment", "stage"])
                .help("Named execution profile")
                .help_heading("Global")
                .display_order(5)
                .default_value("dev")
                .value_parser(["dev", "stage", "prod"])
                .global(true),
        )
        // Required group on the root command.
        .arg(
            Arg::new("fast")
                .long("fast")
                .help("Optimize for speed")
                .help_heading("Mode")
                .display_order(10)
                .action(ArgAction::SetTrue)
                .group("mode"),
        )
        .arg(
            Arg::new("safe")
                .long("safe")
                .help("Optimize for safety")
                .help_heading("Mode")
                .display_order(11)
                .action(ArgAction::SetTrue)
                .group("mode"),
        )
        // Argument relationships and richer value shapes.
        .arg(
            Arg::new("upload")
                .long("upload")
                .help("Upload results to the remote target")
                .help_heading("Actions")
                .display_order(20)
                .action(ArgAction::SetTrue)
                .requires("token")
                .conflicts_with("dry_run"),
        )
        .arg(
            Arg::new("token")
                .long("token")
                .help("Authentication token required by --upload")
                .help_heading("Actions")
                .display_order(21)
                .value_name("TOKEN"),
        )
        .arg(
            Arg::new("dry_run")
                .long("dry-run")
                .help("Preview actions without writing changes")
                .help_heading("Actions")
                .display_order(22)
                .action(ArgAction::SetTrue)
                .conflicts_with("upload"),
        )
        .arg(
            Arg::new("offset")
                .long("offset")
                .help("Integer offset that accepts negative numbers")
                .help_heading("Input")
                .display_order(30)
                .default_value("0")
                .allow_negative_numbers(true)
                .value_parser(value_parser!(i32)),
        )
        .arg(
            Arg::new("tag")
                .short('t')
                .long("tag")
                .visible_alias("label")
                .help("Repeatable tag list; also accepts comma-delimited values")
                .help_heading("Input")
                .display_order(31)
                .action(ArgAction::Append)
                .num_args(1..)
                .value_name("TAG")
                .value_delimiter(','),
        )
        .arg(
            Arg::new("define")
                .long("define")
                .help("Key/value pairs")
                .help_heading("Input")
                .display_order(32)
                .action(ArgAction::Append)
                .num_args(2)
                .value_names(["KEY", "VALUE"]),
        )
        .arg(
            Arg::new("include")
                .long("include")
                .help("Multi-value path list terminated by `;`")
                .help_heading("Input")
                .display_order(33)
                .action(ArgAction::Append)
                .num_args(1..)
                .value_name("PATH")
                .value_hint(ValueHint::AnyPath)
                .value_terminator(";"),
        )
        // Subcommands with aliases, subcommand flags, and their own parser rules.
        .subcommand(
            Command::new("serve")
                .about("Serve a directory over HTTP")
                .visible_alias("http")
                .display_order(1)
                .arg(
                    Arg::new("document_root")
                        .help("Directory to serve")
                        .required(true)
                        .index(1)
                        .value_hint(ValueHint::DirPath),
                )
                .arg(
                    Arg::new("host")
                        .long("host")
                        .help("Bind address")
                        .default_value("127.0.0.1"),
                )
                .arg(
                    Arg::new("port")
                        .long("port")
                        .help("Bind port")
                        .default_value("8080")
                        .value_parser(value_parser!(u16)),
                )
                .arg(
                    Arg::new("feature")
                        .long("feature")
                        .help("Server features; supports repeated and comma-delimited input")
                        .action(ArgAction::Append)
                        .num_args(1..)
                        .value_delimiter(',')
                        .value_parser(["gzip", "brotli", "http2"]),
                ),
        )
        .subcommand(
            Command::new("sync")
                .about("Synchronize artifacts")
                .short_flag('S')
                .long_flag("sync")
                .visible_alias("mirror")
                .display_order(2)
                .arg(
                    Arg::new("target")
                        .long("target")
                        .help("Sync target")
                        .required(true)
                        .value_parser(["local", "s3", "gcs"]),
                )
                .arg(
                    Arg::new("path")
                        .help("Input paths to synchronize")
                        .required(true)
                        .index(1)
                        .action(ArgAction::Append)
                        .num_args(1..)
                        .value_hint(ValueHint::AnyPath),
                )
                .arg(
                    Arg::new("filter")
                        .help("Additional filter expressions after `--`")
                        .index(2)
                        .last(true)
                        .action(ArgAction::Append)
                        .num_args(1..)
                        .allow_hyphen_values(true),
                ),
        )
        .subcommand(
            Command::new("exec")
                .about("Run a program with trailing raw arguments")
                .visible_aliases(["spawn", "run-raw"])
                .display_order(3)
                .arg_required_else_help(true)
                .arg(
                    Arg::new("cwd")
                        .long("cwd")
                        .help("Working directory")
                        .value_hint(ValueHint::DirPath),
                )
                .arg(
                    Arg::new("env_pair")
                        .long("env")
                        .help("Environment pairs; supports repeated and comma-delimited input")
                        .action(ArgAction::Append)
                        .num_args(1..)
                        .value_name("KEY=VALUE")
                        .value_delimiter(','),
                )
                .arg(
                    Arg::new("program")
                        .help("Program to run")
                        .required(true)
                        .index(1)
                        .value_hint(ValueHint::CommandName),
                )
                .arg(
                    Arg::new("argv")
                        .help("Raw trailing command arguments")
                        .index(2)
                        .action(ArgAction::Append)
                        .num_args(1..)
                        .trailing_var_arg(true)
                        .allow_hyphen_values(true)
                        .value_hint(ValueHint::CommandWithArguments),
                ),
        )
        .subcommand(
            Command::new("workflow")
                .about("Command-level subcommand interaction rules")
                .display_order(4)
                .arg_required_else_help(true)
                .subcommand_negates_reqs(true)
                .args_conflicts_with_subcommands(true)
                .arg(
                    Arg::new("template")
                        .long("template")
                        .help("Required unless a subcommand is chosen")
                        .required(true)
                        .value_name("NAME"),
                )
                .subcommand(Command::new("plan").about("Render the workflow plan"))
                .subcommand(Command::new("apply").about("Execute the workflow")),
        )
        .subcommand(
            Command::new("admin")
                .about("Nested command that requires a subcommand")
                .display_order(5)
                .arg_required_else_help(true)
                .subcommand_required(true)
                .subcommand(Command::new("cache").about("Inspect cache state"))
                .subcommand(Command::new("users").about("Inspect user state")),
        )
        .subcommand(
            Command::new("inspect")
                .about("Read-only inspection helpers")
                .visible_alias("show")
                .display_order(6)
                .arg(
                    Arg::new("dump_defaults")
                        .long("dump-defaults")
                        .help("Exclusive flag that must be passed alone")
                        .action(ArgAction::SetTrue)
                        .exclusive(true),
                )
                .arg(
                    Arg::new("item")
                        .help("Optional item to inspect")
                        .index(1)
                        .value_parser(["config", "cache", "state"]),
                ),
        )
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let app = TuiApp::from_command(build_command());
    app.run_with_matches::<_, std::io::Error>(|matches| {
        println!("{matches:#?}");
        Ok(())
    })?;
    Ok(())
}
