use clap::{Arg, ArgAction, Command};

#[derive(Debug, Clone)]
pub struct CommandSpec {
    pub name: String,
    pub about: Option<String>,
    pub help: String,
    pub args: Vec<ArgSpec>,
    pub subcommands: Vec<CommandSpec>,
}

#[derive(Debug, Clone)]
pub struct ArgSpec {
    pub id: String,
    pub name: String,
    pub help: Option<String>,
    pub required: bool,
    pub kind: ArgKind,
    pub default: Option<String>,
    pub possible_values: Vec<String>,
    pub positional_index: Option<usize>,
    pub is_multi: bool,
    pub value_hint: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArgKind {
    Flag,
    Option,
    Positional,
    Enum,
}

impl ArgSpec {
    pub(crate) fn is_flag(&self) -> bool {
        matches!(self.kind, ArgKind::Flag)
    }

    pub(crate) fn is_positional(&self) -> bool {
        self.positional_index.is_some() || matches!(self.kind, ArgKind::Positional)
    }

    pub(crate) fn uses_choice_input(&self) -> bool {
        !self.possible_values.is_empty()
    }

    pub(crate) fn accepts_text_input(&self) -> bool {
        !self.is_flag() && !self.uses_choice_input()
    }
}

pub(crate) fn enum_value_matches_default(arg: &ArgSpec, index: usize) -> bool {
    arg.default
        .as_deref()
        .zip(arg.possible_values.get(index).map(String::as_str))
        .is_some_and(|(default, value)| default == value)
}

impl CommandSpec {
    pub fn from_command(command: &Command) -> Self {
        let mut cmd = command.clone();
        let help = cmd.render_help().to_string();
        let args = cmd
            .get_arguments()
            .filter_map(arg_to_spec)
            .collect::<Vec<_>>();
        let subcommands = cmd
            .get_subcommands()
            .map(CommandSpec::from_command)
            .collect::<Vec<_>>();
        Self {
            name: cmd.get_name().to_string(),
            about: cmd.get_about().map(std::string::ToString::to_string),
            help,
            args,
            subcommands,
        }
    }
}

fn arg_to_spec(arg: &Arg) -> Option<ArgSpec> {
    if arg.is_hide_set() {
        return None;
    }

    let id = arg.get_id().to_string();
    let name = arg
        .get_long()
        .map(|s| format!("--{s}"))
        .or_else(|| arg.get_short().map(|s| format!("-{s}")))
        .unwrap_or_else(|| id.clone());

    let help = arg.get_help().map(std::string::ToString::to_string);
    let required = arg.is_required_set();
    let possible_values = arg
        .get_possible_values()
        .iter()
        .map(|v| v.get_name().to_string())
        .collect::<Vec<_>>();

    let default = arg
        .get_default_values()
        .first()
        .map(|v| v.to_string_lossy().to_string());

    let positional_index = if arg.is_positional() {
        arg.get_index()
    } else {
        None
    };

    let action = arg.get_action();
    let is_multi = arg.get_num_args().is_some_and(|n| n.max_values() > 1);
    let value_hint = match arg.get_value_hint() {
        clap::ValueHint::Unknown => None,
        hint => Some(format!("{hint:?}")),
    };

    let kind = if !possible_values.is_empty() {
        ArgKind::Enum
    } else if arg.is_positional() {
        ArgKind::Positional
    } else if matches!(action, ArgAction::SetTrue | ArgAction::SetFalse) {
        ArgKind::Flag
    } else {
        ArgKind::Option
    };

    Some(ArgSpec {
        id,
        name,
        help,
        required,
        kind,
        default,
        possible_values,
        positional_index,
        is_multi,
        value_hint,
    })
}
