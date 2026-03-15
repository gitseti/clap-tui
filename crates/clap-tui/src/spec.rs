use clap::{Arg, ArgAction, Command};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
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

#[derive(Debug, Clone)]
pub(crate) struct CommandModel {
    pub(crate) name: String,
    pub(crate) about: Option<String>,
    pub(crate) help: String,
    pub(crate) args: Vec<ArgModel>,
    pub(crate) subcommands: Vec<CommandModel>,
}

#[derive(Debug, Clone)]
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
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ArgKind {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ValueCardinality {
    None,
    One,
    Many,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ChoiceSource {
    None,
    Static(Vec<String>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InputPresentation {
    Toggle,
    FreeText {
        multiple: bool,
        positional: bool,
    },
    ChoiceList {
        multiple: bool,
        positional: bool,
    },
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

impl From<Vec<String>> for CommandPath {
    fn from(value: Vec<String>) -> Self {
        Self(value)
    }
}

impl ArgModel {
    pub(crate) fn is_flag(&self) -> bool {
        matches!(self.kind, ArgKind::Flag)
    }

    pub(crate) fn is_positional(&self) -> bool {
        matches!(self.kind, ArgKind::PositionalValue) || self.position.is_some()
    }

    #[allow(dead_code)]
    pub(crate) fn choice_source(&self) -> ChoiceSource {
        if self.choices.is_empty() {
            ChoiceSource::None
        } else {
            ChoiceSource::Static(self.choices.clone())
        }
    }

    pub(crate) fn uses_choice_input(&self) -> bool {
        !self.choices.is_empty()
    }

    pub(crate) fn accepts_text_input(&self) -> bool {
        !self.is_flag() && !self.uses_choice_input()
    }

    pub(crate) fn default_value(&self) -> Option<&str> {
        self.default_values.first().map(String::as_str)
    }

    pub(crate) fn accepts_multiple_values(&self) -> bool {
        matches!(self.value_cardinality, ValueCardinality::Many)
    }

    pub(crate) fn input_presentation(&self) -> InputPresentation {
        if self.is_flag() {
            InputPresentation::Toggle
        } else if self.uses_choice_input() {
            InputPresentation::ChoiceList {
                multiple: self.accepts_multiple_values(),
                positional: self.is_positional(),
            }
        } else {
            InputPresentation::FreeText {
                multiple: self.accepts_multiple_values(),
                positional: self.is_positional(),
            }
        }
    }
}

pub(crate) fn choice_value_matches_default(arg: &ArgModel, value: &str) -> bool {
    arg.default_value() == Some(value)
}

impl CommandModel {
    pub(crate) fn from_command(command: &Command) -> Self {
        let mut cmd = command.clone();
        let help = cmd.render_help().to_string();
        let args = cmd.get_arguments().filter_map(arg_to_model).collect::<Vec<_>>();
        let subcommands = cmd
            .get_subcommands()
            .map(CommandModel::from_command)
            .collect::<Vec<_>>();
        Self {
            name: cmd.get_name().to_string(),
            about: cmd.get_about().map(std::string::ToString::to_string),
            help,
            args,
            subcommands,
        }
    }

    pub(crate) fn resolve_path(&self, path: &[String]) -> Option<&CommandModel> {
        let mut cmd = self;
        for name in path {
            cmd = cmd.subcommands.iter().find(|candidate| &candidate.name == name)?;
        }
        Some(cmd)
    }

    pub(crate) fn normalize_path(&self, path: &[String]) -> Option<CommandPath> {
        self.resolve_path(path)?;
        Some(CommandPath::new(path.to_vec()))
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
            start.split_whitespace().map(str::to_string).collect::<Vec<_>>()
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
}

fn arg_to_model(arg: &Arg) -> Option<ArgModel> {
    if arg.is_hide_set() {
        return None;
    }

    let id = arg.get_id().to_string();
    let display_name = arg
        .get_long()
        .map(|s| format!("--{s}"))
        .or_else(|| arg.get_short().map(|s| format!("-{s}")))
        .unwrap_or_else(|| id.clone());
    let help = arg.get_help().map(std::string::ToString::to_string);
    let required = arg.is_required_set();
    let choices = arg
        .get_possible_values()
        .iter()
        .map(|v| v.get_name().to_string())
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
    let action = arg.get_action();
    let value_cardinality = match arg.get_num_args() {
        Some(num_args) if num_args.max_values() > 1 => ValueCardinality::Many,
        None if matches!(action, ArgAction::SetTrue | ArgAction::SetFalse) => ValueCardinality::None,
        Some(_) | None => ValueCardinality::One,
    };
    let value_hint = match arg.get_value_hint() {
        clap::ValueHint::Unknown => None,
        hint => Some(format!("{hint:?}")),
    };
    let kind = if matches!(action, ArgAction::SetTrue | ArgAction::SetFalse) {
        ArgKind::Flag
    } else if arg.is_positional() {
        ArgKind::PositionalValue
    } else {
        ArgKind::ValueOption
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
    })
}

pub(crate) type CommandSpec = CommandModel;
pub(crate) type ArgSpec = ArgModel;
