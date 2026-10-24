#[cfg(test)]
use std::cell::Cell;
use std::collections::BTreeSet;
use std::ffi::OsString;

use clap::Command;
use clap::error::{ContextKind, ContextValue, ErrorKind};

use crate::input::{AppState, ArgInput, ArgInputState, ArgValue, CommandFormState, InputSource};
use crate::pipeline::{ValidationIssue, ValidationState, ValidationTarget};
use crate::spec::{ArgModel, CommandModel};

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

fn validate_with_clap(state: &AppState, command: Command, argv: &[OsString]) -> ValidationState {
    match command.clone().try_get_matches_from(argv.iter().cloned()) {
        Ok(_) => ValidationState::default(),
        Err(error) if error.kind() == ErrorKind::DisplayVersion => ValidationState::default(),
        Err(error) => {
            if error.kind() == ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand {
                let diagnostic =
                    suppress_help_on_empty(command, state.domain.selected_path().as_slice());
                if let Err(requirement) = diagnostic.try_get_matches_from(argv.iter().cloned())
                    && requirement.kind() != ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand
                {
                    return adapt_clap_error(state, &requirement);
                }
            }
            adapt_clap_error(state, &error)
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

fn adapt_clap_error(state: &AppState, error: &clap::Error) -> ValidationState {
    let issues = match error.kind() {
        ErrorKind::MissingRequiredArgument => missing_requirement_issues(state, error),
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
    let ids = referenced_arg_ids(state, error);
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

fn missing_requirement_issues(state: &AppState, error: &clap::Error) -> Vec<ValidationIssue> {
    let groups = missing_required_group_feedback(state, error);
    let mut issues = groups
        .iter()
        .map(|group| ValidationIssue::arguments(&group.message, group.arg_ids.clone()))
        .collect::<Vec<_>>();
    let ids = referenced_arg_ids(state, error);
    for arg in visible_arg_models(state).filter(|arg| ids.contains(&arg.id)) {
        if groups.iter().any(|group| group.arg_ids.contains(&arg.id)) {
            continue;
        }
        issues.push(ValidationIssue::arguments(
            format!("Provide {}", arg.display_name),
            vec![arg.id.clone()],
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
    let invalid =
        context_string(error, ContextKind::InvalidArg).map(|value| compact_arg_reference(&value));
    let prior =
        context_string(error, ContextKind::PriorArg).map(|value| compact_arg_reference(&value));
    match (invalid, prior) {
        (Some(invalid), Some(prior)) => Some(format!("Remove one of: {invalid} or {prior}")),
        _ => None,
    }
}

fn summary_from_value_context(error: &clap::Error) -> Option<String> {
    let arg = compact_arg_reference(&context_string(error, ContextKind::InvalidArg)?);
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
    let arg = compact_arg_reference(&context_string(error, ContextKind::InvalidArg)?);
    let value = context_string(error, ContextKind::InvalidValue)?;
    let explanation = std::error::Error::source(error)?;
    Some(format!("Invalid value for {arg}: {value}. {explanation}"))
}

fn summary_from_no_equals_context(error: &clap::Error) -> Option<String> {
    let arg = compact_arg_reference(&context_string(error, ContextKind::InvalidArg)?);
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

fn referenced_arg_ids(state: &AppState, error: &clap::Error) -> Vec<String> {
    // Only clap's structured argument references identify controls. Diagnostic
    // prose can mention unrelated flags, examples, or suggested spellings.
    let tokens = context_reference_values(error)
        .iter()
        .flat_map(|value| reference_tokens(value))
        .collect::<BTreeSet<_>>();
    visible_arg_models(state)
        .filter(|arg| {
            arg_reference_candidates(arg)
                .iter()
                .any(|candidate| tokens.contains(candidate))
        })
        .map(|arg| arg.id.clone())
        .collect()
}

fn arg_reference_candidates(arg: &ArgModel) -> Vec<String> {
    let mut candidates = vec![
        arg.id.clone(),
        arg.display_name.clone(),
        arg.display_label().to_string(),
    ];
    if let Some(long) = arg.long() {
        candidates.push(format!("--{long}"));
    }
    if let Some(short) = arg.short() {
        candidates.push(format!("-{short}"));
    }
    candidates.extend(
        arg.visible_aliases()
            .iter()
            .map(|alias| format!("--{alias}")),
    );
    candidates.extend(
        arg.visible_short_aliases()
            .iter()
            .map(|alias| format!("-{alias}")),
    );
    candidates.sort();
    candidates.dedup();
    candidates
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

fn reference_tokens(value: &str) -> BTreeSet<String> {
    let compact = compact_arg_reference(value);
    let mut tokens = BTreeSet::new();
    if !compact.is_empty() {
        tokens.insert(compact.clone());
    }
    if !compact.contains('|')
        && let Some(inner) = compact.strip_prefix('<').and_then(|v| v.strip_suffix('>'))
    {
        tokens.insert(inner.to_string());
    }
    if let Some(members) = composite_reference_members(&compact) {
        tokens.extend(members);
    }
    tokens
}

fn composite_reference_members(value: &str) -> Option<Vec<String>> {
    let trimmed = value.trim();
    let inner = trimmed
        .strip_prefix('<')
        .and_then(|rest| rest.strip_suffix('>'))
        .or_else(|| {
            trimmed
                .strip_prefix('[')
                .and_then(|rest| rest.strip_suffix(']'))
        })?;
    if !inner.contains('|') {
        return None;
    }

    let members = inner
        .split('|')
        .map(compact_arg_reference)
        .filter(|member| !member.is_empty())
        .collect::<Vec<_>>();
    (!members.is_empty()).then_some(members)
}

fn context_string(error: &clap::Error, key: ContextKind) -> Option<String> {
    match error.get(key) {
        Some(ContextValue::String(value)) => Some(value.clone()),
        Some(ContextValue::StyledStr(value)) => Some(value.to_string()),
        _ => None,
    }
}

fn compact_arg_reference(value: &str) -> String {
    let without_value_name = value.split_once(" <").map_or(value, |(prefix, _)| prefix);
    let without_value_name = without_value_name
        .split_once(" [")
        .map_or(without_value_name, |(prefix, _)| prefix);
    let without_equals_value = without_value_name
        .split_once("=<")
        .map_or(without_value_name, |(prefix, _)| prefix);
    let without_equals_value = without_equals_value
        .split_once("=[")
        .map_or(without_equals_value, |(prefix, _)| prefix);
    without_equals_value.trim().to_string()
}

fn extract_arg_token(token: &str) -> Option<String> {
    let trimmed = token.trim_matches(|ch: char| {
        ch == ','
            || ch == ':'
            || ch == ';'
            || ch == ')'
            || ch == '('
            || ch == '.'
            || ch == '>'
            || ch == ']'
    });
    (trimmed.starts_with('-') || trimmed.starts_with('<') || trimmed.starts_with('['))
        .then(|| trimmed.to_string())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MissingRequiredGroupFeedback {
    message: String,
    arg_ids: Vec<String>,
}

fn missing_required_group_feedback(
    state: &AppState,
    error: &clap::Error,
) -> Vec<MissingRequiredGroupFeedback> {
    composite_references(error)
        .into_iter()
        .filter_map(|reference| {
            let tokens = composite_reference_members(&reference)?;
            let args = resolve_reference_args(state, &tokens);
            if args.is_empty() {
                return None;
            }

            let multiple = state
                .domain
                .validation_command
                .as_ref()
                .is_some_and(|root| {
                    let mut command = root.clone();
                    command.build();
                    let selected = state
                        .domain
                        .selected_path()
                        .as_slice()
                        .iter()
                        .try_fold(&command, |parent, name| parent.find_subcommand(name))
                        .unwrap_or(&command);
                    selected.get_groups().any(|group| {
                        let members = group
                            .get_args()
                            .map(clap::Id::as_str)
                            .collect::<BTreeSet<_>>();
                        let actual = args
                            .iter()
                            .map(|arg| arg.id.as_str())
                            .collect::<BTreeSet<_>>();
                        members == actual && group.clone().is_multiple()
                    })
                });
            Some(MissingRequiredGroupFeedback {
                message: format!(
                    "{}: {}",
                    if multiple {
                        "Choose at least one"
                    } else {
                        "Choose one"
                    },
                    args.iter()
                        .map(|arg| arg.display_name.as_str())
                        .collect::<Vec<_>>()
                        .join(" or ")
                ),

                arg_ids: args.into_iter().map(|arg| arg.id.clone()).collect(),
            })
        })
        .collect()
}

fn composite_references(error: &clap::Error) -> Vec<String> {
    let mut seen = BTreeSet::new();
    let mut references = Vec::new();

    for value in context_reference_values(error) {
        for token in reference_tokens(&value) {
            if composite_reference_members(&token).is_some() && seen.insert(token.clone()) {
                references.push(token);
            }
        }
    }

    if let Some(diagnostic) = diagnostic_text(error) {
        for token in diagnostic.split_whitespace().filter_map(extract_arg_token) {
            for reference in reference_tokens(&token) {
                if composite_reference_members(&reference).is_some()
                    && seen.insert(reference.clone())
                {
                    references.push(reference);
                }
            }
        }
    }

    references
}

fn diagnostic_text(error: &clap::Error) -> Option<String> {
    clean_diagnostic_line(&error.to_string(), None)
}

fn resolve_reference_args<'a>(state: &'a AppState, tokens: &[String]) -> Vec<&'a ArgModel> {
    let token_set = tokens.iter().cloned().collect::<BTreeSet<_>>();
    visible_arg_models(state)
        .filter(|arg| {
            arg_reference_candidates(arg)
                .iter()
                .any(|candidate| token_set.contains(candidate))
        })
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
