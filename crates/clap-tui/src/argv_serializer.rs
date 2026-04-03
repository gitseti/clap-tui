use crate::input::{ArgInput, CommandFormState, InputSource};
use crate::spec::{
    ArgModel, CommandModel, EXTERNAL_SUBCOMMAND_ARGS_ID, EXTERNAL_SUBCOMMAND_NAME_ID,
};

#[derive(Debug, Clone)]
struct PositionalPlan {
    position: usize,
    sequence: usize,
    occurrences: Vec<Vec<String>>,
    value_delimiter: Option<char>,
    insert_boundary_before: bool,
    terminator: Option<String>,
}

#[allow(clippy::too_many_lines)]
pub(crate) fn build_argv(
    command: &CommandModel,
    state: &CommandFormState,
    has_following_command: bool,
) -> Vec<String> {
    let mut argv = Vec::new();
    let mut positionals = Vec::new();
    let mut positional_sequence = 0usize;
    let mut saw_later_positionals = false;

    for arg in &command.args {
        let Some(input) = state.input(&arg.id) else {
            continue;
        };
        if should_omit_input(input) {
            continue;
        }
        if matches!(&input.value, ArgInput::Values { .. }) && arg.serializes_as_positional() {
            saw_later_positionals = true;
        }
    }

    for arg in &command.args {
        let Some(input) = state.input(&arg.id) else {
            continue;
        };
        if should_omit_input(input) {
            continue;
        }

        match &input.value {
            ArgInput::Flag { present: true, .. }
                if arg.uses_toggle_semantics() || arg.uses_optional_value_semantics() =>
            {
                argv.push(arg.display_name.clone());
            }
            ArgInput::Values { occurrences } => {
                let non_empty_occurrences = occurrences
                    .iter()
                    .map(|occurrence| {
                        occurrence
                            .values
                            .iter()
                            .filter(|value| !value.is_empty())
                            .cloned()
                            .collect::<Vec<_>>()
                    })
                    .filter(|values| !values.is_empty())
                    .collect::<Vec<_>>();
                if non_empty_occurrences.is_empty() {
                    continue;
                }
                if arg.serializes_as_positional() {
                    if let Some(index) = arg.position {
                        positionals.push(PositionalPlan {
                            position: index,
                            sequence: positional_sequence,
                            occurrences: non_empty_occurrences,
                            value_delimiter: arg.metadata.syntax.value_delimiter,
                            insert_boundary_before: arg.is_last_positional()
                                || arg.is_trailing_var_arg(),
                            terminator: arg.value_terminator().map(str::to_string),
                        });
                        positional_sequence += 1;
                    }
                } else {
                    for values in non_empty_occurrences {
                        serialize_option_occurrence(
                            &mut argv,
                            arg,
                            &values,
                            saw_later_positionals || has_following_command,
                        );
                    }
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

    positionals.sort_by_key(|plan| (plan.position, plan.sequence));
    let mut inserted_boundary = false;
    for (index, plan) in positionals.iter().enumerate() {
        if plan.insert_boundary_before && !inserted_boundary {
            argv.push("--".to_string());
            inserted_boundary = true;
        }
        let join_with_delimiter = plan.value_delimiter.is_some()
            && !(inserted_boundary && command.parser_rules.dont_delimit_trailing_values);
        for occurrence in &plan.occurrences {
            if join_with_delimiter {
                argv.push(join_positional_occurrence(plan.value_delimiter, occurrence));
            } else {
                argv.extend(occurrence.iter().cloned());
            }
        }
        if let Some(terminator) = plan.terminator.as_ref()
            && (index + 1 < positionals.len() || has_following_command)
        {
            argv.push(terminator.clone());
        }
    }
    if !has_following_command {
        append_external_subcommand(command, state, &mut argv);
    }
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

fn serialize_option_occurrence(
    argv: &mut Vec<String>,
    arg: &ArgModel,
    values: &[String],
    needs_terminator: bool,
) {
    if values.is_empty() {
        return;
    }

    if arg.metadata.syntax.require_equals {
        let Some(first) = values.first() else {
            return;
        };
        let attached = if arg.accepts_multiple_values_per_occurrence()
            && arg.metadata.syntax.value_delimiter.is_some()
        {
            join_occurrence_values(arg, values)
        } else {
            first.clone()
        };
        argv.push(format!("{}={attached}", arg.display_name));
        if arg.accepts_multiple_values_per_occurrence()
            && arg.metadata.syntax.value_delimiter.is_none()
        {
            argv.extend(values.iter().skip(1).cloned());
        }
        return;
    }

    argv.push(arg.display_name.clone());
    if arg.accepts_multiple_values_per_occurrence() {
        if arg.metadata.syntax.value_delimiter.is_some() {
            argv.push(join_occurrence_values(arg, values));
        } else {
            argv.extend(values.iter().cloned());
        }
        if needs_terminator && let Some(terminator) = arg.value_terminator() {
            argv.push(terminator.to_string());
        }
    } else if let Some(value) = values.first() {
        argv.push(value.clone());
    }
}

fn join_occurrence_values(arg: &ArgModel, values: &[String]) -> String {
    arg.metadata.syntax.value_delimiter.map_or_else(
        || values.join(" "),
        |delimiter| values.join(&delimiter.to_string()),
    )
}

fn join_positional_occurrence(value_delimiter: Option<char>, values: &[String]) -> String {
    value_delimiter.map_or_else(
        || values.join(" "),
        |delimiter| values.join(&delimiter.to_string()),
    )
}

fn append_external_subcommand(
    command: &CommandModel,
    state: &CommandFormState,
    argv: &mut Vec<String>,
) {
    if !command.parser_rules.allow_external_subcommands {
        return;
    }

    let name = state
        .input(EXTERNAL_SUBCOMMAND_NAME_ID)
        .and_then(|input| match &input.value {
            ArgInput::Values { occurrences } => occurrences
                .iter()
                .flat_map(|occurrence| occurrence.values.iter())
                .find(|value| !value.trim().is_empty())
                .cloned(),
            _ => None,
        });
    let Some(name) = name else {
        return;
    };
    argv.push(name);

    if let Some(input) = state.input(EXTERNAL_SUBCOMMAND_ARGS_ID)
        && let ArgInput::Values { occurrences } = &input.value
    {
        argv.extend(
            occurrences
                .iter()
                .flat_map(|occurrence| occurrence.values.iter())
                .filter(|value| !value.is_empty())
                .cloned(),
        );
    }
}
