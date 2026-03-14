use crate::input::ActiveTab;
use crate::spec::{ArgKind, ArgSpec, CommandSpec};

#[derive(Debug, Clone, Copy)]
pub(crate) struct OrderedArg<'a> {
    pub(crate) order_index: usize,
    pub(crate) arg: &'a ArgSpec,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FieldMetrics {
    pub(crate) label_height: u16,
    pub(crate) input_height: u16,
    pub(crate) gap_height: u16,
    pub(crate) help_height: u16,
    pub(crate) total_height: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FormHit {
    pub(crate) order_index: usize,
    pub(crate) kind: ArgKind,
    pub(crate) arg_id: String,
    pub(crate) in_input: bool,
    pub(crate) in_label: bool,
}

pub(crate) fn ordered_args(command: &CommandSpec) -> Vec<OrderedArg<'_>> {
    let mut positionals = command
        .args
        .iter()
        .filter(|arg| matches!(arg.kind, ArgKind::Positional))
        .filter(|arg| !is_help_arg(arg))
        .collect::<Vec<_>>();
    positionals.sort_by_key(|arg| arg.positional_index.unwrap_or(usize::MAX));

    let mut others = command
        .args
        .iter()
        .filter(|arg| !matches!(arg.kind, ArgKind::Positional))
        .filter(|arg| !is_help_arg(arg))
        .collect::<Vec<_>>();
    others.sort_by_key(|arg| arg.name.clone());

    positionals.extend(others);
    positionals
        .into_iter()
        .enumerate()
        .map(|(order_index, arg)| OrderedArg { order_index, arg })
        .collect()
}

pub(crate) fn visible_args(command: &CommandSpec, active_tab: ActiveTab) -> Vec<OrderedArg<'_>> {
    match active_tab {
        ActiveTab::Options => ordered_args(command)
            .into_iter()
            .filter(|item| !matches!(item.arg.kind, ArgKind::Positional))
            .collect(),
        ActiveTab::Arguments => ordered_args(command)
            .into_iter()
            .filter(|item| matches!(item.arg.kind, ArgKind::Positional))
            .collect(),
        ActiveTab::Help => Vec::new(),
    }
}

pub(crate) fn field_metrics(arg: &ArgSpec) -> FieldMetrics {
    let label_height = 1;
    let input_height = if arg.is_multi { 5 } else { 3 };
    let gap_height = 1;
    let help_height = if arg.help.is_some() || arg.value_hint.is_some() {
        1
    } else {
        0
    };
    FieldMetrics {
        label_height,
        input_height,
        gap_height,
        help_height,
        total_height: label_height + input_height + gap_height + help_height,
    }
}

pub(crate) fn measure_fields_height(args: &[OrderedArg<'_>]) -> u16 {
    args.iter()
        .map(|item| field_metrics(item.arg).total_height)
        .sum()
}

pub(crate) fn measure_help_height(help: &str) -> u16 {
    u16::try_from(help.lines().count()).unwrap_or(u16::MAX)
}

pub(crate) fn field_content_bounds(
    args: &[OrderedArg<'_>],
    selected_index: usize,
) -> Option<(u16, u16)> {
    let mut y: u16 = 0;
    for item in args {
        let metrics = field_metrics(item.arg);
        let input_top = y.saturating_add(metrics.label_height);
        let input_bottom = input_top.saturating_add(metrics.input_height);
        if item.order_index == selected_index {
            return Some((input_top, input_bottom));
        }
        y = y.saturating_add(metrics.total_height);
    }
    None
}

pub(crate) fn hit_test_form_content(args: &[OrderedArg<'_>], content_y: u16) -> Option<FormHit> {
    let mut y: u16 = 0;
    for item in args {
        let metrics = field_metrics(item.arg);
        let input_top = y.saturating_add(metrics.label_height);
        let input_bottom = input_top.saturating_add(metrics.input_height);
        let help_y = input_bottom.saturating_add(metrics.gap_height);

        let in_label = content_y < input_top;
        let in_input = content_y >= input_top && content_y < input_bottom;
        let in_help = metrics.help_height > 0 && content_y == help_y;
        if in_label || in_input || in_help {
            return Some(FormHit {
                order_index: item.order_index,
                kind: item.arg.kind,
                arg_id: item.arg.id.clone(),
                in_input,
                in_label,
            });
        }
        y = y.saturating_add(metrics.total_height);
    }
    None
}

fn is_help_arg(arg: &ArgSpec) -> bool {
    arg.id == "help" || arg.name == "--help" || arg.name == "-h"
}

#[cfg(test)]
mod tests {
    use super::{
        field_content_bounds, field_metrics, hit_test_form_content, measure_fields_height,
        ordered_args, visible_args,
    };
    use crate::input::ActiveTab;
    use crate::spec::{ArgKind, ArgSpec, CommandSpec};

    fn arg(id: &str, name: &str, kind: ArgKind) -> ArgSpec {
        ArgSpec {
            id: id.to_string(),
            name: name.to_string(),
            help: None,
            required: false,
            kind,
            default: None,
            possible_values: Vec::new(),
            positional_index: None,
            is_multi: false,
            value_hint: None,
        }
    }

    fn command(args: Vec<ArgSpec>) -> CommandSpec {
        CommandSpec {
            name: "tool".to_string(),
            about: None,
            help: String::new(),
            args,
            subcommands: Vec::new(),
        }
    }

    #[test]
    fn positional_args_are_ordered_before_options() {
        let mut source = vec![
            arg("verbose", "--verbose", ArgKind::Flag),
            arg("path", "path", ArgKind::Positional),
            arg("alpha", "--alpha", ArgKind::Option),
        ];
        source[1].positional_index = Some(1);
        let command = command(source);

        let ordered = ordered_args(&command);
        let names = ordered
            .into_iter()
            .map(|item| item.arg.name.clone())
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
        positional.positional_index = Some(1);
        let option = arg("target", "--target", ArgKind::Option);
        let command = command(vec![positional, option]);

        assert_eq!(visible_args(&command, ActiveTab::Options).len(), 1);
        assert_eq!(visible_args(&command, ActiveTab::Arguments).len(), 1);
        assert!(visible_args(&command, ActiveTab::Help).is_empty());
    }

    #[test]
    fn field_metrics_match_single_and_multi_line_fields() {
        let single = arg("target", "--target", ArgKind::Option);
        let mut multi = arg("paths", "--path", ArgKind::Option);
        multi.is_multi = true;
        multi.help = Some("paths".to_string());

        assert_eq!(field_metrics(&single).total_height, 5);
        assert_eq!(field_metrics(&multi).total_height, 8);
    }

    #[test]
    fn field_bounds_and_hit_testing_share_same_geometry() {
        let mut positional = arg("path", "path", ArgKind::Positional);
        positional.positional_index = Some(1);
        positional.help = Some("required".to_string());
        let command = command(vec![positional]);
        let visible = visible_args(&command, ActiveTab::Arguments);

        assert_eq!(measure_fields_height(&visible), 6);
        assert_eq!(field_content_bounds(&visible, 0), Some((1, 4)));

        let label_hit = hit_test_form_content(&visible, 0).expect("label hit");
        assert!(label_hit.in_label);
        assert!(!label_hit.in_input);

        let input_hit = hit_test_form_content(&visible, 2).expect("input hit");
        assert!(input_hit.in_input);

        let help_hit = hit_test_form_content(&visible, 5).expect("help hit");
        assert!(!help_hit.in_input);
        assert!(!help_hit.in_label);
    }
}
