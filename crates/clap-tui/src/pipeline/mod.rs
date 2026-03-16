use std::collections::BTreeMap;

use crate::input::AppState;

mod argv;
mod validation;

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
}

pub(crate) fn derive(state: &AppState) -> DerivedState {
    let argv = argv::build_command_line(state);
    DerivedState {
        validation: validation::validate_argv(state, &argv),
        argv,
    }
}

pub(crate) fn build_command_line(state: &AppState) -> Vec<String> {
    argv::build_command_line(state)
}

pub(crate) fn validate_argv(state: &AppState, argv: &[String]) -> ValidationState {
    validation::validate_argv(state, argv)
}

#[cfg(test)]
mod tests {
    use clap::{Arg, ArgAction, Command};

    use super::derive;
    use crate::input::AppState;
    use crate::spec::{ArgKind, ArgSpec, CommandSpec};

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
}
