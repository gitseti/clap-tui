use crate::input::ActiveTab;
use crate::spec::{ArgActionKind, ArgSpec, CommandPath, CommandSpec, format_command_path};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FieldWidget {
    Toggle,
    SingleText,
    RepeatedText,
    SingleChoice,
    MultiChoice,
    Counter,
    OptionalValue,
}

impl FieldWidget {
    pub(crate) fn accepts_text_input(self) -> bool {
        matches!(
            self,
            Self::SingleText | Self::RepeatedText | Self::OptionalValue
        )
    }

    pub(crate) fn uses_choice_popup(self) -> bool {
        matches!(self, Self::SingleChoice | Self::MultiChoice)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct OrderedArg<'a> {
    pub(crate) order_index: usize,
    pub(crate) arg: &'a ArgSpec,
    pub(crate) widget: FieldWidget,
    pub(crate) section_heading: Option<String>,
}

pub(crate) fn single_choice_has_clear_row(
    arg: &ArgSpec,
    widget: FieldWidget,
    required: bool,
) -> bool {
    matches!(widget, FieldWidget::SingleChoice) && !required && !arg.choices.is_empty()
}

pub(crate) fn widget_for(arg: &ArgSpec) -> FieldWidget {
    if arg.uses_count_semantics() {
        FieldWidget::Counter
    } else if arg.uses_optional_value_semantics() {
        FieldWidget::OptionalValue
    } else if arg.uses_toggle_semantics() {
        FieldWidget::Toggle
    } else if arg.has_value_choices() && arg.is_multi_value_input() {
        FieldWidget::MultiChoice
    } else if arg.has_value_choices() {
        FieldWidget::SingleChoice
    } else if arg.is_multi_value_input() {
        FieldWidget::RepeatedText
    } else {
        FieldWidget::SingleText
    }
}

#[cfg(test)]
pub(crate) fn ordered_args(command: &CommandSpec) -> Vec<OrderedArg<'_>> {
    ordered_args_with_heading(command, None)
}

fn ordered_args_with_heading<'a>(
    command: &'a CommandSpec,
    section_heading_prefix: Option<&str>,
) -> Vec<OrderedArg<'a>> {
    let mut positionals = command
        .all_args()
        .into_iter()
        .filter(|arg| arg.is_positional())
        .filter(|arg| is_form_visible_arg(arg))
        .collect::<Vec<_>>();
    positionals.sort_by_key(|arg| (arg.position.unwrap_or(usize::MAX), arg.display_order()));

    let mut others = command
        .all_args()
        .into_iter()
        .filter(|arg| !arg.is_positional())
        .filter(|arg| is_form_visible_arg(arg))
        .collect::<Vec<_>>();
    others.sort_by_key(|arg| (arg.display_order(), arg.display_label().to_string()));

    positionals.extend(others);
    positionals
        .into_iter()
        .enumerate()
        .map(|(order_index, arg)| OrderedArg {
            order_index,
            arg,
            widget: widget_for(arg),
            section_heading: section_heading(arg, section_heading_prefix),
        })
        .collect()
}

#[cfg(test)]
pub(crate) fn visible_args(command: &CommandSpec, active_tab: ActiveTab) -> Vec<OrderedArg<'_>> {
    match active_tab {
        ActiveTab::Inputs => ordered_args(command),
    }
}

pub(crate) fn visible_args_for_path<'a>(
    root: &'a CommandSpec,
    selected_path: &CommandPath,
    active_tab: ActiveTab,
) -> Vec<OrderedArg<'a>> {
    match active_tab {
        ActiveTab::Inputs => visible_input_args_for_path(root, selected_path),
    }
}

fn visible_input_args_for_path<'a>(
    root: &'a CommandSpec,
    selected_path: &CommandPath,
) -> Vec<OrderedArg<'a>> {
    let Some(lineage) = root.command_lineage(selected_path) else {
        return Vec::new();
    };
    let Some(current) = lineage.last().copied() else {
        return Vec::new();
    };

    let mut ordered = ordered_args_with_heading(current, None)
        .into_iter()
        .filter(|item| item.arg.owner_path() == selected_path)
        .collect::<Vec<_>>();

    for owner in lineage.iter().rev().skip(1) {
        let owner_heading = format!(
            "Inherited from {}",
            format_command_path(&root.name, &owner.path)
        );
        ordered.extend(
            ordered_args_with_heading(owner, Some(owner_heading.as_str()))
                .into_iter()
                .filter(|item| item.arg.owner_path() == &owner.path)
                .filter(|item| !item.arg.is_external_subcommand_field())
                .collect::<Vec<_>>(),
        );
    }

    ordered
        .into_iter()
        .enumerate()
        .map(|(order_index, item)| OrderedArg {
            order_index,
            ..item
        })
        .collect()
}

pub(crate) fn visible_arg_pairs<'a>(args: &[OrderedArg<'a>]) -> Vec<(usize, &'a ArgSpec)> {
    args.iter()
        .map(|item| (item.order_index, item.arg))
        .collect()
}

pub(crate) fn field_has_description(arg: &ArgSpec, field_error: Option<&str>) -> bool {
    field_error.is_some() || arg.help.is_some() || arg.value_hint.is_some()
}

pub(crate) fn field_heading<'a>(
    previous_heading: Option<&'a str>,
    item: &'a OrderedArg<'_>,
) -> Option<&'a str> {
    let heading = item
        .section_heading
        .as_deref()
        .filter(|heading| !heading.is_empty());
    if heading.is_some() && heading != previous_heading {
        heading
    } else {
        None
    }
}

pub(crate) fn field_is_in_section(item: &OrderedArg<'_>) -> bool {
    item.section_heading
        .as_deref()
        .is_some_and(|heading| !heading.is_empty())
}

fn is_help_arg(arg: &ArgSpec) -> bool {
    arg.id == "help" || arg.display_name == "--help" || arg.display_name == "-h"
}

fn is_form_visible_arg(arg: &ArgSpec) -> bool {
    !is_help_arg(arg) && arg.action_kind() != ArgActionKind::Version
}

fn section_heading(arg: &ArgSpec, prefix: Option<&str>) -> Option<String> {
    match (
        prefix,
        arg.help_heading().filter(|heading| !heading.is_empty()),
    ) {
        (Some(prefix), Some(heading)) => Some(format!("{prefix} · {heading}")),
        (Some(prefix), None) => Some(prefix.to_string()),
        (None, heading) => heading.map(str::to_string),
    }
}

#[cfg(test)]
mod tests {
    use clap::{Arg, Command};

    use super::{
        FieldWidget, field_heading, ordered_args, visible_args, visible_args_for_path, widget_for,
    };
    use crate::input::ActiveTab;
    use crate::spec::{ArgKind, ArgSpec, CommandPath, CommandSpec};

    fn arg(id: &str, name: &str, kind: ArgKind) -> ArgSpec {
        ArgSpec {
            id: id.to_string(),
            display_name: name.to_string(),
            help: None,
            required: false,
            kind,
            default_values: Vec::new(),
            choices: Vec::new(),
            position: None,
            value_cardinality: crate::spec::ValueCardinality::One,
            value_hint: None,
            ..ArgSpec::default()
        }
    }

    fn command(args: Vec<ArgSpec>) -> CommandSpec {
        CommandSpec {
            name: "tool".to_string(),
            version: None,
            about: None,
            help: String::new(),
            args,
            subcommands: Vec::new(),
            ..CommandSpec::default()
        }
    }

    #[test]
    fn positional_args_are_ordered_before_options() {
        let mut source = vec![
            arg("verbose", "--verbose", ArgKind::Flag),
            arg("path", "path", ArgKind::Positional),
            arg("alpha", "--alpha", ArgKind::Option),
        ];
        source[1].position = Some(1);
        let command = command(source);

        let ordered = ordered_args(&command);
        let names = ordered
            .into_iter()
            .map(|item| item.arg.display_name.clone())
            .collect::<Vec<_>>();

        assert_eq!(names, vec!["path", "--alpha", "--verbose"]);
    }

    #[test]
    fn help_args_are_excluded() {
        let command = command(vec![
            arg("help", "--help", ArgKind::Flag),
            arg("target", "--target", ArgKind::Option),
        ]);

        let ordered = ordered_args(&command);

        assert_eq!(ordered.len(), 1);
        assert_eq!(ordered[0].arg.id, "target");
    }

    #[test]
    fn visible_args_follow_active_tab() {
        let mut positional = arg("path", "path", ArgKind::Positional);
        positional.position = Some(1);
        let option = arg("target", "--target", ArgKind::Option);
        let command = command(vec![positional, option]);

        assert_eq!(visible_args(&command, ActiveTab::Inputs).len(), 2);
    }

    #[test]
    fn visible_args_for_path_keeps_local_fields_primary_and_groups_inherited_owners() {
        let root = CommandSpec::from_command(
            &Command::new("tool")
                .arg(Arg::new("config").long("config"))
                .subcommand(
                    Command::new("build")
                        .arg(Arg::new("target").long("target"))
                        .subcommand(
                            Command::new("release").arg(Arg::new("profile").long("profile")),
                        ),
                ),
        );
        let selected_path = CommandPath::from(vec!["build".to_string(), "release".to_string()]);

        let visible = visible_args_for_path(&root, &selected_path, ActiveTab::Inputs);
        let ids = visible
            .iter()
            .map(|item| item.arg.id.clone())
            .collect::<Vec<_>>();

        assert_eq!(ids, vec!["profile", "target", "config"]);
        assert_eq!(visible[0].section_heading, None);
        assert_eq!(
            visible[1].section_heading.as_deref(),
            Some("Inherited from tool > build")
        );
        assert_eq!(
            visible[2].section_heading.as_deref(),
            Some("Inherited from tool")
        );
    }

    #[test]
    fn autogenerated_version_flags_use_toggle_widget_geometry() {
        let command = CommandSpec::from_command(&Command::new("tool").version("1.2.3"));
        let version = command
            .args
            .iter()
            .find(|arg| arg.display_name == "--version")
            .expect("version arg should be present");

        assert!(matches!(widget_for(version), FieldWidget::Toggle));
    }

    #[test]
    fn autogenerated_version_flags_are_excluded_from_visible_form_args() {
        let command = CommandSpec::from_command(&Command::new("tool").version("1.2.3"));

        assert!(visible_args(&command, ActiveTab::Inputs).is_empty());
    }

    #[test]
    fn heading_rows_contribute_to_form_height_and_offsets() {
        let mut first = arg("first", "--first", ArgKind::Option);
        first.metadata.display.help_heading = Some("Inputs".to_string());
        let mut second = arg("second", "--second", ArgKind::Option);
        second.metadata.display.help_heading = Some("Inputs".to_string());
        let mut third = arg("third", "--third", ArgKind::Option);
        third.metadata.display.help_heading = Some("Outputs".to_string());
        let command = command(vec![first, second, third]);
        let visible = visible_args(&command, ActiveTab::Inputs);

        assert_eq!(field_heading(None, &visible[0]), Some("Inputs"));
        assert_eq!(field_heading(Some("Inputs"), &visible[1]), None);
        assert_eq!(field_heading(Some("Inputs"), &visible[2]), Some("Outputs"));
    }
}
