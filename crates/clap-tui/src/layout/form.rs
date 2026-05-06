use std::collections::{BTreeMap, HashMap};

use ratatui::layout::Rect;

use crate::pipeline::{FieldInstanceId, FieldSemantics};
use crate::query::form::{self, FieldWidget, OrderedArg};
use crate::repeated_field::project_repeated_field_with_input_height;
use crate::spec::ArgSpec;

pub(crate) const LABEL_COLUMN_MIN_WIDTH: u16 = 12;
pub(crate) const LABEL_COLUMN_MAX_WIDTH: u16 = 24;
pub(crate) const COLUMN_GAP_WIDTH: u16 = 1;
pub(crate) const INPUT_RIGHT_PADDING: u16 = 2;
const INPUT_MIN_WIDTH_BEFORE_RIGHT_PADDING: u16 = 27;

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

#[derive(Debug, Clone)]
pub struct FormFieldLayout {
    pub arg_id: String,
    pub heading: Option<Rect>,
    pub label: Option<Rect>,
    pub input: Rect,
    pub input_clip_top: u16,
    pub description: Option<Rect>,
}

pub(crate) struct FormProjectionInput<'a, 'arg> {
    pub(crate) area: Rect,
    pub(crate) content_area: Rect,
    pub(crate) form_scroll: u16,
    pub(crate) active_args: &'a [OrderedArg<'arg>],
    pub(crate) field_errors: &'a BTreeMap<String, String>,
    pub(crate) input_height_overrides: &'a HashMap<String, u16>,
    pub(crate) label_height_overrides: &'a HashMap<String, u16>,
    pub(crate) field_semantics: &'a BTreeMap<FieldInstanceId, FieldSemantics>,
}

pub(crate) fn preferred_label_column_width_with_semantics(
    args: &[OrderedArg<'_>],
    field_semantics: &BTreeMap<FieldInstanceId, FieldSemantics>,
) -> u16 {
    let widest_label = args
        .iter()
        .map(|item| {
            let required = field_semantics
                .get(&FieldInstanceId::from_arg(item.arg))
                .map_or(item.arg.required, |semantics| semantics.required_badge);
            let required_marker = u16::from(required) * 2;
            let label_width =
                u16::try_from(item.arg.display_label().chars().count()).unwrap_or(u16::MAX);
            label_width.saturating_add(required_marker)
        })
        .max()
        .unwrap_or(LABEL_COLUMN_MIN_WIDTH);

    widest_label.clamp(LABEL_COLUMN_MIN_WIDTH, LABEL_COLUMN_MAX_WIDTH)
}

pub(crate) fn field_metrics(arg: &ArgSpec) -> FieldMetrics {
    field_metrics_with_description(arg, form::field_has_description(arg, None))
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
    field_metrics_with_description_and_layout_overrides(
        arg,
        show_description,
        input_height_override,
        None,
    )
}

pub(crate) fn field_metrics_with_description_and_layout_overrides(
    arg: &ArgSpec,
    show_description: bool,
    input_height_override: Option<u16>,
    label_height_override: Option<u16>,
) -> FieldMetrics {
    let widget = form::widget_for(arg);
    let label_height = label_height_override.unwrap_or(1);
    let description_height = u16::from(show_description);
    let input_height = input_height_override.unwrap_or(match widget {
        FieldWidget::Toggle
        | FieldWidget::SingleChoice
        | FieldWidget::MultiChoice
        | FieldWidget::Counter
        | FieldWidget::SingleText
        | FieldWidget::OptionalValue => 3,
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
    field_input_offset_with_description(arg, form::field_has_description(arg, None))
}

pub(crate) fn field_input_offset_with_description(arg: &ArgSpec, show_description: bool) -> u16 {
    let _ = (arg, show_description);
    0
}

#[cfg(test)]
pub(crate) fn field_description_offset(arg: &ArgSpec) -> Option<u16> {
    field_description_offset_with_description(arg, form::field_has_description(arg, None))
}

#[cfg(test)]
pub(crate) fn field_description_offset_with_description(
    arg: &ArgSpec,
    show_description: bool,
) -> Option<u16> {
    field_description_offset_with_description_and_input_height(arg, show_description, None)
}

#[cfg(test)]
pub(crate) fn field_description_offset_with_description_and_input_height(
    arg: &ArgSpec,
    show_description: bool,
    input_height_override: Option<u16>,
) -> Option<u16> {
    field_description_offset_with_layout_overrides(
        arg,
        show_description,
        input_height_override,
        None,
    )
}

pub(crate) fn field_description_offset_with_layout_overrides(
    arg: &ArgSpec,
    show_description: bool,
    input_height_override: Option<u16>,
    label_height_override: Option<u16>,
) -> Option<u16> {
    let metrics = field_metrics_with_description_and_layout_overrides(
        arg,
        show_description,
        input_height_override,
        label_height_override,
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

#[cfg(test)]
pub(crate) fn measure_fields_height_with_errors(
    args: &[OrderedArg<'_>],
    field_errors: &BTreeMap<String, String>,
) -> u16 {
    measure_fields_height_with_overrides(args, field_errors, &HashMap::new())
}

#[cfg(test)]
pub(crate) fn measure_fields_height_with_overrides(
    args: &[OrderedArg<'_>],
    field_errors: &BTreeMap<String, String>,
    input_height_overrides: &HashMap<String, u16>,
) -> u16 {
    measure_fields_height_with_layout_overrides(
        args,
        field_errors,
        input_height_overrides,
        &HashMap::new(),
    )
}

#[cfg(test)]
pub(crate) fn measure_fields_height_with_layout_overrides(
    args: &[OrderedArg<'_>],
    field_errors: &BTreeMap<String, String>,
    input_height_overrides: &HashMap<String, u16>,
    label_height_overrides: &HashMap<String, u16>,
) -> u16 {
    measure_fields_height_with_layout_overrides_and_semantics(
        args,
        field_errors,
        input_height_overrides,
        label_height_overrides,
        &BTreeMap::new(),
    )
}

pub(crate) fn measure_fields_height_with_layout_overrides_and_semantics(
    args: &[OrderedArg<'_>],
    field_errors: &BTreeMap<String, String>,
    input_height_overrides: &HashMap<String, u16>,
    label_height_overrides: &HashMap<String, u16>,
    field_semantics: &BTreeMap<FieldInstanceId, FieldSemantics>,
) -> u16 {
    let mut total = 0;
    let mut previous_heading = None;
    for item in args {
        if form::field_heading(previous_heading, item).is_some() {
            total += 1;
        }
        let semantic_reason = field_semantics
            .get(&FieldInstanceId::from_arg(item.arg))
            .and_then(|semantics| semantics.reason.as_deref());
        total += field_metrics_with_description_and_layout_overrides(
            item.arg,
            form::field_has_description(
                item.arg,
                field_errors
                    .get(&item.arg.id)
                    .map(String::as_str)
                    .or(semantic_reason),
            ),
            input_height_overrides.get(&item.arg.id).copied(),
            label_height_overrides.get(&item.arg.id).copied(),
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

#[cfg(test)]
pub(crate) fn field_content_bounds_with_errors(
    args: &[OrderedArg<'_>],
    selected_index: usize,
    field_errors: &BTreeMap<String, String>,
) -> Option<(u16, u16)> {
    field_content_bounds_with_layout_overrides(
        args,
        selected_index,
        field_errors,
        &HashMap::new(),
        &HashMap::new(),
    )
}

#[cfg(test)]
pub(crate) fn field_content_bounds_with_layout_overrides(
    args: &[OrderedArg<'_>],
    selected_index: usize,
    field_errors: &BTreeMap<String, String>,
    input_height_overrides: &HashMap<String, u16>,
    label_height_overrides: &HashMap<String, u16>,
) -> Option<(u16, u16)> {
    field_content_bounds_with_layout_overrides_and_semantics(
        args,
        selected_index,
        field_errors,
        input_height_overrides,
        label_height_overrides,
        &BTreeMap::new(),
    )
}

pub(crate) fn field_content_bounds_with_layout_overrides_and_semantics(
    args: &[OrderedArg<'_>],
    selected_index: usize,
    field_errors: &BTreeMap<String, String>,
    input_height_overrides: &HashMap<String, u16>,
    label_height_overrides: &HashMap<String, u16>,
    field_semantics: &BTreeMap<FieldInstanceId, FieldSemantics>,
) -> Option<(u16, u16)> {
    let mut y: u16 = 0;
    let mut previous_heading = None;
    for item in args {
        if form::field_heading(previous_heading, item).is_some() {
            y = y.saturating_add(1);
        }
        let semantic_reason = field_semantics
            .get(&FieldInstanceId::from_arg(item.arg))
            .and_then(|semantics| semantics.reason.as_deref());
        let show_description = form::field_has_description(
            item.arg,
            field_errors
                .get(&item.arg.id)
                .map(String::as_str)
                .or(semantic_reason),
        );
        let metrics = field_metrics_with_description_and_layout_overrides(
            item.arg,
            show_description,
            input_height_overrides.get(&item.arg.id).copied(),
            label_height_overrides.get(&item.arg.id).copied(),
        );
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

#[cfg(test)]
pub(crate) fn hit_test_form_content_with_errors(
    args: &[OrderedArg<'_>],
    content_y: u16,
    field_errors: &BTreeMap<String, String>,
) -> Option<FormHit> {
    hit_test_form_content_with_layout_overrides(
        args,
        content_y,
        field_errors,
        &HashMap::new(),
        &HashMap::new(),
    )
}

#[cfg(test)]
pub(crate) fn hit_test_form_content_with_layout_overrides(
    args: &[OrderedArg<'_>],
    content_y: u16,
    field_errors: &BTreeMap<String, String>,
    input_height_overrides: &HashMap<String, u16>,
    label_height_overrides: &HashMap<String, u16>,
) -> Option<FormHit> {
    hit_test_form_content_with_layout_overrides_and_semantics(
        args,
        content_y,
        field_errors,
        input_height_overrides,
        label_height_overrides,
        &BTreeMap::new(),
    )
}

pub(crate) fn hit_test_form_content_with_layout_overrides_and_semantics(
    args: &[OrderedArg<'_>],
    content_y: u16,
    field_errors: &BTreeMap<String, String>,
    input_height_overrides: &HashMap<String, u16>,
    label_height_overrides: &HashMap<String, u16>,
    field_semantics: &BTreeMap<FieldInstanceId, FieldSemantics>,
) -> Option<FormHit> {
    let mut y: u16 = 0;
    let mut previous_heading = None;
    for item in args {
        if form::field_heading(previous_heading, item).is_some() {
            y = y.saturating_add(1);
        }
        let semantic_reason = field_semantics
            .get(&FieldInstanceId::from_arg(item.arg))
            .and_then(|semantics| semantics.reason.as_deref());
        let show_description = form::field_has_description(
            item.arg,
            field_errors
                .get(&item.arg.id)
                .map(String::as_str)
                .or(semantic_reason),
        );
        let metrics = field_metrics_with_description_and_layout_overrides(
            item.arg,
            show_description,
            input_height_overrides.get(&item.arg.id).copied(),
            label_height_overrides.get(&item.arg.id).copied(),
        );
        let input_top = y.saturating_add(field_input_offset_with_description(
            item.arg,
            show_description,
        ));
        let input_bottom = input_top.saturating_add(metrics.input_height);
        let description_top = field_description_offset_with_layout_overrides(
            item.arg,
            show_description,
            input_height_overrides.get(&item.arg.id).copied(),
            label_height_overrides.get(&item.arg.id).copied(),
        )
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

pub(crate) fn field_content_geometry(
    area: Rect,
    in_section: bool,
    preferred_label_width: u16,
) -> (u16, u16, u16, u16) {
    field_content_geometry_with_right_padding(area, in_section, preferred_label_width, true)
}

fn field_description_geometry(
    area: Rect,
    in_section: bool,
    preferred_label_width: u16,
) -> (u16, u16, u16, u16) {
    field_content_geometry_with_right_padding(area, in_section, preferred_label_width, false)
}

fn field_content_geometry_with_right_padding(
    area: Rect,
    _in_section: bool,
    preferred_label_width: u16,
    reserve_right_padding: bool,
) -> (u16, u16, u16, u16) {
    let content_x = area.x;
    let content_width = area.width;
    let gap = COLUMN_GAP_WIDTH.min(content_width.saturating_sub(1));
    let label_width = preferred_label_width
        .min(content_width.saturating_sub(gap).saturating_sub(8))
        .max(LABEL_COLUMN_MIN_WIDTH);
    let input_x = content_x.saturating_add(label_width).saturating_add(gap);
    let available_input_width = content_width
        .saturating_sub(label_width)
        .saturating_sub(gap);
    let right_padding = if reserve_right_padding {
        INPUT_RIGHT_PADDING
            .min(available_input_width.saturating_sub(INPUT_MIN_WIDTH_BEFORE_RIGHT_PADDING))
    } else {
        0
    };
    let input_width = available_input_width.saturating_sub(right_padding);
    (content_x, label_width, input_x, input_width)
}

#[allow(clippy::too_many_lines)]
pub(crate) fn project_visible_form_fields(
    input: &FormProjectionInput<'_, '_>,
) -> Vec<FormFieldLayout> {
    let preferred_label_width =
        preferred_label_column_width_with_semantics(input.active_args, input.field_semantics);
    let mut fields = Vec::new();
    let mut y = i32::from(input.content_area.y) - i32::from(input.form_scroll);
    let mut previous_heading = None;

    for item in input.active_args {
        let heading = form::field_heading(previous_heading, item);
        let semantic_reason = input
            .field_semantics
            .get(&FieldInstanceId::from_arg(item.arg))
            .and_then(|semantics| semantics.reason.as_deref());
        let show_description = form::field_has_description(
            item.arg,
            input
                .field_errors
                .get(&item.arg.id)
                .map(String::as_str)
                .or(semantic_reason),
        );
        let metrics = field_metrics_with_description_and_layout_overrides(
            item.arg,
            show_description,
            input.input_height_overrides.get(&item.arg.id).copied(),
            input.label_height_overrides.get(&item.arg.id).copied(),
        );
        let heading_height = i32::from(u16::from(heading.is_some()));
        let item_bottom = y + heading_height + i32::from(metrics.total_height);
        if y >= i32::from(input.content_area.y) + i32::from(input.content_area.height) {
            break;
        }
        if item_bottom <= i32::from(input.content_area.y) {
            y += heading_height + i32::from(metrics.total_height);
            previous_heading = item.section_heading.as_deref();
            continue;
        }

        let heading = if heading.is_some() && y >= i32::from(input.content_area.y) {
            let rect = clipped_rect(input.area.x, input.area.width, y, 1, input.content_area);
            y += 1;
            rect
        } else if heading.is_some() {
            y += 1;
            None
        } else {
            None
        };
        let (label_x, label_width, input_x, input_width) = field_content_geometry(
            input.area,
            form::field_is_in_section(item),
            preferred_label_width,
        );
        let label = if metrics.label_height > 0 {
            clipped_rect(
                label_x,
                label_width,
                y,
                metrics.label_height,
                input.content_area,
            )
        } else {
            None
        };
        let repeated_projection = matches!(item.widget, FieldWidget::RepeatedText).then(|| {
            let field_top = u16::try_from(
                (y - i32::from(input.content_area.y) + i32::from(input.form_scroll)).max(0),
            )
            .unwrap_or(u16::MAX);
            project_repeated_field_with_input_height(
                input
                    .input_height_overrides
                    .get(&item.arg.id)
                    .copied()
                    .unwrap_or(metrics.input_height),
                field_top,
                input_x,
                input_width,
                show_description,
                metrics.label_height,
            )
        });
        let description_anchor_visible = description_anchor_visible(y, input.content_area);
        let (input_y, field_input, description) = if let Some(projection) = repeated_projection {
            let input_y = i32::from(input.content_area.y) + i32::from(projection.input.y)
                - i32::from(input.form_scroll);
            let Some(field_input) = clipped_rect(
                projection.input.x,
                projection.input.width,
                input_y,
                projection.input.height,
                input.content_area,
            ) else {
                y += i32::from(metrics.total_height);
                previous_heading = item.section_heading.as_deref();
                continue;
            };
            let description = description_anchor_visible
                .then_some(())
                .and(projection.description)
                .and_then(|description| {
                    let description_y = i32::from(input.content_area.y) + i32::from(description.y)
                        - i32::from(input.form_scroll);
                    let (_, _, description_x, description_width) = field_description_geometry(
                        input.area,
                        form::field_is_in_section(item),
                        preferred_label_width,
                    );
                    clipped_rect(
                        description_x,
                        description_width,
                        description_y,
                        description.height,
                        input.content_area,
                    )
                });
            (input_y, field_input, description)
        } else {
            let input_y = y + i32::from(field_input_offset_with_description(
                item.arg,
                show_description,
            ));
            let Some(field_input) = clipped_rect(
                input_x,
                input_width,
                input_y,
                metrics.input_height,
                input.content_area,
            ) else {
                y += i32::from(metrics.total_height);
                previous_heading = item.section_heading.as_deref();
                continue;
            };
            let description = form_description_rect(
                item,
                y,
                input.area,
                input.content_area,
                description_anchor_visible,
                show_description,
                preferred_label_width,
                input.input_height_overrides.get(&item.arg.id).copied(),
                input.label_height_overrides.get(&item.arg.id).copied(),
            );
            (input_y, field_input, description)
        };

        let input_clip_top =
            u16::try_from((i32::from(field_input.y) - input_y).max(0)).unwrap_or(u16::MAX);
        fields.push(FormFieldLayout {
            arg_id: item.arg.id.clone(),
            heading,
            label,
            input: field_input,
            input_clip_top,
            description,
        });

        y += i32::from(metrics.total_height);
        previous_heading = item.section_heading.as_deref();
    }

    fields
}

#[allow(clippy::too_many_arguments)]
fn form_description_rect(
    item: &OrderedArg<'_>,
    y: i32,
    area: Rect,
    content_area: Rect,
    description_anchor_visible: bool,
    show_description: bool,
    preferred_label_width: u16,
    input_height_override: Option<u16>,
    label_height_override: Option<u16>,
) -> Option<Rect> {
    description_anchor_visible.then_some(())?;
    show_description.then_some(())?;
    let description_y = y + i32::from(field_description_offset_with_layout_overrides(
        item.arg,
        show_description,
        input_height_override,
        label_height_override,
    )?);
    let (_, _, input_x, input_width) =
        field_description_geometry(area, form::field_is_in_section(item), preferred_label_width);
    clipped_rect(
        input_x,
        input_width,
        description_y,
        field_metrics_with_description_and_layout_overrides(
            item.arg,
            show_description,
            input_height_override,
            label_height_override,
        )
        .description_height
        .max(1),
        content_area,
    )
}

fn description_anchor_visible(field_top_y: i32, content_area: Rect) -> bool {
    field_top_y >= i32::from(content_area.y)
}

fn intersect_rects(rect: Rect, bounds: Rect) -> Option<Rect> {
    let left = rect.x.max(bounds.x);
    let top = rect.y.max(bounds.y);
    let right = rect
        .x
        .saturating_add(rect.width)
        .min(bounds.x.saturating_add(bounds.width));
    let bottom = rect
        .y
        .saturating_add(rect.height)
        .min(bounds.y.saturating_add(bounds.height));

    if left >= right || top >= bottom {
        return None;
    }

    Some(Rect::new(
        left,
        top,
        right.saturating_sub(left),
        bottom.saturating_sub(top),
    ))
}

fn clipped_rect(x: u16, width: u16, top: i32, height: u16, bounds: Rect) -> Option<Rect> {
    let bounded_top = top.max(i32::from(bounds.y));
    let bounded_bottom = top
        .saturating_add(i32::from(height))
        .min(i32::from(bounds.y.saturating_add(bounds.height)));
    if bounded_top >= bounded_bottom {
        return None;
    }

    let y = u16::try_from(bounded_top).ok()?;
    let clipped_height = u16::try_from(bounded_bottom.saturating_sub(bounded_top)).ok()?;
    intersect_rects(Rect::new(x, y, width, clipped_height), bounds)
}

#[cfg(test)]
mod tests {
    use clap::Command;

    use super::{
        FieldWidget, INPUT_RIGHT_PADDING, field_content_bounds, field_content_geometry,
        field_description_offset, field_input_offset, field_metrics, hit_test_form_content,
        measure_fields_height,
    };
    use crate::input::ActiveTab;
    use crate::query::form::{visible_args, widget_for};
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
    fn field_metrics_match_single_and_multi_line_fields() {
        let single = arg("target", "--target", ArgKind::Option);
        let mut multi = arg("paths", "--path", ArgKind::Option);
        multi.value_cardinality = crate::spec::ValueCardinality::Many;
        multi.help = Some("paths".to_string());
        let mut flag = arg("verbose", "--verbose", ArgKind::Flag);
        flag.help = Some("toggle".to_string());

        assert_eq!(field_metrics(&single).total_height, 4);
        assert_eq!(field_metrics(&multi).total_height, 7);
        assert_eq!(field_metrics(&flag).total_height, 5);
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
        assert_eq!(field_metrics(version).total_height, 5);
        assert_eq!(field_input_offset(version), 0);
        assert_eq!(field_description_offset(version), Some(3));
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

        assert_eq!(measure_fields_height(&visible), 5);
        assert_eq!(field_content_bounds(&visible, 0), Some((0, 3)));

        let input_hit = hit_test_form_content(&visible, 0).expect("input hit");
        assert!(matches!(input_hit.widget, FieldWidget::Toggle));
        assert!(input_hit.in_input);
        assert!(input_hit.in_label);

        let description_hit = hit_test_form_content(&visible, 3).expect("description hit");
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

        assert_eq!(field_content_bounds(&visible, 1), Some((7, 10)));

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

        assert_eq!(
            crate::query::form::field_heading(None, &visible[0]),
            Some("Inputs")
        );
        assert_eq!(
            crate::query::form::field_heading(Some("Inputs"), &visible[1]),
            None
        );
        assert_eq!(
            crate::query::form::field_heading(Some("Inputs"), &visible[2]),
            Some("Outputs")
        );
        assert_eq!(measure_fields_height(&visible), 14);
        assert_eq!(field_content_bounds(&visible, 0), Some((1, 4)));
        assert_eq!(field_content_bounds(&visible, 2), Some((10, 13)));
    }

    #[test]
    fn field_geometry_reserves_right_padding_after_input_column() {
        let area = ratatui::layout::Rect::new(0, 0, 44, 10);
        let (_, _, input_x, input_width) = field_content_geometry(area, false, 12);

        assert_eq!(
            input_x.saturating_add(input_width),
            area.width - INPUT_RIGHT_PADDING
        );
    }

    #[test]
    fn narrow_field_geometry_keeps_input_width_before_right_padding() {
        let area = ratatui::layout::Rect::new(0, 0, 20, 10);
        let (_, _, _, input_width) = field_content_geometry(area, false, 12);

        assert_eq!(input_width, 7);
    }
}
