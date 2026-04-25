use std::ffi::{OsStr, OsString};

use crate::input::{ArgInput, CommandFormState, InputSource};
use crate::spec::{
    ArgModel, CommandModel, CommandPath, EXTERNAL_SUBCOMMAND_ARGS_ID, EXTERNAL_SUBCOMMAND_NAME_ID,
};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct SerializationResult {
    pub(crate) argv: Vec<OsString>,
    pub(crate) provenance: Vec<TokenProvenance>,
    pub(crate) diagnostics: Vec<SerializationDiagnostic>,
}

impl SerializationResult {
    pub(crate) fn is_ok(&self) -> bool {
        self.diagnostics.is_empty()
    }

    pub(crate) fn rendered_posix(&self) -> Option<RenderedCommand> {
        self.is_ok()
            .then(|| render_command(&self.argv, &self.provenance, TargetShell::Posix))
    }

    pub(crate) fn rendered_powershell(&self) -> Option<RenderedCommand> {
        self.is_ok()
            .then(|| render_command(&self.argv, &self.provenance, TargetShell::PowerShell))
    }

    pub(crate) fn rendered_for_platform(&self) -> Option<RenderedCommand> {
        if cfg!(windows) {
            self.rendered_powershell()
        } else {
            self.rendered_posix()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TokenProvenance {
    pub(crate) token_index: usize,
    pub(crate) kind: TokenProvenanceKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TokenProvenanceKind {
    Command {
        path: CommandPath,
    },
    CanonicalSpelling {
        arg_id: String,
        occurrence: Option<usize>,
    },
    AuthoredValue {
        arg_id: String,
        occurrence: Option<usize>,
    },
    DelimiterJoined {
        arg_id: String,
        occurrence: Option<usize>,
    },
    Terminator {
        arg_id: String,
    },
    RawBoundary {
        arg_id: Option<String>,
    },
    SubcommandBoundary {
        path: CommandPath,
    },
    ExternalSubcommandName,
    PreservedExternalSubcommandArg {
        occurrence: Option<usize>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SerializationDiagnostic {
    pub(crate) kind: SerializationDiagnosticKind,
    pub(crate) message: String,
    pub(crate) target: DiagnosticTarget,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SerializationDiagnosticKind {
    OccurrenceGroupingAmbiguity,
    OwnershipAmbiguity,
    HyphenLeadingAmbiguity,
    PositionalTrailingAmbiguity,
    UnsupportedParserShape,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DiagnosticTarget {
    Field(String),
    PositionalSlot(usize),
    Command(CommandPath),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TargetShell {
    Posix,
    /// Windows preview/copy rendering targets `PowerShell` explicitly.
    ///
    /// The rendered string is UI text only; canonical validation and execution use argv tokens.
    PowerShell,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RenderedCommand {
    pub(crate) text: String,
    pub(crate) tokens: Vec<RenderedShellToken>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RenderedShellToken {
    pub(crate) text: String,
    pub(crate) kind: RenderedShellTokenKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RenderedShellTokenKind {
    EntryPoint,
    SubcommandName,
    OptionSpelling,
    Value,
    DelimiterJoinedValue,
    RawBoundary,
    Terminator,
    PreservedExternalToken,
}

#[derive(Debug, Clone)]
struct PositionalPlan {
    arg_id: String,
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
) -> SerializationResult {
    let mut argv = Vec::new();
    let mut provenance = Vec::new();
    let mut diagnostics = Vec::new();
    let mut positionals = Vec::new();
    let mut positional_sequence = 0usize;
    let mut saw_later_positionals = false;
    let mut saw_later_unprotected_positionals = false;

    for arg in &command.args {
        let Some(input) = state.input(&arg.id) else {
            continue;
        };
        if should_omit_input(input) {
            continue;
        }
        if matches!(&input.value, ArgInput::Values { .. }) && arg.serializes_as_positional() {
            saw_later_positionals = true;
            if !(arg.is_last_positional() || arg.is_trailing_var_arg()) {
                saw_later_unprotected_positionals = true;
            }
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
                if let Some(spelling) = canonical_spelling(arg) {
                    push_arg_token(&mut argv, &mut provenance, spelling, arg, None);
                } else {
                    add_unsupported_canonical_spelling_diagnostic(arg, &mut diagnostics);
                }
            }
            ArgInput::Values { occurrences } => {
                let non_empty_occurrences = occurrences
                    .iter()
                    .enumerate()
                    .map(|(index, occurrence)| {
                        let values = occurrence.values.clone();
                        (index, values)
                    })
                    .filter(|(_, values)| !values.is_empty())
                    .collect::<Vec<_>>();
                if non_empty_occurrences.is_empty() {
                    continue;
                }
                add_value_diagnostics(
                    command,
                    arg,
                    &non_empty_occurrences,
                    has_following_command,
                    saw_later_unprotected_positionals,
                    &mut diagnostics,
                );
                if arg.serializes_as_positional() {
                    if let Some(index) = arg.position {
                        positionals.push(PositionalPlan {
                            arg_id: arg.id.clone(),
                            position: index,
                            sequence: positional_sequence,
                            occurrences: non_empty_occurrences
                                .into_iter()
                                .map(|(_, values)| values)
                                .collect(),
                            value_delimiter: arg.metadata.syntax.value_delimiter,
                            insert_boundary_before: arg.is_last_positional()
                                || arg.is_trailing_var_arg(),
                            terminator: arg.value_terminator().map(str::to_string),
                        });
                        positional_sequence += 1;
                    }
                } else {
                    if canonical_spelling(arg).is_none() {
                        add_unsupported_canonical_spelling_diagnostic(arg, &mut diagnostics);
                        continue;
                    }
                    if should_use_attached_delimited_option(arg) {
                        for (occurrence, values) in non_empty_occurrences {
                            serialize_attached_delimited_option_occurrence(
                                &mut argv,
                                &mut provenance,
                                arg,
                                &values,
                                occurrence,
                            );
                        }
                    } else {
                        for (occurrence, values) in non_empty_occurrences {
                            serialize_option_occurrence(
                                &mut argv,
                                &mut provenance,
                                arg,
                                &values,
                                occurrence,
                                saw_later_positionals || has_following_command,
                            );
                        }
                    }
                }
            }
            ArgInput::Count { occurrences, .. } => {
                for _ in 0..*occurrences {
                    if let Some(spelling) = canonical_spelling(arg) {
                        push_arg_token(&mut argv, &mut provenance, spelling, arg, None);
                    } else {
                        add_unsupported_canonical_spelling_diagnostic(arg, &mut diagnostics);
                        break;
                    }
                }
            }
            ArgInput::Flag { .. } => {}
        }
    }

    positionals.sort_by_key(|plan| (plan.position, plan.sequence));
    let mut inserted_boundary = false;
    for (index, plan) in positionals.iter().enumerate() {
        if plan.insert_boundary_before && !inserted_boundary {
            push_token(
                &mut argv,
                &mut provenance,
                "--",
                TokenProvenanceKind::RawBoundary {
                    arg_id: Some(plan.arg_id.clone()),
                },
            );
            inserted_boundary = true;
        }
        let join_with_delimiter = plan.value_delimiter.is_some()
            && !(inserted_boundary && command.parser_rules.dont_delimit_trailing_values);
        for (occurrence_index, occurrence) in plan.occurrences.iter().enumerate() {
            if join_with_delimiter {
                push_token(
                    &mut argv,
                    &mut provenance,
                    join_positional_occurrence(plan.value_delimiter, occurrence),
                    TokenProvenanceKind::DelimiterJoined {
                        arg_id: plan.arg_id.clone(),
                        occurrence: Some(occurrence_index),
                    },
                );
            } else {
                for value in occurrence {
                    push_token(
                        &mut argv,
                        &mut provenance,
                        value,
                        TokenProvenanceKind::AuthoredValue {
                            arg_id: plan.arg_id.clone(),
                            occurrence: Some(occurrence_index),
                        },
                    );
                }
            }
        }
        if let Some(terminator) = plan.terminator.as_ref()
            && (index + 1 < positionals.len() || has_following_command)
        {
            push_token(
                &mut argv,
                &mut provenance,
                terminator,
                TokenProvenanceKind::Terminator {
                    arg_id: plan.arg_id.clone(),
                },
            );
        }
    }
    if !has_following_command {
        append_external_subcommand(command, state, &mut argv, &mut provenance);
    }
    SerializationResult {
        argv,
        provenance,
        diagnostics,
    }
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
    argv: &mut Vec<OsString>,
    provenance: &mut Vec<TokenProvenance>,
    arg: &ArgModel,
    values: &[String],
    occurrence: usize,
    needs_terminator: bool,
) {
    if values.is_empty() {
        return;
    }

    if arg.metadata.syntax.require_equals {
        let Some(first) = values.first() else {
            return;
        };
        let Some(spelling) = canonical_spelling(arg) else {
            return;
        };
        let attached = if arg.accepts_multiple_values_per_occurrence()
            && arg.metadata.syntax.value_delimiter.is_some()
        {
            join_occurrence_values(arg, values)
        } else {
            first.clone()
        };
        push_token(
            argv,
            provenance,
            format!("{spelling}={attached}"),
            TokenProvenanceKind::DelimiterJoined {
                arg_id: arg.id.clone(),
                occurrence: Some(occurrence),
            },
        );
        if arg.accepts_multiple_values_per_occurrence()
            && arg.metadata.syntax.value_delimiter.is_none()
        {
            for value in values.iter().skip(1) {
                push_token(
                    argv,
                    provenance,
                    value,
                    TokenProvenanceKind::AuthoredValue {
                        arg_id: arg.id.clone(),
                        occurrence: Some(occurrence),
                    },
                );
            }
        }
        return;
    }

    push_arg_token(
        argv,
        provenance,
        canonical_spelling(arg).expect("canonical spelling checked before option serialization"),
        arg,
        Some(occurrence),
    );
    if arg.accepts_multiple_values_per_occurrence() {
        if arg.metadata.syntax.value_delimiter.is_some() {
            push_token(
                argv,
                provenance,
                join_occurrence_values(arg, values),
                TokenProvenanceKind::DelimiterJoined {
                    arg_id: arg.id.clone(),
                    occurrence: Some(occurrence),
                },
            );
        } else {
            for value in values {
                push_token(
                    argv,
                    provenance,
                    value,
                    TokenProvenanceKind::AuthoredValue {
                        arg_id: arg.id.clone(),
                        occurrence: Some(occurrence),
                    },
                );
            }
        }
        if needs_terminator && let Some(terminator) = arg.value_terminator() {
            push_token(
                argv,
                provenance,
                terminator,
                TokenProvenanceKind::Terminator {
                    arg_id: arg.id.clone(),
                },
            );
        }
    } else if let Some(value) = values.first() {
        push_token(
            argv,
            provenance,
            value,
            TokenProvenanceKind::AuthoredValue {
                arg_id: arg.id.clone(),
                occurrence: Some(occurrence),
            },
        );
    }
}

fn serialize_attached_delimited_option_occurrence(
    argv: &mut Vec<OsString>,
    provenance: &mut Vec<TokenProvenance>,
    arg: &ArgModel,
    values: &[String],
    occurrence: usize,
) {
    if values.is_empty() {
        return;
    }
    let Some(spelling) = canonical_spelling(arg) else {
        return;
    };

    push_token(
        argv,
        provenance,
        format!("{}={}", spelling, join_occurrence_values(arg, values)),
        TokenProvenanceKind::DelimiterJoined {
            arg_id: arg.id.clone(),
            occurrence: Some(occurrence),
        },
    );
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
    argv: &mut Vec<OsString>,
    provenance: &mut Vec<TokenProvenance>,
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
    push_token(
        argv,
        provenance,
        name,
        TokenProvenanceKind::ExternalSubcommandName,
    );

    if let Some(input) = state.input(EXTERNAL_SUBCOMMAND_ARGS_ID)
        && let ArgInput::Values { occurrences } = &input.value
    {
        for (occurrence_index, value) in
            occurrences
                .iter()
                .enumerate()
                .flat_map(|(occurrence_index, occurrence)| {
                    occurrence
                        .values
                        .iter()
                        .filter(|value| !value.is_empty())
                        .map(move |value| (occurrence_index, value))
                })
        {
            push_token(
                argv,
                provenance,
                value,
                TokenProvenanceKind::PreservedExternalSubcommandArg {
                    occurrence: Some(occurrence_index),
                },
            );
        }
    }
}

pub(crate) fn push_command_token(
    result: &mut SerializationResult,
    token: impl Into<OsString>,
    path: CommandPath,
) {
    push_token(
        &mut result.argv,
        &mut result.provenance,
        token,
        TokenProvenanceKind::Command { path },
    );
}

pub(crate) fn push_subcommand_token(
    result: &mut SerializationResult,
    token: impl Into<OsString>,
    path: CommandPath,
) {
    push_token(
        &mut result.argv,
        &mut result.provenance,
        token,
        TokenProvenanceKind::SubcommandBoundary { path },
    );
}

fn push_arg_token(
    argv: &mut Vec<OsString>,
    provenance: &mut Vec<TokenProvenance>,
    token: String,
    arg: &ArgModel,
    occurrence: Option<usize>,
) {
    push_token(
        argv,
        provenance,
        token,
        TokenProvenanceKind::CanonicalSpelling {
            arg_id: arg.id.clone(),
            occurrence,
        },
    );
}

fn push_token(
    argv: &mut Vec<OsString>,
    provenance: &mut Vec<TokenProvenance>,
    token: impl Into<OsString>,
    kind: TokenProvenanceKind,
) {
    provenance.push(TokenProvenance {
        token_index: argv.len(),
        kind,
    });
    argv.push(token.into());
}

fn canonical_spelling(arg: &ArgModel) -> Option<String> {
    arg.long()
        .map(|long| format!("--{long}"))
        .or_else(|| arg.short().map(|short| format!("-{short}")))
        .or_else(|| {
            arg.display_name
                .starts_with('-')
                .then(|| arg.display_name.clone())
        })
}

fn should_use_attached_delimited_option(arg: &ArgModel) -> bool {
    !arg.serializes_as_positional()
        && arg.accepts_multiple_values_per_occurrence()
        && arg.metadata.syntax.value_delimiter.is_some()
}

fn add_value_diagnostics(
    command: &CommandModel,
    arg: &ArgModel,
    occurrences: &[(usize, Vec<String>)],
    has_following_command: bool,
    has_following_unprotected_positional: bool,
    diagnostics: &mut Vec<SerializationDiagnostic>,
) {
    if !arg.serializes_as_positional()
        && arg.accepts_multiple_values_per_occurrence()
        && arg.metadata.cardinality.unbounded
        && arg.metadata.syntax.value_delimiter.is_none()
        && arg.metadata.syntax.value_terminator.is_none()
        && !arg.metadata.syntax.require_equals
        && (has_following_unprotected_positional
            || (has_following_command && !command.parser_rules.subcommand_precedence_over_arg))
    {
        diagnostics.push(SerializationDiagnostic {
            kind: SerializationDiagnosticKind::OwnershipAmbiguity,
            message: format!(
                "{} has variable values before a positional; ownership cannot be serialized uniquely",
                arg.display_label()
            ),
            target: DiagnosticTarget::Field(arg.id.clone()),
        });
    }

    if !arg.serializes_as_positional()
        && !arg.metadata.syntax.allow_hyphen_values
        && !arg.metadata.syntax.allow_negative_numbers
    {
        for (_, values) in occurrences {
            if values
                .iter()
                .enumerate()
                .any(|(index, value)| hyphen_leading_value_is_unsafe(arg, index, value))
            {
                diagnostics.push(SerializationDiagnostic {
                    kind: SerializationDiagnosticKind::HyphenLeadingAmbiguity,
                    message: format!(
                        "{} contains a hyphen-leading value that could parse as an option",
                        arg.display_label()
                    ),
                    target: DiagnosticTarget::Field(arg.id.clone()),
                });
                break;
            }
        }
    }
}

fn add_unsupported_canonical_spelling_diagnostic(
    arg: &ArgModel,
    diagnostics: &mut Vec<SerializationDiagnostic>,
) {
    diagnostics.push(SerializationDiagnostic {
        kind: SerializationDiagnosticKind::UnsupportedParserShape,
        message: format!(
            "{} cannot be serialized because it has no canonical option spelling",
            arg.display_label()
        ),
        target: DiagnosticTarget::Field(arg.id.clone()),
    });
}

fn hyphen_leading_value_is_unsafe(arg: &ArgModel, value_index: usize, value: &str) -> bool {
    if !value.starts_with('-') || value == "-" {
        return false;
    }

    if should_use_attached_delimited_option(arg) {
        return false;
    }

    !(arg.metadata.syntax.require_equals && value_index == 0)
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn render_shell(argv: &[OsString], shell: TargetShell) -> String {
    render_command(argv, &[], shell).text
}

pub(crate) fn render_command(
    argv: &[OsString],
    provenance: &[TokenProvenance],
    shell: TargetShell,
) -> RenderedCommand {
    let tokens = argv
        .iter()
        .enumerate()
        .map(|(index, token)| RenderedShellToken {
            text: render_token(token, shell),
            kind: rendered_token_kind(index, token, provenance),
        })
        .collect::<Vec<_>>();
    let text = tokens
        .iter()
        .map(|token| token.text.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    RenderedCommand { text, tokens }
}

fn rendered_token_kind(
    token_index: usize,
    token: &OsStr,
    provenance: &[TokenProvenance],
) -> RenderedShellTokenKind {
    let Some(kind) = provenance
        .iter()
        .find(|provenance| provenance.token_index == token_index)
        .map(|provenance| &provenance.kind)
    else {
        return RenderedShellTokenKind::Value;
    };

    match kind {
        TokenProvenanceKind::Command { .. } => RenderedShellTokenKind::EntryPoint,
        TokenProvenanceKind::SubcommandBoundary { .. }
        | TokenProvenanceKind::ExternalSubcommandName => RenderedShellTokenKind::SubcommandName,
        TokenProvenanceKind::CanonicalSpelling { .. } => RenderedShellTokenKind::OptionSpelling,
        TokenProvenanceKind::DelimiterJoined { .. } => {
            if token.to_string_lossy().starts_with('-') {
                RenderedShellTokenKind::OptionSpelling
            } else {
                RenderedShellTokenKind::DelimiterJoinedValue
            }
        }
        TokenProvenanceKind::AuthoredValue { .. } => RenderedShellTokenKind::Value,
        TokenProvenanceKind::RawBoundary { .. } => RenderedShellTokenKind::RawBoundary,
        TokenProvenanceKind::Terminator { .. } => RenderedShellTokenKind::Terminator,
        TokenProvenanceKind::PreservedExternalSubcommandArg { .. } => {
            RenderedShellTokenKind::PreservedExternalToken
        }
    }
}

fn render_token(token: &OsStr, shell: TargetShell) -> String {
    let text = token.to_string_lossy();
    match shell {
        TargetShell::Posix => render_posix_token(&text),
        TargetShell::PowerShell => render_powershell_token(&text),
    }
}

fn render_posix_token(token: &str) -> String {
    if token.is_empty() {
        return "''".to_string();
    }
    if token.bytes().all(|byte| {
        byte.is_ascii_alphanumeric()
            || matches!(byte, b'_' | b'-' | b'.' | b'/' | b':' | b'=' | b',' | b'+')
    }) {
        return token.to_string();
    }
    format!("'{}'", token.replace('\'', "'\\''"))
}

fn render_powershell_token(token: &str) -> String {
    if token.is_empty() {
        return "''".to_string();
    }
    if token.chars().all(|ch| {
        ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.' | '/' | ':' | '=' | ',' | '+')
    }) {
        return token.to_string();
    }
    format!("'{}'", token.replace('\'', "''"))
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;

    use super::{
        RenderedShellTokenKind, TargetShell, TokenProvenance, TokenProvenanceKind, render_command,
        render_shell,
    };
    use crate::spec::CommandPath;

    fn os_vec(values: &[&str]) -> Vec<OsString> {
        values.iter().map(OsString::from).collect()
    }

    #[test]
    fn rendered_command_text_is_joined_from_rendered_tokens() {
        let argv = os_vec(&[
            "clap-features",
            "serve",
            "--feature=gzip",
            "-literal",
            "--",
            "external arg",
        ]);
        let provenance = vec![
            TokenProvenance {
                token_index: 0,
                kind: TokenProvenanceKind::Command {
                    path: CommandPath::default(),
                },
            },
            TokenProvenance {
                token_index: 1,
                kind: TokenProvenanceKind::SubcommandBoundary {
                    path: CommandPath::new(vec!["serve".to_string()]),
                },
            },
            TokenProvenance {
                token_index: 2,
                kind: TokenProvenanceKind::DelimiterJoined {
                    arg_id: "feature".to_string(),
                    occurrence: Some(0),
                },
            },
            TokenProvenance {
                token_index: 3,
                kind: TokenProvenanceKind::AuthoredValue {
                    arg_id: "value".to_string(),
                    occurrence: Some(0),
                },
            },
            TokenProvenance {
                token_index: 4,
                kind: TokenProvenanceKind::RawBoundary { arg_id: None },
            },
            TokenProvenance {
                token_index: 5,
                kind: TokenProvenanceKind::PreservedExternalSubcommandArg {
                    occurrence: Some(0),
                },
            },
        ];

        let rendered = render_command(&argv, &provenance, TargetShell::Posix);

        assert_eq!(
            rendered.text,
            rendered
                .tokens
                .iter()
                .map(|token| token.text.as_str())
                .collect::<Vec<_>>()
                .join(" ")
        );
        assert_eq!(
            rendered
                .tokens
                .iter()
                .map(|token| token.kind)
                .collect::<Vec<_>>(),
            vec![
                RenderedShellTokenKind::EntryPoint,
                RenderedShellTokenKind::SubcommandName,
                RenderedShellTokenKind::OptionSpelling,
                RenderedShellTokenKind::Value,
                RenderedShellTokenKind::RawBoundary,
                RenderedShellTokenKind::PreservedExternalToken,
            ]
        );
        assert_eq!(
            rendered
                .tokens
                .iter()
                .map(|token| token.text.as_str())
                .collect::<Vec<_>>(),
            vec![
                "clap-features",
                "serve",
                "--feature=gzip",
                "-literal",
                "--",
                "'external arg'",
            ]
        );
    }

    #[test]
    fn rendered_command_uses_same_quoting_as_flat_shell_rendering() {
        let argv = os_vec(&["tool", "two words", "it's", "", "$HOME"]);

        let rendered = render_command(&argv, &[], TargetShell::Posix);

        assert_eq!(rendered.text, render_shell(&argv, TargetShell::Posix));
        assert_eq!(rendered.text, "tool 'two words' 'it'\\''s' '' '$HOME'");
        assert!(
            rendered
                .tokens
                .iter()
                .all(|token| token.kind == RenderedShellTokenKind::Value)
        );
    }
}
