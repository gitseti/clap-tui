use std::collections::HashMap;

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, BorderType, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
};

use crate::config::TuiConfig;
use crate::form_editor;
use crate::frame_snapshot::FrameSnapshot;
use crate::input::{ArgInput, ArgInputState, ArgValue, Focus, InputSource, UiState};
use crate::pipeline::{EffectiveArgValue, EffectiveValueSource};
use crate::query::form::{self, FieldWidget, field_metrics};
use crate::repeated_field::{
    project_repeated_field, repeated_add_rect, repeated_input_height, repeated_remove_rect,
    repeated_row_textarea_rect,
};
use crate::spec::{ArgSpec, choice_value_matches_default, format_command_path};

use super::{screen::ScreenView, styles};

const REPEATED_CONTROL_REMOVE: &str = " - ";
const REPEATED_CONTROL_ADD: &str = " + ";

#[allow(dead_code)]
pub(crate) fn populate_layout(
    ui: &UiState,
    area: Rect,
    vm: &ScreenView<'_>,
    frame_snapshot: &mut FrameSnapshot,
) {
    let input_height_overrides = repeated_input_height_overrides(ui, vm);
    let label_height_overrides = label_height_overrides(vm);
    crate::frame_snapshot::populate_form_layout(
        ui,
        area,
        &vm.active_args,
        &vm.command.help,
        &vm.validation,
        &vm.field_semantics,
        &input_height_overrides,
        &label_height_overrides,
        frame_snapshot,
    );
}

pub(crate) fn render_form(
    frame: &mut Frame<'_>,
    ui: &UiState,
    config: &TuiConfig,
    vm: &ScreenView<'_>,
    frame_snapshot: &FrameSnapshot,
) {
    let Some(area) = frame_snapshot.layout.form else {
        return;
    };
    let frame_layout = &frame_snapshot.layout;
    let content_area = frame_layout.form_view.unwrap_or(area);
    let input_height_overrides = repeated_input_height_overrides(ui, vm);
    let label_height_overrides = label_height_overrides(vm);
    let content_height = form::measure_fields_height_with_layout_overrides_and_semantics(
        &vm.active_args,
        &vm.validation.field_errors,
        &input_height_overrides,
        &label_height_overrides,
        &vm.field_semantics,
    );
    let viewport_height = content_area.height;
    let form_scroll = ui.form_scroll(frame_snapshot);

    if ui.help_open {
        render_help_overlay(
            frame,
            config,
            area,
            ui.help_scroll(frame_snapshot),
            &vm.command.help,
        );
        return;
    }

    render_fields(frame, ui, config, vm, frame_snapshot);

    if content_height > viewport_height {
        let scroll_steps = usize::from(frame_snapshot.form_scroll_max.saturating_add(1));
        let mut scrollbar_state = ScrollbarState::new(scroll_steps)
            .position(usize::from(form_scroll))
            .viewport_content_length(usize::from(viewport_height));
        let scrollbar_area = Rect::new(
            content_area.x,
            content_area.y,
            content_area.width.saturating_add(1),
            content_area.height,
        );
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .track_symbol(Some("┃"))
            .thumb_symbol("█")
            .begin_style(styles::scrollbar_cap(
                config,
                matches!(ui.focus, Focus::Form),
            ))
            .end_style(styles::scrollbar_cap(
                config,
                matches!(ui.focus, Focus::Form),
            ))
            .thumb_style(styles::scrollbar_thumb(
                config,
                matches!(ui.focus, Focus::Form),
            ))
            .track_style(styles::scrollbar_track(config));
        frame.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
    }
}

fn repeated_input_height_overrides(ui: &UiState, vm: &ScreenView<'_>) -> HashMap<String, u16> {
    vm.active_args
        .iter()
        .filter(|item| matches!(item.widget, FieldWidget::RepeatedText))
        .map(|item| {
            let current_value = effective_compatibility_value(vm, item.arg);
            let selected_values = effective_selected_values(vm, item.arg);
            let value = display_value(item.widget, current_value.as_ref(), &selected_values);
            (
                item.arg.id.clone(),
                repeated_input_height(ui, item.arg, &value),
            )
        })
        .collect()
}

fn label_height_overrides(_: &ScreenView<'_>) -> HashMap<String, u16> {
    HashMap::new()
}

#[allow(clippy::too_many_lines)]
fn render_fields(
    frame: &mut Frame<'_>,
    ui: &UiState,
    config: &TuiConfig,
    vm: &ScreenView<'_>,
    frame_snapshot: &FrameSnapshot,
) {
    for field in &frame_snapshot.layout.form_fields {
        let Some(item) = vm
            .active_args
            .iter()
            .find(|item| item.arg.id == field.arg_id)
        else {
            continue;
        };
        let selected = item.order_index == ui.selected_arg_index && matches!(ui.focus, Focus::Form);
        let field_error = vm.validation.field_errors.get(&item.arg.id);
        let is_primary_invalid =
            frame_snapshot.first_invalid_field_id() == Some(item.arg.id.as_str());
        let source_badge = effective_source_badge(vm, item.arg);
        let required_badge = vm.field_required(item.arg);
        let can_edit = vm.field_can_edit(item.arg);
        let semantic_reason = vm
            .field_semantics(item.arg)
            .and_then(|semantics| semantics.reason.as_deref());

        if let Some(heading_rect) = field.heading
            && let Some(heading) = item.section_heading.as_deref()
        {
            frame.render_widget(
                Paragraph::new(section_heading_line(config, heading, heading_rect.width)),
                heading_rect,
            );
        }

        if let Some(label_rect) = field.label {
            let label_style = if field_error.is_some() {
                if is_primary_invalid {
                    styles::label(config, selected)
                        .fg(config.theme.error)
                        .add_modifier(Modifier::BOLD)
                } else {
                    styles::label(config, selected).fg(config.theme.error)
                }
            } else {
                styles::label(config, selected)
            };
            let mut spans = vec![Span::styled(
                item.arg.display_label().to_string(),
                label_style,
            )];
            if required_badge {
                spans.push(Span::raw(" "));
                spans.push(Span::styled("*", styles::required_prompt(config)));
            }
            frame.render_widget(Paragraph::new(Line::from(spans)), label_rect);
        }

        let current_value = effective_compatibility_value(vm, item.arg);
        let selected_values = effective_selected_values(vm, item.arg);
        let value = display_value(item.widget, current_value.as_ref(), &selected_values);
        let expected_input_height = expected_input_height(ui, item.arg, item.widget, &value);
        let input_is_truncated = field.input.height < expected_input_height;
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

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(field_border_style(
                config,
                selected,
                field_error.is_some(),
                is_primary_invalid,
            ));
        let fill_style = styles::input(config, selected);
        let text_style = if !can_edit {
            styles::placeholder(config)
        } else if shows_choice_placeholder && required_badge {
            styles::required_prompt(config)
        } else if uses_muted_non_user_value
            || is_default
            || shows_choice_placeholder
            || shows_text_placeholder
            || shows_passive_toggle
        {
            styles::placeholder(config)
        } else {
            Style::default().fg(config.theme.text)
        };

        if matches!(item.widget, FieldWidget::Toggle) {
            render_flag_toggle(frame, config, field.input, selected, value == "[x]", block);
        } else if matches!(
            item.widget,
            FieldWidget::SingleChoice | FieldWidget::MultiChoice | FieldWidget::Counter
        ) {
            let display = if value.is_empty() {
                compact_placeholder(item.arg, item.widget, required_badge)
            } else {
                value.as_str()
            };
            render_compact_control(
                frame,
                config,
                field.input,
                item.widget,
                display,
                selected,
                uses_muted_non_user_value || is_default || shows_choice_placeholder,
                matches!(
                    item.widget,
                    FieldWidget::SingleChoice | FieldWidget::MultiChoice
                ) && ui.dropdown_open.as_deref() == Some(&item.arg.id),
                block,
            );
        } else if matches!(item.widget, FieldWidget::OptionalValue) {
            render_optional_value(
                frame,
                ui,
                item.arg,
                selected,
                field.input,
                config,
                vm.inputs
                    .as_ref()
                    .and_then(|inputs| inputs.input(&item.arg.id)),
                current_value.as_ref(),
                &value,
                source_badge,
                vm.effective_values.get(&item.arg.id),
                block,
                text_style,
            );
        } else if matches!(item.widget, FieldWidget::RepeatedText) {
            render_repeated_text_field(
                frame,
                ui,
                item.arg,
                &value,
                field.input,
                field.input_clip_top,
                config,
                block,
                text_style,
                selected,
                field_error.is_none(),
                required_badge,
            );
        } else if input_is_truncated {
            frame.render_widget(
                Paragraph::new(repeated_display_lines(
                    item.arg,
                    item.widget,
                    &value,
                    required_badge,
                ))
                .style(fill_style.patch(text_style)),
                field.input,
            );
        } else if selected {
            let editor =
                form_editor::editor_for_render(ui, item.arg.owner_path(), item.arg, &value);
            let mut textarea = editor.to_textarea(editor.selection_anchor());
            textarea.set_block(block.style(styles::input(config, true)));
            let base_style = Style::default()
                .fg(text_style.fg.unwrap_or(config.theme.text))
                .bg(config.theme.surface_raised);
            textarea.set_style(base_style);
            textarea.set_cursor_line_style(base_style);
            textarea.set_cursor_style(
                Style::default()
                    .bg(config.theme.accent)
                    .add_modifier(Modifier::BOLD),
            );
            textarea.set_selection_style(
                Style::default()
                    .fg(config.theme.text)
                    .add_modifier(Modifier::REVERSED),
            );
            if value.is_empty()
                && field_error.is_none()
                && let Some(placeholder) =
                    required_empty_prompt(item.arg, item.widget, required_badge)
            {
                textarea.set_placeholder_text(placeholder);
                textarea.set_placeholder_style(styles::placeholder(config));
            }
            frame.render_widget(textarea.widget(), field.input);
            place_textarea_cursor(frame, &textarea, field.input);
        } else {
            frame.render_widget(
                Paragraph::new(repeated_display_lines(
                    item.arg,
                    item.widget,
                    &value,
                    required_badge,
                ))
                .block(block)
                .style(fill_style.patch(text_style)),
                field.input,
            );
        }
        if let (Some(help), Some(help_rect)) = (
            field_help_text(
                vm.root,
                item.arg,
                item.widget,
                &vm.selected_path,
                FieldHelpContext {
                    selected,
                    field_error: field_error.map(String::as_str),
                    effective_value: vm.effective_values.get(&item.arg.id),
                    semantic_reason,
                },
            ),
            field.description,
        ) {
            frame.render_widget(
                Paragraph::new(Line::from(Span::raw(help))).style(if field_error.is_some() {
                    Style::default().fg(config.theme.error)
                } else {
                    styles::help(config)
                }),
                help_rect,
            );
        }
    }
}

fn display_value(
    widget: FieldWidget,
    current_value: Option<&ArgValue>,
    selected_values: &[String],
) -> String {
    match widget {
        FieldWidget::Toggle => match current_value {
            Some(ArgValue::Bool(true)) => "[x]".to_string(),
            _ => "[ ]".to_string(),
        },
        FieldWidget::Counter => match current_value {
            Some(ArgValue::Text(text)) => text.clone(),
            _ => "0".to_string(),
        },
        FieldWidget::SingleChoice => match current_value {
            Some(ArgValue::Choice(value)) => value.clone(),
            Some(ArgValue::Text(text)) => text.clone(),
            _ => String::new(),
        },
        FieldWidget::MultiChoice => match selected_values {
            [] => String::new(),
            [single] => single.clone(),
            many => format!("{} selected", many.len()),
        },
        FieldWidget::SingleText | FieldWidget::RepeatedText | FieldWidget::OptionalValue => {
            match current_value {
                Some(ArgValue::Text(text)) => text.clone(),
                _ => String::new(),
            }
        }
    }
}

fn compact_placeholder(_arg: &ArgSpec, widget: FieldWidget, required: bool) -> &'static str {
    match widget {
        FieldWidget::Counter => "0",
        FieldWidget::MultiChoice if required => "Press Space to choose",
        FieldWidget::SingleChoice if required => "Press Enter to choose",
        _ => "Select...",
    }
}

fn expected_input_height(ui: &UiState, arg: &ArgSpec, widget: FieldWidget, value: &str) -> u16 {
    match widget {
        FieldWidget::RepeatedText => repeated_input_height(ui, arg, value),
        _ => field_metrics(arg).input_height,
    }
}

fn repeated_display_lines(
    arg: &ArgSpec,
    widget: FieldWidget,
    value: &str,
    required: bool,
) -> Vec<Line<'static>> {
    if value.is_empty() {
        return required_empty_prompt(arg, widget, required)
            .map_or_else(Vec::new, |placeholder| vec![Line::from(placeholder)]);
    }

    value
        .lines()
        .map(|line| Line::from(line.to_string()))
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn render_repeated_text_field(
    frame: &mut Frame<'_>,
    ui: &UiState,
    arg: &ArgSpec,
    value: &str,
    area: Rect,
    input_clip_top: u16,
    config: &TuiConfig,
    _block: Block<'_>,
    text_style: Style,
    selected: bool,
    show_placeholder: bool,
    required: bool,
) {
    let editor = form_editor::editor_for_render(ui, arg.owner_path(), arg, value);
    let total_rows = editor.row_count().max(1);
    let current_row = editor.current_row();
    if total_rows <= 1 {
        let textarea_rect = repeated_row_textarea_rect(area, true, true);
        let placeholder = (show_placeholder && value.is_empty())
            .then(|| required_empty_prompt(arg, FieldWidget::RepeatedText, required))
            .flatten();
        let cursor_col =
            (current_row == 0).then_some(u16::try_from(editor.cursor().col).unwrap_or(u16::MAX));
        render_repeated_row_textarea(
            frame,
            config,
            textarea_rect,
            value,
            placeholder,
            text_style,
            selected,
            selected,
            cursor_col,
        );
        render_repeated_row_controls(config, frame, area, selected, false, true, true);
        return;
    }
    let projection = project_repeated_field(ui, arg, value, 0, area.x, area.width, false, 1);
    let visible_rows =
        usize::from(area.height / crate::repeated_field::REPEATED_ROW_HEIGHT).min(total_rows);
    if visible_rows == 0 {
        return;
    }
    let start_row = repeated_visible_start_row(visible_rows, projection.rows.len(), input_clip_top);

    for visible_index in 0..visible_rows {
        let row_index = start_row + visible_index;
        let Some(row) = projection.row(row_index) else {
            continue;
        };
        let row_rect = Rect::new(
            area.x,
            area.y.saturating_add(
                u16::try_from(visible_index)
                    .unwrap_or(u16::MAX)
                    .saturating_mul(crate::repeated_field::REPEATED_ROW_HEIGHT),
            ),
            row.width,
            row.height,
        );
        let is_last_row = row_index + 1 == total_rows;
        let textarea_rect = repeated_row_textarea_rect(row_rect, true, is_last_row);
        let active_row = selected && row_index == current_row;
        let line = editor.lines().get(row_index).cloned().unwrap_or_default();
        let placeholder = (show_placeholder && row_index == 0 && line.is_empty())
            .then(|| required_empty_prompt(arg, FieldWidget::RepeatedText, required))
            .flatten();

        render_repeated_row_textarea(
            frame,
            config,
            textarea_rect,
            line.as_str(),
            placeholder,
            text_style,
            active_row,
            active_row && selected,
            active_row.then_some(u16::try_from(editor.cursor().col).unwrap_or(u16::MAX)),
        );

        render_repeated_row_controls(
            config,
            frame,
            row_rect,
            active_row,
            total_rows > 1,
            true,
            is_last_row,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn render_repeated_row_textarea(
    frame: &mut Frame<'_>,
    config: &TuiConfig,
    area: Rect,
    value: &str,
    placeholder: Option<String>,
    text_style: Style,
    selected_row: bool,
    place_cursor: bool,
    cursor_col: Option<u16>,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(styles::field_border(config, selected_row, false));
    if selected_row {
        let mut textarea = tui_textarea::TextArea::new(vec![value.to_string()]);
        textarea.set_block(block.style(styles::input(config, true)));
        let base_style = Style::default()
            .fg(text_style.fg.unwrap_or(config.theme.text))
            .bg(config.theme.surface_raised);
        textarea.set_style(base_style);
        textarea.set_cursor_line_style(base_style);
        textarea.set_cursor_style(
            Style::default()
                .bg(config.theme.accent)
                .add_modifier(Modifier::BOLD),
        );
        textarea.set_selection_style(
            Style::default()
                .fg(config.theme.text)
                .add_modifier(Modifier::REVERSED),
        );
        if let Some(placeholder) = placeholder {
            textarea.set_placeholder_text(placeholder);
            textarea.set_placeholder_style(styles::placeholder(config));
        }
        if let Some(cursor_col) = cursor_col {
            textarea.move_cursor(tui_textarea::CursorMove::Jump(0, cursor_col));
        }
        frame.render_widget(textarea.widget(), area);
        if place_cursor {
            place_textarea_cursor(frame, &textarea, area);
        }
    } else {
        frame.render_widget(
            Paragraph::new(if value.is_empty() {
                placeholder.unwrap_or_default()
            } else {
                value.to_string()
            })
            .block(block.style(styles::input(config, false)))
            .style(if value.is_empty() {
                styles::placeholder(config)
            } else {
                text_style
            }),
            area,
        );
    }
}

#[allow(clippy::fn_params_excessive_bools)]
fn render_repeated_row_controls(
    config: &TuiConfig,
    frame: &mut Frame<'_>,
    row_rect: Rect,
    active: bool,
    can_remove: bool,
    show_remove: bool,
    show_add: bool,
) {
    if show_remove && let Some(remove_rect) = repeated_remove_rect(row_rect, show_remove, show_add)
    {
        frame.render_widget(
            Paragraph::new(REPEATED_CONTROL_REMOVE).style(styles::compact_control_affordance(
                config, active, can_remove,
            )),
            remove_rect,
        );
    }
    if show_add && let Some(add_rect) = repeated_add_rect(row_rect) {
        frame.render_widget(
            Paragraph::new(REPEATED_CONTROL_ADD)
                .style(styles::compact_control_affordance(config, active, true)),
            add_rect,
        );
    }
}

fn repeated_visible_start_row(
    visible_rows: usize,
    total_rows: usize,
    input_clip_top: u16,
) -> usize {
    let clipped_rows = usize::from(
        input_clip_top.saturating_add(crate::repeated_field::REPEATED_ROW_HEIGHT.saturating_sub(1))
            / crate::repeated_field::REPEATED_ROW_HEIGHT,
    );
    clipped_rows.min(total_rows.saturating_sub(visible_rows.min(total_rows)))
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

#[allow(clippy::too_many_arguments)]
fn render_optional_value(
    frame: &mut Frame<'_>,
    ui: &UiState,
    arg: &ArgSpec,
    selected: bool,
    area: Rect,
    config: &TuiConfig,
    current_input: Option<&ArgInputState>,
    current_value: Option<&ArgValue>,
    value: &str,
    source: Option<EffectiveValueSource>,
    effective_value: Option<&EffectiveArgValue>,
    block: Block<'_>,
    text_style: Style,
) {
    match optional_value_visual_state(current_input, current_value, value, source, effective_value)
    {
        OptionalValueVisualState::Explicit if selected => {
            render_textarea_field(frame, ui, arg, value, None, area, config, block, text_style);
        }
        OptionalValueVisualState::Explicit => {
            frame.render_widget(
                Paragraph::new(value.to_string())
                    .block(block.style(styles::input(config, selected)))
                    .style(text_style),
                area,
            );
        }
        OptionalValueVisualState::Present { detail } if selected => {
            render_textarea_field(
                frame,
                ui,
                arg,
                "",
                Some(format!("Present · {detail}")),
                area,
                config,
                block,
                text_style,
            );
        }
        OptionalValueVisualState::Present { detail } => {
            frame.render_widget(
                Paragraph::new(Line::from(vec![
                    Span::styled(" Present ", styles::checkbox_chip(config, selected, true)),
                    Span::raw(" "),
                    Span::styled(detail, styles::placeholder(config)),
                ]))
                .block(block.style(styles::input(config, selected)))
                .style(Style::default()),
                area,
            );
        }
        OptionalValueVisualState::Off { detail } if selected => {
            render_textarea_field(
                frame,
                ui,
                arg,
                "",
                Some(format!("Off · {detail}")),
                area,
                config,
                block,
                text_style,
            );
        }
        OptionalValueVisualState::Off { detail } => {
            frame.render_widget(
                Paragraph::new(Line::from(vec![
                    Span::styled(" Off ", styles::checkbox_chip(config, selected, false)),
                    Span::raw(" "),
                    Span::styled(detail, styles::placeholder(config)),
                ]))
                .block(block.style(styles::input(config, selected)))
                .style(Style::default()),
                area,
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn render_textarea_field(
    frame: &mut Frame<'_>,
    ui: &UiState,
    arg: &ArgSpec,
    value: &str,
    placeholder: Option<String>,
    area: Rect,
    config: &TuiConfig,
    block: Block<'_>,
    text_style: Style,
) {
    let editor = form_editor::editor_for_render(ui, arg.owner_path(), arg, value);
    let mut textarea = editor.to_textarea(editor.selection_anchor());
    textarea.set_block(block.style(styles::input(config, true)));
    let base_style = Style::default()
        .fg(text_style.fg.unwrap_or(config.theme.text))
        .bg(config.theme.surface_raised);
    textarea.set_style(base_style);
    textarea.set_cursor_line_style(base_style);
    textarea.set_cursor_style(
        Style::default()
            .bg(config.theme.accent)
            .add_modifier(Modifier::BOLD),
    );
    textarea.set_selection_style(
        Style::default()
            .fg(config.theme.text)
            .add_modifier(Modifier::REVERSED),
    );
    if let Some(placeholder) = placeholder {
        textarea.set_placeholder_text(placeholder);
        textarea.set_placeholder_style(styles::placeholder(config));
    }
    frame.render_widget(textarea.widget(), area);
    place_textarea_cursor(frame, &textarea, area);
}

enum OptionalValueVisualState {
    Explicit,
    Present { detail: String },
    Off { detail: String },
}

fn optional_value_visual_state(
    current_input: Option<&ArgInputState>,
    current_value: Option<&ArgValue>,
    value: &str,
    source: Option<EffectiveValueSource>,
    effective_value: Option<&EffectiveArgValue>,
) -> OptionalValueVisualState {
    if let Some(input) = current_input {
        match &input.value {
            ArgInput::Flag { present: true, .. } => {
                return OptionalValueVisualState::Present {
                    detail: present_detail(effective_value),
                };
            }
            ArgInput::Values { occurrences }
                if occurrences
                    .iter()
                    .any(|occurrence| occurrence.values.iter().any(|entry| !entry.is_empty())) =>
            {
                let input_source = input.input_source().map(optional_input_source_label);
                if input.touched || matches!(input.input_source(), Some(InputSource::User)) {
                    return OptionalValueVisualState::Explicit;
                }
                return OptionalValueVisualState::Off {
                    detail: off_detail(input_source, value),
                };
            }
            _ => {}
        }
    }

    match current_value {
        Some(ArgValue::Bool(true)) => OptionalValueVisualState::Present {
            detail: present_detail(effective_value),
        },
        Some(ArgValue::Text(_)) if !value.is_empty() => OptionalValueVisualState::Off {
            detail: off_detail(source.map(optional_effective_source_label), value),
        },
        _ => OptionalValueVisualState::Off {
            detail: "Right/Space enables".to_string(),
        },
    }
}

fn present_detail(effective_value: Option<&EffectiveArgValue>) -> String {
    effective_value
        .filter(|effective| effective.source == EffectiveValueSource::DefaultMissing)
        .filter(|effective| !effective.values.is_empty())
        .map_or_else(
            || "bare flag, type to add a value".to_string(),
            |effective| format!("bare flag, implicit: {}", effective.values.join(" ")),
        )
}

fn off_detail(source: Option<&'static str>, value: &str) -> String {
    match (source, value.is_empty()) {
        (Some(source), false) => format!("{source}: {value}"),
        (None, false) => format!("effective: {value}"),
        _ => "Right/Space enables".to_string(),
    }
}

fn optional_input_source_label(source: InputSource) -> &'static str {
    match source {
        InputSource::User => "value",
        InputSource::Default => "default",
        InputSource::Env => "env",
    }
}

fn optional_effective_source_label(source: EffectiveValueSource) -> &'static str {
    match source {
        EffectiveValueSource::User => "value",
        EffectiveValueSource::Default => "default",
        EffectiveValueSource::Env => "env",
        EffectiveValueSource::DefaultMissing => "default-missing",
        EffectiveValueSource::ConditionalDefault => "conditional",
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

#[allow(clippy::too_many_arguments)]
fn render_flag_toggle(
    frame: &mut Frame<'_>,
    config: &TuiConfig,
    area: Rect,
    selected: bool,
    enabled: bool,
    block: Block<'_>,
) {
    let spans = vec![
        Span::styled(
            if enabled { " [✓] " } else { " [ ] " },
            styles::checkbox_chip(config, selected, enabled),
        ),
        Span::raw(" "),
        Span::styled(
            if enabled { "Enabled" } else { "Disabled" },
            styles::compact_control_value(config, selected, !enabled),
        ),
    ];
    let line = Line::from(spans);
    frame.render_widget(
        Paragraph::new(line)
            .block(block.style(styles::input(config, selected)))
            .style(styles::flag_toggle(config, selected)),
        area,
    );
}

#[allow(clippy::too_many_arguments)]
fn render_compact_control(
    frame: &mut Frame<'_>,
    config: &TuiConfig,
    area: Rect,
    widget: FieldWidget,
    value: &str,
    selected: bool,
    is_default: bool,
    open: bool,
    block: Block<'_>,
) {
    let input = Paragraph::new(compact_control_line(
        config,
        widget,
        value,
        area.width.saturating_sub(2),
        selected,
        is_default,
        open,
    ))
    .block(block.style(styles::input(config, selected)))
    .style(styles::compact_control(config, selected));
    frame.render_widget(input, area);
}

fn compact_control_line(
    config: &TuiConfig,
    widget: FieldWidget,
    value: &str,
    inner_width: u16,
    selected: bool,
    is_default: bool,
    open: bool,
) -> Line<'static> {
    let value_style = styles::compact_control_value(config, selected, is_default);
    let affordance = match widget {
        FieldWidget::Counter => " -  + ",
        _ if open => " ^ ",
        _ => " v ",
    };
    let affordance_width = u16::try_from(affordance.chars().count()).unwrap_or(6);
    let available_value = inner_width.saturating_sub(affordance_width + 1);
    let value_width = u16::try_from(value.chars().count()).unwrap_or(available_value);
    let padding = available_value.saturating_sub(value_width.saturating_add(1));
    if matches!(widget, FieldWidget::Counter) {
        return Line::from(vec![
            Span::raw(" "),
            Span::styled(value.to_string(), value_style),
            Span::raw(" ".repeat(usize::from(padding))),
            Span::styled(
                " - ",
                styles::compact_control_affordance(config, selected, true),
            ),
            Span::raw(" "),
            Span::styled(
                " + ",
                styles::compact_control_affordance(config, selected, true),
            ),
        ]);
    }
    Line::from(vec![
        Span::raw(" "),
        Span::styled(value.to_string(), value_style),
        Span::raw(" ".repeat(usize::from(padding))),
        Span::styled(
            affordance,
            styles::compact_control_affordance(config, selected, open),
        ),
    ])
}

fn place_textarea_cursor(frame: &mut Frame<'_>, textarea: &tui_textarea::TextArea<'_>, area: Rect) {
    if area.width < 3 || area.height < 3 {
        return;
    }
    let (row, col) = textarea.cursor();
    let inner_x = area.x.saturating_add(1);
    let inner_y = area.y.saturating_add(1);
    let inner_w = area.width.saturating_sub(2);
    let inner_h = area.height.saturating_sub(2);
    if inner_w == 0 || inner_h == 0 {
        return;
    }
    let x = inner_x
        .saturating_add(u16::try_from(col).unwrap_or(inner_w.saturating_sub(1)))
        .min(inner_x + inner_w - 1);
    let y = inner_y
        .saturating_add(u16::try_from(row).unwrap_or(inner_h.saturating_sub(1)))
        .min(inner_y + inner_h - 1);
    frame.set_cursor_position((x, y));
}

fn render_help_overlay(
    frame: &mut Frame<'_>,
    config: &TuiConfig,
    area: Rect,
    scroll: u16,
    help: &str,
) {
    let popup = crate::frame_snapshot::help_overlay_popup_rect(area);
    let block = Block::default()
        .title(Line::from(vec![
            Span::raw(" "),
            Span::styled("Help", styles::preview_title(config)),
            Span::raw(" "),
        ]))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(styles::panel_border(config, true))
        .style(styles::overlay_panel(config, true));
    let inner = crate::frame_snapshot::help_overlay_content_rect(area);
    frame.render_widget(block, popup);
    frame.render_widget(
        Paragraph::new(help.to_string())
            .style(Style::default().fg(config.theme.text))
            .scroll((scroll, 0)),
        inner,
    );

    let visible_height = inner.height;
    if usize::from(visible_height) < help.lines().count() {
        let steps = usize::from(
            u16::try_from(help.lines().count())
                .unwrap_or(u16::MAX)
                .saturating_sub(visible_height),
        )
        .saturating_add(1);
        let mut scrollbar_state = ScrollbarState::new(steps)
            .position(usize::from(scroll))
            .viewport_content_length(usize::from(visible_height));
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .track_symbol(Some("┃"))
            .thumb_symbol("█")
            .begin_style(styles::scrollbar_cap(config, true))
            .end_style(styles::scrollbar_cap(config, true))
            .thumb_style(styles::scrollbar_thumb(config, true))
            .track_style(styles::scrollbar_track(config));
        frame.render_stateful_widget(scrollbar, inner, &mut scrollbar_state);
    }
}

#[derive(Debug, Clone, Copy)]
struct FieldHelpContext<'a> {
    selected: bool,
    field_error: Option<&'a str>,
    effective_value: Option<&'a EffectiveArgValue>,
    semantic_reason: Option<&'a str>,
}

fn field_help_text(
    root: &crate::spec::CommandSpec,
    arg: &ArgSpec,
    widget: FieldWidget,
    selected_path: &crate::spec::CommandPath,
    help: FieldHelpContext<'_>,
) -> Option<String> {
    if let Some(field_error) = help.field_error {
        return Some(field_error.to_string());
    }

    let mut parts = Vec::new();
    if let Some(reason) = help.semantic_reason {
        parts.push(reason.to_string());
    }
    let primary_help = if help.selected {
        arg.long_help()
            .filter(|long_help| Some(*long_help) != arg.help.as_deref())
            .map(str::to_string)
            .or_else(|| arg.help.clone())
            .or_else(|| arg.value_hint.clone())
    } else {
        arg.help.clone().or_else(|| arg.value_hint.clone())
    };
    if let Some(help) = primary_help {
        parts.push(help);
    }
    if let Some(effective_value) = help.effective_value {
        match effective_value.source {
            EffectiveValueSource::DefaultMissing if !effective_value.values.is_empty() => parts
                .push(format!(
                    "Implicit value: {}",
                    render_effective_value(arg, &effective_value.values)
                )),
            EffectiveValueSource::ConditionalDefault => {
                parts.push("Value is default-derived under the current conditions.".to_string());
            }
            _ => {}
        }
    }
    if help.selected && arg.is_inherited_for(selected_path) && !selected_path.is_empty() {
        parts.push(format!(
            "Defined on {}. Editing here updates that shared option for commands in this lineage.",
            format_command_path(&root.name, arg.owner_path())
        ));
    }
    if help.selected
        && let Some(hint) = widget_help_hint(widget)
    {
        parts.push(hint.to_string());
    }

    (!parts.is_empty()).then(|| parts.join("  "))
}

fn section_heading_line(config: &TuiConfig, heading: &str, width: u16) -> Line<'static> {
    let title = format!(" {heading} ");
    let title_width = u16::try_from(title.chars().count()).unwrap_or(width);
    if width <= title_width.saturating_add(2) {
        return Line::from(Span::styled(
            heading.to_string(),
            Style::default()
                .fg(config.theme.result_accent)
                .add_modifier(Modifier::BOLD),
        ));
    }

    let line_width = width.saturating_sub(title_width);
    Line::from(vec![
        Span::styled(
            title,
            Style::default()
                .fg(config.theme.result_accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "─".repeat(usize::from(line_width)),
            Style::default().fg(config.theme.divider),
        ),
    ])
}

fn widget_help_hint(widget: FieldWidget) -> Option<&'static str> {
    match widget {
        FieldWidget::RepeatedText => Some(
            "Enter moves or adds a line. Up/Down switches lines. Backspace removes an empty line. Alt+Up/Down reorders.",
        ),
        FieldWidget::MultiChoice => Some("Space toggles choices. Enter finishes selection."),
        FieldWidget::Counter => Some("Right/+ increments. Left/- decrements."),
        FieldWidget::OptionalValue => Some("Right enables. Left/Delete disables."),
        _ => None,
    }
}

fn required_empty_prompt(arg: &ArgSpec, widget: FieldWidget, required: bool) -> Option<String> {
    let _ = arg;
    if !required {
        return None;
    }

    Some(match widget {
        FieldWidget::RepeatedText | FieldWidget::SingleText => {
            "Enter a value to continue".to_string()
        }
        FieldWidget::SingleChoice => "Press Enter to choose a value".to_string(),
        FieldWidget::MultiChoice => "Press Space to choose at least one value".to_string(),
        _ => return None,
    })
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

fn effective_compatibility_value(vm: &ScreenView<'_>, arg: &ArgSpec) -> Option<ArgValue> {
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

fn effective_selected_values(vm: &ScreenView<'_>, arg: &ArgSpec) -> Vec<String> {
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

fn render_effective_value(arg: &ArgSpec, values: &[String]) -> String {
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
    use clap::{Arg, ArgAction, Command};
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui::style::Modifier;

    use super::{
        FieldHelpContext, FieldWidget, compact_control_line, field_help_text, populate_layout,
        repeated_add_rect, repeated_remove_rect, repeated_row_textarea_rect,
    };
    use crate::TuiConfig;
    use crate::frame_snapshot::{FormFieldLayout, FrameSnapshot};
    use crate::input::{ActiveTab, AppState, Focus, UiState};
    use crate::query::form::{visible_args, visible_args_for_path, widget_for};
    use crate::spec::{ArgKind, ArgSpec, CommandSpec, ValueCardinality};
    use crate::ui::form::render_form;
    use crate::ui::screen::ScreenView;
    use ratatui::layout::Rect;

    fn command() -> CommandSpec {
        CommandSpec {
            name: "tool".to_string(),
            version: None,
            about: None,
            help: String::new(),
            args: Vec::new(),
            subcommands: Vec::new(),
            ..CommandSpec::default()
        }
    }

    fn ui_state() -> UiState {
        UiState {
            focus: Focus::Form,
            active_tab: ActiveTab::Inputs,
            last_non_help_tab: ActiveTab::Inputs,
            help_open: false,
            help_scroll: 0,
            selected_arg_index: 0,
            search_query: String::new(),
            editors: crate::editor_state::EditorState::default(),
            dropdown_open: None,
            dropdown_scroll: 0,
            dropdown_cursor: 0,
            sidebar_scroll: 0,
            form_scroll: 0,
            hover: None,
            hover_tab: None,
            mouse_select: None,
        }
    }

    fn option_arg(id: &str, name: &str) -> ArgSpec {
        ArgSpec {
            id: id.to_string(),
            display_name: name.to_string(),
            help: None,
            required: false,
            kind: ArgKind::Option,
            default_values: Vec::new(),
            choices: Vec::new(),
            position: None,
            value_cardinality: ValueCardinality::One,
            value_hint: None,
            ..ArgSpec::default()
        }
    }

    fn choice_arg(id: &str, name: &str, choices: &[&str]) -> ArgSpec {
        let mut arg = option_arg(id, name);
        arg.choices = choices.iter().map(|choice| (*choice).to_string()).collect();
        arg
    }

    fn buffer_text(backend: &TestBackend) -> String {
        backend
            .buffer()
            .content
            .iter()
            .map(ratatui::buffer::Cell::symbol)
            .collect::<String>()
    }

    fn cell_bg(backend: &TestBackend, x: u16, y: u16) -> ratatui::style::Color {
        backend.buffer()[(x, y)].bg
    }

    fn cell_fg(backend: &TestBackend, x: u16, y: u16) -> ratatui::style::Color {
        backend.buffer()[(x, y)].fg
    }

    #[test]
    fn layout_phase_has_no_tab_geometry_for_single_inputs_view() {
        let command = command();
        let vm = ScreenView {
            command: &command,
            root: &command,
            selected_path: crate::spec::CommandPath::default(),
            tree_rows: Vec::new(),
            sidebar_scroll: 0,
            active_args: Vec::new(),
            authoritative_argv: Vec::new(),
            rendered_command: None,
            validation: crate::pipeline::ValidationState::default(),
            effective_values: std::collections::BTreeMap::new(),
            field_semantics: std::collections::BTreeMap::new(),
            inputs: None,
        };
        let mut snapshot = FrameSnapshot::default();

        populate_layout(
            &ui_state(),
            ratatui::layout::Rect::new(2, 3, 40, 12),
            &vm,
            &mut snapshot,
        );

        assert!(snapshot.layout.form_tabs.is_empty());
        assert_eq!(
            snapshot.layout.form_view,
            Some(ratatui::layout::Rect::new(2, 3, 40, 12))
        );
    }

    #[test]
    fn layout_phase_uses_full_height_without_tab_strip() {
        let command = command();
        let vm = ScreenView {
            command: &command,
            root: &command,
            selected_path: crate::spec::CommandPath::default(),
            tree_rows: Vec::new(),
            sidebar_scroll: 0,
            active_args: Vec::new(),
            authoritative_argv: Vec::new(),
            rendered_command: None,
            validation: crate::pipeline::ValidationState::default(),
            effective_values: std::collections::BTreeMap::new(),
            field_semantics: std::collections::BTreeMap::new(),
            inputs: None,
        };
        let mut snapshot = FrameSnapshot::default();

        populate_layout(
            &ui_state(),
            ratatui::layout::Rect::new(2, 3, 12, 6),
            &vm,
            &mut snapshot,
        );

        assert!(snapshot.layout.form_tabs.is_empty());
        assert_eq!(
            snapshot.layout.form_view,
            Some(ratatui::layout::Rect::new(2, 3, 12, 6))
        );
    }

    #[test]
    fn layout_phase_uses_help_overlay_inner_viewport_for_scroll_range() {
        let command = CommandSpec {
            name: "tool".to_string(),
            version: None,
            about: None,
            help: (1..=10)
                .map(|line| format!("line {line}"))
                .collect::<Vec<_>>()
                .join("\n"),
            args: Vec::new(),
            subcommands: Vec::new(),
            ..CommandSpec::default()
        };
        let vm = ScreenView {
            command: &command,
            root: &command,
            selected_path: crate::spec::CommandPath::default(),
            tree_rows: Vec::new(),
            sidebar_scroll: 0,
            active_args: Vec::new(),
            authoritative_argv: Vec::new(),
            rendered_command: None,
            validation: crate::pipeline::ValidationState::default(),
            effective_values: std::collections::BTreeMap::new(),
            field_semantics: std::collections::BTreeMap::new(),
            inputs: None,
        };
        let mut snapshot = FrameSnapshot::default();

        populate_layout(
            &ui_state(),
            ratatui::layout::Rect::new(2, 3, 40, 8),
            &vm,
            &mut snapshot,
        );

        assert_eq!(snapshot.help_scroll_max, 4);
    }

    #[test]
    fn help_overlay_skips_rendering_fields_underneath() {
        let mut command = command();
        command.help = "Command help".to_string();
        command.args = vec![option_arg("config", "--config")];
        let vm = ScreenView {
            command: &command,
            root: &command,
            selected_path: crate::spec::CommandPath::default(),
            tree_rows: Vec::new(),
            sidebar_scroll: 0,
            active_args: visible_args(&command, ActiveTab::Inputs),
            authoritative_argv: Vec::new(),
            rendered_command: None,
            validation: crate::pipeline::ValidationState::default(),
            effective_values: std::collections::BTreeMap::new(),
            field_semantics: std::collections::BTreeMap::new(),
            inputs: None,
        };
        let mut snapshot = FrameSnapshot::default();
        let mut ui = ui_state();
        ui.help_open = true;

        populate_layout(
            &ui,
            ratatui::layout::Rect::new(0, 0, 40, 8),
            &vm,
            &mut snapshot,
        );

        let mut terminal = Terminal::new(TestBackend::new(40, 8)).expect("terminal");
        terminal
            .draw(|frame| {
                render_form(frame, &ui, &TuiConfig::default(), &vm, &snapshot);
            })
            .expect("draw");

        let rendered = buffer_text(terminal.backend());
        assert!(rendered.contains("Command help"));
        assert!(!rendered.contains("--config"));
    }

    #[test]
    fn invalid_field_uses_local_error_text() {
        let mut arg = option_arg("name", "--name");
        arg.required = true;
        let root = command();

        let help = field_help_text(
            &root,
            &arg,
            FieldWidget::SingleText,
            &crate::spec::CommandPath::default(),
            FieldHelpContext {
                selected: false,
                field_error: Some("Required argument"),
                effective_value: None,
                semantic_reason: None,
            },
        )
        .expect("help text");

        assert_eq!(help, "Required argument");
    }

    #[test]
    fn secondary_invalid_field_keeps_plain_error_text() {
        let mut arg = option_arg("name", "--name");
        arg.required = true;
        let root = command();

        let help = field_help_text(
            &root,
            &arg,
            FieldWidget::SingleText,
            &crate::spec::CommandPath::default(),
            FieldHelpContext {
                selected: false,
                field_error: Some("Required argument"),
                effective_value: None,
                semantic_reason: None,
            },
        )
        .expect("help text");

        assert_eq!(help, "Required argument");
    }

    #[test]
    fn open_choice_control_uses_open_affordance() {
        let line = compact_control_line(
            &TuiConfig::default(),
            FieldWidget::SingleChoice,
            "dev",
            20,
            false,
            false,
            true,
        );
        let rendered = line
            .spans
            .iter()
            .map(|span| span.content.to_string())
            .collect::<String>();

        assert!(rendered.contains('^'));
        assert!(rendered.contains("dev"));
    }

    #[test]
    fn layout_places_option_label_and_input_on_the_same_row() {
        let mut config = option_arg("config", "--config");
        config.help = Some("Path to the main config file".to_string());
        let command = CommandSpec {
            name: "tool".to_string(),
            version: None,
            about: None,
            help: String::new(),
            args: vec![config],
            subcommands: Vec::new(),
            ..CommandSpec::default()
        };
        let vm = ScreenView {
            command: &command,
            root: &command,
            selected_path: crate::spec::CommandPath::default(),
            tree_rows: Vec::new(),
            sidebar_scroll: 0,
            active_args: visible_args(&command, ActiveTab::Inputs),
            authoritative_argv: Vec::new(),
            rendered_command: None,
            validation: crate::pipeline::ValidationState::default(),
            effective_values: std::collections::BTreeMap::new(),
            field_semantics: std::collections::BTreeMap::new(),
            inputs: None,
        };
        let mut snapshot = FrameSnapshot::default();

        populate_layout(
            &ui_state(),
            ratatui::layout::Rect::new(0, 0, 40, 12),
            &vm,
            &mut snapshot,
        );

        let field = snapshot.layout.form_fields.first().expect("field layout");
        let label = field.label.expect("label rect");
        assert_eq!(field.input.y, label.y);
        assert!(field.input.x > label.x);
    }

    #[test]
    fn form_renders_help_heading_and_combined_label() {
        let mut include = option_arg("include", "--include");
        include.metadata.identifiers.display_label = "-I, --include".to_string();
        include.metadata.display.help_heading = Some("Inputs".to_string());
        let command = CommandSpec {
            name: "tool".to_string(),
            version: None,
            about: None,
            help: String::new(),
            args: vec![include],
            subcommands: Vec::new(),
            ..CommandSpec::default()
        };
        let vm = ScreenView {
            command: &command,
            root: &command,
            selected_path: crate::spec::CommandPath::default(),
            tree_rows: Vec::new(),
            sidebar_scroll: 0,
            active_args: visible_args(&command, ActiveTab::Inputs),
            authoritative_argv: Vec::new(),
            rendered_command: None,
            validation: crate::pipeline::ValidationState::default(),
            effective_values: std::collections::BTreeMap::new(),
            field_semantics: std::collections::BTreeMap::new(),
            inputs: None,
        };
        let mut snapshot = FrameSnapshot::default();
        let ui = ui_state();

        populate_layout(
            &ui,
            ratatui::layout::Rect::new(0, 0, 40, 8),
            &vm,
            &mut snapshot,
        );

        let mut terminal = Terminal::new(TestBackend::new(40, 8)).expect("terminal");
        terminal
            .draw(|frame| {
                render_form(frame, &ui, &TuiConfig::default(), &vm, &snapshot);
            })
            .expect("draw");

        let rendered = buffer_text(terminal.backend());
        assert!(rendered.contains(" Inputs "));
        assert!(rendered.contains("-I, --include"));
    }

    #[test]
    fn unselected_single_text_field_renders_required_placeholder() {
        let mut arg = option_arg("input", "--input");
        arg.required = true;
        let command = CommandSpec {
            name: "tool".to_string(),
            version: None,
            about: None,
            help: String::new(),
            args: vec![arg],
            subcommands: Vec::new(),
            ..CommandSpec::default()
        };
        let vm = ScreenView {
            command: &command,
            root: &command,
            selected_path: crate::spec::CommandPath::default(),
            tree_rows: Vec::new(),
            sidebar_scroll: 0,
            active_args: visible_args(&command, ActiveTab::Inputs),
            authoritative_argv: Vec::new(),
            rendered_command: None,
            validation: crate::pipeline::ValidationState::default(),
            effective_values: std::collections::BTreeMap::new(),
            field_semantics: std::collections::BTreeMap::new(),
            inputs: None,
        };
        let mut snapshot = FrameSnapshot::default();
        let mut ui = ui_state();
        ui.focus = Focus::Sidebar;

        populate_layout(
            &ui,
            ratatui::layout::Rect::new(0, 0, 40, 8),
            &vm,
            &mut snapshot,
        );

        let mut terminal = Terminal::new(TestBackend::new(40, 8)).expect("terminal");
        terminal
            .draw(|frame| {
                render_form(frame, &ui, &TuiConfig::default(), &vm, &snapshot);
            })
            .expect("draw");

        let rendered = buffer_text(terminal.backend());
        assert!(rendered.contains("Enter a value to continue"));
    }

    #[test]
    fn section_heading_uses_lightweight_section_layout() {
        let mut include = option_arg("include", "--include");
        include.metadata.display.help_heading = Some("Global".to_string());
        let mut config = option_arg("config", "--config");
        config.metadata.display.help_heading = Some("Global".to_string());
        let command = CommandSpec {
            name: "tool".to_string(),
            version: None,
            about: None,
            help: String::new(),
            args: vec![include, config],
            subcommands: Vec::new(),
            ..CommandSpec::default()
        };
        let vm = ScreenView {
            command: &command,
            root: &command,
            selected_path: crate::spec::CommandPath::default(),
            tree_rows: Vec::new(),
            sidebar_scroll: 0,
            active_args: visible_args(&command, ActiveTab::Inputs),
            authoritative_argv: Vec::new(),
            rendered_command: None,
            validation: crate::pipeline::ValidationState::default(),
            effective_values: std::collections::BTreeMap::new(),
            field_semantics: std::collections::BTreeMap::new(),
            inputs: None,
        };
        let mut snapshot = FrameSnapshot::default();

        populate_layout(
            &ui_state(),
            ratatui::layout::Rect::new(0, 0, 40, 12),
            &vm,
            &mut snapshot,
        );

        let first = &snapshot.layout.form_fields[0];
        let second = &snapshot.layout.form_fields[1];

        assert!(first.section_rail.is_none());
        assert!(first.section_right_rail.is_none());
        assert!(first.section_cap.is_none());
        assert!(second.section_rail.is_none());
        assert!(second.section_right_rail.is_none());
        assert!(second.section_cap.is_none());
        assert_eq!(first.label.expect("label rect").x, 1);
        assert_eq!(second.input.x, first.input.x);
        assert_eq!(second.input.width, first.input.width);
    }

    #[test]
    fn section_rendering_shows_heading_rule_without_box_frame() {
        let mut include = option_arg("include", "--include");
        include.metadata.display.help_heading = Some("Global".to_string());
        let command = CommandSpec {
            name: "tool".to_string(),
            version: None,
            about: None,
            help: String::new(),
            args: vec![include],
            subcommands: Vec::new(),
            ..CommandSpec::default()
        };
        let vm = ScreenView {
            command: &command,
            root: &command,
            selected_path: crate::spec::CommandPath::default(),
            tree_rows: Vec::new(),
            sidebar_scroll: 0,
            active_args: visible_args(&command, ActiveTab::Inputs),
            authoritative_argv: Vec::new(),
            rendered_command: None,
            validation: crate::pipeline::ValidationState::default(),
            effective_values: std::collections::BTreeMap::new(),
            field_semantics: std::collections::BTreeMap::new(),
            inputs: None,
        };
        let mut snapshot = FrameSnapshot::default();
        let ui = ui_state();

        populate_layout(
            &ui,
            ratatui::layout::Rect::new(0, 0, 40, 8),
            &vm,
            &mut snapshot,
        );

        let mut terminal = Terminal::new(TestBackend::new(40, 8)).expect("terminal");
        terminal
            .draw(|frame| {
                render_form(frame, &ui, &TuiConfig::default(), &vm, &snapshot);
            })
            .expect("draw");

        let rendered = buffer_text(terminal.backend());
        assert!(rendered.contains(" Global "));
        assert!(rendered.contains('─'));
    }

    #[test]
    fn selected_field_help_uses_long_help_and_value_names() {
        let mut include = option_arg("include", "--include");
        include.help = Some("Include path".to_string());
        include.metadata.display.long_help = Some("Include one or more paths".to_string());
        include.metadata.values.value_names = vec!["PATH".to_string()];
        let root = command();

        let help = field_help_text(
            &root,
            &include,
            FieldWidget::SingleText,
            &crate::spec::CommandPath::default(),
            FieldHelpContext {
                selected: true,
                field_error: None,
                effective_value: None,
                semantic_reason: None,
            },
        )
        .expect("selected help text");

        assert!(help.contains("Include one or more paths"));
        assert!(!help.contains("Expects: PATH"));
    }

    #[test]
    fn layout_phase_clips_scrolled_fields_to_form_view() {
        let command = CommandSpec {
            name: "tool".to_string(),
            version: None,
            about: None,
            help: String::new(),
            args: vec![
                option_arg("target", "--target"),
                option_arg("output", "--output"),
                option_arg("mode", "--mode"),
            ],
            subcommands: Vec::new(),
            ..CommandSpec::default()
        };
        let vm = ScreenView {
            command: &command,
            root: &command,
            selected_path: crate::spec::CommandPath::default(),
            tree_rows: Vec::new(),
            sidebar_scroll: 0,
            active_args: visible_args(&command, ActiveTab::Inputs),
            authoritative_argv: Vec::new(),
            rendered_command: None,
            validation: crate::pipeline::ValidationState::default(),
            effective_values: std::collections::BTreeMap::new(),
            field_semantics: std::collections::BTreeMap::new(),
            inputs: None,
        };
        let mut snapshot = FrameSnapshot::default();
        let mut ui = ui_state();
        ui.form_scroll = 1;

        populate_layout(
            &ui,
            ratatui::layout::Rect::new(2, 3, 40, 6),
            &vm,
            &mut snapshot,
        );

        let form_view = snapshot.layout.form_view.expect("form view");
        let field = snapshot
            .layout
            .form_fields
            .first()
            .expect("visible field layout");

        assert_eq!(field.label, None);
        assert!(field.input.y >= form_view.y);
        assert!(field.input.y + field.input.height <= form_view.y + form_view.height);
    }

    #[test]
    fn clipped_text_inputs_keep_their_bordered_rendering() {
        let command = CommandSpec {
            name: "tool".to_string(),
            version: None,
            about: None,
            help: String::new(),
            args: vec![option_arg("path", "--path")],
            subcommands: Vec::new(),
            ..CommandSpec::default()
        };
        let vm = ScreenView {
            command: &command,
            root: &command,
            selected_path: crate::spec::CommandPath::default(),
            tree_rows: Vec::new(),
            sidebar_scroll: 0,
            active_args: visible_args(&command, ActiveTab::Inputs),
            authoritative_argv: Vec::new(),
            rendered_command: None,
            validation: crate::pipeline::ValidationState::default(),
            effective_values: std::collections::BTreeMap::new(),
            field_semantics: std::collections::BTreeMap::new(),
            inputs: None,
        };
        let mut snapshot = FrameSnapshot::default();
        let ui = ui_state();

        populate_layout(
            &ui,
            ratatui::layout::Rect::new(0, 0, 40, 2),
            &vm,
            &mut snapshot,
        );
        let field = snapshot
            .layout
            .form_fields
            .first()
            .expect("clipped field layout");
        assert!(
            field.input.height
                < crate::query::form::field_metrics(vm.active_args[0].arg).input_height
        );

        let mut terminal = Terminal::new(TestBackend::new(40, 2)).expect("terminal");
        terminal
            .draw(|frame| {
                render_form(frame, &ui, &TuiConfig::default(), &vm, &snapshot);
            })
            .expect("draw");

        assert_eq!(
            terminal.backend().buffer()[(field.input.x, field.input.y)].symbol(),
            " "
        );
    }

    #[test]
    fn top_clipped_text_inputs_render_their_closing_edge_instead_of_reopening() {
        let command = CommandSpec {
            name: "tool".to_string(),
            version: None,
            about: None,
            help: String::new(),
            args: vec![option_arg("path", "--path")],
            subcommands: Vec::new(),
            ..CommandSpec::default()
        };
        let vm = ScreenView {
            command: &command,
            root: &command,
            selected_path: crate::spec::CommandPath::default(),
            tree_rows: Vec::new(),
            sidebar_scroll: 0,
            active_args: visible_args(&command, ActiveTab::Inputs),
            authoritative_argv: Vec::new(),
            rendered_command: None,
            validation: crate::pipeline::ValidationState::default(),
            effective_values: std::collections::BTreeMap::new(),
            field_semantics: std::collections::BTreeMap::new(),
            inputs: None,
        };
        let mut snapshot = FrameSnapshot::default();
        let mut ui = ui_state();
        ui.form_scroll = 2;

        populate_layout(
            &ui,
            ratatui::layout::Rect::new(0, 0, 40, 3),
            &vm,
            &mut snapshot,
        );
        let field = snapshot
            .layout
            .form_fields
            .first()
            .expect("clipped field layout");
        assert_eq!(field.input.y, 0);
        let mut terminal = Terminal::new(TestBackend::new(40, 3)).expect("terminal");
        terminal
            .draw(|frame| {
                render_form(frame, &ui, &TuiConfig::default(), &vm, &snapshot);
            })
            .expect("draw");

        assert_ne!(
            terminal.backend().buffer()[(field.input.x, field.input.y)].symbol(),
            "╭"
        );
    }

    #[test]
    fn descriptions_do_not_render_when_the_field_top_row_is_scrolled_offscreen() {
        let mut define = option_arg("define", "--define");
        define.help = Some("Key/value pairs".to_string());
        let mut include = option_arg("include", "--include");
        include.help = Some("Multi-value path list".to_string());
        let command = CommandSpec {
            name: "tool".to_string(),
            version: None,
            about: None,
            help: String::new(),
            args: vec![define, include],
            subcommands: Vec::new(),
            ..CommandSpec::default()
        };
        let vm = ScreenView {
            command: &command,
            root: &command,
            selected_path: crate::spec::CommandPath::default(),
            tree_rows: Vec::new(),
            sidebar_scroll: 0,
            active_args: visible_args(&command, ActiveTab::Inputs),
            authoritative_argv: Vec::new(),
            rendered_command: None,
            validation: crate::pipeline::ValidationState::default(),
            effective_values: std::collections::BTreeMap::new(),
            field_semantics: std::collections::BTreeMap::new(),
            inputs: None,
        };
        let mut snapshot = FrameSnapshot::default();
        let mut ui = ui_state();
        ui.form_scroll = 6;

        populate_layout(
            &ui,
            ratatui::layout::Rect::new(0, 0, 40, 4),
            &vm,
            &mut snapshot,
        );

        let field = snapshot
            .layout
            .form_fields
            .first()
            .expect("visible field layout");
        assert_eq!(field.arg_id, "include");
        assert!(field.label.is_none());
        assert!(field.description.is_none());

        let mut terminal = Terminal::new(TestBackend::new(40, 4)).expect("terminal");
        terminal
            .draw(|frame| {
                render_form(frame, &ui, &TuiConfig::default(), &vm, &snapshot);
            })
            .expect("draw");

        let rendered = buffer_text(terminal.backend());
        assert!(!rendered.contains("Multi-value path list"));
    }

    #[test]
    fn clipped_section_does_not_reintroduce_heading_for_next_visible_field() {
        let mut first = option_arg("upload", "--upload");
        first.metadata.display.help_heading = Some("Actions".to_string());
        first.metadata.display.display_order = 1;
        let mut second = option_arg("token", "--token");
        second.metadata.display.help_heading = Some("Actions".to_string());
        second.metadata.display.display_order = 2;
        let command = CommandSpec {
            name: "tool".to_string(),
            version: None,
            about: None,
            help: String::new(),
            args: vec![first, second],
            subcommands: Vec::new(),
            ..CommandSpec::default()
        };
        let vm = ScreenView {
            command: &command,
            root: &command,
            selected_path: crate::spec::CommandPath::default(),
            tree_rows: Vec::new(),
            sidebar_scroll: 0,
            active_args: visible_args(&command, ActiveTab::Inputs),
            authoritative_argv: Vec::new(),
            rendered_command: None,
            validation: crate::pipeline::ValidationState::default(),
            effective_values: std::collections::BTreeMap::new(),
            field_semantics: std::collections::BTreeMap::new(),
            inputs: None,
        };
        let mut snapshot = FrameSnapshot::default();
        let mut ui = ui_state();
        ui.form_scroll = 5;

        populate_layout(
            &ui,
            ratatui::layout::Rect::new(0, 0, 40, 4),
            &vm,
            &mut snapshot,
        );

        let visible_fields = &snapshot.layout.form_fields;
        assert_eq!(visible_fields.len(), 1);
        assert_eq!(visible_fields[0].arg_id, "token");
        assert_eq!(visible_fields[0].heading, None);
    }

    #[test]
    fn later_section_heading_appears_when_its_boundary_enters_the_viewport() {
        let mut first = option_arg("upload", "--upload");
        first.metadata.display.help_heading = Some("Actions".to_string());
        first.metadata.display.display_order = 1;
        let mut second = option_arg("dry_run", "--dry-run");
        second.metadata.display.help_heading = Some("Actions".to_string());
        second.metadata.display.display_order = 2;
        let mut third = option_arg("offset", "--offset");
        third.metadata.display.help_heading = Some("Input".to_string());
        third.metadata.display.display_order = 3;
        let command = CommandSpec {
            name: "tool".to_string(),
            version: None,
            about: None,
            help: String::new(),
            args: vec![first, second, third],
            subcommands: Vec::new(),
            ..CommandSpec::default()
        };
        let vm = ScreenView {
            command: &command,
            root: &command,
            selected_path: crate::spec::CommandPath::default(),
            tree_rows: Vec::new(),
            sidebar_scroll: 0,
            active_args: visible_args(&command, ActiveTab::Inputs),
            authoritative_argv: Vec::new(),
            rendered_command: None,
            validation: crate::pipeline::ValidationState::default(),
            effective_values: std::collections::BTreeMap::new(),
            field_semantics: std::collections::BTreeMap::new(),
            inputs: None,
        };
        let mut snapshot = FrameSnapshot::default();
        let mut ui = ui_state();
        ui.form_scroll = 9;

        populate_layout(
            &ui,
            ratatui::layout::Rect::new(0, 0, 40, 5),
            &vm,
            &mut snapshot,
        );

        let visible_fields = &snapshot.layout.form_fields;
        assert_eq!(visible_fields.len(), 1);
        assert_eq!(visible_fields[0].arg_id, "offset");
        assert!(visible_fields[0].heading.is_some());
    }

    #[test]
    fn repeated_text_fields_do_not_reserve_extra_height_by_default() {
        let mut multi = option_arg("include", "--include");
        multi.value_cardinality = ValueCardinality::Many;
        let command = CommandSpec {
            name: "tool".to_string(),
            version: None,
            about: None,
            help: String::new(),
            args: vec![multi],
            subcommands: Vec::new(),
            ..CommandSpec::default()
        };
        let vm = ScreenView {
            command: &command,
            root: &command,
            selected_path: crate::spec::CommandPath::default(),
            tree_rows: Vec::new(),
            sidebar_scroll: 0,
            active_args: visible_args(&command, ActiveTab::Inputs),
            authoritative_argv: Vec::new(),
            rendered_command: None,
            validation: crate::pipeline::ValidationState::default(),
            effective_values: std::collections::BTreeMap::new(),
            field_semantics: std::collections::BTreeMap::new(),
            inputs: None,
        };
        let mut snapshot = FrameSnapshot::default();

        populate_layout(
            &ui_state(),
            ratatui::layout::Rect::new(0, 0, 50, 6),
            &vm,
            &mut snapshot,
        );

        let field = snapshot
            .layout
            .form_fields
            .first()
            .expect("repeated field layout");
        assert_eq!(field.input.height, 3);
    }

    #[test]
    fn empty_repeated_text_field_renders_as_a_normal_textarea() {
        let mut multi = option_arg("tag", "--tag");
        multi.value_cardinality = ValueCardinality::Many;
        multi.help = Some("Repeatable tag list".to_string());
        let command = CommandSpec {
            name: "tool".to_string(),
            version: None,
            about: None,
            help: String::new(),
            args: vec![multi],
            subcommands: Vec::new(),
            ..CommandSpec::default()
        };
        let vm = ScreenView {
            command: &command,
            root: &command,
            selected_path: crate::spec::CommandPath::default(),
            tree_rows: Vec::new(),
            sidebar_scroll: 0,
            active_args: visible_args(&command, ActiveTab::Inputs),
            authoritative_argv: Vec::new(),
            rendered_command: None,
            validation: crate::pipeline::ValidationState::default(),
            effective_values: std::collections::BTreeMap::new(),
            field_semantics: std::collections::BTreeMap::new(),
            inputs: None,
        };
        let mut snapshot = FrameSnapshot::default();
        let ui = ui_state();

        populate_layout(
            &ui,
            ratatui::layout::Rect::new(0, 0, 50, 8),
            &vm,
            &mut snapshot,
        );

        let field = snapshot
            .layout
            .form_fields
            .first()
            .expect("repeated field layout");
        assert_eq!(field.input.height, 3);
        assert_eq!(
            field.description.expect("description rect").y,
            field.input.y.saturating_add(field.input.height)
        );

        let mut terminal = Terminal::new(TestBackend::new(50, 8)).expect("terminal");
        terminal
            .draw(|frame| {
                render_form(frame, &ui, &TuiConfig::default(), &vm, &snapshot);
            })
            .expect("draw");

        let rendered = buffer_text(terminal.backend());
        assert!(rendered.contains(" - "));
        assert!(rendered.contains(" + "));
        assert!(rendered.contains("Repeatable tag list"));
        assert_eq!(
            terminal.backend().buffer()[(field.input.x, field.input.y)].symbol(),
            "╭"
        );
    }

    #[test]
    fn optional_choice_without_value_renders_select_placeholder() {
        let command = CommandSpec {
            name: "tool".to_string(),
            version: None,
            about: None,
            help: String::new(),
            args: vec![choice_arg("color", "--color", &["red", "green", "blue"])],
            subcommands: Vec::new(),
            ..CommandSpec::default()
        };
        let vm = ScreenView {
            command: &command,
            root: &command,
            selected_path: crate::spec::CommandPath::default(),
            tree_rows: Vec::new(),
            sidebar_scroll: 0,
            active_args: visible_args(&command, ActiveTab::Inputs),
            authoritative_argv: Vec::new(),
            rendered_command: None,
            validation: crate::pipeline::ValidationState::default(),
            effective_values: std::collections::BTreeMap::new(),
            field_semantics: std::collections::BTreeMap::new(),
            inputs: None,
        };
        let mut snapshot = FrameSnapshot::default();
        let ui = ui_state();

        populate_layout(
            &ui,
            ratatui::layout::Rect::new(0, 0, 40, 8),
            &vm,
            &mut snapshot,
        );

        let mut terminal = Terminal::new(TestBackend::new(40, 8)).expect("terminal");
        terminal
            .draw(|frame| {
                render_form(frame, &ui, &TuiConfig::default(), &vm, &snapshot);
            })
            .expect("draw");

        assert!(buffer_text(terminal.backend()).contains("Select..."));
    }

    #[test]
    fn required_choice_empty_state_is_instructional() {
        let mut required_choice = choice_arg("mode", "--mode", &["fast", "safe"]);
        required_choice.required = true;
        let command = CommandSpec {
            name: "tool".to_string(),
            version: None,
            about: None,
            help: String::new(),
            args: vec![required_choice],
            subcommands: Vec::new(),
            ..CommandSpec::default()
        };
        let vm = ScreenView {
            command: &command,
            root: &command,
            selected_path: crate::spec::CommandPath::default(),
            tree_rows: Vec::new(),
            sidebar_scroll: 0,
            active_args: visible_args(&command, ActiveTab::Inputs),
            authoritative_argv: Vec::new(),
            rendered_command: None,
            validation: crate::pipeline::ValidationState::default(),
            effective_values: std::collections::BTreeMap::new(),
            field_semantics: std::collections::BTreeMap::new(),
            inputs: None,
        };
        let mut snapshot = FrameSnapshot::default();
        let ui = ui_state();

        populate_layout(
            &ui,
            ratatui::layout::Rect::new(0, 0, 50, 8),
            &vm,
            &mut snapshot,
        );

        let mut terminal = Terminal::new(TestBackend::new(50, 8)).expect("terminal");
        terminal
            .draw(|frame| {
                render_form(frame, &ui, &TuiConfig::default(), &vm, &snapshot);
            })
            .expect("draw");

        assert!(buffer_text(terminal.backend()).contains("Press Enter to choose"));
    }

    #[test]
    fn filled_default_backed_choice_renders_with_placeholder_tone() {
        let config = TuiConfig::default();
        let line = compact_control_line(
            &config,
            FieldWidget::SingleChoice,
            "fast",
            20,
            false,
            true,
            false,
        );

        assert_eq!(line.spans[1].style.fg, Some(config.theme.metadata));
        assert!(!line.spans[1].style.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn repeated_text_fields_render_clear_add_and_remove_controls() {
        let mut multi = option_arg("include", "--include");
        multi.value_cardinality = ValueCardinality::Many;
        let command = CommandSpec {
            name: "tool".to_string(),
            version: None,
            about: None,
            help: String::new(),
            args: vec![multi],
            subcommands: Vec::new(),
            ..CommandSpec::default()
        };
        let mut state = AppState::new(command.clone());
        state.domain.set_text_value("include", "alpha\nbeta");
        state.domain.mark_touched("include");
        let derived = crate::pipeline::derive(&state);
        let vm = ScreenView {
            command: &command,
            root: &command,
            selected_path: crate::spec::CommandPath::default(),
            tree_rows: Vec::new(),
            sidebar_scroll: 0,
            active_args: visible_args(&command, ActiveTab::Inputs),
            authoritative_argv: derived.authoritative_argv,
            rendered_command: None,
            validation: derived.validation,
            effective_values: derived.effective_values,
            field_semantics: derived.field_semantics,
            inputs: state.domain.current_form(),
        };
        let mut snapshot = FrameSnapshot::default();
        let mut ui = ui_state();
        ui.selected_arg_index = 1;

        populate_layout(
            &ui,
            ratatui::layout::Rect::new(0, 0, 50, 10),
            &vm,
            &mut snapshot,
        );

        let mut terminal = Terminal::new(TestBackend::new(50, 10)).expect("terminal");
        terminal
            .draw(|frame| {
                render_form(frame, &ui, &TuiConfig::default(), &vm, &snapshot);
            })
            .expect("draw");

        let rendered = buffer_text(terminal.backend());
        assert!(!rendered.contains("Add value"));
        assert!(!rendered.contains("Remove"));
        assert!(rendered.contains(" - "));
        assert!(rendered.contains(" + "));
    }

    #[test]
    fn repeated_text_rows_use_an_external_remove_gutter() {
        let row_rect = Rect::new(10, 1, 30, 3);

        assert_eq!(
            repeated_row_textarea_rect(row_rect, true, false),
            Rect::new(10, 1, 21, 3)
        );
        assert_eq!(
            repeated_remove_rect(row_rect, true, false),
            Some(Rect::new(34, 2, 3, 1))
        );
        assert_eq!(
            repeated_row_textarea_rect(row_rect, true, true),
            Rect::new(10, 1, 21, 3)
        );
        assert_eq!(
            repeated_remove_rect(row_rect, true, true),
            Some(Rect::new(32, 2, 3, 1))
        );
        assert_eq!(repeated_add_rect(row_rect), Some(Rect::new(36, 2, 3, 1)));
    }

    #[test]
    fn clipped_single_row_repeated_controls_do_not_render_below_the_visible_row() {
        let row_rect = Rect::new(10, 1, 30, 1);

        assert_eq!(repeated_remove_rect(row_rect, true, true), None);
        assert_eq!(repeated_add_rect(row_rect), None);
    }

    #[test]
    fn clipped_repeated_text_fields_still_render_as_row_editors() {
        let mut multi = option_arg("include", "--include");
        multi.value_cardinality = ValueCardinality::Many;
        let command = CommandSpec {
            name: "tool".to_string(),
            version: None,
            about: None,
            help: String::new(),
            args: vec![multi],
            subcommands: Vec::new(),
            ..CommandSpec::default()
        };
        let mut state = AppState::new(command.clone());
        state.domain.set_text_value("include", "alpha\nbeta\ngamma");
        state.domain.mark_touched("include");
        let arg = state
            .domain
            .current_command()
            .args
            .iter()
            .find(|arg| arg.id == "include")
            .cloned()
            .expect("include arg");
        crate::form_editor::set_cursor_from_click(&mut state, &arg, 2, 0);
        let mut ui = ui_state();
        ui.editors = std::mem::take(&mut state.ui.editors);
        let derived = crate::pipeline::derive(&state);
        let vm = ScreenView {
            command: &command,
            root: &command,
            selected_path: crate::spec::CommandPath::default(),
            tree_rows: Vec::new(),
            sidebar_scroll: 0,
            active_args: visible_args(&command, ActiveTab::Inputs),
            authoritative_argv: derived.authoritative_argv,
            rendered_command: None,
            validation: derived.validation,
            effective_values: derived.effective_values,
            field_semantics: derived.field_semantics,
            inputs: state.domain.current_form(),
        };
        let mut snapshot = FrameSnapshot::default();
        let ui = ui_state();

        populate_layout(&ui, Rect::new(0, 0, 60, 4), &vm, &mut snapshot);
        let field = snapshot.layout.form_fields.first().expect("field layout");

        let mut terminal = Terminal::new(TestBackend::new(60, 4)).expect("terminal");
        terminal
            .draw(|frame| {
                render_form(frame, &ui, &TuiConfig::default(), &vm, &snapshot);
            })
            .expect("draw");

        assert_eq!(
            terminal.backend().buffer()[(field.input.x, field.input.y)].symbol(),
            "╭"
        );
    }

    #[test]
    fn clipped_repeated_text_fields_follow_outer_scroll_order_instead_of_cursor() {
        let mut multi = option_arg("include", "--include");
        multi.value_cardinality = ValueCardinality::Many;
        let command = CommandSpec {
            name: "tool".to_string(),
            version: None,
            about: None,
            help: String::new(),
            args: vec![multi],
            subcommands: Vec::new(),
            ..CommandSpec::default()
        };
        let mut state = AppState::new(command.clone());
        state.domain.set_text_value("include", "alpha\nbeta\ngamma");
        state.domain.mark_touched("include");
        let arg = state
            .domain
            .current_command()
            .args
            .iter()
            .find(|arg| arg.id == "include")
            .cloned()
            .expect("include arg");
        crate::form_editor::set_cursor_from_click(&mut state, &arg, 2, 0);
        let mut ui = ui_state();
        ui.editors = std::mem::take(&mut state.ui.editors);
        let derived = crate::pipeline::derive(&state);
        let vm = ScreenView {
            command: &command,
            root: &command,
            selected_path: crate::spec::CommandPath::default(),
            tree_rows: Vec::new(),
            sidebar_scroll: 0,
            active_args: visible_args(&command, ActiveTab::Inputs),
            authoritative_argv: derived.authoritative_argv,
            rendered_command: None,
            validation: derived.validation,
            effective_values: derived.effective_values,
            field_semantics: derived.field_semantics,
            inputs: state.domain.current_form(),
        };
        let mut snapshot = FrameSnapshot::default();
        snapshot.layout.form = Some(Rect::new(0, 0, 60, 5));
        snapshot.layout.form_view = Some(Rect::new(0, 0, 60, 5));
        snapshot
            .layout
            .form_inputs
            .insert("include".to_string(), Rect::new(10, 1, 30, 3));
        snapshot.layout.form_fields.push(FormFieldLayout {
            arg_id: "include".to_string(),
            heading: None,
            section_rail: None,
            section_right_rail: None,
            section_cap: None,
            label: Some(Rect::new(0, 1, 9, 1)),
            input: Rect::new(10, 1, 30, 3),
            input_clip_top: 3,
            description: None,
        });

        let mut terminal = Terminal::new(TestBackend::new(60, 5)).expect("terminal");
        terminal
            .draw(|frame| {
                render_form(frame, &ui, &TuiConfig::default(), &vm, &snapshot);
            })
            .expect("draw");

        let rendered = buffer_text(terminal.backend());
        assert!(rendered.contains("beta"));
        assert!(!rendered.contains("gamma"));
    }

    #[test]
    fn selected_text_input_fills_the_entire_input_surface() {
        let mut state = AppState::from_command(
            &Command::new("tool").arg(Arg::new("host").long("host").action(ArgAction::Set)),
        );
        state.domain.set_text_value("host", "127.0.0.1");
        state.domain.mark_touched("host");

        let current = state.domain.current_command().clone();
        let root = state.domain.root.clone();
        let derived = crate::pipeline::derive(&state);
        let vm = ScreenView {
            command: &current,
            root: &root,
            selected_path: crate::spec::CommandPath::default(),
            tree_rows: Vec::new(),
            sidebar_scroll: 0,
            active_args: visible_args(&current, ActiveTab::Inputs),
            authoritative_argv: derived.authoritative_argv,
            rendered_command: None,
            validation: derived.validation,
            effective_values: derived.effective_values,
            field_semantics: derived.field_semantics,
            inputs: state.domain.current_form(),
        };
        let mut snapshot = FrameSnapshot::default();
        let ui = ui_state();

        populate_layout(
            &ui,
            ratatui::layout::Rect::new(0, 0, 40, 8),
            &vm,
            &mut snapshot,
        );
        let field = snapshot.layout.form_fields.first().expect("field layout");

        let mut terminal = Terminal::new(TestBackend::new(40, 8)).expect("terminal");
        terminal
            .draw(|frame| {
                render_form(frame, &ui, &TuiConfig::default(), &vm, &snapshot);
            })
            .expect("draw");

        assert_eq!(
            cell_bg(
                terminal.backend(),
                field.input.x + field.input.width - 2,
                field.input.y + 1,
            ),
            TuiConfig::default().theme.surface_raised
        );
    }

    #[test]
    fn selected_default_backed_text_input_keeps_value_in_primary_text_color() {
        let state = AppState::from_command(
            &Command::new("tool").arg(
                Arg::new("host")
                    .long("host")
                    .action(ArgAction::Set)
                    .default_value("127.0.0.1"),
            ),
        );

        let current = state.domain.current_command().clone();
        let root = state.domain.root.clone();
        let derived = crate::pipeline::derive(&state);
        let vm = ScreenView {
            command: &current,
            root: &root,
            selected_path: crate::spec::CommandPath::default(),
            tree_rows: Vec::new(),
            sidebar_scroll: 0,
            active_args: visible_args(&current, ActiveTab::Inputs),
            authoritative_argv: derived.authoritative_argv,
            rendered_command: None,
            validation: derived.validation,
            effective_values: derived.effective_values,
            field_semantics: derived.field_semantics,
            inputs: state.domain.current_form(),
        };
        let mut snapshot = FrameSnapshot::default();
        let ui = ui_state();

        populate_layout(
            &ui,
            ratatui::layout::Rect::new(0, 0, 40, 8),
            &vm,
            &mut snapshot,
        );
        let field = snapshot.layout.form_fields.first().expect("field layout");

        let mut terminal = Terminal::new(TestBackend::new(40, 8)).expect("terminal");
        terminal
            .draw(|frame| {
                render_form(frame, &ui, &TuiConfig::default(), &vm, &snapshot);
            })
            .expect("draw");

        assert_eq!(
            cell_fg(terminal.backend(), field.input.x + 1, field.input.y + 1),
            TuiConfig::default().theme.metadata
        );
    }

    #[test]
    fn counter_fields_render_stepper_affordance_instead_of_dropdown_chevron() {
        let command = AppState::from_command(
            &Command::new("tool").arg(Arg::new("verbose").short('v').action(ArgAction::Count)),
        );
        let current = command.domain.current_command().clone();
        let root = command.domain.root.clone();
        let derived = crate::pipeline::derive(&command);
        let vm = ScreenView {
            command: &current,
            root: &root,
            selected_path: crate::spec::CommandPath::default(),
            tree_rows: Vec::new(),
            sidebar_scroll: 0,
            active_args: visible_args(&current, ActiveTab::Inputs),
            authoritative_argv: derived.authoritative_argv,
            rendered_command: None,
            validation: derived.validation,
            effective_values: derived.effective_values,
            field_semantics: derived.field_semantics,
            inputs: command.domain.current_form(),
        };
        let mut snapshot = FrameSnapshot::default();
        let ui = ui_state();

        populate_layout(
            &ui,
            ratatui::layout::Rect::new(0, 0, 40, 8),
            &vm,
            &mut snapshot,
        );

        let mut terminal = Terminal::new(TestBackend::new(40, 8)).expect("terminal");
        terminal
            .draw(|frame| {
                render_form(frame, &ui, &TuiConfig::default(), &vm, &snapshot);
            })
            .expect("draw");

        let rendered = buffer_text(terminal.backend());
        assert!(rendered.contains(" - "));
        assert!(rendered.contains(" + "));
        assert!(!rendered.contains(" ^ "));
    }

    #[test]
    fn descendant_form_shows_inherited_global_badge() {
        let mut state = AppState::from_command(
            &Command::new("tool")
                .arg(
                    Arg::new("verbose")
                        .long("verbose")
                        .action(ArgAction::SetTrue)
                        .global(true),
                )
                .subcommand(
                    Command::new("build")
                        .arg(Arg::new("target").long("target"))
                        .subcommand(Command::new("release")),
                ),
        );
        state
            .select_command_path(&["build".to_string(), "release".to_string()])
            .expect("valid descendant path");

        let current = state.domain.current_command().clone();
        let root = state.domain.root.clone();
        let selected_path = state.domain.selected_path().clone();
        let derived = crate::pipeline::derive(&state);
        let vm = ScreenView {
            command: &current,
            root: &root,
            selected_path: selected_path.clone(),
            tree_rows: Vec::new(),
            sidebar_scroll: 0,
            active_args: visible_args_for_path(&root, &selected_path, ActiveTab::Inputs),
            authoritative_argv: derived.authoritative_argv,
            rendered_command: None,
            validation: derived.validation,
            effective_values: derived.effective_values,
            field_semantics: derived.field_semantics,
            inputs: state.domain.current_form(),
        };
        let mut snapshot = FrameSnapshot::default();
        let ui = ui_state();

        populate_layout(
            &ui,
            ratatui::layout::Rect::new(0, 0, 60, 10),
            &vm,
            &mut snapshot,
        );

        let mut terminal = Terminal::new(TestBackend::new(60, 10)).expect("terminal");
        terminal
            .draw(|frame| {
                render_form(frame, &ui, &TuiConfig::default(), &vm, &snapshot);
            })
            .expect("draw");

        let rendered = buffer_text(terminal.backend());
        assert!(rendered.contains("Inherited"));
    }

    #[test]
    fn descendant_form_groups_ancestor_owned_fields_by_owner() {
        let mut state = AppState::from_command(
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
        state
            .select_command_path(&["build".to_string(), "release".to_string()])
            .expect("valid descendant path");

        let current = state.domain.current_command().clone();
        let root = state.domain.root.clone();
        let selected_path = state.domain.selected_path().clone();
        let derived = crate::pipeline::derive(&state);
        let vm = ScreenView {
            command: &current,
            root: &root,
            selected_path: selected_path.clone(),
            tree_rows: Vec::new(),
            sidebar_scroll: 0,
            active_args: visible_args_for_path(&root, &selected_path, ActiveTab::Inputs),
            authoritative_argv: derived.authoritative_argv,
            rendered_command: None,
            validation: derived.validation,
            effective_values: derived.effective_values,
            field_semantics: derived.field_semantics,
            inputs: state.domain.current_form(),
        };
        let mut snapshot = FrameSnapshot::default();
        let ui = ui_state();

        populate_layout(
            &ui,
            ratatui::layout::Rect::new(0, 0, 72, 16),
            &vm,
            &mut snapshot,
        );

        let mut terminal = Terminal::new(TestBackend::new(72, 16)).expect("terminal");
        terminal
            .draw(|frame| {
                render_form(frame, &ui, &TuiConfig::default(), &vm, &snapshot);
            })
            .expect("draw");

        let rendered = buffer_text(terminal.backend());
        assert!(rendered.contains("--profile"));
        assert!(rendered.contains("--target"));
        assert!(rendered.contains("--config"));
        assert!(rendered.contains("Inherited from tool > build"));
        assert!(rendered.contains("Inherited from tool"));
    }

    #[test]
    fn descendant_form_exposes_ancestor_option_that_also_appears_in_preview() {
        let mut state = AppState::from_command(
            &Command::new("tool")
                .arg(Arg::new("config").long("config"))
                .subcommand(Command::new("build").subcommand(Command::new("release"))),
        );
        state
            .select_command_path(&["build".to_string(), "release".to_string()])
            .expect("valid descendant path");
        state.domain.set_text_value("config", "prod.toml");

        let selected_path = state.domain.selected_path().clone();
        let active_args =
            visible_args_for_path(&state.domain.root, &selected_path, ActiveTab::Inputs);
        let derived = crate::pipeline::derive(&state);

        assert!(active_args.iter().any(|item| item.arg.id == "config"));
        assert_eq!(
            derived.authoritative_argv,
            vec![
                "tool".to_string(),
                "--config".to_string(),
                "prod.toml".to_string(),
                "build".to_string(),
                "release".to_string(),
            ]
        );
    }

    #[test]
    fn selected_inherited_field_explains_owner_and_shared_edit_scope() {
        let mut state = AppState::from_command(
            &Command::new("tool")
                .arg(Arg::new("config").long("config").global(true))
                .subcommand(Command::new("build").subcommand(Command::new("release"))),
        );
        state
            .select_command_path(&["build".to_string(), "release".to_string()])
            .expect("valid descendant path");
        let arg = state
            .domain
            .current_command()
            .args
            .iter()
            .find(|arg| arg.id == "config")
            .expect("inherited config arg");
        let selected_path =
            crate::spec::CommandPath::from(vec!["build".to_string(), "release".to_string()]);
        let help = field_help_text(
            &state.domain.root,
            arg,
            widget_for(arg),
            &selected_path,
            FieldHelpContext {
                selected: true,
                field_error: None,
                effective_value: None,
                semantic_reason: None,
            },
        )
        .expect("override help");

        assert!(help.contains("Defined on tool"));
        assert!(help.contains("updates that shared option"));
    }

    #[test]
    fn compact_help_overlay_keeps_help_content_visible() {
        let command = CommandSpec {
            name: "tool".to_string(),
            version: None,
            about: None,
            help: "Line one\nLine two\nLine three".to_string(),
            args: Vec::new(),
            subcommands: Vec::new(),
            ..CommandSpec::default()
        };
        let vm = ScreenView {
            command: &command,
            root: &command,
            selected_path: crate::spec::CommandPath::default(),
            tree_rows: Vec::new(),
            sidebar_scroll: 0,
            active_args: Vec::new(),
            authoritative_argv: Vec::new(),
            rendered_command: None,
            validation: crate::pipeline::ValidationState::default(),
            effective_values: std::collections::BTreeMap::new(),
            field_semantics: std::collections::BTreeMap::new(),
            inputs: None,
        };
        let mut snapshot = FrameSnapshot::default();
        let mut ui = ui_state();
        ui.help_open = true;

        populate_layout(
            &ui,
            ratatui::layout::Rect::new(0, 0, 18, 6),
            &vm,
            &mut snapshot,
        );

        let mut terminal = Terminal::new(TestBackend::new(18, 6)).expect("terminal");
        terminal
            .draw(|frame| {
                render_form(frame, &ui, &TuiConfig::default(), &vm, &snapshot);
            })
            .expect("draw");

        let rendered = buffer_text(terminal.backend());
        assert!(rendered.contains("Help"));
        assert!(rendered.contains("Line one"));
    }

    #[test]
    fn selected_optional_value_without_explicit_text_renders_editor_state() {
        let mut state = AppState::from_command(
            &Command::new("tool").arg(
                Arg::new("color")
                    .long("color")
                    .action(ArgAction::Set)
                    .num_args(0..=1)
                    .require_equals(true)
                    .default_missing_value("always"),
            ),
        );
        state.domain.toggle_optional_value_flag("color", true);
        state.ui.focus = Focus::Form;

        let current = state.domain.current_command().clone();
        let root = state.domain.root.clone();
        let derived = crate::pipeline::derive(&state);
        let vm = ScreenView {
            command: &current,
            root: &root,
            selected_path: crate::spec::CommandPath::default(),
            tree_rows: Vec::new(),
            sidebar_scroll: 0,
            active_args: visible_args(&current, ActiveTab::Inputs),
            authoritative_argv: derived.authoritative_argv,
            rendered_command: None,
            validation: derived.validation,
            effective_values: derived.effective_values,
            field_semantics: derived.field_semantics,
            inputs: state.domain.current_form(),
        };
        let mut snapshot = FrameSnapshot::default();
        let ui = ui_state();

        populate_layout(
            &ui,
            ratatui::layout::Rect::new(0, 0, 60, 10),
            &vm,
            &mut snapshot,
        );

        let mut terminal = Terminal::new(TestBackend::new(60, 10)).expect("terminal");
        terminal
            .draw(|frame| {
                render_form(frame, &ui, &TuiConfig::default(), &vm, &snapshot);
            })
            .expect("draw");

        let rendered = buffer_text(terminal.backend());
        assert!(rendered.contains("Present"));
        assert!(rendered.contains("bare flag"));
    }

    #[test]
    fn default_backed_optional_value_renders_as_off_state() {
        let mut state = AppState::from_command(
            &Command::new("tool").arg(
                Arg::new("color")
                    .long("color")
                    .action(ArgAction::Set)
                    .num_args(0..=1)
                    .require_equals(true)
                    .default_value("auto")
                    .default_missing_value("always"),
            ),
        );
        state.ui.focus = Focus::Form;

        let current = state.domain.current_command().clone();
        let root = state.domain.root.clone();
        let derived = crate::pipeline::derive(&state);
        let vm = ScreenView {
            command: &current,
            root: &root,
            selected_path: crate::spec::CommandPath::default(),
            tree_rows: Vec::new(),
            sidebar_scroll: 0,
            active_args: visible_args(&current, ActiveTab::Inputs),
            authoritative_argv: derived.authoritative_argv,
            rendered_command: None,
            validation: derived.validation,
            effective_values: derived.effective_values,
            field_semantics: derived.field_semantics,
            inputs: state.domain.current_form(),
        };
        let mut snapshot = FrameSnapshot::default();
        let ui = ui_state();

        populate_layout(
            &ui,
            ratatui::layout::Rect::new(0, 0, 60, 10),
            &vm,
            &mut snapshot,
        );

        let mut terminal = Terminal::new(TestBackend::new(60, 10)).expect("terminal");
        terminal
            .draw(|frame| {
                render_form(frame, &ui, &TuiConfig::default(), &vm, &snapshot);
            })
            .expect("draw");

        let rendered = buffer_text(terminal.backend());
        assert!(rendered.contains("Off"));
        assert!(rendered.contains("default: auto"));
    }

    #[test]
    fn commands_with_external_subcommands_render_external_flow_fields() {
        let command =
            CommandSpec::from_command(&Command::new("tool").allow_external_subcommands(true));
        let vm = ScreenView {
            command: &command,
            root: &command,
            selected_path: crate::spec::CommandPath::default(),
            tree_rows: Vec::new(),
            sidebar_scroll: 0,
            active_args: visible_args(&command, ActiveTab::Inputs),
            authoritative_argv: Vec::new(),
            rendered_command: None,
            validation: crate::pipeline::ValidationState::default(),
            effective_values: std::collections::BTreeMap::new(),
            field_semantics: std::collections::BTreeMap::new(),
            inputs: None,
        };
        let mut snapshot = FrameSnapshot::default();
        let ui = ui_state();

        populate_layout(
            &ui,
            ratatui::layout::Rect::new(0, 0, 60, 12),
            &vm,
            &mut snapshot,
        );

        let mut terminal = Terminal::new(TestBackend::new(60, 12)).expect("terminal");
        terminal
            .draw(|frame| {
                render_form(frame, &ui, &TuiConfig::default(), &vm, &snapshot);
            })
            .expect("draw");

        let rendered = buffer_text(terminal.backend());
        assert!(rendered.contains("External subcommand"));
        assert!(rendered.contains("Trailing args"));
    }
}
