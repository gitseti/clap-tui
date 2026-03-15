use crate::input::{AppState, ArgValue};

pub(crate) fn build_argv(state: &AppState) -> Vec<String> {
    let mut command_line = vec![state.command.root.name.clone()];
    command_line.extend(state.command.selected_path.iter().cloned());

    let inputs = state.current_inputs();
    let current_args = state.current_command().args.iter();
    let mut positionals: Vec<(usize, usize, String)> = Vec::new();
    let mut positional_sequence = 0_usize;

    for arg in current_args {
        let is_touched = state.is_touched(&arg.id);
        if arg.default.is_some() && !is_touched {
            continue;
        }
        match inputs.and_then(|i| i.values.get(&arg.id)) {
            Some(ArgValue::Bool(true)) => {
                command_line.push(arg.name.clone());
            }
            Some(ArgValue::Text(value)) if !value.is_empty() => {
                if arg.is_positional() {
                    if let Some(idx) = arg.positional_index {
                        if arg.is_multi {
                            for part in value.lines().filter(|s| !s.trim().is_empty()) {
                                positionals.push((idx, positional_sequence, part.to_string()));
                                positional_sequence += 1;
                            }
                        } else {
                            positionals.push((idx, positional_sequence, value.clone()));
                            positional_sequence += 1;
                        }
                    }
                } else if arg.is_multi {
                    for part in value.lines().filter(|s| !s.trim().is_empty()) {
                        command_line.push(arg.name.clone());
                        command_line.push(part.to_string());
                    }
                } else {
                    command_line.push(arg.name.clone());
                    command_line.push(value.clone());
                }
            }
            Some(ArgValue::Enum(idx)) => {
                if let Some(val) = arg.possible_values.get(*idx) {
                    if arg.is_positional() {
                        if let Some(positional_index) = arg.positional_index {
                            positionals.push((positional_index, positional_sequence, val.clone()));
                            positional_sequence += 1;
                        }
                    } else {
                        command_line.push(arg.name.clone());
                        command_line.push(val.clone());
                    }
                }
            }
            _ => {}
        }
    }

    positionals.sort_by_key(|(idx, seq, _)| (*idx, *seq));
    for (_, _, value) in positionals {
        command_line.push(value);
    }

    command_line
}

#[allow(dead_code)]
pub(crate) fn missing_required(state: &AppState) -> Vec<String> {
    let inputs = state.current_inputs();
    state
        .current_command()
        .args
        .iter()
        .filter(|arg| arg.required)
        .filter_map(|arg| match inputs.and_then(|i| i.values.get(&arg.id)) {
            Some(ArgValue::Text(value)) if !value.is_empty() => None,
            Some(ArgValue::Bool(true) | ArgValue::Enum(_)) => None,
            _ => Some(arg.name.clone()),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::build_argv;
    use crate::input::{AppState, ArgValue};
    use crate::spec::{ArgKind, ArgSpec, CommandSpec};

    fn arg(id: &str, name: &str, kind: ArgKind) -> ArgSpec {
        ArgSpec {
            id: id.to_string(),
            name: name.to_string(),
            help: None,
            required: false,
            kind,
            default: None,
            possible_values: Vec::new(),
            positional_index: None,
            is_multi: false,
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
        name.default = Some("world".to_string());
        let mut state = app_state(vec![name]);
        state.ensure_defaults();

        assert_eq!(build_argv(&state), vec!["tool".to_string()]);
    }

    #[test]
    fn touched_text_defaults_are_preserved() {
        let mut name = arg("name", "--name", ArgKind::Option);
        name.default = Some("world".to_string());
        let mut state = app_state(vec![name]);
        state.ensure_defaults();
        state.set_text_value("name", "world".to_string());
        state.mark_touched("name");

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
        color.possible_values = vec!["red".to_string(), "blue".to_string()];
        let mut state = app_state(vec![flag, color]);
        state.ensure_defaults();
        state.toggle_flag("verbose");
        state.mark_touched("verbose");
        state
            .current_inputs_mut()
            .values
            .insert("color".to_string(), ArgValue::Enum(1));
        state.mark_touched("color");

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
        color.default = Some("blue".to_string());
        color.possible_values = vec!["red".to_string(), "blue".to_string()];
        let mut state = app_state(vec![color]);
        state.ensure_defaults();

        assert_eq!(build_argv(&state), vec!["tool".to_string()]);
    }

    #[test]
    fn touched_enum_defaults_are_preserved() {
        let mut color = arg("color", "--color", ArgKind::Enum);
        color.default = Some("blue".to_string());
        color.possible_values = vec!["red".to_string(), "blue".to_string()];
        let mut state = app_state(vec![color]);
        state.ensure_defaults();
        state
            .current_inputs_mut()
            .values
            .insert("color".to_string(), ArgValue::Enum(1));
        state.mark_touched("color");

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
        paths.is_multi = true;
        let mut state = app_state(vec![paths]);
        state.set_text_value("path", "a\nb".to_string());
        state.mark_touched("path");

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
        input.positional_index = Some(2);
        input.is_multi = true;
        let mut output = arg("output", "output", ArgKind::Positional);
        output.positional_index = Some(1);
        let mut state = app_state(vec![input, output]);
        state.set_text_value("input", "a\nb".to_string());
        state.mark_touched("input");
        state.set_text_value("output", "dest".to_string());
        state.mark_touched("output");

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
        mode.positional_index = Some(1);
        mode.possible_values = vec!["fast".to_string(), "slow".to_string()];
        let mut state = app_state(vec![mode]);
        state
            .current_inputs_mut()
            .values
            .insert("mode".to_string(), ArgValue::Enum(1));
        state.mark_touched("mode");

        assert_eq!(
            build_argv(&state),
            vec!["tool".to_string(), "slow".to_string()]
        );
    }
}
