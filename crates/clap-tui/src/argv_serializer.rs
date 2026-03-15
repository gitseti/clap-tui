use crate::input::{ArgValue, CommandFormState};
use crate::spec::{CommandModel, InputPresentation};

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
            Some(ArgValue::Bool(true)) => argv.push(arg.display_name.clone()),
            Some(ArgValue::Text(value)) if !value.is_empty() => match arg.input_presentation() {
                InputPresentation::FreeText {
                    multiple: true,
                    positional: true,
                } => {
                    for part in value.lines().filter(|s| !s.trim().is_empty()) {
                        if let Some(index) = arg.position {
                            positionals.push((index, positional_sequence, part.to_string()));
                            positional_sequence += 1;
                        }
                    }
                }
                InputPresentation::FreeText {
                    multiple: false,
                    positional: true,
                } => {
                    if let Some(index) = arg.position {
                        positionals.push((index, positional_sequence, value.clone()));
                        positional_sequence += 1;
                    }
                }
                InputPresentation::FreeText {
                    multiple: true,
                    positional: false,
                } => {
                    for part in value.lines().filter(|s| !s.trim().is_empty()) {
                        argv.push(arg.display_name.clone());
                        argv.push(part.to_string());
                    }
                }
                InputPresentation::FreeText {
                    multiple: false,
                    positional: false,
                } => {
                    argv.push(arg.display_name.clone());
                    argv.push(value.clone());
                }
                InputPresentation::Toggle | InputPresentation::ChoiceList { .. } => {}
            },
            Some(ArgValue::Choice(value)) => match arg.input_presentation() {
                InputPresentation::ChoiceList {
                    positional: true, ..
                } => {
                    if let Some(index) = arg.position {
                        positionals.push((index, positional_sequence, value.clone()));
                        positional_sequence += 1;
                    }
                }
                InputPresentation::ChoiceList {
                    positional: false, ..
                } => {
                    argv.push(arg.display_name.clone());
                    argv.push(value.clone());
                }
                InputPresentation::Toggle | InputPresentation::FreeText { .. } => {}
            },
            _ => {}
        }
    }

    positionals.sort_by_key(|(index, sequence, _)| (*index, *sequence));
    argv.extend(positionals.into_iter().map(|(_, _, value)| value));
    argv
}

pub(crate) fn missing_required(command: &CommandModel, state: &CommandFormState) -> Vec<String> {
    command
        .args
        .iter()
        .filter(|arg| arg.required)
        .filter_map(|arg| match state.values.get(&arg.id) {
            Some(ArgValue::Text(value)) if !value.is_empty() => None,
            Some(ArgValue::Bool(true) | ArgValue::Choice(_)) => None,
            _ => Some(arg.display_name.clone()),
        })
        .collect()
}
