use std::collections::BTreeMap;

use clap::ArgMatches;
use clap::parser::ValueSource;

use crate::input::{AppState, ArgInput, CommandFormState};
use crate::spec::ArgSpec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EffectiveValueSource {
    User,
    Default,
    Env,
    DefaultMissing,
    ConditionalDefault,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EffectiveArgValue {
    pub(crate) source: EffectiveValueSource,
    pub(crate) values: Vec<String>,
}

pub(crate) fn derive_effective_values(
    state: &AppState,
    argv: &[String],
) -> BTreeMap<String, EffectiveArgValue> {
    let Some(command) = state.domain.validation_command.as_ref() else {
        return BTreeMap::new();
    };
    let Ok(matches) = command.clone().try_get_matches_from(argv.iter().cloned()) else {
        return BTreeMap::new();
    };
    let Some(current_matches) =
        matches_for_selected_path(&matches, state.domain.selected_path().as_slice())
    else {
        return BTreeMap::new();
    };

    let current_form = state.domain.current_form().unwrap_or_default();
    state
        .domain
        .current_command()
        .all_args()
        .into_iter()
        .filter(|arg| !arg.is_help_action() && !arg.is_external_subcommand_field())
        .filter_map(|arg| {
            let source = current_matches.value_source(&arg.id)?;
            let values = raw_values_for_arg(current_matches, arg);
            Some((
                arg.id.clone(),
                EffectiveArgValue {
                    source: effective_source(arg, &current_form, source, &values),
                    values,
                },
            ))
        })
        .collect()
}

fn matches_for_selected_path<'a>(
    matches: &'a ArgMatches,
    path: &[String],
) -> Option<&'a ArgMatches> {
    let mut current = matches;
    for segment in path {
        let (name, next) = current.subcommand()?;
        if name != segment {
            return None;
        }
        current = next;
    }
    Some(current)
}

fn raw_values_for_arg(matches: &ArgMatches, arg: &ArgSpec) -> Vec<String> {
    if arg.accepts_values() {
        return matches
            .get_raw_occurrences(&arg.id)
            .map(|occurrences| {
                occurrences
                    .flat_map(|occurrence| {
                        occurrence.map(|value| value.to_string_lossy().to_string())
                    })
                    .collect()
            })
            .or_else(|| {
                matches.get_raw(&arg.id).map(|values| {
                    values
                        .map(|value| value.to_string_lossy().to_string())
                        .collect()
                })
            })
            .unwrap_or_default();
    }

    if arg.uses_count_semantics() {
        let count = matches.get_count(&arg.id);
        return if count > 0 {
            vec![count.to_string()]
        } else {
            Vec::new()
        };
    }

    if arg.is_flag() && matches.contains_id(&arg.id) {
        return vec![matches.get_flag(&arg.id).to_string()];
    }

    Vec::new()
}

fn effective_source(
    arg: &ArgSpec,
    current_form: &CommandFormState,
    source: ValueSource,
    values: &[String],
) -> EffectiveValueSource {
    match source {
        ValueSource::CommandLine => {
            if arg.uses_optional_value_semantics()
                && current_form.input(&arg.id).is_some_and(|input| {
                    matches!(input.value, ArgInput::Flag { present: true, .. })
                })
            {
                EffectiveValueSource::DefaultMissing
            } else {
                EffectiveValueSource::User
            }
        }
        ValueSource::DefaultValue => {
            if arg.has_conditional_defaults()
                && (arg.default_values.is_empty() || values != arg.default_values)
            {
                EffectiveValueSource::ConditionalDefault
            } else {
                EffectiveValueSource::Default
            }
        }
        ValueSource::EnvVariable => EffectiveValueSource::Env,
        _ => EffectiveValueSource::User,
    }
}
