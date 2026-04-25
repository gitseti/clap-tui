use clap::{Arg, ArgAction, ArgGroup, Command, ValueHint, value_parser};
use clap_tui::TuiApp;

fn build_command() -> Command {
    Command::new("clap-features")
        .version("0.1.0")
        .about("Manual compatibility and stress fixture for clap feature coverage")
        .long_about(
            "A broad clap command graph used to exercise clap features in clap-tui. \
             This example is a diagnostic compatibility fixture rather than a learning-oriented CLI.",
        )
        .next_line_help(true)
        .arg_required_else_help(true)
        .subcommand_negates_reqs(true)
        .allow_external_subcommands(true)
        .external_subcommand_value_parser(value_parser!(String))
        .group(
            ArgGroup::new("required_mode_group")
                .args(["required_mode_fast", "required_mode_safe"])
                .required(true),
        )
        .arg(
            Arg::new("global_counted_verbosity")
                .short('v')
                .long("global-counted-verbosity")
                .help("Counted global flag")
                .long_help("Repeat to increase the count, for example `-v`, `-vv`, or `-vvv`.")
                .help_heading("Global")
                .display_order(1)
                .action(ArgAction::Count)
                .global(true),
        )
        .arg(
            Arg::new("global_conflicts_with_count")
                .long("global-conflicts-with-count")
                .help("Global flag that conflicts with --global-counted-verbosity")
                .help_heading("Global")
                .display_order(2)
                .action(ArgAction::SetTrue)
                .conflicts_with("global_counted_verbosity")
                .global(true),
        )
        .arg(
            Arg::new("global_optional_value_enum")
                .long("global-optional-value-enum")
                .visible_alias("global-optional-value-enum-alias")
                .help("Global option with an optional value and default missing value")
                .long_help(
                    "Optional-value flag. `--global-optional-value-enum` uses the default missing \
                     value, while `--global-optional-value-enum=never` sets an explicit value.",
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
            Arg::new("global_config_path")
                .short('c')
                .long("global-config-path")
                .aliases(["config-path-alias"])
                .visible_alias("cfg-path")
                .help("Global file path option")
                .help_heading("Global")
                .display_order(4)
                .value_name("FILE")
                .value_hint(ValueHint::FilePath)
                .global(true),
        )
        .arg(
            Arg::new("global_defaulted_value_enum")
                .long("global-defaulted-value-enum")
                .visible_aliases(["global-visible-alias-a", "global-visible-alias-b"])
                .help("Global option with a defaulted enumerated value")
                .help_heading("Global")
                .display_order(5)
                .default_value("dev")
                .value_parser(["dev", "stage", "prod"])
                .global(true),
        )
        .arg(
            Arg::new("required_mode_fast")
                .long("required-mode-fast")
                .help("First member of a required ArgGroup")
                .help_heading("Groups")
                .display_order(10)
                .action(ArgAction::SetTrue)
                .group("required_mode_group"),
        )
        .arg(
            Arg::new("required_mode_safe")
                .long("required-mode-safe")
                .help("Second member of a required ArgGroup")
                .help_heading("Groups")
                .display_order(11)
                .action(ArgAction::SetTrue)
                .group("required_mode_group"),
        )
        .arg(
            Arg::new("debug")
                .long("debug")
                .help("Boolean flag used by --conflicts-with-debug")
                .help_heading("Relations")
                .display_order(20)
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("conflicts_with_debug")
                .long("conflicts-with-debug")
                .help("Boolean flag that conflicts with --debug")
                .help_heading("Relations")
                .display_order(21)
                .action(ArgAction::SetTrue)
                .conflicts_with("debug"),
        )
        .arg(
            Arg::new("requires_config")
                .long("requires-config")
                .help("Boolean flag that requires --global-config-path")
                .help_heading("Relations")
                .display_order(22)
                .action(ArgAction::SetTrue)
                .requires("global_config_path"),
        )
        .arg(
            Arg::new("allow_negative_integer")
                .long("allow-negative-integer")
                .help("Integer option that accepts negative numbers")
                .help_heading("Input")
                .display_order(30)
                .default_value("0")
                .allow_negative_numbers(true)
                .value_parser(value_parser!(i32)),
        )
        .arg(
            Arg::new("multiple_values")
                .short('m')
                .long("multiple-values")
                .visible_alias("multiple-values-alias")
                .help("Repeatable option that also accepts comma-delimited values")
                .help_heading("Input")
                .display_order(31)
                .action(ArgAction::Append)
                .num_args(1..)
                .value_name("VALUE")
                .value_delimiter(','),
        )
        .arg(
            Arg::new("key_value_pair")
                .long("key-value-pair")
                .help("Option that captures two values per occurrence")
                .help_heading("Input")
                .display_order(32)
                .action(ArgAction::Append)
                .num_args(2)
                .value_names(["KEY", "VALUE"]),
        )
        .arg(
            Arg::new("terminated_paths")
                .long("terminated-paths")
                .help("Multi-value path list terminated by `;`")
                .help_heading("Input")
                .display_order(33)
                .action(ArgAction::Append)
                .num_args(1..)
                .value_name("PATH")
                .value_hint(ValueHint::AnyPath)
                .value_terminator(";"),
        )
        .subcommand(
            Command::new("required-args")
                .about("Required positional arguments plus defaults and repeated values")
                .display_order(1)
                .arg(
                    Arg::new("required_path")
                        .help("Required positional path")
                        .required(true)
                        .index(1)
                        .value_hint(ValueHint::DirPath),
                )
                .arg(
                    Arg::new("defaulted_host")
                        .long("defaulted-host")
                        .help("Option with a default value")
                        .default_value("127.0.0.1"),
                )
                .arg(
                    Arg::new("defaulted_port")
                        .long("defaulted-port")
                        .help("Option with a default value parser")
                        .default_value("8080")
                        .value_parser(value_parser!(u16)),
                )
                .arg(
                    Arg::new("repeated_value_enum")
                        .long("repeated-value-enum")
                        .help("Repeated value enum input with comma-delimited values")
                        .action(ArgAction::Append)
                        .num_args(1..)
                        .value_delimiter(',')
                        .value_parser(["gzip", "brotli", "http2"]),
                ),
        )
        .subcommand(
            Command::new("repeated-values")
                .about("Repeated positional values plus a trailing `last(true)` argument")
                .short_flag('R')
                .long_flag("repeated-values")
                .visible_alias("repeated-values-alias")
                .display_order(2)
                .arg(
                    Arg::new("required_target")
                        .long("required-target")
                        .help("Required enumerated option")
                        .required(true)
                        .value_parser(["local", "s3", "gcs"]),
                )
                .arg(
                    Arg::new("repeated_paths")
                        .help("Required repeated positional values")
                        .required(true)
                        .index(1)
                        .action(ArgAction::Append)
                        .num_args(1..)
                        .value_hint(ValueHint::AnyPath),
                )
                .arg(
                    Arg::new("last_filters")
                        .help("Additional filter expressions after `--`")
                        .index(2)
                        .last(true)
                        .action(ArgAction::Append)
                        .num_args(1..)
                        .allow_hyphen_values(true),
                ),
        )
        .subcommand(
            Command::new("trailing-var-args")
                .about("Trailing raw arguments, command hints, and repeated env pairs")
                .visible_aliases(["spawn-alias", "run-raw-alias"])
                .display_order(3)
                .arg_required_else_help(true)
                .arg(
                    Arg::new("optional_cwd")
                        .long("optional-cwd")
                        .help("Optional working directory")
                        .value_hint(ValueHint::DirPath),
                )
                .arg(
                    Arg::new("repeated_env_pair")
                        .long("repeated-env-pair")
                        .help("Repeated option with comma-delimited KEY=VALUE items")
                        .action(ArgAction::Append)
                        .num_args(1..)
                        .value_name("KEY=VALUE")
                        .value_delimiter(','),
                )
                .arg(
                    Arg::new("required_program")
                        .help("Required command name")
                        .required(true)
                        .index(1)
                        .value_hint(ValueHint::CommandName),
                )
                .arg(
                    Arg::new("raw_argv")
                        .help("Trailing raw command arguments")
                        .index(2)
                        .action(ArgAction::Append)
                        .num_args(1..)
                        .trailing_var_arg(true)
                        .allow_hyphen_values(true)
                        .value_hint(ValueHint::CommandWithArguments),
                ),
        )
        .subcommand(
            Command::new("args-conflict-with-subcommands")
                .about("args_conflicts_with_subcommands and subcommand_negates_reqs coverage")
                .display_order(4)
                .arg_required_else_help(true)
                .subcommand_negates_reqs(true)
                .args_conflicts_with_subcommands(true)
                .arg(
                    Arg::new("required_template")
                        .long("required-template")
                        .help("Required unless a subcommand is chosen")
                        .required(true)
                        .value_name("NAME"),
                )
                .subcommand(Command::new("plan").about("Render the workflow plan"))
                .subcommand(Command::new("apply").about("Execute the workflow")),
        )
        .subcommand(
            Command::new("nested-subcommands")
                .about("Nested subcommands with subcommand_required")
                .display_order(5)
                .arg_required_else_help(true)
                .subcommand_required(true)
                .subcommand(Command::new("cache").about("Inspect cache state"))
                .subcommand(
                    Command::new("users")
                        .about("Inspect user state")
                        .arg_required_else_help(true)
                        .subcommand_required(true)
                        .subcommand(
                            Command::new("list").about("List known users").arg(
                                Arg::new("user_status")
                                    .long("user-status")
                                    .help("Filter users by account state")
                                    .value_parser(["active", "disabled", "pending"]),
                            ),
                        )
                        .subcommand(
                            Command::new("sessions")
                                .about("Inspect active user sessions")
                                .arg(
                                    Arg::new("session_user")
                                        .long("session-user")
                                        .help("Only show sessions for this user")
                                        .value_name("USER"),
                                ),
                        ),
                ),
        )
        .subcommand(
            Command::new("exclusive-flag")
                .about("Exclusive flag coverage")
                .visible_alias("exclusive-flag-alias")
                .display_order(6)
                .arg(
                    Arg::new("dump_defaults")
                        .long("dump-defaults")
                        .help("Exclusive flag that must be passed alone")
                        .action(ArgAction::SetTrue)
                        .exclusive(true),
                )
                .arg(
                    Arg::new("optional_item")
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
