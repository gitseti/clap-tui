use std::collections::BTreeMap;

use crate::input::ActiveTab;
use crate::spec::{ArgActionKind, ArgSpec, CommandPath, CommandSpec, format_command_path};

pub(crate) const SECTION_FIELD_INDENT: u16 = 1;
pub(crate) const LABEL_COLUMN_MIN_WIDTH: u16 = 12;
pub(crate) const LABEL_COLUMN_MAX_WIDTH: u16 = 24;
pub(crate) const COLUMN_GAP_WIDTH: u16 = 1;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::struct_field_names)]
pub(crate) struct FieldMetrics {
    pub(crate) label_height: u16,
    pub(crate) description_height: u16,
    pub(crate) input_height: u16,
    pub(crate) gap_height: u16,
    pub(crate) total_height: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools)]
pub(crate) struct FormHit {
    pub(crate) order_index: usize,
    pub(crate) arg_id: String,
    pub(crate) widget: FieldWidget,
    pub(crate) in_input: bool,
    pub(crate) in_label: bool,
    pub(crate) in_description: bool,
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

pub(crate) fn preferred_label_column_width(args: &[OrderedArg<'_>]) -> u16 {
    let widest_label = args
        .iter()
        .map(|item| {
            let required_marker = u16::from(item.arg.required) * 2;
            let label_width =
                u16::try_from(item.arg.display_label().chars().count()).unwrap_or(u16::MAX);
            label_width.saturating_add(required_marker)
        })
        .max()
        .unwrap_or(LABEL_COLUMN_MIN_WIDTH);

    widest_label.clamp(LABEL_COLUMN_MIN_WIDTH, LABEL_COLUMN_MAX_WIDTH)
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn field_metrics(arg: &ArgSpec) -> FieldMetrics {
    field_metrics_with_description(arg, field_has_description(arg, None))
}

pub(crate) fn field_metrics_with_description(
    arg: &ArgSpec,
    show_description: bool,
) -> FieldMetrics {
    field_metrics_with_description_and_input_height(arg, show_description, None)
}

pub(crate) fn field_metrics_with_description_and_input_height(
    arg: &ArgSpec,
    show_description: bool,
    input_height_override: Option<u16>,
) -> FieldMetrics {
    let widget = widget_for(arg);
    let label_height = 1;
    let description_height = u16::from(show_description);
    let input_height = input_height_override.unwrap_or(match widget {
        FieldWidget::Toggle
        | FieldWidget::SingleChoice
        | FieldWidget::MultiChoice
        | FieldWidget::Counter => 1,
        FieldWidget::SingleText | FieldWidget::OptionalValue => 3,
        FieldWidget::RepeatedText => 5,
    });
    let gap_height = 1;
    let content_height = label_height.max(input_height);
    FieldMetrics {
        label_height,
        description_height,
        input_height,
        gap_height,
        total_height: content_height + description_height + gap_height,
    }
}

#[cfg(test)]
pub(crate) fn field_input_offset(arg: &ArgSpec) -> u16 {
    field_input_offset_with_description(arg, field_has_description(arg, None))
}

pub(crate) fn field_input_offset_with_description(arg: &ArgSpec, show_description: bool) -> u16 {
    let _ = (arg, show_description);
    0
}

#[cfg(test)]
pub(crate) fn field_description_offset(arg: &ArgSpec) -> Option<u16> {
    field_description_offset_with_description(arg, field_has_description(arg, None))
}

pub(crate) fn field_description_offset_with_description(
    arg: &ArgSpec,
    show_description: bool,
) -> Option<u16> {
    field_description_offset_with_description_and_input_height(arg, show_description, None)
}

pub(crate) fn field_description_offset_with_description_and_input_height(
    arg: &ArgSpec,
    show_description: bool,
    input_height_override: Option<u16>,
) -> Option<u16> {
    let metrics = field_metrics_with_description_and_input_height(
        arg,
        show_description,
        input_height_override,
    );
    if metrics.description_height == 0 {
        None
    } else {
        Some(metrics.label_height.max(metrics.input_height))
    }
}

#[cfg(test)]
pub(crate) fn measure_fields_height(args: &[OrderedArg<'_>]) -> u16 {
    measure_fields_height_with_errors(args, &BTreeMap::new())
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn measure_fields_height_with_errors(
    args: &[OrderedArg<'_>],
    field_errors: &BTreeMap<String, String>,
) -> u16 {
    measure_fields_height_with_overrides(args, field_errors, &std::collections::HashMap::new())
}

pub(crate) fn measure_fields_height_with_overrides(
    args: &[OrderedArg<'_>],
    field_errors: &BTreeMap<String, String>,
    input_height_overrides: &std::collections::HashMap<String, u16>,
) -> u16 {
    let mut total = 0;
    let mut previous_heading = None;
    for item in args {
        if field_heading(previous_heading, item).is_some() {
            total += 1;
        }
        total += field_metrics_with_description_and_input_height(
            item.arg,
            field_has_description(item.arg, field_errors.get(&item.arg.id).map(String::as_str)),
            input_height_overrides.get(&item.arg.id).copied(),
        )
        .total_height;
        previous_heading = item.section_heading.as_deref();
    }
    total
}

pub(crate) fn measure_help_height(help: &str) -> u16 {
    u16::try_from(help.lines().count()).unwrap_or(u16::MAX)
}

#[cfg(test)]
pub(crate) fn field_content_bounds(
    args: &[OrderedArg<'_>],
    selected_index: usize,
) -> Option<(u16, u16)> {
    field_content_bounds_with_errors(args, selected_index, &BTreeMap::new())
}

pub(crate) fn field_content_bounds_with_errors(
    args: &[OrderedArg<'_>],
    selected_index: usize,
    field_errors: &BTreeMap<String, String>,
) -> Option<(u16, u16)> {
    let mut y: u16 = 0;
    let mut previous_heading = None;
    for item in args {
        if field_heading(previous_heading, item).is_some() {
            y = y.saturating_add(1);
        }
        let show_description =
            field_has_description(item.arg, field_errors.get(&item.arg.id).map(String::as_str));
        let metrics = field_metrics_with_description(item.arg, show_description);
        let input_top = y.saturating_add(field_input_offset_with_description(
            item.arg,
            show_description,
        ));
        let input_bottom = input_top.saturating_add(metrics.input_height);
        if item.order_index == selected_index {
            return Some((input_top, input_bottom));
        }
        y = y.saturating_add(metrics.total_height);
        previous_heading = item.section_heading.as_deref();
    }
    None
}

#[cfg(test)]
pub(crate) fn hit_test_form_content(args: &[OrderedArg<'_>], content_y: u16) -> Option<FormHit> {
    hit_test_form_content_with_errors(args, content_y, &BTreeMap::new())
}

pub(crate) fn hit_test_form_content_with_errors(
    args: &[OrderedArg<'_>],
    content_y: u16,
    field_errors: &BTreeMap<String, String>,
) -> Option<FormHit> {
    let mut y: u16 = 0;
    let mut previous_heading = None;
    for item in args {
        if field_heading(previous_heading, item).is_some() {
            y = y.saturating_add(1);
        }
        let show_description =
            field_has_description(item.arg, field_errors.get(&item.arg.id).map(String::as_str));
        let metrics = field_metrics_with_description(item.arg, show_description);
        let input_top = y.saturating_add(field_input_offset_with_description(
            item.arg,
            show_description,
        ));
        let input_bottom = input_top.saturating_add(metrics.input_height);
        let description_top = field_description_offset_with_description(item.arg, show_description)
            .map(|offset| y.saturating_add(offset));
        let label_bottom = y.saturating_add(metrics.label_height);

        let in_label = metrics.label_height > 0 && content_y >= y && content_y < label_bottom;
        let in_description = description_top.is_some_and(|top| {
            content_y >= top && content_y < top.saturating_add(metrics.description_height)
        });
        let in_input = content_y >= input_top && content_y < input_bottom;
        if in_label || in_description || in_input {
            return Some(FormHit {
                order_index: item.order_index,
                arg_id: item.arg.id.clone(),
                widget: item.widget,
                in_input,
                in_label,
                in_description,
            });
        }
        y = y.saturating_add(metrics.total_height);
        previous_heading = item.section_heading.as_deref();
    }
    None
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

pub(crate) fn field_section_ends(current: &OrderedArg<'_>, next: Option<&OrderedArg<'_>>) -> bool {
    let Some(current_heading) = current
        .section_heading
        .as_deref()
        .filter(|heading| !heading.is_empty())
    else {
        return false;
    };

    next.and_then(|item| item.section_heading.as_deref())
        .filter(|heading| !heading.is_empty())
        != Some(current_heading)
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
        FieldWidget, field_content_bounds, field_description_offset, field_heading,
        field_input_offset, field_metrics, hit_test_form_content, measure_fields_height,
        ordered_args, visible_args, visible_args_for_path, widget_for,
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
    fn field_metrics_match_single_and_multi_line_fields() {
        let single = arg("target", "--target", ArgKind::Option);
        let mut multi = arg("paths", "--path", ArgKind::Option);
        multi.value_cardinality = crate::spec::ValueCardinality::Many;
        multi.help = Some("paths".to_string());
        let mut flag = arg("verbose", "--verbose", ArgKind::Flag);
        flag.help = Some("toggle".to_string());

        assert_eq!(field_metrics(&single).total_height, 4);
        assert_eq!(field_metrics(&multi).total_height, 7);
        assert_eq!(field_metrics(&flag).total_height, 3);
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
        assert_eq!(field_metrics(version).total_height, 3);
        assert_eq!(field_input_offset(version), 0);
        assert_eq!(field_description_offset(version), Some(1));
    }

    #[test]
    fn autogenerated_version_flags_are_excluded_from_visible_form_args() {
        let command = CommandSpec::from_command(&Command::new("tool").version("1.2.3"));

        assert!(visible_args(&command, ActiveTab::Inputs).is_empty());
    }

    #[test]
    fn described_options_place_help_below_inline_input_content() {
        let mut option = arg("config", "--config", ArgKind::Option);
        option.help = Some("Path to config".to_string());
        let flag = arg("quiet", "--quiet", ArgKind::Flag);

        assert_eq!(field_description_offset(&option), Some(3));
        assert_eq!(field_input_offset(&option), 0);
        assert_eq!(field_description_offset(&flag), None);
        assert_eq!(field_input_offset(&flag), 0);
    }

    #[test]
    fn field_bounds_and_hit_testing_share_same_geometry() {
        let mut positional = arg("path", "path", ArgKind::Positional);
        positional.position = Some(1);
        positional.help = Some("required".to_string());
        let command = command(vec![positional]);
        let visible = visible_args(&command, ActiveTab::Inputs);

        assert_eq!(measure_fields_height(&visible), 5);
        assert_eq!(field_content_bounds(&visible, 0), Some((0, 3)));

        let label_hit = hit_test_form_content(&visible, 0).expect("label hit");
        assert!(label_hit.in_label);
        assert!(label_hit.in_input);
        assert!(!label_hit.in_description);

        let input_hit = hit_test_form_content(&visible, 1).expect("input hit");
        assert!(matches!(input_hit.widget, FieldWidget::SingleText));
        assert!(input_hit.in_input);
        assert!(!input_hit.in_description);

        let description_hit = hit_test_form_content(&visible, 3).expect("description hit");
        assert!(description_hit.in_description);
        assert!(!description_hit.in_input);

        assert!(hit_test_form_content(&visible, 4).is_none());
    }

    #[test]
    fn flag_metrics_and_hit_testing_use_compact_control_row() {
        let mut flag = arg("verbose", "--verbose", ArgKind::Flag);
        flag.help = Some("Enable verbose output".to_string());
        let command = command(vec![flag]);
        let visible = visible_args(&command, ActiveTab::Inputs);

        assert_eq!(measure_fields_height(&visible), 3);
        assert_eq!(field_content_bounds(&visible, 0), Some((0, 1)));

        let input_hit = hit_test_form_content(&visible, 0).expect("input hit");
        assert!(matches!(input_hit.widget, FieldWidget::Toggle));
        assert!(input_hit.in_input);
        assert!(input_hit.in_label);

        let description_hit = hit_test_form_content(&visible, 1).expect("description hit");
        assert!(description_hit.in_description);
        assert!(!description_hit.in_input);
    }

    #[test]
    fn hit_testing_offsets_follow_preceding_multiline_field_height() {
        let mut multi = arg("paths", "--path", ArgKind::Option);
        multi.value_cardinality = crate::spec::ValueCardinality::Many;
        multi.help = Some("multiple paths".to_string());
        let mut flag = arg("verbose", "--verbose", ArgKind::Flag);
        flag.help = Some("Enable verbose output".to_string());
        let command = command(vec![multi, flag]);
        let visible = visible_args(&command, ActiveTab::Inputs);

        assert_eq!(field_content_bounds(&visible, 1), Some((7, 8)));

        let second_input = hit_test_form_content(&visible, 7).expect("second field input");
        assert_eq!(second_input.arg_id, "verbose");
        assert!(second_input.in_input);
        assert!(matches!(second_input.widget, FieldWidget::Toggle));
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
        assert_eq!(measure_fields_height(&visible), 14);
        assert_eq!(field_content_bounds(&visible, 0), Some((1, 4)));
        assert_eq!(field_content_bounds(&visible, 2), Some((10, 13)));
    }
}
