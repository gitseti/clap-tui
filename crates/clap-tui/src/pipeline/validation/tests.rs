use super::*;
use crate::pipeline::derive;
use clap::{Arg, ArgAction, ArgGroup};

fn modes(multiple: bool, help: bool) -> Command {
    Command::new("tool")
        .arg_required_else_help(help)
        .group(
            ArgGroup::new("mode")
                .args(["fast", "safe"])
                .required(true)
                .multiple(multiple),
        )
        .arg(Arg::new("fast").long("fast").action(ArgAction::SetTrue))
        .arg(Arg::new("safe").long("safe").action(ArgAction::SetTrue))
}

#[test]
fn empty_groups_use_clap_requirements_with_correct_multiplicity() {
    for help in [false, true] {
        for multiple in [false, true] {
            let mut state = AppState::from_command(&modes(multiple, help));
            let validation = derive(&state).validation;
            let prefix = if multiple {
                "Choose at least one"
            } else {
                "Choose one"
            };
            assert_eq!(
                validation.summary(),
                Some(format!("{prefix}: --fast or --safe"))
            );
            assert_eq!(validation.issues.len(), 1);
            assert!(validation.field_error("safe").is_some());
            assert!(validation.inline_message("fast").is_some());
            assert!(validation.inline_message("safe").is_none());
            state.domain.toggle_flag_touched("safe");
            assert!(derive(&state).validation.is_valid());
        }
    }
}

#[test]
fn ambiguous_group_references_do_not_guess_multiplicity() {
    for help in [false, true] {
        for policies in [[true, false], [false, true]] {
            let command = policies.into_iter().enumerate().fold(
                Command::new("tool")
                    .arg_required_else_help(help)
                    .arg(Arg::new("fast").long("fast").action(ArgAction::SetTrue))
                    .arg(Arg::new("safe").long("safe").action(ArgAction::SetTrue)),
                |command, (index, multiple)| {
                    command.group(
                        ArgGroup::new(format!("mode{index}"))
                            .args(["fast", "safe"])
                            .required(true)
                            .multiple(multiple),
                    )
                },
            );
            let mut state = AppState::from_command(&command);
            let validation = derive(&state).validation;
            assert_eq!(
                validation.summary().as_deref(),
                Some("Choose from: --fast or --safe")
            );
            assert_eq!(validation.issues[0].argument_ids(), ["fast", "safe"]);
            state.domain.toggle_flag_touched("fast");
            assert!(derive(&state).validation.is_valid());
            state.domain.toggle_flag_touched("safe");
            assert!(!derive(&state).validation.is_valid());
        }
    }
}

#[test]
fn help_recovery_respects_lineage_and_negated_parent_requirements() {
    let child = Command::new("child")
        .arg_required_else_help(true)
        .group(
            ArgGroup::new("child_mode")
                .args(["quick", "secure"])
                .required(true)
                .multiple(true),
        )
        .arg(Arg::new("quick").long("quick").action(ArgAction::SetTrue))
        .arg(Arg::new("secure").long("secure").action(ArgAction::SetTrue));
    let command = modes(false, true)
        .subcommand_negates_reqs(true)
        .subcommand(child);
    let mut state = AppState::from_command(&command);
    state.select_command_path(&["child".into()]).unwrap();
    assert_eq!(
        derive(&state).validation.summary().as_deref(),
        Some("Choose at least one: --quick or --secure")
    );
    state.domain.toggle_flag_touched("quick");
    assert!(derive(&state).validation.is_valid());
}

#[test]
fn defaults_and_inherited_options_follow_clap_validity() {
    let command = Command::new("tool")
        .arg_required_else_help(true)
        .arg(
            Arg::new("config")
                .long("config")
                .global(true)
                .default_value("default.toml"),
        )
        .subcommand(Command::new("child"));
    let mut state = AppState::from_command(&command);
    state.select_command_path(&["child".into()]).unwrap();
    assert!(derive(&state).validation.is_valid());
    let default_group = modes(false, false).mut_arg("fast", |arg| arg.default_value("true"));
    let expected = default_group.clone().try_get_matches_from(["tool"]).is_ok();
    assert_eq!(
        derive(&AppState::from_command(&default_group))
            .validation
            .is_valid(),
        expected
    );
}

#[test]
fn dependency_and_conditional_requirements_target_the_missing_argument() {
    let command = Command::new("tool")
        .arg(
            Arg::new("enable")
                .long("enable")
                .action(ArgAction::SetTrue)
                .requires("config"),
        )
        .arg(Arg::new("config").long("config"));
    let mut state = AppState::from_command(&command);
    state.domain.toggle_flag_touched("enable");
    let validation = derive(&state).validation;
    assert_eq!(validation.summary().as_deref(), Some("Provide --config"));
    assert_eq!(validation.issues[0].argument_ids(), ["config"]);
    assert!(validation.field_error("enable").is_none());
    state.domain.set_text_value("config", "settings.toml");
    assert!(derive(&state).validation.is_valid());
}

#[test]
fn enum_range_and_custom_validator_explanations_are_preserved() {
    let command = Command::new("tool").arg(
        Arg::new("color")
            .long("color")
            .value_parser(["red", "green"]),
    );
    let mut state = AppState::from_command(&command);
    state.domain.set_text_value("color", "orange");
    let message = derive(&state).validation.summary().unwrap();
    assert!(message.contains("--color") && message.contains("red, green"));
    let command = Command::new("tool").arg(
        Arg::new("port")
            .long("port")
            .value_parser(clap::value_parser!(u16)),
    );
    let mut state = AppState::from_command(&command);
    state.domain.set_text_value("port", "99999");
    assert!(
        derive(&state)
            .validation
            .field_error("port")
            .unwrap()
            .contains("0..=65535")
    );
    let command = Command::new("tool").arg(Arg::new("name").long("name").value_parser(
        |_: &str| -> Result<String, String> {
            Err("use a lowercase project name\nstart with a letter".into())
        },
    ));
    let mut state = AppState::from_command(&command);
    state.domain.set_text_value("name", "UPPER");
    assert!(
        derive(&state)
            .validation
            .field_error("name")
            .unwrap()
            .contains("use a lowercase project name")
    );
}

#[test]
fn unknown_diagnostics_have_no_speculative_jump_target() {
    let state = AppState::from_command(&Command::new("tool").arg(Arg::new("name").long("name")));
    let validation = validate_argv(&state, &["tool".into(), "--unknown".into()]);
    assert!(!validation.is_valid());
    assert!(!validation.has_target());
    assert!(validation.summary().unwrap().contains("--unknown"));
}

#[test]
fn conditional_requirement_tracks_the_active_trigger() {
    let command = Command::new("tool")
        .arg(
            Arg::new("mode")
                .long("mode")
                .value_parser(["local", "remote"]),
        )
        .arg(
            Arg::new("token")
                .long("token")
                .required_if_eq("mode", "remote"),
        );
    let mut state = AppState::from_command(&command);
    state.domain.set_text_value("mode", "local");
    assert!(derive(&state).validation.is_valid());
    state.domain.set_text_value("mode", "remote");
    assert_eq!(
        derive(&state).validation.summary().as_deref(),
        Some("Provide --token")
    );
    state.domain.set_text_value("mode", "local");
    assert!(derive(&state).validation.is_valid());
}

#[test]
fn equal_messages_do_not_collapse_distinct_issues_and_form_order_wins() {
    let state = AppState::from_command(
        &Command::new("tool")
            .arg(Arg::new("first").long("first").display_order(1))
            .arg(Arg::new("second").long("second").display_order(2)),
    );
    let args = crate::query::selectors::visible_form_args(
        &state.domain.root,
        state.domain.selected_path(),
        crate::input::ActiveTab::Inputs,
    );
    let mut validation = ValidationState {
        issues: vec![
            ValidationIssue::arguments("Provide a value", vec!["second".into()]),
            ValidationIssue::arguments("Provide a value", vec!["first".into()]),
        ],
    };
    validation.order_by_form(&args);
    assert_eq!(validation.issues.len(), 2);
    assert_eq!(validation.issues[0].argument_ids(), ["first"]);
    assert_eq!(
        validation.summary().as_deref(),
        Some("Provide a value (+1 more)")
    );
}

#[test]
fn validator_prose_cannot_target_an_unrelated_argument() {
    let command = Command::new("tool")
        .arg(
            Arg::new("name")
                .long("name")
                .value_parser(|_: &str| -> Result<String, String> {
                    Err("use --other only for another purpose".into())
                }),
        )
        .arg(Arg::new("other").long("other"));
    let mut state = AppState::from_command(&command);
    state.domain.set_text_value("name", "bad");
    let validation = derive(&state).validation;
    assert_eq!(validation.issues[0].argument_ids(), ["name"]);
    assert!(validation.field_error("other").is_none());
}

#[test]
fn parent_group_multiplicity_survives_child_selection() {
    let command = modes(true, false).subcommand(Command::new("child"));
    let mut state = AppState::from_command(&command);
    state.select_command_path(&["child".into()]).unwrap();
    assert_eq!(
        derive(&state).validation.summary().as_deref(),
        Some("Choose at least one: --fast or --safe")
    );
}

#[test]
fn active_parent_and_child_groups_keep_their_own_semantics() {
    let child = Command::new("child")
        .group(
            ArgGroup::new("child_mode")
                .args(["quick", "secure"])
                .required(true)
                .multiple(true),
        )
        .arg(Arg::new("quick").long("quick").action(ArgAction::SetTrue))
        .arg(Arg::new("secure").long("secure").action(ArgAction::SetTrue));
    let command = modes(false, false).subcommand(child);
    let mut state = AppState::from_command(&command);
    state.select_command_path(&["child".into()]).unwrap();
    let validation = derive(&state).validation;
    // Clap reports the child's failure first; do not enumerate extra errors.
    assert_eq!(validation.issues.len(), 1);
    assert_eq!(
        validation.issues[0].message,
        "Choose at least one: --quick or --secure"
    );
    state.domain.toggle_flag_touched("quick");
    assert_eq!(
        derive(&state).validation.summary().as_deref(),
        Some("Choose one: --fast or --safe")
    );
    state.domain.toggle_flag_touched("fast");
    assert!(derive(&state).validation.is_valid());
}

#[test]
fn overlapping_groups_keep_each_instruction_at_the_shared_field() {
    let command = Command::new("tool")
        .arg(
            Arg::new("shared")
                .long("shared")
                .action(ArgAction::SetTrue)
                .display_order(0),
        )
        .arg(
            Arg::new("a")
                .long("a")
                .action(ArgAction::SetTrue)
                .display_order(1),
        )
        .arg(
            Arg::new("b")
                .long("b")
                .action(ArgAction::SetTrue)
                .display_order(2),
        )
        .group(ArgGroup::new("first").args(["shared", "a"]).required(true))
        .group(ArgGroup::new("second").args(["shared", "b"]).required(true));
    let validation = derive(&AppState::from_command(&command)).validation;
    assert_eq!(validation.issues.len(), 2);
    assert_eq!(
        validation.inline_message("shared").as_deref(),
        Some("Choose one: --shared or --a\nChoose one: --shared or --b")
    );
    assert!(validation.inline_message("a").is_none());
    assert!(validation.inline_message("b").is_none());
    assert!(validation.field_error("a").is_some());
    assert!(validation.field_error("b").is_some());
}

#[test]
fn distinct_issues_with_equal_text_are_not_deduplicated_inline() {
    let validation = ValidationState {
        issues: vec![
            ValidationIssue::arguments("Choose an alternative", vec!["shared".into(), "a".into()]),
            ValidationIssue::arguments("Choose an alternative", vec!["shared".into(), "b".into()]),
        ],
    };
    assert_eq!(
        validation.inline_message("shared").as_deref(),
        Some("Choose an alternative\nChoose an alternative")
    );
}
