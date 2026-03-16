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
    DerivedState {
        argv: argv::build_command_line(state),
        validation: validation::build_validation_state(state),
    }
}

pub(crate) fn build_command_line(state: &AppState) -> Vec<String> {
    argv::build_command_line(state)
}

#[cfg(test)]
mod tests {
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
        })
    }

    #[test]
    fn derived_state_keeps_preview_and_run_argv_aligned() {
        let mut state = app_state(vec![arg("verbose", "--verbose", ArgKind::Flag)]);
        state.domain.toggle_flag_touched("verbose");

        let derived = derive(&state);

        assert_eq!(derived.argv, vec!["tool".to_string(), "--verbose".to_string()]);
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
            Some("Missing required: --name")
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
}
