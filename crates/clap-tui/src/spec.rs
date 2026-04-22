use clap::{Arg, ArgAction, Command};

pub(crate) const EXTERNAL_SUBCOMMAND_NAME_ID: &str = "__external_subcommand_name";
pub(crate) const EXTERNAL_SUBCOMMAND_ARGS_ID: &str = "__external_subcommand_args";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub(crate) struct CommandPath(Vec<String>);

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub(crate) struct ResolvedCommand<'a> {
    pub(crate) path: CommandPath,
    pub(crate) command: &'a CommandModel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SelectionError {
    UnknownPath,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct CommandModel {
    pub(crate) name: String,
    pub(crate) version: Option<String>,
    pub(crate) about: Option<String>,
    pub(crate) help: String,
    #[allow(dead_code)]
    pub(crate) path: CommandPath,
    #[allow(dead_code)]
    pub(crate) parser_rules: CommandParserRules,
    pub(crate) display: CommandDisplay,
    pub(crate) args: Vec<ArgModel>,
    pub(crate) virtual_args: Vec<ArgModel>,
    pub(crate) subcommands: Vec<CommandModel>,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct ArgModel {
    pub(crate) id: String,
    pub(crate) display_name: String,
    pub(crate) help: Option<String>,
    pub(crate) required: bool,
    pub(crate) kind: ArgKind,
    pub(crate) default_values: Vec<String>,
    pub(crate) choices: Vec<String>,
    pub(crate) position: Option<usize>,
    pub(crate) value_cardinality: ValueCardinality,
    pub(crate) value_hint: Option<String>,
    #[allow(dead_code)]
    pub(crate) metadata: ArgMetadata,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[allow(clippy::struct_excessive_bools)]
pub(crate) struct CommandParserRules {
    pub(crate) subcommand_required: bool,
    pub(crate) arg_required_else_help: bool,
    pub(crate) args_conflicts_with_subcommands: bool,
    pub(crate) subcommand_negates_reqs: bool,
    pub(crate) allow_missing_positional: bool,
    pub(crate) subcommand_precedence_over_arg: bool,
    pub(crate) allow_external_subcommands: bool,
    pub(crate) dont_delimit_trailing_values: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct ArgMetadata {
    pub(crate) identifiers: ArgIdentifiers,
    pub(crate) ownership: ArgOwnership,
    pub(crate) placement: ArgPlacement,
    pub(crate) action: ArgActionMetadata,
    pub(crate) cardinality: ArgCardinality,
    pub(crate) syntax: ArgSyntax,
    pub(crate) values: ArgValueMetadata,
    pub(crate) defaults: ArgDefaults,
    pub(crate) display: ArgDisplay,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct ArgOwnership {
    pub(crate) owner_path: CommandPath,
    pub(crate) inherited_global: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct ArgIdentifiers {
    pub(crate) short: Option<char>,
    pub(crate) long: Option<String>,
    pub(crate) visible_short_aliases: Vec<char>,
    pub(crate) visible_aliases: Vec<String>,
    pub(crate) display_label: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum ArgPlacementKind {
    #[default]
    Option,
    Positional,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct ArgPlacement {
    pub(crate) kind: ArgPlacementKind,
    pub(crate) index: Option<usize>,
    pub(crate) global: bool,
    pub(crate) last: bool,
    pub(crate) trailing_var_arg: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum ArgActionKind {
    #[default]
    Set,
    Append,
    Count,
    SetTrue,
    SetFalse,
    Help,
    HelpShort,
    HelpLong,
    Version,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum ArgValueArity {
    None,
    #[default]
    Required,
    Optional,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct ArgActionMetadata {
    pub(crate) kind: ArgActionKind,
    pub(crate) value_arity: ArgValueArity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct ArgCardinality {
    pub(crate) min_values: usize,
    pub(crate) max_values: Option<usize>,
    pub(crate) unbounded: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct ArgSyntax {
    pub(crate) require_equals: bool,
    pub(crate) value_delimiter: Option<char>,
    pub(crate) value_terminator: Option<String>,
    pub(crate) allow_hyphen_values: bool,
    pub(crate) allow_negative_numbers: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct ArgValueMetadata {
    pub(crate) possible_values: Vec<PossibleValueMetadata>,
    pub(crate) value_names: Vec<String>,
    pub(crate) value_hint: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct PossibleValueMetadata {
    pub(crate) name: String,
    pub(crate) help: Option<String>,
    pub(crate) hidden: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[allow(clippy::struct_excessive_bools)]
pub(crate) struct ArgDefaults {
    pub(crate) default_values: Vec<String>,
    pub(crate) env: Option<String>,
    pub(crate) has_conditional_defaults: bool,
    pub(crate) hide_default_values: bool,
    pub(crate) hide_env: bool,
    pub(crate) hide_env_values: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct ArgDisplay {
    pub(crate) help: Option<String>,
    pub(crate) long_help: Option<String>,
    pub(crate) help_heading: Option<String>,
    pub(crate) display_order: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct CommandDisplay {
    pub(crate) display_order: usize,
    pub(crate) visible_aliases: Vec<String>,
    pub(crate) display_label: String,
    pub(crate) subcommand_help_heading: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum ArgKind {
    #[default]
    Flag,
    ValueOption,
    PositionalValue,
}

impl ArgKind {
    #[allow(dead_code)]
    #[allow(non_upper_case_globals)]
    pub(crate) const Option: Self = Self::ValueOption;

    #[allow(dead_code)]
    #[allow(non_upper_case_globals)]
    pub(crate) const Positional: Self = Self::PositionalValue;

    #[allow(dead_code)]
    #[allow(non_upper_case_globals)]
    pub(crate) const Enum: Self = Self::ValueOption;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum ValueCardinality {
    None,
    #[default]
    One,
    Many,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ChoiceSource {
    None,
    Static(Vec<String>),
}

impl CommandPath {
    pub(crate) fn new(parts: Vec<String>) -> Self {
        Self(parts)
    }

    pub(crate) fn as_slice(&self) -> &[String] {
        &self.0
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub(crate) fn to_key(&self, root: &str) -> String {
        let mut parts = vec![root.to_string()];
        parts.extend(self.0.iter().cloned());
        parts.join("::")
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &String> {
        self.0.iter()
    }

    pub(crate) fn storage_key(&self) -> String {
        self.0.join("::")
    }
}

pub(crate) fn format_command_path(root_name: &str, path: &CommandPath) -> String {
    let mut parts = vec![root_name.to_string()];
    parts.extend(path.iter().cloned());
    parts.join(" > ")
}

impl From<Vec<String>> for CommandPath {
    fn from(value: Vec<String>) -> Self {
        Self(value)
    }
}

#[allow(dead_code)]
impl ArgModel {
    pub(crate) fn short(&self) -> Option<char> {
        self.metadata.identifiers.short
    }

    pub(crate) fn long(&self) -> Option<&str> {
        self.metadata.identifiers.long.as_deref()
    }

    pub(crate) fn visible_short_aliases(&self) -> &[char] {
        &self.metadata.identifiers.visible_short_aliases
    }

    pub(crate) fn visible_aliases(&self) -> &[String] {
        &self.metadata.identifiers.visible_aliases
    }

    pub(crate) fn display_label(&self) -> &str {
        if self.metadata.identifiers.display_label.is_empty() {
            &self.display_name
        } else {
            &self.metadata.identifiers.display_label
        }
    }

    pub(crate) fn owner_path(&self) -> &CommandPath {
        &self.metadata.ownership.owner_path
    }

    pub(crate) fn is_flag(&self) -> bool {
        matches!(self.kind, ArgKind::Flag)
    }

    pub(crate) fn is_positional(&self) -> bool {
        matches!(self.kind, ArgKind::PositionalValue) || self.position.is_some()
    }

    pub(crate) fn is_global(&self) -> bool {
        self.metadata.placement.global
    }

    pub(crate) fn is_inherited_global(&self) -> bool {
        self.metadata.ownership.inherited_global
    }

    pub(crate) fn is_inherited_for(&self, selected_path: &CommandPath) -> bool {
        self.owner_path() != selected_path
    }

    pub(crate) fn action_kind(&self) -> ArgActionKind {
        self.metadata.action.kind
    }

    pub(crate) fn accepts_optional_values(&self) -> bool {
        self.metadata.action.value_arity == ArgValueArity::Optional
    }

    pub(crate) fn is_help_action(&self) -> bool {
        matches!(
            self.action_kind(),
            ArgActionKind::Help | ArgActionKind::HelpShort | ArgActionKind::HelpLong
        )
    }

    pub(crate) fn uses_count_semantics(&self) -> bool {
        self.action_kind() == ArgActionKind::Count
    }

    pub(crate) fn accepts_values(&self) -> bool {
        self.metadata.action.value_arity != ArgValueArity::None
    }

    pub(crate) fn has_value_choices(&self) -> bool {
        !self.choices.is_empty()
    }

    pub(crate) fn default_value(&self) -> Option<&str> {
        self.default_values.first().map(String::as_str)
    }

    pub(crate) fn has_conditional_defaults(&self) -> bool {
        self.metadata.defaults.has_conditional_defaults
    }

    pub(crate) fn accepts_multiple_values_per_occurrence(&self) -> bool {
        matches!(self.value_cardinality, ValueCardinality::Many)
    }

    pub(crate) fn accepts_multiple_values(&self) -> bool {
        self.accepts_multiple_values_per_occurrence()
    }

    pub(crate) fn allows_multiple_occurrences(&self) -> bool {
        self.action_kind() == ArgActionKind::Append
    }

    pub(crate) fn is_multi_value_input(&self) -> bool {
        self.accepts_multiple_values_per_occurrence() || self.allows_multiple_occurrences()
    }

    pub(crate) fn uses_toggle_semantics(&self) -> bool {
        self.is_flag() && !self.uses_count_semantics() && !self.uses_optional_value_semantics()
    }

    pub(crate) fn uses_optional_value_semantics(&self) -> bool {
        self.accepts_optional_values()
            && !self.is_positional()
            && !self.uses_count_semantics()
            && !self.is_multi_value_input()
    }

    pub(crate) fn serializes_as_positional(&self) -> bool {
        self.is_positional()
    }

    pub(crate) fn is_last_positional(&self) -> bool {
        self.metadata.placement.last
    }

    pub(crate) fn is_trailing_var_arg(&self) -> bool {
        self.metadata.placement.trailing_var_arg
    }

    pub(crate) fn value_terminator(&self) -> Option<&str> {
        self.metadata.syntax.value_terminator.as_deref()
    }

    #[allow(dead_code)]
    pub(crate) fn choice_source(&self) -> ChoiceSource {
        if self.choices.is_empty() {
            ChoiceSource::None
        } else {
            ChoiceSource::Static(self.choices.clone())
        }
    }

    pub(crate) fn display_order(&self) -> usize {
        self.metadata.display.display_order
    }

    pub(crate) fn help_heading(&self) -> Option<&str> {
        self.metadata.display.help_heading.as_deref()
    }

    pub(crate) fn long_help(&self) -> Option<&str> {
        self.metadata.display.long_help.as_deref()
    }

    pub(crate) fn value_names(&self) -> &[String] {
        &self.metadata.values.value_names
    }

    pub(crate) fn choice_metadata(&self, value: &str) -> Option<&PossibleValueMetadata> {
        self.metadata
            .values
            .possible_values
            .iter()
            .find(|candidate| candidate.name == value)
    }

    pub(crate) fn is_external_subcommand_name(&self) -> bool {
        self.id == EXTERNAL_SUBCOMMAND_NAME_ID
    }

    pub(crate) fn is_external_subcommand_args(&self) -> bool {
        self.id == EXTERNAL_SUBCOMMAND_ARGS_ID
    }

    pub(crate) fn is_external_subcommand_field(&self) -> bool {
        self.is_external_subcommand_name() || self.is_external_subcommand_args()
    }
}

pub(crate) fn choice_value_matches_default(arg: &ArgModel, value: &str) -> bool {
    arg.default_value() == Some(value)
}

const AUTOGENERATED_HELP_SUBCOMMAND_ABOUT: &str =
    "Print this message or the help of the given subcommand(s)";

#[allow(dead_code)]
impl CommandModel {
    pub(crate) fn from_command(command: &Command) -> Self {
        Self::from_command_with_path(command, CommandPath::default(), &[])
    }

    fn from_command_with_path(
        command: &Command,
        path: CommandPath,
        inherited_globals: &[(String, CommandPath)],
    ) -> Self {
        let mut cmd = command.clone();
        let help = cmd.render_help().to_string();
        let parser_rules = CommandParserRules {
            subcommand_required: cmd.is_subcommand_required_set(),
            arg_required_else_help: cmd.is_arg_required_else_help_set(),
            args_conflicts_with_subcommands: cmd.is_args_conflicts_with_subcommands_set(),
            subcommand_negates_reqs: cmd.is_subcommand_negates_reqs_set(),
            allow_missing_positional: cmd.is_allow_missing_positional_set(),
            subcommand_precedence_over_arg: cmd.is_subcommand_precedence_over_arg_set(),
            allow_external_subcommands: cmd.is_allow_external_subcommands_set(),
            dont_delimit_trailing_values: cmd.is_dont_delimit_trailing_values_set(),
        };
        let display = CommandDisplay {
            display_order: cmd.get_display_order(),
            visible_aliases: cmd
                .get_visible_aliases()
                .map(str::to_string)
                .collect::<Vec<_>>(),
            display_label: command_display_label(&cmd),
            subcommand_help_heading: cmd.get_subcommand_help_heading().map(str::to_string),
        };
        let args = cmd
            .get_arguments()
            .filter_map(|arg| arg_to_model(arg, &path, inherited_globals))
            .collect::<Vec<_>>();
        let mut child_inherited_globals = inherited_globals.to_vec();
        for arg in &args {
            if arg.is_global() && !arg.is_inherited_global() {
                child_inherited_globals.retain(|(id, _)| id != &arg.id);
                child_inherited_globals.push((arg.id.clone(), path.clone()));
            }
        }
        let mut subcommands = cmd
            .get_subcommands()
            .filter(|subcommand| !is_autogenerated_help_subcommand(&cmd, subcommand))
            .filter(|subcommand| !subcommand.is_hide_set())
            .map(|subcommand| {
                let mut child_path = path.as_slice().to_vec();
                child_path.push(subcommand.get_name().to_string());
                CommandModel::from_command_with_path(
                    subcommand,
                    CommandPath::new(child_path),
                    &child_inherited_globals,
                )
            })
            .collect::<Vec<_>>();
        subcommands.sort_by(|left, right| {
            left.display_order()
                .cmp(&right.display_order())
                .then_with(|| left.name.cmp(&right.name))
        });
        let virtual_args = if parser_rules.allow_external_subcommands {
            external_subcommand_virtual_args(&path)
        } else {
            Vec::new()
        };
        Self {
            name: cmd.get_name().to_string(),
            version: cmd.get_version().map(std::string::ToString::to_string),
            about: cmd.get_about().map(std::string::ToString::to_string),
            help,
            path,
            parser_rules,
            display,
            args,
            virtual_args,
            subcommands,
        }
    }

    pub(crate) fn resolve_path(&self, path: &[String]) -> Option<&CommandModel> {
        let mut cmd = self;
        for name in path {
            cmd = cmd
                .subcommands
                .iter()
                .find(|candidate| &candidate.name == name)?;
        }
        Some(cmd)
    }

    pub(crate) fn normalize_path(&self, path: &[String]) -> Option<CommandPath> {
        self.resolve_path(path)?;
        Some(CommandPath::new(path.to_vec()))
    }

    pub(crate) fn command_lineage(&self, path: &CommandPath) -> Option<Vec<&CommandModel>> {
        let mut lineage = Vec::with_capacity(path.as_slice().len() + 1);
        let mut cmd = self;
        lineage.push(cmd);
        for name in path.iter() {
            cmd = cmd
                .subcommands
                .iter()
                .find(|candidate| &candidate.name == name)?;
            lineage.push(cmd);
        }
        Some(lineage)
    }

    pub(crate) fn args_defined_on_path(
        &self,
        path: &CommandPath,
    ) -> Option<Vec<(&CommandPath, &ArgSpec)>> {
        Some(
            self.command_lineage(path)?
                .into_iter()
                .flat_map(|command| {
                    command
                        .all_args()
                        .into_iter()
                        .filter(|arg| is_invocation_arg(arg))
                        .map(move |arg| (&command.path, arg))
                })
                .collect(),
        )
    }

    pub(crate) fn inherited_global_args(
        &self,
        path: &CommandPath,
    ) -> Option<Vec<(&CommandPath, &ArgSpec)>> {
        let lineage = self.command_lineage(path)?;
        let inherited_len = lineage.len().saturating_sub(1);
        Some(
            lineage
                .into_iter()
                .take(inherited_len)
                .flat_map(|command| {
                    command
                        .all_args()
                        .into_iter()
                        .filter(|arg| is_invocation_arg(arg) && arg.is_global())
                        .map(move |arg| (&command.path, arg))
                })
                .collect(),
        )
    }

    pub(crate) fn effective_args_for_path(
        &self,
        path: &CommandPath,
    ) -> Option<Vec<(&CommandPath, &ArgSpec)>> {
        self.args_defined_on_path(path)
    }

    pub(crate) fn owning_command_for_arg(
        &self,
        path: &CommandPath,
        arg_id: &str,
    ) -> Option<&CommandModel> {
        let (owner_path, _) = self
            .effective_args_for_path(path)?
            .into_iter()
            .find(|(_, arg)| arg.id == arg_id)?;
        self.resolve_path(owner_path.as_slice())
    }

    pub(crate) fn expand_prefix_keys(&self, path: &CommandPath) -> Vec<String> {
        let mut keys = Vec::with_capacity(path.as_slice().len() + 1);
        let mut parts = vec![self.name.clone()];
        keys.push(parts.join("::"));
        for part in path.iter() {
            parts.push(part.clone());
            keys.push(parts.join("::"));
        }
        keys
    }

    pub(crate) fn find_path_by_search_path(&self, start: &str) -> Option<CommandPath> {
        let path = if start.contains("::") {
            start.split("::").map(str::to_string).collect::<Vec<_>>()
        } else {
            start
                .split_whitespace()
                .map(str::to_string)
                .collect::<Vec<_>>()
        };
        self.normalize_path(&path)
    }

    pub(crate) fn resolved<'a>(&'a self, path: &CommandPath) -> ResolvedCommand<'a> {
        let command = self
            .resolve_path(path.as_slice())
            .expect("command path is validated before storage");
        ResolvedCommand {
            path: path.clone(),
            command,
        }
    }

    pub(crate) fn display_order(&self) -> usize {
        self.display.display_order
    }

    pub(crate) fn visible_aliases(&self) -> &[String] {
        &self.display.visible_aliases
    }

    pub(crate) fn display_label(&self) -> &str {
        if self.display.display_label.is_empty() {
            &self.name
        } else {
            &self.display.display_label
        }
    }

    pub(crate) fn subcommand_help_heading(&self) -> Option<&str> {
        self.display.subcommand_help_heading.as_deref()
    }

    pub(crate) fn all_args(&self) -> Vec<&ArgModel> {
        self.args.iter().chain(self.virtual_args.iter()).collect()
    }
}

fn is_autogenerated_help_subcommand(command: &Command, subcommand: &Command) -> bool {
    !command.is_disable_help_subcommand_set()
        && subcommand.get_name() == "help"
        && subcommand
            .get_about()
            .is_some_and(|about| about.to_string() == AUTOGENERATED_HELP_SUBCOMMAND_ABOUT)
        && subcommand.is_disable_help_flag_set()
        && subcommand.is_disable_version_flag_set()
}

#[allow(clippy::too_many_lines)]
fn arg_to_model(
    arg: &Arg,
    path: &CommandPath,
    inherited_globals: &[(String, CommandPath)],
) -> Option<ArgModel> {
    if arg.is_hide_set() {
        return None;
    }

    let id = arg.get_id().to_string();
    let inherited_owner_path = inherited_globals
        .iter()
        .rev()
        .find(|(inherited_id, _)| inherited_id == &id)
        .map(|(_, owner_path)| owner_path.clone());
    let short = arg.get_short();
    let long = arg.get_long().map(str::to_string);
    let visible_short_aliases = arg.get_visible_short_aliases().unwrap_or_default();
    let visible_aliases = arg
        .get_visible_aliases()
        .unwrap_or_default()
        .into_iter()
        .map(str::to_string)
        .collect::<Vec<_>>();
    let display_name = primary_display_name(arg, &id);
    let display_label = display_label(arg, &id);
    let help = arg.get_help().map(std::string::ToString::to_string);
    let long_help = arg.get_long_help().map(std::string::ToString::to_string);
    let required = arg.is_required_set();
    let possible_values = arg.get_possible_values();
    let choices = possible_values
        .iter()
        .filter(|value| !value.is_hide_set())
        .map(|value| value.get_name().to_string())
        .collect::<Vec<_>>();
    let possible_value_metadata = possible_values
        .iter()
        .map(|value| PossibleValueMetadata {
            name: value.get_name().to_string(),
            help: value.get_help().map(std::string::ToString::to_string),
            hidden: value.is_hide_set(),
        })
        .collect::<Vec<_>>();
    let value_names = arg
        .get_value_names()
        .unwrap_or_default()
        .iter()
        .map(std::string::ToString::to_string)
        .collect::<Vec<_>>();
    let default_values = arg
        .get_default_values()
        .iter()
        .map(|v| v.to_string_lossy().to_string())
        .collect::<Vec<_>>();
    let position = if arg.is_positional() {
        arg.get_index()
    } else {
        None
    };
    let action_kind = arg_action_kind(arg.get_action());
    let cardinality = cardinality_for(arg, action_kind);
    let value_arity = if cardinality.max_values == Some(0) {
        ArgValueArity::None
    } else if cardinality.min_values == 0 {
        ArgValueArity::Optional
    } else {
        ArgValueArity::Required
    };
    let value_cardinality = compat_value_cardinality(cardinality);
    let value_hint = match arg.get_value_hint() {
        clap::ValueHint::Unknown => None,
        hint => Some(format!("{hint:?}")),
    };
    let kind = if is_zero_arity_flag_action(action_kind) {
        ArgKind::Flag
    } else if arg.is_positional() {
        ArgKind::PositionalValue
    } else {
        ArgKind::ValueOption
    };
    let metadata = ArgMetadata {
        identifiers: ArgIdentifiers {
            short,
            long,
            visible_short_aliases,
            visible_aliases,
            display_label,
        },
        ownership: ArgOwnership {
            owner_path: inherited_owner_path.clone().unwrap_or_else(|| path.clone()),
            inherited_global: inherited_owner_path.is_some() && arg.is_global_set(),
        },
        placement: ArgPlacement {
            kind: if position.is_some() {
                ArgPlacementKind::Positional
            } else {
                ArgPlacementKind::Option
            },
            index: position,
            global: arg.is_global_set(),
            last: arg.is_last_set(),
            trailing_var_arg: arg.is_trailing_var_arg_set(),
        },
        action: ArgActionMetadata {
            kind: action_kind,
            value_arity,
        },
        cardinality,
        syntax: ArgSyntax {
            require_equals: arg.is_require_equals_set(),
            value_delimiter: arg.get_value_delimiter(),
            value_terminator: arg
                .get_value_terminator()
                .map(std::string::ToString::to_string),
            allow_hyphen_values: arg.is_allow_hyphen_values_set(),
            allow_negative_numbers: arg.is_allow_negative_numbers_set(),
        },
        values: ArgValueMetadata {
            possible_values: possible_value_metadata,
            value_names,
            value_hint: value_hint.clone(),
        },
        defaults: ArgDefaults {
            default_values: default_values.clone(),
            env: arg.get_env().map(|env| env.to_string_lossy().to_string()),
            has_conditional_defaults: arg_debug_field_is_non_empty(arg, "default_vals_ifs"),
            hide_default_values: arg.is_hide_default_value_set(),
            hide_env: arg.is_hide_env_set(),
            hide_env_values: arg.is_hide_env_values_set(),
        },
        display: ArgDisplay {
            help: help.clone(),
            long_help,
            help_heading: arg.get_help_heading().map(str::to_string),
            display_order: arg.get_display_order(),
        },
    };

    Some(ArgModel {
        id,
        display_name,
        help,
        required,
        kind,
        default_values,
        choices,
        position,
        value_cardinality,
        value_hint,
        metadata,
    })
}

fn primary_display_name(arg: &Arg, id: &str) -> String {
    arg.get_long()
        .map(|name| format!("--{name}"))
        .or_else(|| arg.get_short().map(|name| format!("-{name}")))
        .unwrap_or_else(|| id.to_string())
}

fn command_display_label(command: &Command) -> String {
    let name = command.get_name();
    let aliases = command.get_visible_aliases().collect::<Vec<_>>();
    if aliases.is_empty() {
        name.to_string()
    } else {
        format!("{name} ({})", aliases.join(", "))
    }
}

fn external_subcommand_virtual_args(path: &CommandPath) -> Vec<ArgModel> {
    vec![
        ArgModel {
            id: EXTERNAL_SUBCOMMAND_NAME_ID.to_string(),
            display_name: "External subcommand".to_string(),
            help: Some(
                "Run an unknown subcommand without selecting a known tree node.".to_string(),
            ),
            required: false,
            kind: ArgKind::ValueOption,
            default_values: Vec::new(),
            choices: Vec::new(),
            position: None,
            value_cardinality: ValueCardinality::One,
            value_hint: None,
            metadata: ArgMetadata {
                identifiers: ArgIdentifiers {
                    display_label: "External subcommand".to_string(),
                    ..ArgIdentifiers::default()
                },
                ownership: ArgOwnership {
                    owner_path: path.clone(),
                    inherited_global: false,
                },
                action: ArgActionMetadata {
                    kind: ArgActionKind::Set,
                    value_arity: ArgValueArity::Required,
                },
                cardinality: ArgCardinality {
                    min_values: 1,
                    max_values: Some(1),
                    unbounded: false,
                },
                values: ArgValueMetadata {
                    value_names: vec!["COMMAND".to_string()],
                    ..ArgValueMetadata::default()
                },
                display: ArgDisplay {
                    help_heading: Some("External".to_string()),
                    display_order: 9_998,
                    ..ArgDisplay::default()
                },
                ..ArgMetadata::default()
            },
        },
        ArgModel {
            id: EXTERNAL_SUBCOMMAND_ARGS_ID.to_string(),
            display_name: "Trailing args".to_string(),
            help: Some(
                "Add trailing tokens for the external subcommand, one token per row.".to_string(),
            ),
            required: false,
            kind: ArgKind::ValueOption,
            default_values: Vec::new(),
            choices: Vec::new(),
            position: None,
            value_cardinality: ValueCardinality::One,
            value_hint: None,
            metadata: ArgMetadata {
                identifiers: ArgIdentifiers {
                    display_label: "Trailing args".to_string(),
                    ..ArgIdentifiers::default()
                },
                ownership: ArgOwnership {
                    owner_path: path.clone(),
                    inherited_global: false,
                },
                action: ArgActionMetadata {
                    kind: ArgActionKind::Append,
                    value_arity: ArgValueArity::Required,
                },
                cardinality: ArgCardinality {
                    min_values: 1,
                    max_values: Some(1),
                    unbounded: false,
                },
                values: ArgValueMetadata {
                    value_names: vec!["ARG".to_string()],
                    ..ArgValueMetadata::default()
                },
                display: ArgDisplay {
                    help_heading: Some("External".to_string()),
                    display_order: 9_999,
                    ..ArgDisplay::default()
                },
                ..ArgMetadata::default()
            },
        },
    ]
}

fn arg_debug_field_is_non_empty(arg: &Arg, field: &str) -> bool {
    let debug = format!("{arg:?}");
    let prefix = format!("{field}: [");
    let Some(start) = debug.find(&prefix) else {
        return false;
    };
    let rest = &debug[start + prefix.len()..];
    let Some(end) = rest.find(']') else {
        return false;
    };
    !rest[..end].trim().is_empty()
}

fn display_label(arg: &Arg, id: &str) -> String {
    if arg.is_positional() {
        return arg
            .get_value_names()
            .and_then(|names| names.first())
            .map_or_else(|| id.to_string(), std::string::ToString::to_string);
    }

    match (arg.get_short(), arg.get_long()) {
        (Some(short), Some(long)) => format!("-{short}, --{long}"),
        (Some(short), None) => format!("-{short}"),
        (None, Some(long)) => format!("--{long}"),
        (None, None) => id.to_string(),
    }
}

fn arg_action_kind(action: &ArgAction) -> ArgActionKind {
    match action {
        ArgAction::Append => ArgActionKind::Append,
        ArgAction::Count => ArgActionKind::Count,
        ArgAction::SetTrue => ArgActionKind::SetTrue,
        ArgAction::SetFalse => ArgActionKind::SetFalse,
        ArgAction::Help => ArgActionKind::Help,
        ArgAction::HelpShort => ArgActionKind::HelpShort,
        ArgAction::HelpLong => ArgActionKind::HelpLong,
        ArgAction::Version => ArgActionKind::Version,
        _ => ArgActionKind::Set,
    }
}

fn is_zero_arity_flag_action(action_kind: ArgActionKind) -> bool {
    matches!(
        action_kind,
        ArgActionKind::SetTrue
            | ArgActionKind::SetFalse
            | ArgActionKind::Help
            | ArgActionKind::HelpShort
            | ArgActionKind::HelpLong
            | ArgActionKind::Version
    )
}

fn cardinality_for(arg: &Arg, action_kind: ArgActionKind) -> ArgCardinality {
    if let Some(num_args) = arg.get_num_args() {
        let max_values = num_args.max_values();
        return ArgCardinality {
            min_values: num_args.min_values(),
            max_values: (max_values != usize::MAX).then_some(max_values),
            unbounded: max_values == usize::MAX,
        };
    }

    if matches!(
        action_kind,
        ArgActionKind::Count
            | ArgActionKind::Help
            | ArgActionKind::HelpShort
            | ArgActionKind::HelpLong
            | ArgActionKind::Version
    ) {
        return ArgCardinality {
            min_values: 0,
            max_values: Some(0),
            unbounded: false,
        };
    }

    if matches!(
        action_kind,
        ArgActionKind::SetTrue | ArgActionKind::SetFalse
    ) {
        return ArgCardinality {
            min_values: 0,
            max_values: Some(0),
            unbounded: false,
        };
    }

    ArgCardinality {
        min_values: 1,
        max_values: Some(1),
        unbounded: false,
    }
}

fn compat_value_cardinality(cardinality: ArgCardinality) -> ValueCardinality {
    match cardinality.max_values {
        Some(0) => ValueCardinality::None,
        Some(max) if max > 1 => ValueCardinality::Many,
        None if cardinality.unbounded => ValueCardinality::Many,
        Some(_) | None => ValueCardinality::One,
    }
}

#[allow(dead_code)]
fn is_invocation_arg(arg: &ArgSpec) -> bool {
    !arg.is_help_action() && !arg.is_inherited_global()
}

pub(crate) type CommandSpec = CommandModel;
pub(crate) type ArgSpec = ArgModel;

#[cfg(test)]
mod tests {
    use clap::{
        Arg, ArgAction, Command,
        builder::{ArgPredicate, PossibleValue},
        value_parser,
    };

    use super::{ArgActionKind, ArgKind, ArgValueArity, CommandPath, CommandSpec};

    #[test]
    fn extracts_rich_arg_metadata_without_losing_compatibility_fields() {
        let command = Command::new("tool").arg(
            Arg::new("include")
                .short('I')
                .visible_short_alias('i')
                .long("include")
                .visible_alias("inc")
                .help("Include path")
                .long_help("Include one or more paths")
                .help_heading("Inputs")
                .display_order(7)
                .action(ArgAction::Append)
                .num_args(1..)
                .value_delimiter(',')
                .value_terminator(";")
                .allow_hyphen_values(true)
                .allow_negative_numbers(true)
                .require_equals(true)
                .default_values(["src", "tests"])
                .env("TOOL_INCLUDE")
                .value_names(["PATH"])
                .value_parser(value_parser!(String))
                .global(true),
        );

        let spec = CommandSpec::from_command(&command);
        let arg = &spec.args[0];

        assert_eq!(arg.display_name, "--include");
        assert_eq!(arg.display_label(), "-I, --include");
        assert_eq!(arg.short(), Some('I'));
        assert_eq!(arg.long(), Some("include"));
        assert_eq!(arg.visible_short_aliases(), &['i']);
        assert_eq!(arg.visible_aliases(), &["inc".to_string()]);
        assert_eq!(arg.action_kind(), ArgActionKind::Append);
        assert_eq!(arg.metadata.action.value_arity, ArgValueArity::Required);
        assert!(arg.is_global());
        assert_eq!(arg.metadata.cardinality.min_values, 1);
        assert!(arg.metadata.cardinality.unbounded);
        assert_eq!(arg.metadata.syntax.value_delimiter, Some(','));
        assert_eq!(arg.metadata.syntax.value_terminator.as_deref(), Some(";"));
        assert!(arg.metadata.syntax.require_equals);
        assert!(arg.metadata.syntax.allow_hyphen_values);
        assert!(arg.metadata.syntax.allow_negative_numbers);
        assert_eq!(arg.default_values, vec!["src", "tests"]);
        assert_eq!(arg.metadata.defaults.env.as_deref(), Some("TOOL_INCLUDE"));
        assert_eq!(arg.metadata.values.value_names, vec!["PATH"]);
        assert_eq!(arg.metadata.display.help_heading.as_deref(), Some("Inputs"));
        assert_eq!(
            arg.metadata.display.long_help.as_deref(),
            Some("Include one or more paths")
        );
        assert_eq!(arg.metadata.display.display_order, 7);
    }

    #[test]
    fn tracks_optional_value_variants_and_command_parser_rules() {
        let command = Command::new("tool")
            .subcommand_required(true)
            .arg_required_else_help(true)
            .args_conflicts_with_subcommands(true)
            .subcommand_negates_reqs(true)
            .allow_missing_positional(true)
            .subcommand_precedence_over_arg(true)
            .allow_external_subcommands(true)
            .dont_delimit_trailing_values(true)
            .arg(
                Arg::new("color")
                    .long("color")
                    .action(ArgAction::SetTrue)
                    .num_args(0..=1),
            )
            .subcommand(Command::new("run"));

        let spec = CommandSpec::from_command(&command);
        let arg = &spec.args[0];

        assert_eq!(arg.action_kind(), ArgActionKind::SetTrue);
        assert_eq!(arg.metadata.action.value_arity, ArgValueArity::Optional);
        assert_eq!(arg.metadata.cardinality.min_values, 0);
        assert_eq!(arg.metadata.cardinality.max_values, Some(1));
        assert!(spec.parser_rules.subcommand_required);
        assert!(spec.parser_rules.arg_required_else_help);
        assert!(spec.parser_rules.args_conflicts_with_subcommands);
        assert!(spec.parser_rules.subcommand_negates_reqs);
        assert!(spec.parser_rules.allow_missing_positional);
        assert!(spec.parser_rules.subcommand_precedence_over_arg);
        assert!(spec.parser_rules.allow_external_subcommands);
        assert!(spec.parser_rules.dont_delimit_trailing_values);
    }

    #[test]
    fn autogenerated_version_flags_are_modeled_as_zero_arity_flags() {
        let spec = CommandSpec::from_command(&Command::new("tool").version("1.2.3"));
        let arg = spec
            .args
            .iter()
            .find(|arg| arg.action_kind() == ArgActionKind::Version)
            .expect("version arg should be present");

        assert_eq!(arg.id, "version");
        assert_eq!(arg.display_name, "--version");
        assert_eq!(arg.action_kind(), ArgActionKind::Version);
        assert_eq!(arg.metadata.action.value_arity, ArgValueArity::None);
        assert_eq!(arg.kind, ArgKind::Flag);
    }

    #[test]
    fn extracts_subcommand_display_metadata_and_sorts_by_display_order() {
        let spec = CommandSpec::from_command(
            &Command::new("tool")
                .subcommand_help_heading("Applets")
                .subcommand(Command::new("beta").visible_alias("b").display_order(2))
                .subcommand(Command::new("alpha").visible_alias("a").display_order(2)),
        );

        assert_eq!(spec.subcommand_help_heading(), Some("Applets"));
        assert_eq!(
            spec.subcommands
                .iter()
                .map(|command| command.name.as_str())
                .collect::<Vec<_>>(),
            vec!["alpha", "beta"]
        );
        assert_eq!(spec.subcommands[0].visible_aliases(), &["a".to_string()]);
        assert_eq!(spec.subcommands[0].display_label(), "alpha (a)");
    }

    #[test]
    fn allow_external_subcommands_adds_virtual_form_fields() {
        let spec =
            CommandSpec::from_command(&Command::new("tool").allow_external_subcommands(true));

        assert!(
            spec.args
                .iter()
                .all(|arg| !arg.is_external_subcommand_field())
        );
        assert_eq!(spec.virtual_args.len(), 2);
        assert!(spec.virtual_args[0].is_external_subcommand_name());
        assert!(spec.virtual_args[1].is_external_subcommand_args());
    }

    #[test]
    fn extracts_choice_help_and_hides_hidden_values_from_visible_choices() {
        let spec = CommandSpec::from_command(&Command::new("tool").arg(
            Arg::new("mode").long("mode").value_parser([
                PossibleValue::new("fast").help("Fast path"),
                PossibleValue::new("secret").hide(true),
            ]),
        ));
        let arg = &spec.args[0];

        assert_eq!(arg.choices, vec!["fast"]);
        assert_eq!(
            arg.choice_metadata("fast")
                .and_then(|choice| choice.help.as_deref()),
            Some("Fast path")
        );
        assert!(
            arg.choice_metadata("secret")
                .is_some_and(|choice| choice.hidden)
        );
    }

    #[test]
    fn extracts_conditional_default_metadata_presence() {
        let spec = CommandSpec::from_command(
            &Command::new("tool")
                .arg(Arg::new("flag").long("flag").action(ArgAction::SetTrue))
                .arg(Arg::new("mode").long("mode").default_value_if(
                    "flag",
                    ArgPredicate::IsPresent,
                    Some("auto"),
                )),
        );

        assert!(spec.args[1].has_conditional_defaults());
    }

    #[test]
    fn resolves_path_inheritance_and_owner_for_global_args() {
        let spec = CommandSpec::from_command(
            &Command::new("tool")
                .arg(Arg::new("verbose").long("verbose").global(true))
                .subcommand(
                    Command::new("build")
                        .arg(Arg::new("target").long("target"))
                        .subcommand(Command::new("release")),
                ),
        );
        let path = CommandPath::new(vec!["build".to_string(), "release".to_string()]);

        let defined = spec
            .args_defined_on_path(&path)
            .expect("path should resolve")
            .into_iter()
            .map(|(owner, arg)| (owner.as_slice().to_vec(), arg.id.clone()))
            .collect::<Vec<_>>();
        assert_eq!(
            defined,
            vec![
                (Vec::<String>::new(), "verbose".to_string()),
                (vec!["build".to_string()], "target".to_string()),
            ]
        );

        let inherited = spec
            .inherited_global_args(&path)
            .expect("path should resolve")
            .into_iter()
            .map(|(owner, arg)| (owner.as_slice().to_vec(), arg.id.clone()))
            .collect::<Vec<_>>();
        assert_eq!(
            inherited,
            vec![(Vec::<String>::new(), "verbose".to_string())]
        );

        let effective = spec
            .effective_args_for_path(&path)
            .expect("path should resolve")
            .into_iter()
            .map(|(owner, arg)| (owner.as_slice().to_vec(), arg.id.clone()))
            .collect::<Vec<_>>();
        assert_eq!(
            effective,
            vec![
                (Vec::<String>::new(), "verbose".to_string()),
                (vec!["build".to_string()], "target".to_string()),
            ]
        );

        let owner = spec
            .owning_command_for_arg(&path, "verbose")
            .expect("global arg owner should resolve");
        assert!(owner.path.is_empty());

        let owner = spec
            .owning_command_for_arg(&path, "target")
            .expect("command-local arg owner should resolve");
        assert_eq!(owner.path.as_slice(), &["build".to_string()]);
    }

    #[test]
    fn omits_autogenerated_help_subcommands_from_spec() {
        let spec = CommandSpec::from_command(
            &Command::new("tool")
                .subcommand_required(true)
                .subcommand(Command::new("admin").subcommand(Command::new("cache"))),
        );

        let top_level = spec
            .subcommands
            .iter()
            .map(|subcommand| subcommand.name.as_str())
            .collect::<Vec<_>>();
        assert_eq!(top_level, vec!["admin"]);

        let admin = spec
            .subcommands
            .iter()
            .find(|subcommand| subcommand.name == "admin")
            .expect("admin subcommand should be present");
        let admin_children = admin
            .subcommands
            .iter()
            .map(|subcommand| subcommand.name.as_str())
            .collect::<Vec<_>>();
        assert_eq!(admin_children, vec!["cache"]);
    }

    #[test]
    fn keeps_explicit_help_subcommands_when_help_subcommand_is_disabled() {
        let spec = CommandSpec::from_command(
            &Command::new("tool")
                .disable_help_subcommand(true)
                .subcommand(Command::new("help").about("Custom help workflow")),
        );

        let names = spec
            .subcommands
            .iter()
            .map(|subcommand| subcommand.name.as_str())
            .collect::<Vec<_>>();
        assert_eq!(names, vec!["help"]);
        assert_eq!(
            spec.subcommands[0].about.as_deref(),
            Some("Custom help workflow")
        );
    }

    #[test]
    fn omits_hidden_subcommands_from_spec() {
        let spec = CommandSpec::from_command(
            &Command::new("tool")
                .subcommand(Command::new("visible"))
                .subcommand(Command::new("hidden").hide(true)),
        );

        let names = spec
            .subcommands
            .iter()
            .map(|subcommand| subcommand.name.as_str())
            .collect::<Vec<_>>();
        assert_eq!(names, vec!["visible"]);
    }
}
