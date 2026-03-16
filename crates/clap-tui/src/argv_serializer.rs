use crate::input::{ArgValue, CommandFormState};
use crate::spec::CommandModel;

pub(crate) fn build_argv(command: &CommandModel, state: &CommandFormState) -> Vec<String> {
    let mut argv = Vec::new();
    let mut positionals: Vec<(usize, usize, String)> = Vec::new();
    let mut positional_sequence = 0usize;

    for arg in &command.args {
        let is_touched = state.touched.contains(&arg.id);
        if !is_touched && !arg.default_values.is_empty() {
            continue;
        }

        match state.values.get(&arg.id) {
            Some(ArgValue::Bool(true)) if arg.uses_toggle_semantics() => {
                argv.push(arg.display_name.clone());
            }
            Some(ArgValue::Text(value)) if !value.is_empty() => {
                if arg.accepts_multiple_values() && arg.serializes_as_positional() {
                    for part in value.lines().filter(|s| !s.trim().is_empty()) {
                        if let Some(index) = arg.position {
                            positionals.push((index, positional_sequence, part.to_string()));
                            positional_sequence += 1;
                        }
                    }
                } else if arg.serializes_as_positional() {
                    if let Some(index) = arg.position {
                        positionals.push((index, positional_sequence, value.clone()));
                        positional_sequence += 1;
                    }
                } else if arg.accepts_multiple_values() {
                    for part in value.lines().filter(|s| !s.trim().is_empty()) {
                        argv.push(arg.display_name.clone());
                        argv.push(part.to_string());
                    }
                } else {
                    argv.push(arg.display_name.clone());
                    argv.push(value.clone());
                }
            }
            Some(ArgValue::Choice(value)) if arg.uses_choice_semantics() => {
                if arg.serializes_as_positional() {
                    if let Some(index) = arg.position {
                        positionals.push((index, positional_sequence, value.clone()));
                        positional_sequence += 1;
                    }
                } else {
                    argv.push(arg.display_name.clone());
                    argv.push(value.clone());
                }
            }
            _ => {}
        }
    }

    positionals.sort_by_key(|(index, sequence, _)| (*index, *sequence));
    argv.extend(positionals.into_iter().map(|(_, _, value)| value));
    argv
}
