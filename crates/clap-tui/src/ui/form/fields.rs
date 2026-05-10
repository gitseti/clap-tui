use std::borrow::Cow;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph, Widget};

use crate::config::TuiConfig;
use crate::input::{ArgInput, ArgInputState, ArgValue, Focus, InputSource, UiState};
use crate::layout::form::FormFieldLayout;
use crate::pipeline::{EffectiveArgValue, EffectiveValueSource};
use crate::query::form::FieldWidget;
use crate::spec::{ArgSpec, choice_value_matches_default};

use super::{compact, help, optional_value, repeated, styles, text};
use crate::ui::screen::ScreenView;

#[derive(Debug)]
#[allow(clippy::struct_excessive_bools)]
pub(super) struct FieldRenderModel<'a> {
    pub(super) arg: &'a ArgSpec,
    pub(super) widget: FieldWidget,
    pub(super) value: Cow<'a, str>,
    pub(super) current_input: Option<&'a ArgInputState>,
    pub(super) effective_value: Option<&'a EffectiveArgValue>,
    pub(super) selected: bool,
    pub(super) field_error: Option<&'a str>,
    pub(super) is_primary_invalid: bool,
    pub(super) source_badge: Option<EffectiveValueSource>,
    pub(super) required: bool,
    pub(super) semantic_reason: Option<&'a str>,
    pub(super) shows_choice_placeholder: bool,
    pub(super) is_default: bool,
    pub(super) uses_muted_non_user_value: bool,
    pub(super) dropdown_open: bool,
    pub(super) block: Block<'static>,
    pub(super) fill_style: Style,
    pub(super) text_style: Style,
}

pub(super) fn render_fields<'a>(
    buffer: &mut Buffer,
    ui: &UiState,
    config: &TuiConfig,
    vm: &'a ScreenView<'a>,
    fields: &[FormFieldLayout],
    first_invalid_field_id: Option<&str>,
) -> Option<(u16, u16)> {
    let mut cursor = None;
    for field in fields {
        let Some(item) = vm
            .active_args
            .iter()
            .find(|item| item.arg.id == field.arg_id)
        else {
            continue;
        };
        let model = FieldRenderModel::new(item, ui, config, vm, first_invalid_field_id);

        render_heading(buffer, config, field, item.section_heading.as_deref());
        render_label(buffer, config, field, &model);
        cursor = render_clipped_widget(buffer, ui, config, field, &model).or(cursor);
        help::render_field_help(
            buffer,
            config,
            field.description,
            vm.root,
            &vm.selected_path,
            &model,
        );
    }
    cursor
}

fn render_clipped_widget(
    buffer: &mut Buffer,
    ui: &UiState,
    config: &TuiConfig,
    field: &FormFieldLayout,
    model: &FieldRenderModel<'_>,
) -> Option<(u16, u16)> {
    if field.input.width == 0 || field.input.height == 0 {
        return None;
    }

    if matches!(model.widget, FieldWidget::RepeatedText) {
        return repeated::render_repeated_text_field(buffer, ui, field, config, model);
    }

    let unclipped = field.input_clip_top == 0 && field.input.height == field.full_input_height;
    if unclipped {
        return dispatch_fixed_widget(buffer, ui, config, field.input, model);
    }

    let full_area = Rect::new(0, 0, field.input.width, field.full_input_height);
    let mut local_buffer = Buffer::empty(full_area);
    let cursor = dispatch_fixed_widget(&mut local_buffer, ui, config, full_area, model);
    super::blit(
        buffer,
        &local_buffer,
        (0, field.input_clip_top),
        (field.input.x, field.input.y),
        (field.input.width, field.input.height),
    );
    cursor.and_then(|cursor| clipped_cursor_position(cursor, field))
}

fn clipped_cursor_position(cursor: (u16, u16), field: &FormFieldLayout) -> Option<(u16, u16)> {
    let (x, y) = cursor;
    if x >= field.input.width || y >= field.full_input_height {
        return None;
    }
    if y < field.input_clip_top || y >= field.input_clip_top.saturating_add(field.input.height) {
        return None;
    }
    Some((
        field.input.x.saturating_add(x),
        field
            .input
            .y
            .saturating_add(y.saturating_sub(field.input_clip_top)),
    ))
}

impl<'a> FieldRenderModel<'a> {
    fn new(
        item: &crate::query::form::OrderedArg<'a>,
        ui: &UiState,
        config: &TuiConfig,
        vm: &'a ScreenView<'a>,
        first_invalid_field_id: Option<&str>,
    ) -> Self {
        let selected = item.order_index == ui.selected_arg_index && matches!(ui.focus, Focus::Form);
        let field_error = vm
            .validation
            .field_errors
            .get(&item.arg.id)
            .map(String::as_str);
        let is_primary_invalid = first_invalid_field_id == Some(item.arg.id.as_str());
        let source_badge = effective_source_badge(vm, item.arg);
        let required = vm.field_required(item.arg);
        let can_edit = vm.field_can_edit(item.arg);
        let semantic_reason = vm
            .field_semantics(item.arg)
            .and_then(|semantics| semantics.reason.as_deref());
        let current_input = vm
            .inputs
            .as_ref()
            .and_then(|inputs| inputs.input(&item.arg.id));
        let current_value = effective_compatibility_value(vm, item.arg);
        let selected_values = effective_selected_values(vm, item.arg);
        let value = field_display_value(
            item.widget,
            item.arg,
            current_value.as_ref(),
            &selected_values,
            current_input,
            vm.effective_values.get(&item.arg.id),
        );
        let shows_choice_placeholder = matches!(
            item.widget,
            FieldWidget::SingleChoice | FieldWidget::MultiChoice
        ) && value.is_empty();
        let is_touched = vm
            .inputs
            .as_ref()
            .is_some_and(|inputs| inputs.is_touched(&item.arg.id));
        let is_default = value_matches_default(item.arg, current_value.as_ref(), is_touched);
        let uses_muted_non_user_value = !value.is_empty()
            && !is_touched
            && source_badge.is_some_and(|source| source != EffectiveValueSource::User);
        let shows_text_placeholder = value.is_empty()
            && matches!(
                item.widget,
                FieldWidget::SingleText | FieldWidget::RepeatedText
            );
        let shows_passive_toggle =
            matches!(item.widget, FieldWidget::Toggle) && !is_touched && value == "[ ]";
        let fill_style = styles::input(config, selected);
        let text_style = text_style(
            config,
            RenderTone {
                can_edit,
                shows_choice_placeholder,
                required,
                uses_muted_non_user_value,
                is_default,
                shows_text_placeholder,
                shows_passive_toggle,
            },
        );
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(field_border_style(
                config,
                selected,
                field_error.is_some(),
                is_primary_invalid,
            ));

        Self {
            arg: item.arg,
            widget: item.widget,
            value,
            current_input,
            effective_value: vm.effective_values.get(&item.arg.id),
            selected,
            field_error,
            is_primary_invalid,
            source_badge,
            required,
            semantic_reason,
            shows_choice_placeholder,
            is_default,
            uses_muted_non_user_value,
            dropdown_open: matches!(
                item.widget,
                FieldWidget::SingleChoice | FieldWidget::MultiChoice
            ) && ui.dropdown_open.as_deref() == Some(&item.arg.id),
            block,
            fill_style,
            text_style,
        }
    }
}

fn field_display_value<'a>(
    widget: FieldWidget,
    arg: &ArgSpec,
    current_value: Option<&ArgValue>,
    selected_values: &[String],
    current_input: Option<&'a ArgInputState>,
    effective_value: Option<&'a EffectiveArgValue>,
) -> Cow<'a, str> {
    let display = display_value(widget, current_value, selected_values);
    let source_display = display_value_from_sources(widget, arg, current_input, effective_value);
    if source_display.as_ref() == display.as_ref() {
        source_display
    } else {
        Cow::Owned(display.into_owned())
    }
}

#[derive(Debug, Clone, Copy)]
#[allow(clippy::struct_excessive_bools)]
struct RenderTone {
    can_edit: bool,
    shows_choice_placeholder: bool,
    required: bool,
    uses_muted_non_user_value: bool,
    is_default: bool,
    shows_text_placeholder: bool,
    shows_passive_toggle: bool,
}

fn text_style(config: &TuiConfig, tone: RenderTone) -> Style {
    if !tone.can_edit {
        styles::placeholder(config)
    } else if tone.shows_choice_placeholder && tone.required {
        styles::required_prompt(config)
    } else if tone.uses_muted_non_user_value
        || tone.is_default
        || tone.shows_choice_placeholder
        || tone.shows_text_placeholder
        || tone.shows_passive_toggle
    {
        styles::placeholder(config)
    } else {
        Style::default().fg(config.theme.text)
    }
}

fn render_heading(
    buffer: &mut Buffer,
    config: &TuiConfig,
    field: &FormFieldLayout,
    heading: Option<&str>,
) {
    if let Some(heading_rect) = field.heading
        && let Some(heading) = heading
    {
        Paragraph::new(help::section_heading_line(
            config,
            heading,
            heading_rect.width,
        ))
        .render(heading_rect, buffer);
    }
}

fn render_label(
    buffer: &mut Buffer,
    config: &TuiConfig,
    field: &FormFieldLayout,
    model: &FieldRenderModel<'_>,
) {
    let Some(label_rect) = field.label else {
        return;
    };
    let label_style = if model.field_error.is_some() {
        if model.is_primary_invalid {
            styles::label(config, model.selected)
                .fg(config.theme.error)
                .add_modifier(Modifier::BOLD)
        } else {
            styles::label(config, model.selected).fg(config.theme.error)
        }
    } else {
        styles::label(config, model.selected)
    };
    let mut spans = vec![Span::styled(
        model.arg.display_label().to_string(),
        label_style,
    )];
    if model.required {
        spans.push(Span::raw(" "));
        spans.push(Span::styled("*", styles::required_prompt(config)));
    }
    Paragraph::new(Line::from(spans)).render(label_rect, buffer);
}

fn dispatch_fixed_widget(
    buffer: &mut Buffer,
    ui: &UiState,
    config: &TuiConfig,
    area: Rect,
    model: &FieldRenderModel<'_>,
) -> Option<(u16, u16)> {
    match model.widget {
        FieldWidget::Toggle => {
            compact::render_flag_toggle(buffer, config, area, model);
            None
        }
        FieldWidget::SingleChoice | FieldWidget::MultiChoice | FieldWidget::Counter => {
            compact::render_compact_control(buffer, config, area, model);
            None
        }
        FieldWidget::OptionalValue => {
            optional_value::render_optional_value(buffer, ui, area, config, model)
        }
        FieldWidget::SingleText => text::render_text_field(buffer, ui, area, config, model),
        FieldWidget::RepeatedText => {
            unreachable!("RepeatedText is dispatched directly to render_repeated_text_field")
        }
    }
}

pub(super) fn display_value<'a>(
    widget: FieldWidget,
    current_value: Option<&'a ArgValue>,
    selected_values: &'a [String],
) -> Cow<'a, str> {
    match widget {
        FieldWidget::Toggle => match current_value {
            Some(ArgValue::Bool(true)) => Cow::Borrowed("[x]"),
            _ => Cow::Borrowed("[ ]"),
        },
        FieldWidget::Counter => match current_value {
            Some(ArgValue::Text(text)) => Cow::Borrowed(text.as_str()),
            _ => Cow::Borrowed("0"),
        },
        FieldWidget::SingleChoice => match current_value {
            Some(ArgValue::Choice(value) | ArgValue::Text(value)) => Cow::Borrowed(value.as_str()),
            _ => Cow::Borrowed(""),
        },
        FieldWidget::MultiChoice => match selected_values {
            [] => Cow::Borrowed(""),
            [single] => Cow::Borrowed(single.as_str()),
            many => Cow::Owned(format!("{} selected", many.len())),
        },
        FieldWidget::SingleText | FieldWidget::RepeatedText | FieldWidget::OptionalValue => {
            match current_value {
                Some(ArgValue::Text(text)) => Cow::Borrowed(text.as_str()),
                _ => Cow::Borrowed(""),
            }
        }
    }
}

fn display_value_from_sources<'a>(
    widget: FieldWidget,
    arg: &ArgSpec,
    current_input: Option<&'a ArgInputState>,
    effective_value: Option<&'a EffectiveArgValue>,
) -> Cow<'a, str> {
    if let Some(input) = current_input
        && let Some(value) = input_display_value(widget, arg, input)
    {
        return value;
    }

    let Some(effective_value) =
        effective_value.filter(|value| value.source != EffectiveValueSource::User)
    else {
        return match widget {
            FieldWidget::Toggle => Cow::Borrowed("[ ]"),
            FieldWidget::Counter => Cow::Borrowed("0"),
            _ => Cow::Borrowed(""),
        };
    };
    if effective_value.values.is_empty() {
        return Cow::Borrowed("");
    }

    match widget {
        FieldWidget::Toggle => Cow::Borrowed("[ ]"),
        FieldWidget::Counter
        | FieldWidget::SingleText
        | FieldWidget::RepeatedText
        | FieldWidget::OptionalValue => render_values(arg, &effective_value.values),
        FieldWidget::SingleChoice => effective_value
            .values
            .first()
            .map_or(Cow::Borrowed(""), |value| Cow::Borrowed(value.as_str())),
        FieldWidget::MultiChoice => selected_values_display(effective_value.values.iter()),
    }
}

fn input_display_value<'a>(
    widget: FieldWidget,
    arg: &ArgSpec,
    input: &'a ArgInputState,
) -> Option<Cow<'a, str>> {
    match (&input.value, widget) {
        (ArgInput::Flag { present, .. }, FieldWidget::Toggle) => Some(if *present {
            Cow::Borrowed("[x]")
        } else {
            Cow::Borrowed("[ ]")
        }),
        (ArgInput::Flag { .. }, FieldWidget::OptionalValue) => Some(Cow::Borrowed("")),
        (ArgInput::Count { occurrences, .. }, FieldWidget::Counter) => {
            Some(Cow::Owned(occurrences.to_string()))
        }
        (ArgInput::Values { occurrences }, FieldWidget::SingleChoice) => occurrences
            .iter()
            .flat_map(|occurrence| occurrence.values.iter())
            .find(|value| !value.is_empty())
            .map(|value| Cow::Borrowed(value.as_str()))
            .or(Some(Cow::Borrowed(""))),
        (ArgInput::Values { occurrences }, FieldWidget::MultiChoice) => {
            Some(selected_values_display(
                occurrences
                    .iter()
                    .flat_map(|occurrence| occurrence.values.iter()),
            ))
        }
        (
            ArgInput::Values { occurrences },
            FieldWidget::SingleText | FieldWidget::RepeatedText | FieldWidget::OptionalValue,
        ) => Some(render_occurrences(arg, occurrences)),
        _ => None,
    }
}

fn selected_values_display<'a>(values: impl Iterator<Item = &'a String>) -> Cow<'a, str> {
    let mut first = None;
    let mut count = 0usize;
    for value in values.filter(|value| !value.is_empty()) {
        count += 1;
        if first.is_none() {
            first = Some(value.as_str());
        }
        if count > 1 {
            return Cow::Owned(format!("{count} selected"));
        }
    }

    first.map_or(Cow::Borrowed(""), Cow::Borrowed)
}

fn render_occurrences<'a>(
    arg: &ArgSpec,
    occurrences: &'a [crate::input::InputValueOccurrence],
) -> Cow<'a, str> {
    let mut borrowed = None;
    let mut parts = Vec::new();
    for occurrence in occurrences {
        let rendered = render_values(arg, &occurrence.values);
        if rendered.is_empty() {
            continue;
        }
        match rendered {
            Cow::Borrowed(value) if borrowed.is_none() && parts.is_empty() => {
                borrowed = Some(value);
            }
            Cow::Borrowed(value) => {
                if let Some(previous) = borrowed.take() {
                    parts.push(previous.to_string());
                }
                parts.push(value.to_string());
            }
            Cow::Owned(value) => {
                if let Some(previous) = borrowed.take() {
                    parts.push(previous.to_string());
                }
                parts.push(value);
            }
        }
    }

    match (borrowed, parts.as_slice()) {
        (Some(value), []) => Cow::Borrowed(value),
        (None, []) => Cow::Borrowed(""),
        _ => Cow::Owned(parts.join("\n")),
    }
}

fn render_values<'a>(arg: &ArgSpec, values: &'a [String]) -> Cow<'a, str> {
    match values {
        [] => Cow::Borrowed(""),
        [single] => Cow::Borrowed(single.as_str()),
        many => Cow::Owned(crate::input::render_occurrence_text(arg, many)),
    }
}

fn field_border_style(
    config: &TuiConfig,
    selected: bool,
    has_error: bool,
    is_primary_invalid: bool,
) -> Style {
    if has_error && (selected || is_primary_invalid) {
        Style::default().fg(config.theme.error)
    } else if has_error {
        styles::field_border(config, false, true)
    } else if selected {
        styles::field_border(config, true, false)
    } else {
        styles::field_border(config, false, false)
    }
}

fn value_matches_default(arg: &ArgSpec, value: Option<&ArgValue>, is_touched: bool) -> bool {
    match value {
        Some(ArgValue::Choice(value)) => !is_touched && choice_value_matches_default(arg, value),
        Some(ArgValue::Text(text)) => !is_touched && arg.default_value() == Some(text.as_str()),
        Some(ArgValue::Bool(enabled)) => {
            !is_touched
                && matches!(
                    (arg.default_value(), *enabled),
                    (Some("true"), true) | (Some("false"), false)
                )
        }
        None => false,
    }
}

fn effective_source_badge(vm: &ScreenView<'_>, arg: &ArgSpec) -> Option<EffectiveValueSource> {
    if let Some(source) = vm.effective_values.get(&arg.id).map(|value| value.source) {
        return Some(source);
    }

    vm.inputs
        .as_ref()
        .filter(|inputs| !inputs.is_touched(&arg.id))
        .and_then(|inputs| inputs.input_source(&arg.id))
        .map(|source| match source {
            InputSource::User => EffectiveValueSource::User,
            InputSource::Default => EffectiveValueSource::Default,
            InputSource::Env => EffectiveValueSource::Env,
        })
}

pub(super) fn effective_compatibility_value(
    vm: &ScreenView<'_>,
    arg: &ArgSpec,
) -> Option<ArgValue> {
    let input_value = vm
        .inputs
        .as_ref()
        .and_then(|inputs| inputs.compatibility_value(arg));
    if input_value.is_some() {
        return input_value;
    }

    let effective_value = vm.effective_values.get(&arg.id)?;
    if effective_value.source == EffectiveValueSource::User {
        return None;
    }
    if effective_value.values.is_empty() {
        return None;
    }

    if arg.uses_optional_value_semantics() {
        return Some(ArgValue::Text(render_effective_value(
            arg,
            &effective_value.values,
        )));
    }

    if arg.has_value_choices() && !arg.is_multi_value_input() {
        return effective_value
            .values
            .first()
            .cloned()
            .map(ArgValue::Choice);
    }

    Some(ArgValue::Text(render_effective_value(
        arg,
        &effective_value.values,
    )))
}

pub(super) fn effective_selected_values(vm: &ScreenView<'_>, arg: &ArgSpec) -> Vec<String> {
    let selected_values = vm
        .inputs
        .as_ref()
        .map_or_else(Vec::new, |inputs| inputs.selected_values(arg));
    if !selected_values.is_empty() {
        return selected_values;
    }

    vm.effective_values
        .get(&arg.id)
        .filter(|value| value.source != EffectiveValueSource::User)
        .map(|value| value.values.clone())
        .unwrap_or_default()
}

pub(super) fn render_effective_value(arg: &ArgSpec, values: &[String]) -> String {
    if let Some(delimiter) = arg.metadata.syntax.value_delimiter {
        values.join(&delimiter.to_string())
    } else if arg.accepts_multiple_values_per_occurrence() || arg.allows_multiple_occurrences() {
        values.join("\n")
    } else {
        values.first().cloned().unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use super::{display_value, display_value_from_sources};
    use crate::input::{ArgInput, ArgInputState, ArgValue, InputSource, InputValueOccurrence};
    use crate::pipeline::{EffectiveArgValue, EffectiveValueSource};
    use crate::query::form::FieldWidget;
    use crate::spec::{ArgKind, ArgSpec, ValueCardinality};

    fn option_arg() -> ArgSpec {
        ArgSpec {
            id: "target".to_string(),
            display_name: "--target".to_string(),
            kind: ArgKind::Option,
            value_cardinality: ValueCardinality::One,
            ..ArgSpec::default()
        }
    }

    #[test]
    fn display_value_borrows_existing_text_when_possible() {
        let value = ArgValue::Text("target/debug".to_string());

        assert!(matches!(
            display_value(FieldWidget::SingleText, Some(&value), &[]),
            Cow::Borrowed("target/debug")
        ));
    }

    #[test]
    fn display_value_allocates_only_for_multi_choice_summary() {
        let selected = vec!["red".to_string(), "blue".to_string()];

        assert!(matches!(
            display_value(FieldWidget::MultiChoice, None, &selected),
            Cow::Owned(summary) if summary == "2 selected"
        ));
    }

    #[test]
    fn display_projection_borrows_input_text_when_possible() {
        let arg = option_arg();
        let input = ArgInputState {
            value: ArgInput::Values {
                occurrences: vec![InputValueOccurrence {
                    values: vec!["target/debug".to_string()],
                    source: InputSource::User,
                }],
            },
            touched: true,
        };

        assert!(matches!(
            display_value_from_sources(FieldWidget::SingleText, &arg, Some(&input), None),
            Cow::Borrowed("target/debug")
        ));
    }

    #[test]
    fn display_projection_borrows_effective_single_value_when_possible() {
        let arg = option_arg();
        let effective = EffectiveArgValue {
            source: EffectiveValueSource::Default,
            values: vec!["release".to_string()],
        };

        assert!(matches!(
            display_value_from_sources(FieldWidget::SingleText, &arg, None, Some(&effective)),
            Cow::Borrowed("release")
        ));
    }
}
