use std::collections::BTreeMap;

use crate::input::AppState;

mod argv;
mod effective;
mod validation;

pub(crate) use effective::{EffectiveArgValue, EffectiveValueSource};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct ValidationState {
    pub(crate) is_valid: bool,
    pub(crate) summary: Option<String>,
    pub(crate) field_errors: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DerivedState {
    pub(crate) argv: Vec<String>,
    pub(crate) validation: ValidationState,
    pub(crate) effective_values: BTreeMap<String, EffectiveArgValue>,
}

pub(crate) fn derive(state: &AppState) -> DerivedState {
    let argv = argv::build_command_line(state);
    let parse_argv = argv::build_parse_command_line(state);
    DerivedState {
        effective_values: effective::derive_effective_values(state, &parse_argv),
        validation: validation::validate_argv(state, &parse_argv),
        argv,
    }
}

#[cfg(test)]
pub(crate) fn build_command_line(state: &AppState) -> Vec<String> {
    argv::build_command_line(state)
}

#[cfg(test)]
pub(crate) fn validate_argv(state: &AppState, argv: &[String]) -> ValidationState {
    validation::validate_argv(state, argv)
}

#[cfg(test)]
pub(crate) fn validation_call_count() -> usize {
    validation::validation_call_count()
}

#[cfg(test)]
pub(crate) fn reset_validation_call_count() {
    validation::reset_validation_call_count();
}

#[cfg(test)]
mod tests {
    use clap::{Arg, ArgAction, Command, builder::ArgPredicate};

    use super::{EffectiveValueSource, derive};
    use crate::input::{AppState, InputSource, InputValueOccurrence};
    use crate::spec::{
        ArgKind, ArgSpec, CommandSpec, EXTERNAL_SUBCOMMAND_ARGS_ID, EXTERNAL_SUBCOMMAND_NAME_ID,
    };

    fn arg(id: &str, name: &str, kind: ArgKind) -> ArgSpec {
        ArgSpec {
            id: id.to_string(),
            display_name: name.to_string(),
            help: None,
            required: false,
            kind,
            default_values: Vec::new(),
            choices: Vec::new(),
            position: None,
            value_cardinality: crate::spec::ValueCardinality::One,
            value_hint: None,
            ..ArgSpec::default()
        }
    }

    fn app_state(args: Vec<ArgSpec>) -> AppState {
        AppState::new(CommandSpec {
            name: "tool".to_string(),
            version: None,
            about: None,
            help: String::new(),
            args,
            subcommands: Vec::new(),
            ..CommandSpec::default()
        })
    }

    #[test]
    fn derived_state_keeps_preview_and_run_argv_aligned() {
        let mut state = app_state(vec![arg("verbose", "--verbose", ArgKind::Flag)]);
        state.domain.toggle_flag_touched("verbose");

        let derived = derive(&state);

        assert_eq!(
            derived.argv,
            vec!["tool".to_string(), "--verbose".to_string()]
        );
        assert!(derived.validation.is_valid);
    }

    #[test]
    fn validation_tracks_missing_required_args_by_id() {
        let mut name = arg("name", "--name", ArgKind::Option);
        name.required = true;
        let state = app_state(vec![name]);

        let derived = derive(&state);

        assert!(!derived.validation.is_valid);
        assert_eq!(
            derived.validation.field_errors.get("name"),
            Some(&"Required argument".to_string())
        );
        assert_eq!(
            derived.validation.summary.as_deref(),
            Some("Missing required argument: --name")
        );
    }

    #[test]
    fn optional_choice_without_explicit_default_is_omitted_from_argv() {
        let mut color = arg("color", "--color", ArgKind::Enum);
        color.choices = vec!["red".to_string(), "green".to_string(), "blue".to_string()];
        let state = app_state(vec![color]);

        let derived = derive(&state);

        assert_eq!(derived.argv, vec!["tool".to_string()]);
        assert!(derived.validation.is_valid);
    }

    #[test]
    fn required_choice_without_explicit_default_stays_invalid_until_selected() {
        let mut color = arg("color", "--color", ArgKind::Enum);
        color.required = true;
        color.choices = vec!["red".to_string(), "green".to_string(), "blue".to_string()];
        let state = app_state(vec![color]);

        let derived = derive(&state);

        assert!(!derived.validation.is_valid);
        assert_eq!(
            derived.validation.field_errors.get("color"),
            Some(&"Required argument".to_string())
        );
    }

    #[test]
    fn derived_state_builds_full_invocation_argv_from_owned_command_forms() {
        let root = CommandSpec::from_command(
            &Command::new("tool")
                .arg(
                    Arg::new("verbose")
                        .long("verbose")
                        .action(clap::ArgAction::SetTrue)
                        .global(true),
                )
                .subcommand(
                    Command::new("build")
                        .arg(Arg::new("target").long("target"))
                        .subcommand(Command::new("release")),
                ),
        );
        let mut state = AppState::new(root);
        state
            .select_command_path(&["build".to_string()])
            .expect("valid path");
        state
            .domain
            .set_text_value("target", "wasm32-unknown-unknown");
        state.domain.mark_touched("target");
        state
            .select_command_path(&["build".to_string(), "release".to_string()])
            .expect("valid path");
        state.domain.toggle_flag_touched("verbose");

        let derived = derive(&state);

        assert_eq!(
            derived.argv,
            vec![
                "tool".to_string(),
                "--verbose".to_string(),
                "build".to_string(),
                "--target".to_string(),
                "wasm32-unknown-unknown".to_string(),
                "release".to_string(),
            ]
        );
    }

    #[test]
    fn clap_backed_validation_reports_conflicts_from_preview_argv() {
        let mut state = AppState::from_command(
            &Command::new("tool")
                .arg(
                    Arg::new("debug")
                        .long("debug")
                        .action(clap::ArgAction::SetTrue)
                        .conflicts_with("quiet"),
                )
                .arg(
                    Arg::new("quiet")
                        .long("quiet")
                        .action(clap::ArgAction::SetTrue),
                ),
        );
        state.domain.toggle_flag_touched("debug");
        state.domain.toggle_flag_touched("quiet");

        let derived = derive(&state);

        assert!(!derived.validation.is_valid);
        assert!(
            derived
                .validation
                .summary
                .as_deref()
                .is_some_and(|summary| summary == "Conflicting arguments: --debug, --quiet")
        );
        assert!(derived.validation.field_errors.contains_key("debug"));
        assert!(derived.validation.field_errors.contains_key("quiet"));
    }

    #[test]
    fn derived_state_remains_stable_after_env_source_changes() {
        let path = std::env::var("PATH").expect("PATH should exist for env-backed default tests");
        let mut state = AppState::from_command(
            &Command::new("tool").arg(Arg::new("config").long("config").env("PATH").required(true)),
        );

        let initial = derive(&state);
        assert_eq!(initial.argv, vec!["tool".to_string()]);
        assert!(initial.validation.is_valid);
        assert_eq!(
            initial
                .effective_values
                .get("config")
                .expect("effective value for env-backed config")
                .source,
            EffectiveValueSource::Env
        );
        assert_eq!(
            initial
                .effective_values
                .get("config")
                .expect("effective value for env-backed config")
                .values,
            vec![path]
        );

        state.domain.root.args[0].metadata.defaults.env =
            Some("CLAP_TUI_TEST_ENV_DERIVED_UNUSED".to_string());
        state.domain.validation_command = Some(
            Command::new("tool").arg(
                Arg::new("config")
                    .long("config")
                    .env("CLAP_TUI_TEST_ENV_DERIVED_UNUSED")
                    .required(true),
            ),
        );

        let after_mutation = derive(&state);
        assert_eq!(after_mutation.argv, initial.argv);
        assert_eq!(after_mutation.validation, initial.validation);
        assert_eq!(after_mutation.effective_values, initial.effective_values);
    }

    #[test]
    fn append_options_serialize_as_repeated_occurrences() {
        let mut state = AppState::from_command(
            &Command::new("tool").arg(
                Arg::new("include")
                    .long("include")
                    .action(ArgAction::Append)
                    .num_args(1),
            ),
        );
        state.domain.set_text_value("include", "src\ntests");
        state.domain.mark_touched("include");

        let derived = derive(&state);

        assert_eq!(
            derived.argv,
            vec![
                "tool".to_string(),
                "--include".to_string(),
                "src".to_string(),
                "--include".to_string(),
                "tests".to_string(),
            ]
        );
        assert!(derived.validation.is_valid);
    }

    #[test]
    fn optional_value_flags_serialize_presence_without_forcing_a_value() {
        let mut state = AppState::from_command(
            &Command::new("tool").arg(
                Arg::new("color")
                    .long("color")
                    .action(ArgAction::SetTrue)
                    .num_args(0..=1),
            ),
        );
        state.domain.toggle_optional_value_flag("color", true);

        let derived = derive(&state);

        assert_eq!(
            derived.argv,
            vec!["tool".to_string(), "--color".to_string()]
        );
        assert!(derived.validation.is_valid);
    }

    #[test]
    fn default_missing_optional_values_report_implicit_source_without_extra_preview_tokens() {
        let mut state = AppState::from_command(
            &Command::new("tool").arg(
                Arg::new("color")
                    .long("color")
                    .action(ArgAction::Set)
                    .default_value("auto")
                    .num_args(0..=1)
                    .require_equals(true)
                    .default_missing_value("always"),
            ),
        );
        state.domain.toggle_optional_value_flag("color", true);

        let derived = derive(&state);
        let effective = derived
            .effective_values
            .get("color")
            .expect("effective source for color");

        assert_eq!(
            derived.argv,
            vec!["tool".to_string(), "--color".to_string()]
        );
        assert_eq!(effective.source, EffectiveValueSource::DefaultMissing);
        assert_eq!(effective.values, vec!["always".to_string()]);
        assert!(derived.validation.is_valid);
    }

    #[test]
    fn conditional_defaults_report_derived_source_without_extra_preview_tokens() {
        let mut state = AppState::from_command(
            &Command::new("tool")
                .arg(Arg::new("flag").long("flag").action(ArgAction::SetTrue))
                .arg(Arg::new("mode").long("mode").default_value_if(
                    "flag",
                    ArgPredicate::IsPresent,
                    Some("auto"),
                )),
        );
        state.domain.toggle_flag_touched("flag");

        let derived = derive(&state);
        let effective = derived
            .effective_values
            .get("mode")
            .expect("effective source for mode");

        assert_eq!(derived.argv, vec!["tool".to_string(), "--flag".to_string()]);
        assert_eq!(effective.source, EffectiveValueSource::ConditionalDefault);
        assert_eq!(effective.values, vec!["auto".to_string()]);
        assert!(derived.validation.is_valid);
    }

    #[test]
    fn multi_select_choices_keep_append_style_occurrences() {
        let mut state = AppState::from_command(
            &Command::new("tool").arg(
                Arg::new("color")
                    .long("color")
                    .action(ArgAction::Append)
                    .num_args(1)
                    .value_parser(["red", "green", "blue"]),
            ),
        );
        state.domain.toggle_choice_value_touched("color", "red");
        state.domain.toggle_choice_value_touched("color", "blue");

        let derived = derive(&state);

        assert_eq!(
            derived.argv,
            vec![
                "tool".to_string(),
                "--color".to_string(),
                "red".to_string(),
                "--color".to_string(),
                "blue".to_string(),
            ]
        );
        assert!(derived.validation.is_valid);
    }

    #[test]
    fn grouped_multi_value_choices_stay_within_one_occurrence() {
        let mut state = AppState::from_command(
            &Command::new("tool").arg(
                Arg::new("color")
                    .long("color")
                    .num_args(1..)
                    .value_parser(["red", "green", "blue"]),
            ),
        );
        state.domain.toggle_choice_value_touched("color", "red");
        state.domain.toggle_choice_value_touched("color", "blue");

        let derived = derive(&state);

        assert_eq!(
            derived.argv,
            vec![
                "tool".to_string(),
                "--color".to_string(),
                "red".to_string(),
                "blue".to_string(),
            ]
        );
        assert!(derived.validation.is_valid);
    }

    #[test]
    fn count_flags_serialize_as_repeated_flag_occurrences() {
        let mut state = AppState::from_command(
            &Command::new("tool").arg(Arg::new("verbose").short('v').action(ArgAction::Count)),
        );
        state.domain.increment_counter("verbose");
        state.domain.increment_counter("verbose");
        state.domain.increment_counter("verbose");

        let derived = derive(&state);

        assert_eq!(
            derived.argv,
            vec![
                "tool".to_string(),
                "-v".to_string(),
                "-v".to_string(),
                "-v".to_string(),
            ]
        );
        assert!(derived.validation.is_valid);
    }

    #[test]
    fn allow_missing_positional_follows_command_level_parser_rules() {
        let mut state = AppState::from_command(
            &Command::new("tool")
                .allow_missing_positional(true)
                .arg(Arg::new("arg1"))
                .arg(Arg::new("arg2").required(true)),
        );
        state.domain.set_text_value("arg2", "other");
        state.domain.mark_touched("arg2");

        let derived = derive(&state);

        assert_eq!(derived.argv, vec!["tool".to_string(), "other".to_string()]);
        assert!(derived.validation.is_valid);
    }

    #[test]
    fn subcommand_precedence_over_arg_keeps_selected_subcommand_parseable() {
        let mut state = AppState::from_command(
            &Command::new("cmd")
                .subcommand_precedence_over_arg(true)
                .subcommand(Command::new("sub"))
                .arg(
                    Arg::new("arg")
                        .long("arg")
                        .action(ArgAction::Set)
                        .num_args(1..),
                ),
        );
        state.domain.set_text_value("arg", "1\n2\n3");
        state.domain.mark_touched("arg");
        state
            .select_command_path(&["sub".to_string()])
            .expect("valid subcommand");

        let derived = derive(&state);

        assert_eq!(
            derived.argv,
            vec![
                "cmd".to_string(),
                "--arg".to_string(),
                "1".to_string(),
                "2".to_string(),
                "3".to_string(),
                "sub".to_string(),
            ]
        );
        assert!(derived.validation.is_valid);
    }

    #[test]
    fn help_style_missing_positional_uses_missing_argument_summary_not_about() {
        let state = AppState::from_command(
            &Command::new("tool")
                .about("Run the selected tool")
                .arg_required_else_help(true)
                .arg(Arg::new("path").required(true)),
        );

        let derived = derive(&state);

        assert!(!derived.validation.is_valid);
        assert_eq!(
            derived.validation.summary.as_deref(),
            Some("Missing required argument: path")
        );
        assert_eq!(
            derived.validation.field_errors.get("path"),
            Some(&"Required argument".to_string())
        );
        assert!(
            derived
                .validation
                .summary
                .as_deref()
                .is_some_and(|summary| !summary.contains("Run the selected tool"))
        );
    }

    #[test]
    fn help_style_missing_input_without_required_args_uses_generic_summary() {
        let state = AppState::from_command(
            &Command::new("tool")
                .about("Run the selected tool")
                .arg_required_else_help(true)
                .arg(
                    Arg::new("verbose")
                        .long("verbose")
                        .action(ArgAction::SetTrue),
                ),
        );

        let derived = derive(&state);

        assert!(!derived.validation.is_valid);
        assert_eq!(
            derived.validation.summary.as_deref(),
            Some("Missing required input")
        );
        assert!(derived.validation.field_errors.is_empty());
    }

    #[test]
    fn missing_subcommand_uses_explicit_summary() {
        let state = AppState::from_command(
            &Command::new("tool")
                .about("Run the selected tool")
                .subcommand_required(true)
                .subcommand(Command::new("build")),
        );

        let derived = derive(&state);

        assert!(!derived.validation.is_valid);
        assert_eq!(
            derived.validation.summary.as_deref(),
            Some("Missing required subcommand")
        );
        assert!(derived.validation.field_errors.is_empty());
    }

    #[test]
    fn multiple_missing_required_args_are_pluralized() {
        let state = AppState::from_command(
            &Command::new("tool")
                .arg(Arg::new("name").long("name").required(true))
                .arg(Arg::new("path").required(true)),
        );

        let derived = derive(&state);

        assert!(!derived.validation.is_valid);
        assert_eq!(
            derived.validation.summary.as_deref(),
            Some("Missing required arguments: --name, path")
        );
        assert!(derived.validation.field_errors.contains_key("name"));
        assert!(derived.validation.field_errors.contains_key("path"));
    }

    #[test]
    fn version_display_actions_validate_as_successful_terminal_actions() {
        let mut state = AppState::from_command(&Command::new("tool").version("1.2.3"));
        let version_id = state
            .domain
            .current_command()
            .args
            .iter()
            .find(|arg| arg.action_kind() == crate::spec::ArgActionKind::Version)
            .expect("version arg should be present")
            .id
            .clone();
        state.domain.toggle_flag_touched(&version_id);

        let derived = derive(&state);

        assert_eq!(
            derived.argv,
            vec!["tool".to_string(), "--version".to_string()]
        );
        assert!(derived.validation.is_valid);
        assert_eq!(derived.validation.summary, None);
        assert!(derived.validation.field_errors.is_empty());
    }

    #[test]
    fn invalid_value_summary_uses_arg_and_value_context() {
        let state = AppState::from_command(
            &Command::new("tool").arg(
                Arg::new("color")
                    .long("color")
                    .value_parser(["red", "green"]),
            ),
        );
        let argv = vec![
            "tool".to_string(),
            "--color".to_string(),
            "orange".to_string(),
        ];

        let validation = super::validate_argv(&state, &argv);

        assert!(!validation.is_valid);
        assert_eq!(
            validation.summary.as_deref(),
            Some("Invalid value for --color: orange")
        );
        assert_eq!(
            validation.field_errors.get("color"),
            Some(&"Invalid value for --color: orange".to_string())
        );
    }

    #[test]
    fn no_equals_summary_uses_option_specific_message() {
        let state = AppState::from_command(
            &Command::new("tool").arg(
                Arg::new("color")
                    .long("color")
                    .action(ArgAction::Set)
                    .require_equals(true),
            ),
        );
        let argv = vec!["tool".to_string(), "--color".to_string(), "red".to_string()];

        let validation = super::validate_argv(&state, &argv);

        assert!(!validation.is_valid);
        assert_eq!(
            validation.summary.as_deref(),
            Some("Option requires '=': --color")
        );
        assert_eq!(
            validation.field_errors.get("color"),
            Some(&"Option requires '=': --color".to_string())
        );
    }

    #[test]
    fn require_equals_with_delimited_values_uses_attached_value_shape() {
        let root = Command::new("tool")
            .arg(
                Arg::new("input")
                    .long("input")
                    .require_equals(true)
                    .num_args(1..)
                    .value_delimiter(','),
            )
            .subcommand(Command::new("run"));
        let mut state = AppState::from_command(&root);
        state.domain.replace_occurrences(
            "input",
            vec![InputValueOccurrence {
                values: vec!["alpha".to_string(), "beta".to_string()],
                source: InputSource::User,
            }],
        );
        state
            .select_command_path(&["run".to_string()])
            .expect("valid subcommand");

        let derived = derive(&state);

        assert_eq!(
            derived.argv,
            vec![
                "tool".to_string(),
                "--input=alpha,beta".to_string(),
                "run".to_string(),
            ]
        );
        assert!(derived.validation.is_valid);
    }

    #[test]
    fn last_positional_inserts_double_dash_boundary() {
        let mut state = AppState::from_command(
            &Command::new("tool")
                .arg(Arg::new("mode").long("mode"))
                .arg(Arg::new("cmd").last(true).required(true)),
        );
        state.domain.set_text_value("mode", "fast");
        state.domain.set_text_value("cmd", "--not-a-flag");

        let derived = derive(&state);

        assert_eq!(
            derived.argv,
            vec![
                "tool".to_string(),
                "--mode".to_string(),
                "fast".to_string(),
                "--".to_string(),
                "--not-a-flag".to_string(),
            ]
        );
        assert!(derived.validation.is_valid);
    }

    #[test]
    fn trailing_var_arg_inserts_double_dash_boundary() {
        let mut state = AppState::from_command(
            &Command::new("tool").arg(
                Arg::new("args")
                    .trailing_var_arg(true)
                    .num_args(1..)
                    .required(true),
            ),
        );
        state.domain.set_text_value("args", "--flag\nsubcommand");

        let derived = derive(&state);

        assert_eq!(
            derived.argv,
            vec![
                "tool".to_string(),
                "--".to_string(),
                "--flag".to_string(),
                "subcommand".to_string(),
            ]
        );
        assert!(derived.validation.is_valid);
    }

    #[test]
    fn trailing_var_arg_does_not_satisfy_previous_required_positional() {
        let mut state = AppState::from_command(
            &Command::new("tool")
                .arg(Arg::new("program").required(true).index(1))
                .arg(
                    Arg::new("argv")
                        .index(2)
                        .action(ArgAction::Append)
                        .num_args(1..)
                        .trailing_var_arg(true)
                        .allow_hyphen_values(true),
                ),
        );
        state.domain.set_text_value("argv", "a");

        let derived = derive(&state);

        assert_eq!(
            derived.argv,
            vec!["tool".to_string(), "--".to_string(), "a".to_string()]
        );
        assert!(!derived.validation.is_valid);
        assert_eq!(
            derived.validation.field_errors.get("program"),
            Some(&"Required argument".to_string())
        );
        assert_eq!(
            derived.validation.summary.as_deref(),
            Some("Missing required argument: program")
        );
    }

    #[test]
    fn dont_delimit_trailing_values_keeps_grouped_trailing_values_as_separate_tokens() {
        let mut state = AppState::from_command(
            &Command::new("tool").dont_delimit_trailing_values(true).arg(
                Arg::new("args")
                    .trailing_var_arg(true)
                    .num_args(1..)
                    .value_delimiter(',')
                    .required(true),
            ),
        );
        state.domain.replace_occurrences(
            "args",
            vec![
                InputValueOccurrence {
                    values: vec!["alpha".to_string(), "beta".to_string()],
                    source: InputSource::User,
                },
                InputValueOccurrence {
                    values: vec!["gamma".to_string()],
                    source: InputSource::User,
                },
            ],
        );

        let derived = derive(&state);

        assert_eq!(
            derived.argv,
            vec![
                "tool".to_string(),
                "--".to_string(),
                "alpha".to_string(),
                "beta".to_string(),
                "gamma".to_string(),
            ]
        );
        assert!(derived.validation.is_valid);
    }

    #[test]
    fn raw_positional_captures_trailing_tokens_after_boundary() {
        let mut state = AppState::from_command(
            &Command::new("tool").arg(Arg::new("raw").raw(true).required(true)),
        );
        state.domain.replace_occurrences(
            "raw",
            vec![InputValueOccurrence {
                values: vec!["--flag".to_string(), "subcommand".to_string()],
                source: InputSource::User,
            }],
        );

        let derived = derive(&state);

        assert_eq!(
            derived.argv,
            vec![
                "tool".to_string(),
                "--".to_string(),
                "--flag".to_string(),
                "subcommand".to_string(),
            ]
        );
        assert!(derived.validation.is_valid);
    }

    #[test]
    fn external_subcommand_flow_preserves_unknown_name_and_trailing_args() {
        let mut state =
            AppState::from_command(&Command::new("tool").allow_external_subcommands(true));
        state
            .domain
            .set_text_value(EXTERNAL_SUBCOMMAND_NAME_ID, "plugin");
        state
            .domain
            .set_text_value(EXTERNAL_SUBCOMMAND_ARGS_ID, "--flag\nvalue");

        let derived = derive(&state);

        assert_eq!(
            derived.argv,
            vec![
                "tool".to_string(),
                "plugin".to_string(),
                "--flag".to_string(),
                "value".to_string(),
            ]
        );
        assert!(derived.validation.is_valid);
    }

    #[test]
    fn known_subcommand_selection_takes_precedence_over_external_subcommand_fields() {
        let mut state = AppState::from_command(
            &Command::new("tool")
                .allow_external_subcommands(true)
                .subcommand(Command::new("build")),
        );
        state
            .domain
            .set_text_value(EXTERNAL_SUBCOMMAND_NAME_ID, "plugin");
        state
            .domain
            .set_text_value(EXTERNAL_SUBCOMMAND_ARGS_ID, "--flag\nvalue");
        state
            .select_command_path(&["build".to_string()])
            .expect("valid subcommand");

        let derived = derive(&state);

        assert_eq!(derived.argv, vec!["tool".to_string(), "build".to_string()]);
        assert!(derived.validation.is_valid);
    }

    #[test]
    fn positional_value_terminator_is_emitted_before_following_positionals() {
        let mut state = AppState::from_command(
            &Command::new("tool")
                .arg(
                    Arg::new("inputs")
                        .num_args(1..)
                        .required(true)
                        .value_terminator(";"),
                )
                .arg(Arg::new("target").required(true)),
        );
        state.domain.set_text_value("inputs", "alpha\nbeta");
        state.domain.set_text_value("target", "release");

        let derived = derive(&state);

        assert_eq!(
            derived.argv,
            vec![
                "tool".to_string(),
                "alpha".to_string(),
                "beta".to_string(),
                ";".to_string(),
                "release".to_string(),
            ]
        );
        assert!(derived.validation.is_valid);
    }

    #[test]
    fn option_value_terminator_is_emitted_before_following_subcommand() {
        let root = Command::new("tool")
            .arg(
                Arg::new("inputs")
                    .long("input")
                    .num_args(1..)
                    .value_terminator(";"),
            )
            .subcommand(Command::new("run"));
        let mut state = AppState::from_command(&root);
        state.domain.set_text_value("inputs", "alpha\nbeta");
        state
            .select_command_path(&["run".to_string()])
            .expect("valid subcommand");

        let derived = derive(&state);

        assert_eq!(
            derived.argv,
            vec![
                "tool".to_string(),
                "--input".to_string(),
                "alpha".to_string(),
                "beta".to_string(),
                ";".to_string(),
                "run".to_string(),
            ]
        );
        assert!(derived.validation.is_valid);
    }

    #[test]
    fn hyphen_prefixed_positional_values_are_preserved_when_allowed() {
        let mut state = AppState::from_command(
            &Command::new("tool").arg(Arg::new("pattern").allow_hyphen_values(true).required(true)),
        );
        state.domain.set_text_value("pattern", "-weird");

        let derived = derive(&state);

        assert_eq!(derived.argv, vec!["tool".to_string(), "-weird".to_string()]);
        assert!(derived.validation.is_valid);
    }

    #[test]
    fn negative_positional_values_are_preserved_when_allowed() {
        let mut state = AppState::from_command(
            &Command::new("tool").arg(
                Arg::new("count")
                    .allow_negative_numbers(true)
                    .value_parser(clap::value_parser!(i64))
                    .required(true),
            ),
        );
        state.domain.set_text_value("count", "-42");

        let derived = derive(&state);

        assert_eq!(derived.argv, vec!["tool".to_string(), "-42".to_string()]);
        assert!(derived.validation.is_valid);
    }

    #[test]
    fn args_conflicting_with_subcommands_are_reported_through_validation() {
        let mut state = AppState::from_command(
            &Command::new("tool")
                .args_conflicts_with_subcommands(true)
                .arg(
                    Arg::new("verbose")
                        .long("verbose")
                        .action(ArgAction::SetTrue),
                )
                .subcommand(Command::new("build")),
        );
        state.domain.toggle_flag_touched("verbose");
        state
            .select_command_path(&["build".to_string()])
            .expect("valid subcommand");

        let derived = derive(&state);

        assert!(!derived.validation.is_valid);
        assert!(
            derived
                .validation
                .summary
                .as_deref()
                .is_some_and(|summary| summary.contains("--verbose"))
        );
    }
}
