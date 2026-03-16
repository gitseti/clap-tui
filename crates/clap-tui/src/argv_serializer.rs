use crate::input::{ArgInput, CommandFormState, InputSource};
use crate::spec::CommandModel;

pub(crate) fn build_argv(command: &CommandModel, state: &CommandFormState) -> Vec<String> {
    let mut argv = Vec::new();
    let mut positionals: Vec<(usize, usize, String)> = Vec::new();
    let mut positional_sequence = 0usize;

    for arg in &command.args {
        let Some(input) = state.input(&arg.id) else {
            continue;
        };
        if should_omit_input(input) {
            continue;
        }

        match &input.value {
            ArgInput::Flag { present: true, .. } if arg.uses_toggle_semantics() => {
                argv.push(arg.display_name.clone());
            }
            ArgInput::Values { occurrences } => {
                let values = occurrences
                    .iter()
                    .flat_map(|occurrence| occurrence.values.iter())
                    .filter(|value| !value.is_empty())
                    .cloned()
                    .collect::<Vec<_>>();
                if values.is_empty() {
                    continue;
                }
                if arg.accepts_multiple_values() && arg.serializes_as_positional() {
                    for value in values {
                        if let Some(index) = arg.position {
                            positionals.push((index, positional_sequence, value));
                            positional_sequence += 1;
                        }
                    }
                } else if arg.serializes_as_positional() {
                    if let Some(index) = arg.position {
                        positionals.push((index, positional_sequence, values[0].clone()));
                        positional_sequence += 1;
                    }
                } else if arg.accepts_multiple_values() {
                    for value in values {
                        argv.push(arg.display_name.clone());
                        argv.push(value);
                    }
                } else {
                    argv.push(arg.display_name.clone());
                    argv.push(values[0].clone());
                }
            }
            ArgInput::Count { occurrences, .. } => {
                for _ in 0..*occurrences {
                    argv.push(arg.display_name.clone());
                }
            }
            ArgInput::Flag { .. } => {}
        }
    }

    positionals.sort_by_key(|(index, sequence, _)| (*index, *sequence));
    argv.extend(positionals.into_iter().map(|(_, _, value)| value));
    argv
}

fn should_omit_input(input: &crate::input::ArgInputState) -> bool {
    if input.touched {
        return false;
    }

    match &input.value {
        ArgInput::Flag { source, .. } | ArgInput::Count { source, .. } => {
            *source != InputSource::User
        }
        ArgInput::Values { occurrences } => occurrences
            .iter()
            .all(|occurrence| occurrence.source != InputSource::User),
    }
}
