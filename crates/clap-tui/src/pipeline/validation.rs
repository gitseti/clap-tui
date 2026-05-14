#[cfg(test)]
use std::cell::Cell;
use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsString;

use clap::Command;
use clap::error::{ContextKind, ContextValue, ErrorKind};

use crate::input::{AppState, ArgInput, ArgInputState, ArgValue, CommandFormState, InputSource};
use crate::pipeline::ValidationState;
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
    match command.try_get_matches_from(argv.iter().cloned()) {
        Ok(_) => ValidationState {
            is_valid: true,
            summary: None,
            field_errors: BTreeMap::new(),
        },
        Err(error) if error.kind() == ErrorKind::DisplayVersion => ValidationState {
            is_valid: true,
            summary: None,
            field_errors: BTreeMap::new(),
        },
        Err(error) => adapt_clap_error(state, &error),
    }
}

fn adapt_clap_error(state: &AppState, error: &clap::Error) -> ValidationState {
    let summary = summary_from_error_kind_and_context(state, error);
    let field_errors = infer_field_errors(state, error, summary.as_deref());

    ValidationState {
        is_valid: false,
        summary,
        field_errors,
    }
}

fn summary_from_error_kind_and_context(state: &AppState, error: &clap::Error) -> Option<String> {
    match error.kind() {
        ErrorKind::MissingRequiredArgument => summary_for_missing_required_error(state, error),
        ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand => {
            summary_for_help_style_missing(state)
                .or_else(|| clean_diagnostic_line(&error.to_string(), current_about(state)))
        }
        ErrorKind::MissingSubcommand => Some("Missing required subcommand".to_string()),
        ErrorKind::ArgumentConflict => summary_from_conflict_context(error)
            .or_else(|| clean_diagnostic_line(&error.to_string(), current_about(state))),
        ErrorKind::InvalidValue => summary_from_value_context(error)
            .or_else(|| clean_diagnostic_line(&error.to_string(), current_about(state))),
        ErrorKind::NoEquals => summary_from_no_equals_context(error)
            .or_else(|| clean_diagnostic_line(&error.to_string(), current_about(state))),
        ErrorKind::TooFewValues
        | ErrorKind::TooManyValues
        | ErrorKind::WrongNumberOfValues
        | ErrorKind::UnknownArgument
        | ErrorKind::InvalidSubcommand
        | ErrorKind::ValueValidation => {
            clean_diagnostic_line(&error.to_string(), current_about(state))
        }
        _ => clean_diagnostic_line(&error.to_string(), current_about(state)),
    }
}

fn summary_for_missing_required_error(state: &AppState, error: &clap::Error) -> Option<String> {
    let mut parts = Vec::new();
    if let Some(summary) = summary_for_missing_required(state) {
        parts.push(summary);
    }
    parts.extend(
        missing_required_group_feedback(state, error)
            .into_iter()
            .map(|group| group.message),
    );

    match parts.as_slice() {
        [] => clean_diagnostic_line(&error.to_string(), current_about(state)),
        [single] => Some(single.clone()),
        many => Some(many.join("; ")),
    }
}

fn summary_for_missing_required(state: &AppState) -> Option<String> {
    let missing = missing_required_args(state);
    match missing.as_slice() {
        [] => None,
        [single] => Some(format!("Missing required argument: {single}")),
        many => Some(format!("Missing required arguments: {}", many.join(", "))),
    }
}

fn summary_for_help_style_missing(state: &AppState) -> Option<String> {
    if let Some(summary) = summary_for_missing_required(state) {
        return Some(summary);
    }

    let current_command = state.domain.current_command();
    if current_command.parser_rules.subcommand_required && !current_command.subcommands.is_empty() {
        return Some("Missing required subcommand".to_string());
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
        (Some(invalid), Some(prior)) => Some(format!("Conflicting arguments: {invalid}, {prior}")),
        _ => None,
    }
}

fn summary_from_value_context(error: &clap::Error) -> Option<String> {
    let arg = compact_arg_reference(&context_string(error, ContextKind::InvalidArg)?);
    let value = context_string(error, ContextKind::InvalidValue)?;
    Some(format!("Invalid value for {arg}: {value}"))
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

fn infer_field_errors(
    state: &AppState,
    error: &clap::Error,
    summary: Option<&str>,
) -> BTreeMap<String, String> {
    let summary = summary.unwrap_or("Invalid input").to_string();
    let mut field_errors = BTreeMap::new();

    match error.kind() {
        ErrorKind::MissingRequiredArgument
        | ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand => {
            for arg in missing_required_arg_models(state) {
                field_errors.insert(arg.id.clone(), "Required argument".to_string());
            }
            for group in missing_required_group_feedback(state, error) {
                for arg_id in group.arg_ids {
                    field_errors.insert(arg_id, group.message.clone());
                }
            }
        }
        ErrorKind::ArgumentConflict => {
            for arg_id in referenced_arg_ids(state, error, Some(&summary)) {
                field_errors.insert(arg_id, summary.clone());
            }
        }
        ErrorKind::InvalidValue
        | ErrorKind::NoEquals
        | ErrorKind::TooFewValues
        | ErrorKind::TooManyValues
        | ErrorKind::WrongNumberOfValues
        | ErrorKind::ValueValidation => {
            let referenced = referenced_arg_ids(state, error, Some(&summary));
            if referenced.len() == 1 {
                field_errors.insert(referenced[0].clone(), summary);
            }
        }
        _ => {}
    }

    field_errors
}

fn missing_required_args(state: &AppState) -> Vec<String> {
    missing_required_arg_models(state)
        .into_iter()
        .map(|arg| arg.display_name.clone())
        .collect()
}

fn missing_required_arg_models(state: &AppState) -> Vec<&ArgModel> {
    let current_form = state.domain.current_form().unwrap_or_default();
    visible_arg_models(state)
        .filter(|arg| arg.required && !arg.is_help_action())
        .filter(|arg| arg_is_missing(arg, &current_form))
        .collect()
}

fn referenced_arg_ids(state: &AppState, error: &clap::Error, summary: Option<&str>) -> Vec<String> {
    let tokens = referenced_tokens(error, summary);
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

fn referenced_tokens(error: &clap::Error, summary: Option<&str>) -> BTreeSet<String> {
    let mut tokens = BTreeSet::new();

    for value in context_reference_values(error) {
        tokens.extend(reference_tokens(&value));
    }

    if let Some(diagnostic) = diagnostic_text(error) {
        for token in diagnostic.split_whitespace().filter_map(extract_arg_token) {
            tokens.extend(reference_tokens(&token));
        }
    }

    if let Some(summary) = summary {
        for token in summary.split_whitespace().filter_map(extract_arg_token) {
            tokens.extend(reference_tokens(&token));
        }
    }

    tokens
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

            Some(MissingRequiredGroupFeedback {
                message: format!(
                    "Choose one of: {}",
                    args.iter()
                        .map(|arg| arg.display_name.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
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
    let mut field_errors = BTreeMap::new();
    let mut missing = Vec::new();

    for arg in command
        .args
        .iter()
        .filter(|arg| arg.required && !arg.is_help_action())
    {
        if arg_is_missing(arg, form) {
            field_errors.insert(arg.id.clone(), "Required argument".to_string());
            missing.push(arg.display_name.clone());
        }
    }

    let summary = match missing.as_slice() {
        [] => None,
        [single] => Some(format!("Missing required argument: {single}")),
        many => Some(format!("Missing required arguments: {}", many.join(", "))),
    };

    ValidationState {
        is_valid: field_errors.is_empty(),
        summary,
        field_errors,
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
