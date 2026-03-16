use crate::input::ActiveTab;
use crate::spec::{ArgSpec, CommandSpec};

#[derive(Debug, Clone, Copy)]
pub(crate) struct OrderedArg<'a> {
    pub(crate) order_index: usize,
    pub(crate) arg: &'a ArgSpec,
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
    pub(crate) is_flag: bool,
    pub(crate) uses_choice_input: bool,
    pub(crate) accepts_text_input: bool,
    pub(crate) in_input: bool,
    pub(crate) in_label: bool,
    pub(crate) in_description: bool,
}

pub(crate) fn ordered_args(command: &CommandSpec) -> Vec<OrderedArg<'_>> {
    let mut positionals = command
        .args
        .iter()
        .filter(|arg| arg.is_positional())
        .filter(|arg| !is_help_arg(arg))
        .collect::<Vec<_>>();
    positionals.sort_by_key(|arg| arg.position.unwrap_or(usize::MAX));

    let mut others = command
        .args
        .iter()
        .filter(|arg| !arg.is_positional())
        .filter(|arg| !is_help_arg(arg))
        .collect::<Vec<_>>();
    others.sort_by_key(|arg| arg.display_name.clone());

    positionals.extend(others);
    positionals
        .into_iter()
        .enumerate()
        .map(|(order_index, arg)| OrderedArg { order_index, arg })
        .collect()
}

pub(crate) fn visible_args(command: &CommandSpec, active_tab: ActiveTab) -> Vec<OrderedArg<'_>> {
    match active_tab {
        ActiveTab::Inputs => ordered_args(command),
    }
}

pub(crate) fn field_metrics(arg: &ArgSpec) -> FieldMetrics {
    let label_height = u16::from(!arg.is_flag());
    let description_height = u16::from(arg.help.is_some() || arg.value_hint.is_some());
    let input_height = if arg.is_flag() || arg.uses_choice_input() {
        1
    } else if arg.accepts_multiple_values() {
        5
    } else {
        3
    };
    let gap_height = 1;
    FieldMetrics {
        label_height,
        description_height,
        input_height,
        gap_height,
        total_height: label_height + description_height + input_height + gap_height,
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
        let description_top = input_bottom;

        let in_label = metrics.label_height > 0 && content_y >= y && content_y < input_top;
        let in_description = metrics.description_height > 0
            && content_y >= description_top
            && content_y < description_top.saturating_add(metrics.description_height);
        let in_input = content_y >= input_top && content_y < input_bottom;
        if in_label || in_description || in_input {
            return Some(FormHit {
                order_index: item.order_index,
                arg_id: item.arg.id.clone(),
                is_flag: item.arg.is_flag(),
                uses_choice_input: item.arg.uses_choice_input(),
                accepts_text_input: item.arg.accepts_text_input(),
                in_input,
                in_label,
                in_description,
            });
        }
        y = y.saturating_add(metrics.total_height);
    }
    None
}

fn is_help_arg(arg: &ArgSpec) -> bool {
    arg.id == "help" || arg.display_name == "--help" || arg.display_name == "-h"
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
            display_name: name.to_string(),
            help: None,
            required: false,
            kind,
            default_values: Vec::new(),
            choices: Vec::new(),
            position: None,
            value_cardinality: crate::spec::ValueCardinality::One,
            value_hint: None,
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
    fn field_metrics_match_single_and_multi_line_fields() {
        let single = arg("target", "--target", ArgKind::Option);
        let mut multi = arg("paths", "--path", ArgKind::Option);
        multi.value_cardinality = crate::spec::ValueCardinality::Many;
        multi.help = Some("paths".to_string());
        let mut flag = arg("verbose", "--verbose", ArgKind::Flag);
        flag.help = Some("toggle".to_string());

        assert_eq!(field_metrics(&single).total_height, 5);
        assert_eq!(field_metrics(&multi).total_height, 8);
        assert_eq!(field_metrics(&flag).total_height, 3);
    }

    #[test]
    fn field_bounds_and_hit_testing_share_same_geometry() {
        let mut positional = arg("path", "path", ArgKind::Positional);
        positional.position = Some(1);
        positional.help = Some("required".to_string());
        let command = command(vec![positional]);
        let visible = visible_args(&command, ActiveTab::Inputs);

        assert_eq!(measure_fields_height(&visible), 6);
        assert_eq!(field_content_bounds(&visible, 0), Some((1, 4)));

        let label_hit = hit_test_form_content(&visible, 0).expect("label hit");
        assert!(label_hit.in_label);
        assert!(!label_hit.in_input);
        assert!(!label_hit.in_description);

        let input_hit = hit_test_form_content(&visible, 1).expect("input hit");
        assert!(input_hit.accepts_text_input);
        assert!(input_hit.in_input);
        assert!(!input_hit.in_label);
        assert!(!input_hit.in_description);

        let description_hit = hit_test_form_content(&visible, 4).expect("description hit");
        assert!(description_hit.in_description);
        assert!(!description_hit.in_label);
        assert!(!description_hit.in_input);

        assert!(hit_test_form_content(&visible, 5).is_none());
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
        assert!(input_hit.is_flag);
        assert!(input_hit.in_input);
        assert!(!input_hit.in_label);

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

        assert_eq!(field_content_bounds(&visible, 1), Some((8, 9)));

        let second_input = hit_test_form_content(&visible, 8).expect("second field input");
        assert_eq!(second_input.arg_id, "verbose");
        assert!(second_input.in_input);
        assert!(second_input.is_flag);
    }
}
