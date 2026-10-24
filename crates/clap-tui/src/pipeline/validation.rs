#[cfg(test)]
use std::cell::Cell;
use std::collections::BTreeSet;
use std::ffi::OsString;

use clap::Command;
use clap::error::{ContextKind, ContextValue, ErrorKind};

use crate::input::{AppState, ArgInput, ArgInputState, ArgValue, CommandFormState, InputSource};
use crate::pipeline::{FieldInstanceId, ValidationIssue, ValidationState, ValidationTarget};
use crate::spec::{ArgModel, CommandModel};

mod references;
use references::{ArgumentReference, Reference};

#[cfg(test)]
thread_local! {
    static VALIDATION_CALL_COUNT: Cell<usize> = const { Cell::new(0) };
}

pub(crate) fn validate_argv(state: &AppState, argv: &[OsString]) -> ValidationState {
    #[cfg(test)]
    VALIDATION_CALL_COUNT.with(|count| count.set(count.get() + 1));

    if let Some(command) = state.domain.validation_command.as_ref() {
        return validate_with_clap(state, command.clone(), argv);
    }

    fallback_validation_state(
        state.domain.current_command(),
        &state.domain.current_form().unwrap_or_default(),
    )
}

#[cfg(test)]
pub(crate) fn validation_call_count() -> usize {
    VALIDATION_CALL_COUNT.with(Cell::get)
}

#[cfg(test)]
pub(crate) fn reset_validation_call_count() {
    VALIDATION_CALL_COUNT.with(|count| count.set(0));
}

fn validate_with_clap(
    state: &AppState,
    mut command: Command,
    argv: &[OsString],
) -> ValidationState {
    command.build();
    match command.try_get_matches_from_mut(argv.iter().cloned()) {
        Ok(_) => ValidationState::default(),
        Err(error) if error.kind() == ErrorKind::DisplayVersion => ValidationState::default(),
        Err(error) => {
            if error.kind() == ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand {
                let diagnostic = suppress_help_on_empty(
                    command.clone(),
                    state.domain.selected_path().as_slice(),
                );
                if let Err(requirement) = diagnostic.try_get_matches_from(argv.iter().cloned())
                    && requirement.kind() != ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand
                {
                    return adapt_clap_error(state, &requirement, &command);
                }
            }
            adapt_clap_error(state, &error, &command)
        }
    }
}

fn suppress_help_on_empty(command: Command, path: &[String]) -> Command {
    let command = command.arg_required_else_help(false);
    match path.split_first() {
        Some((name, rest)) => {
            command.mut_subcommand(name, |child| suppress_help_on_empty(child, rest))
        }
        None => command,
    }
}

fn adapt_clap_error(state: &AppState, error: &clap::Error, command: &Command) -> ValidationState {
    let references = context_reference_values(error)
        .iter()
        .filter_map(|value| Reference::parse(value))
        .collect::<Vec<_>>();
    let issues = match error.kind() {
        ErrorKind::MissingRequiredArgument => {
            missing_requirement_issues(state, &references, command)
        }
        ErrorKind::MissingSubcommand => vec![ValidationIssue {
            message: "Choose a command".into(),
            target: ValidationTarget::CommandSelection,
        }],
        _ => Vec::new(),
    };
    if !issues.is_empty() {
        return ValidationState { issues };
    }
    let message = match error.kind() {
        ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand => {
            summary_for_help_style_missing(state)
        }
        ErrorKind::ArgumentConflict => summary_from_conflict_context(error),
        ErrorKind::NoEquals => summary_from_no_equals_context(error),
        ErrorKind::InvalidValue => summary_from_value_context(error),
        ErrorKind::ValueValidation => summary_from_validator(error),
        _ => None,
    }
    .or_else(|| clean_diagnostic_line(&error.to_string(), current_about(state)))
    .unwrap_or_else(|| "Invalid input".into());
    let ids = resolve_reference_args(state, &references)
        .into_iter()
        .map(|arg| arg.id.clone())
        .collect::<Vec<_>>();
    let target = if error.kind() == ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand
        && state
            .domain
            .current_command()
            .parser_rules
            .subcommand_required
    {
        ValidationTarget::CommandSelection
    } else if ids.is_empty() {
        ValidationTarget::None
    } else {
        ValidationTarget::Arguments(ids)
    };
    ValidationState {
        issues: vec![ValidationIssue { message, target }],
    }
}

fn missing_requirement_issues(
    state: &AppState,
    references: &[Reference],
    command: &Command,
) -> Vec<ValidationIssue> {
    let mut issues = missing_group_issues(state, references, command);
    let candidates = visible_arg_models(state).collect::<Vec<_>>();
    for reference in references {
        if !matches!(reference, Reference::Argument(_)) {
            continue;
        }
        let args = reference.resolve(&candidates).unwrap_or_default();
        let message = args.first().map_or_else(
            || format!("Provide {reference}"),
            |arg| format!("Provide {}", arg.display_name),
        );
        issues.push(ValidationIssue::arguments(
            message,
            args.into_iter().map(|arg| arg.id.clone()).collect(),
        ));
    }
    issues
}

fn summary_for_help_style_missing(state: &AppState) -> Option<String> {
    let current_command = state.domain.current_command();
    if current_command.parser_rules.subcommand_required && !current_command.subcommands.is_empty() {
        return Some("Choose a command".to_string());
    }

    if current_command.parser_rules.arg_required_else_help && !current_command_has_user_input(state)
    {
        return Some("Missing required input".to_string());
    }

    None
}

fn summary_from_conflict_context(error: &clap::Error) -> Option<String> {
    let invalid = context_arg_label(error, ContextKind::InvalidArg);
    let prior = context_arg_label(error, ContextKind::PriorArg);
    match (invalid, prior) {
        (Some(invalid), Some(prior)) => Some(format!("Remove one of: {invalid} or {prior}")),
        _ => None,
    }
}

fn summary_from_value_context(error: &clap::Error) -> Option<String> {
    let arg = context_arg_label(error, ContextKind::InvalidArg)?;
    let value = context_string(error, ContextKind::InvalidValue)?;
    let mut message = format!("Invalid value for {arg}: {value}");
    if let Some(ContextValue::Strings(values)) = error.get(ContextKind::ValidValue)
        && !values.is_empty()
    {
        message.push_str(". Choose: ");
        message.push_str(&values.join(", "));
    }
    Some(message)
}

fn summary_from_validator(error: &clap::Error) -> Option<String> {
    let arg = context_arg_label(error, ContextKind::InvalidArg)?;
    let value = context_string(error, ContextKind::InvalidValue)?;
    let explanation = std::error::Error::source(error)?;
    Some(format!("Invalid value for {arg}: {value}. {explanation}"))
}

fn summary_from_no_equals_context(error: &clap::Error) -> Option<String> {
    let arg = context_arg_label(error, ContextKind::InvalidArg)?;
    Some(format!("Option requires '=': {arg}"))
}

fn clean_diagnostic_line(error_text: &str, current_about: Option<&str>) -> Option<String> {
    error_text
        .lines()
        .map(str::trim)
        .take_while(|line| !line.starts_with("Usage:") && !line.starts_with("For more information"))
        .find_map(|line| {
            if line.is_empty() {
                return None;
            }
            if current_about.is_some_and(|about| line == about.trim()) {
                return None;
            }
            if let Some(diagnostic) = line.strip_prefix("error: ") {
                return Some(diagnostic.trim().to_string());
            }
            if line.starts_with("error:") {
                return Some(line.trim_start_matches("error:").trim().to_string());
            }
            None
        })
        .or_else(|| {
            error_text
                .lines()
                .map(str::trim)
                .take_while(|line| {
                    !line.starts_with("Usage:") && !line.starts_with("For more information")
                })
                .find_map(|line| {
                    if line.is_empty() || current_about.is_some_and(|about| line == about.trim()) {
                        return None;
                    }
                    (!line.ends_with(':')).then(|| line.to_string())
                })
        })
}

fn context_reference_values(error: &clap::Error) -> Vec<String> {
    let mut values = Vec::new();

    for key in [
        ContextKind::InvalidArg,
        ContextKind::PriorArg,
        ContextKind::TrailingArg,
    ] {
        if let Some(value) = error.get(key) {
            match value {
                ContextValue::String(value) => values.push(value.clone()),
                ContextValue::Strings(items) => values.extend(items.iter().cloned()),
                ContextValue::StyledStr(value) => values.push(value.to_string()),
                ContextValue::StyledStrs(items) => {
                    values.extend(items.iter().map(ToString::to_string));
                }
                _ => {}
            }
        }
    }

    values
}

fn context_string(error: &clap::Error, key: ContextKind) -> Option<String> {
    match error.get(key) {
        Some(ContextValue::String(value)) => Some(value.clone()),
        Some(ContextValue::StyledStr(value)) => Some(value.to_string()),
        _ => None,
    }
}

fn context_arg_label(error: &clap::Error, key: ContextKind) -> Option<String> {
    Reference::parse(&context_string(error, key)?).map(|reference| reference.to_string())
}

// A group belongs to the command that defines it. A child's selection does not
// transfer its parent's requirements or multiplicity into the child's scope.
struct GroupDefinition<'a> {
    owner: &'a CommandModel,
    members: BTreeSet<&'a str>,
    multiple: bool,
}

impl<'a> GroupDefinition<'a> {
    fn matching_args(&self, references: &[ArgumentReference]) -> Option<Vec<&'a ArgModel>> {
        let args = self
            .owner
            .all_args()
            .into_iter()
            .filter(|arg| self.members.contains(arg.id.as_str()))
            .collect::<Vec<_>>();
        if args.len() != self.members.len() || references.len() != args.len() {
            return None;
        }
        // Consume one member per reference, preserving repeated-label counts.
        let mut unmatched = args.clone();
        for reference in references {
            let index = unmatched.iter().position(|arg| reference.matches(arg))?;
            unmatched.swap_remove(index);
        }
        Some(args)
    }
}

fn group_definitions<'a>(state: &'a AppState, command: &'a Command) -> Vec<GroupDefinition<'a>> {
    let mut definitions = Vec::new();
    let Some(lineage) = state
        .domain
        .root
        .command_lineage(state.domain.selected_path())
    else {
        return definitions;
    };
    let mut scope = command;
    for (index, owner) in lineage.into_iter().enumerate() {
        if index > 0 {
            let Some(child) = scope.find_subcommand(&owner.name) else {
                break;
            };
            scope = child;
        }
        definitions.extend(scope.get_groups().map(|group| GroupDefinition {
            owner,
            members: group.get_args().map(clap::Id::as_str).collect(),
            multiple: group.clone().is_multiple(),
        }));
    }
    definitions
}

fn missing_group_issues(
    state: &AppState,
    references: &[Reference],
    command: &Command,
) -> Vec<ValidationIssue> {
    let definitions = group_definitions(state, command);
    references
        .iter()
        .filter_map(|reference| {
            let Reference::Group(members) = reference else {
                return None;
            };
            let matches = definitions
                .iter()
                .filter_map(|group| {
                    group
                        .matching_args(members)
                        .map(|args| (args, group.multiple))
                })
                .collect::<Vec<_>>();
            // Clap's composite reference names the members, not the group.
            // Only a unique match gives us a reliable multiplicity policy.
            let (args, prefix) = match matches.as_slice() {
                [(args, multiple)] => (
                    args.clone(),
                    if *multiple {
                        "Choose at least one"
                    } else {
                        "Choose one"
                    },
                ),
                _ => (
                    resolve_reference_args(state, std::slice::from_ref(reference)),
                    "Choose from",
                ),
            };
            let alternatives = if args.is_empty() {
                members
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(" or ")
            } else {
                args.iter()
                    .map(|arg| arg.display_name.as_str())
                    .collect::<Vec<_>>()
                    .join(" or ")
            };
            Some(ValidationIssue::arguments(
                format!("{prefix}: {alternatives}"),
                args.into_iter().map(|arg| arg.id.clone()).collect(),
            ))
        })
        .collect()
}

fn resolve_reference_args<'a>(state: &'a AppState, references: &[Reference]) -> Vec<&'a ArgModel> {
    // Only structured argument contexts identify controls; diagnostic prose
    // can mention unrelated flags, examples, or suggested spellings.
    let candidates = visible_arg_models(state).collect::<Vec<_>>();
    let resolved = references
        .iter()
        .filter_map(|reference| reference.resolve(&candidates))
        .flatten()
        .map(FieldInstanceId::from_arg)
        .collect::<BTreeSet<_>>();
    candidates
        .into_iter()
        .filter(|arg| resolved.contains(&FieldInstanceId::from_arg(arg)))
        .collect()
}

fn visible_arg_models(state: &AppState) -> impl Iterator<Item = &ArgModel> {
    state
        .domain
        .root
        .effective_args_for_path(state.domain.selected_path())
        .into_iter()
        .flatten()
        .map(|(_, arg)| arg)
        .filter(|arg| {
            !arg.is_external_subcommand_field() || arg.owner_path() == state.domain.selected_path()
        })
}

fn current_about(state: &AppState) -> Option<&str> {
    state.domain.current_command().about.as_deref()
}

fn current_command_has_user_input(state: &AppState) -> bool {
    state.domain.current_form().is_some_and(|form| {
        state
            .domain
            .current_command()
            .args
            .iter()
            .filter(|arg| !arg.is_help_action())
            .any(|arg| {
                form.input(&arg.id)
                    .is_some_and(input_state_is_user_provided)
            })
    })
}

fn input_state_is_user_provided(input: &ArgInputState) -> bool {
    if input.touched {
        return true;
    }

    match &input.value {
        ArgInput::Flag { present, source } => *present && *source == InputSource::User,
        ArgInput::Count {
            occurrences,
            source,
        } => *occurrences > 0 && *source == InputSource::User,
        ArgInput::Values { occurrences } => occurrences.iter().any(|occurrence| {
            occurrence.source == InputSource::User
                && occurrence.values.iter().any(|value| !value.is_empty())
        }),
    }
}

fn fallback_validation_state(command: &CommandModel, form: &CommandFormState) -> ValidationState {
    ValidationState {
        issues: command
            .args
            .iter()
            .filter(|arg| arg.required && !arg.is_help_action() && arg_is_missing(arg, form))
            .map(|arg| {
                ValidationIssue::arguments(
                    format!("Provide {}", arg.display_name),
                    vec![arg.id.clone()],
                )
            })
            .collect(),
    }
}

fn arg_is_missing(arg: &ArgModel, form: &CommandFormState) -> bool {
    match form.compatibility_value(arg) {
        Some(ArgValue::Text(value)) => value.is_empty(),
        Some(ArgValue::Bool(value)) => !value,
        Some(ArgValue::Choice(_)) => false,
        None => true,
    }
}

#[cfg(test)]
mod tests;
