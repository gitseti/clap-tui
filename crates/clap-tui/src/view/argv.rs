use crate::argv_serializer;
use crate::input::AppState;

pub(crate) fn build_argv(state: &AppState) -> Vec<String> {
    let mut command_line = vec![state.domain.root.name.clone()];
    command_line.extend(state.domain.selected_path().iter().cloned());
    let form = state.domain.current_form().cloned().unwrap_or_default();
    command_line.extend(argv_serializer::build_argv(state.domain.current_command(), &form));
    command_line
}

#[allow(dead_code)]
pub(crate) fn missing_required(state: &AppState) -> Vec<String> {
    argv_serializer::missing_required(
        state.domain.current_command(),
        &state.domain.current_form().cloned().unwrap_or_default(),
    )
}

#[cfg(test)]
mod tests {
    use super::build_argv;
    use crate::input::{AppState, ArgValue};
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
            about: None,
            help: String::new(),
            args,
            subcommands: Vec::new(),
        })
    }

    #[test]
    fn untouched_defaults_are_omitted() {
        let mut name = arg("name", "--name", ArgKind::Option);
        name.default_values = vec!["world".to_string()];
        let mut state = app_state(vec![name]);
        state.domain.ensure_defaults();

        assert_eq!(build_argv(&state), vec!["tool".to_string()]);
    }

    #[test]
    fn touched_text_defaults_are_preserved() {
        let mut name = arg("name", "--name", ArgKind::Option);
        name.default_values = vec!["world".to_string()];
        let mut state = app_state(vec![name]);
        state.domain.ensure_defaults();
        state.domain.set_text_value("name", "world".to_string());
        state.domain.mark_touched("name");

        assert_eq!(
            build_argv(&state),
            vec![
                "tool".to_string(),
                "--name".to_string(),
                "world".to_string()
            ]
        );
    }

    #[test]
    fn flags_and_enums_serialize_correctly() {
        let flag = arg("verbose", "--verbose", ArgKind::Flag);
        let mut color = arg("color", "--color", ArgKind::Enum);
        color.choices = vec!["red".to_string(), "blue".to_string()];
        let mut state = app_state(vec![flag, color]);
        state.domain.ensure_defaults();
        state.domain.toggle_flag("verbose");
        state.domain.mark_touched("verbose");
        state
            .domain
            .current_form_mut()
            .values
            .insert("color".to_string(), ArgValue::Choice("blue".to_string()));
        state.domain.mark_touched("color");

        assert_eq!(
            build_argv(&state),
            vec![
                "tool".to_string(),
                "--verbose".to_string(),
                "--color".to_string(),
                "blue".to_string(),
            ]
        );
    }

    #[test]
    fn untouched_enum_defaults_are_omitted() {
        let mut color = arg("color", "--color", ArgKind::Enum);
        color.default_values = vec!["blue".to_string()];
        color.choices = vec!["red".to_string(), "blue".to_string()];
        let mut state = app_state(vec![color]);
        state.domain.ensure_defaults();

        assert_eq!(build_argv(&state), vec!["tool".to_string()]);
    }

    #[test]
    fn touched_enum_defaults_are_preserved() {
        let mut color = arg("color", "--color", ArgKind::Enum);
        color.default_values = vec!["blue".to_string()];
        color.choices = vec!["red".to_string(), "blue".to_string()];
        let mut state = app_state(vec![color]);
        state.domain.ensure_defaults();
        state
            .domain
            .current_form_mut()
            .values
            .insert("color".to_string(), ArgValue::Choice("blue".to_string()));
        state.domain.mark_touched("color");

        assert_eq!(
            build_argv(&state),
            vec![
                "tool".to_string(),
                "--color".to_string(),
                "blue".to_string()
            ]
        );
    }

    #[test]
    fn multi_value_options_repeat_flag_value_pairs() {
        let mut paths = arg("path", "--path", ArgKind::Option);
        paths.value_cardinality = crate::spec::ValueCardinality::Many;
        let mut state = app_state(vec![paths]);
        state.domain.set_text_value("path", "a\nb".to_string());
        state.domain.mark_touched("path");

        assert_eq!(
            build_argv(&state),
            vec![
                "tool".to_string(),
                "--path".to_string(),
                "a".to_string(),
                "--path".to_string(),
                "b".to_string(),
            ]
        );
    }

    #[test]
    fn multi_value_positionals_preserve_positional_ordering() {
        let mut input = arg("input", "input", ArgKind::Positional);
        input.position = Some(2);
        input.value_cardinality = crate::spec::ValueCardinality::Many;
        let mut output = arg("output", "output", ArgKind::Positional);
        output.position = Some(1);
        let mut state = app_state(vec![input, output]);
        state.domain.set_text_value("input", "a\nb".to_string());
        state.domain.mark_touched("input");
        state.domain.set_text_value("output", "dest".to_string());
        state.domain.mark_touched("output");

        assert_eq!(
            build_argv(&state),
            vec![
                "tool".to_string(),
                "dest".to_string(),
                "a".to_string(),
                "b".to_string(),
            ]
        );
    }

    #[test]
    fn positional_enums_serialize_as_plain_values() {
        let mut mode = arg("mode", "mode", ArgKind::Enum);
        mode.position = Some(1);
        mode.choices = vec!["fast".to_string(), "slow".to_string()];
        let mut state = app_state(vec![mode]);
        state
            .domain
            .current_form_mut()
            .values
            .insert("mode".to_string(), ArgValue::Choice("slow".to_string()));
        state.domain.mark_touched("mode");

        assert_eq!(
            build_argv(&state),
            vec!["tool".to_string(), "slow".to_string()]
        );
    }
}
