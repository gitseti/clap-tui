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
use crate::spec::{ArgSpec, choice_value_matches_default, format_command_path};

use super::{screen::ScreenView, styles};

#[allow(dead_code)]
pub(crate) fn populate_layout(
    ui: &UiState,
    area: Rect,
    vm: &ScreenView<'_>,
    frame_snapshot: &mut FrameSnapshot,
) {
    crate::frame_snapshot::populate_form_layout(
        ui,
        area,
        &vm.active_args,
        &vm.command.help,
        &vm.validation,
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
    let content_height =
        form::measure_fields_height_with_errors(&vm.active_args, &vm.validation.field_errors);
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
        let input_is_truncated = text_input_is_truncated(item.arg, field.input);
        let field_error = vm.validation.field_errors.get(&item.arg.id);
        let validation_summary = vm.validation.summary.as_deref();
        let is_primary_invalid =
            frame_snapshot.first_invalid_field_id() == Some(item.arg.id.as_str());
        let input_state = vm
            .inputs
            .as_ref()
            .and_then(|inputs| inputs.input(&item.arg.id));
        let source_badge = effective_source_badge(vm, item.arg);
        let badges = field_badges(
            config,
            item.arg,
            &vm.selected_path,
            source_badge,
            input_state,
        );

        if let Some(heading_rect) = field.heading {
            if let Some(heading) = item.section_heading.as_deref() {
                frame.render_widget(
                    Paragraph::new(section_heading_line(config, heading, heading_rect.width)),
                    heading_rect,
                );
            }
        }
        if let Some(rail_rect) = field.section_rail {
            frame.render_widget(
                Paragraph::new(
                    (0..rail_rect.height)
                        .map(|_| Line::from("│"))
                        .collect::<Vec<_>>(),
                )
                .style(Style::default().fg(config.theme.divider)),
                rail_rect,
            );
        }
        if let Some(cap_rect) = field.section_cap {
            frame.render_widget(
                Paragraph::new(section_cap_line(cap_rect.width))
                    .style(Style::default().fg(config.theme.divider)),
                cap_rect,
            );
        }

        if let Some(label_rect) = field.label {
            let mut spans = vec![Span::styled(
                item.arg.display_label().to_string(),
                if field_error.is_some() {
                    if is_primary_invalid {
                        styles::label(config, selected)
                            .fg(config.theme.error)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        styles::label(config, selected).fg(config.theme.error)
                    }
                } else {
                    styles::label(config, selected)
                },
            )];
            if item.arg.required {
                spans.push(Span::raw(" "));
                spans.push(Span::styled("*", styles::required_prompt(config)));
            }
            spans.extend(badges.clone());
            frame.render_widget(Paragraph::new(Line::from(spans)), label_rect);
        }

        let current_value = effective_compatibility_value(vm, item.arg);
        let selected_values = effective_selected_values(vm, item.arg);
        let value = display_value(item.widget, current_value.as_ref(), &selected_values);
        let shows_choice_placeholder = matches!(
            item.widget,
            FieldWidget::SingleChoice | FieldWidget::MultiChoice
        ) && value.is_empty();
        let is_default = value_matches_default(
            item.arg,
            current_value.as_ref(),
            vm.inputs
                .as_ref()
                .is_some_and(|inputs| inputs.is_touched(&item.arg.id)),
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
        let fill_style = styles::input(config, selected);
        let text_style = if shows_choice_placeholder && item.arg.required {
            styles::required_prompt(config)
        } else if is_default || shows_choice_placeholder {
            styles::placeholder(config)
        } else {
            Style::default().fg(config.theme.text)
        };

        if matches!(item.widget, FieldWidget::Toggle) {
            render_flag_toggle(
                frame,
                config,
                field.input,
                selected,
                item.arg.display_label(),
                &value,
                &badges,
                text_style,
            );
        } else if matches!(
            item.widget,
            FieldWidget::SingleChoice | FieldWidget::MultiChoice | FieldWidget::Counter
        ) {
            let display = if value.is_empty() {
                compact_placeholder(item.arg, item.widget)
            } else {
                value.as_str()
            };
            let input = Paragraph::new(compact_control_line(
                config,
                item.widget,
                display,
                field.input.width,
                selected,
                is_default || shows_choice_placeholder,
                matches!(
                    item.widget,
                    FieldWidget::SingleChoice | FieldWidget::MultiChoice
                ) && ui.dropdown_open.as_deref() == Some(&item.arg.id),
            ))
            .style(styles::compact_control(config, selected));
            frame.render_widget(input, field.input);
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
        } else if input_is_truncated {
            frame.render_widget(
                Paragraph::new(repeated_display_lines(item.arg, item.widget, &value))
                    .style(fill_style.patch(text_style)),
                field.input,
            );
        } else if selected {
            let editor =
                form_editor::editor_for_render(ui, item.arg.owner_path(), item.arg, &value);
            let mut textarea = editor.to_textarea(editor.selection_anchor());
            textarea.set_block(block);
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
                    .bg(config.theme.surface_raised)
                    .add_modifier(Modifier::REVERSED),
            );
            if value.is_empty()
                && field_error.is_none()
                && let Some(placeholder) = required_empty_prompt(item.arg, item.widget)
            {
                textarea.set_placeholder_text(placeholder);
                textarea.set_placeholder_style(styles::required_prompt(config));
            }
            frame.render_widget(textarea.widget(), field.input);
            place_textarea_cursor(frame, &textarea, field.input);
        } else {
            frame.render_widget(
                Paragraph::new(repeated_display_lines(item.arg, item.widget, &value))
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
                    validation_summary,
                    field_error: field_error.map(String::as_str),
                    is_primary_invalid,
                    effective_value: vm.effective_values.get(&item.arg.id),
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

fn text_input_is_truncated(arg: &ArgSpec, input: Rect) -> bool {
    form::widget_for(arg).accepts_text_input() && input.height < field_metrics(arg).input_height
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

fn compact_placeholder(arg: &ArgSpec, widget: FieldWidget) -> &'static str {
    match widget {
        FieldWidget::Counter => "0",
        FieldWidget::MultiChoice if arg.required => "Press Space to choose",
        FieldWidget::SingleChoice if arg.required => "Press Enter to choose",
        _ => "Select...",
    }
}

fn repeated_display_lines(arg: &ArgSpec, widget: FieldWidget, value: &str) -> Vec<Line<'static>> {
    if !matches!(widget, FieldWidget::RepeatedText) {
        if value.is_empty()
            && let Some(prompt) = required_empty_prompt(arg, widget)
        {
            return vec![Line::from(prompt)];
        }
        return vec![Line::from(value.to_string())];
    }

    if value.is_empty() {
        return vec![Line::from(
            required_empty_prompt(arg, widget).unwrap_or_else(|| "No values added".to_string()),
        )];
    }

    value
        .lines()
        .enumerate()
        .map(|(index, line)| Line::from(format!("{:>2}. {}", index + 1, line)))
        .collect()
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
                    .block(block)
                    .style(styles::input(config, selected).patch(text_style)),
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
                .block(block)
                .style(styles::input(config, selected)),
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
                .block(block)
                .style(styles::input(config, selected)),
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
    textarea.set_block(block);
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
            .bg(config.theme.surface_raised)
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
    label: &str,
    value: &str,
    badges: &[Span<'static>],
    text_style: Style,
) {
    let enabled = value == "[x]";
    let mut spans = vec![
        Span::styled(
            if enabled { " [x] " } else { " [ ] " },
            styles::checkbox_chip(config, selected, enabled),
        ),
        Span::raw(" "),
        Span::styled(
            label.to_string(),
            if selected {
                Style::default()
                    .fg(config.theme.text)
                    .add_modifier(Modifier::BOLD)
            } else {
                text_style.add_modifier(Modifier::BOLD)
            },
        ),
    ];
    spans.extend(badges.iter().cloned());
    let line = Line::from(spans);
    frame.render_widget(
        Paragraph::new(line).style(styles::flag_toggle(config, selected)),
        area,
    );
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
    let affordance_style = styles::compact_control_affordance(
        config,
        selected,
        open || matches!(widget, FieldWidget::Counter),
    );
    let affordance = match widget {
        FieldWidget::Counter => " +/- ",
        _ if open => " ^ ",
        _ => " v ",
    };
    let affordance_width = u16::try_from(affordance.chars().count()).unwrap_or(3);
    let available_value = inner_width.saturating_sub(affordance_width + 1);
    let value_width = u16::try_from(value.chars().count()).unwrap_or(available_value);
    let padding = available_value.saturating_sub(value_width.saturating_add(1));
    Line::from(vec![
        Span::raw(" "),
        Span::styled(value.to_string(), value_style),
        Span::raw(" ".repeat(usize::from(padding))),
        Span::styled(affordance, affordance_style),
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
            Span::raw("Help"),
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
    validation_summary: Option<&'a str>,
    field_error: Option<&'a str>,
    is_primary_invalid: bool,
    effective_value: Option<&'a EffectiveArgValue>,
}

fn field_help_text(
    root: &crate::spec::CommandSpec,
    arg: &ArgSpec,
    widget: FieldWidget,
    selected_path: &crate::spec::CommandPath,
    help: FieldHelpContext<'_>,
) -> Option<String> {
    if let Some(field_error) = help.field_error {
        if help.is_primary_invalid
            && let Some(summary) = help.validation_summary
        {
            return Some(format!("{summary}  {field_error}"));
        }
        return Some(field_error.to_string());
    }

    let mut parts = Vec::new();
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
    if !arg.value_names().is_empty() {
        parts.push(format!("Expects: {}", arg.value_names().join(" ")));
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
    let left = "╭─";
    let left_width = u16::try_from(left.chars().count()).unwrap_or(2);

    if width <= title_width.saturating_add(left_width) {
        return Line::from(Span::styled(
            heading.to_string(),
            Style::default()
                .fg(config.theme.dim)
                .add_modifier(Modifier::BOLD),
        ));
    }

    let right_width = width.saturating_sub(left_width.saturating_add(title_width));
    Line::from(vec![
        Span::styled(left, Style::default().fg(config.theme.divider)),
        Span::styled(
            title,
            Style::default()
                .fg(config.theme.dim)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "─".repeat(usize::from(right_width)),
            Style::default().fg(config.theme.divider),
        ),
    ])
}

fn section_cap_line(width: u16) -> Line<'static> {
    if width <= 1 {
        return Line::from("╰");
    }

    Line::from(format!(
        "╰{}",
        "─".repeat(usize::from(width.saturating_sub(1)))
    ))
}

fn widget_help_hint(widget: FieldWidget) -> Option<&'static str> {
    match widget {
        FieldWidget::RepeatedText => {
            Some("Enter adds rows. Alt+Up/Down reorders. Ctrl+Delete removes.")
        }
        FieldWidget::MultiChoice => Some("Space toggles choices. Enter finishes selection."),
        FieldWidget::Counter => Some("Right/+ increments. Left/- decrements."),
        FieldWidget::OptionalValue => Some("Right enables. Left/Delete disables."),
        _ => None,
    }
}

fn field_badges(
    config: &TuiConfig,
    arg: &ArgSpec,
    selected_path: &crate::spec::CommandPath,
    source: Option<EffectiveValueSource>,
    input_state: Option<&ArgInputState>,
) -> Vec<Span<'static>> {
    let mut spans = Vec::new();

    if arg.is_inherited_for(selected_path) {
        spans.extend(chip_spans(
            "Inherited",
            styles::metadata_badge(config, styles::MetadataKind::Inherited),
        ));
    } else if arg.is_global() && arg.help_heading() != Some("Global") {
        spans.extend(chip_spans(
            "Global",
            styles::metadata_badge(config, styles::MetadataKind::Global),
        ));
    }

    if let Some(source) = source {
        let label = match source {
            EffectiveValueSource::User => None,
            EffectiveValueSource::Default => {
                if suppress_default_badge(arg, input_state) {
                    None
                } else {
                    Some("Default")
                }
            }
            EffectiveValueSource::Env => Some("Env"),
            EffectiveValueSource::DefaultMissing => Some("Default-missing"),
            EffectiveValueSource::ConditionalDefault => Some("Conditional"),
        };
        if let Some(label) = label {
            let kind = match source {
                EffectiveValueSource::Default => styles::MetadataKind::Default,
                EffectiveValueSource::Env => styles::MetadataKind::Env,
                EffectiveValueSource::DefaultMissing => styles::MetadataKind::Implicit,
                EffectiveValueSource::ConditionalDefault => styles::MetadataKind::Conditional,
                EffectiveValueSource::User => unreachable!("user source has no badge label"),
            };
            spans.extend(chip_spans(label, styles::metadata_badge(config, kind)));
        }
    }

    spans
}

fn required_empty_prompt(arg: &ArgSpec, widget: FieldWidget) -> Option<String> {
    if !arg.required {
        return None;
    }

    Some(match widget {
        FieldWidget::RepeatedText => "Press Enter to add the first value".to_string(),
        FieldWidget::SingleText => "Enter a value to continue".to_string(),
        FieldWidget::SingleChoice => "Press Enter to choose a value".to_string(),
        FieldWidget::MultiChoice => "Press Space to choose at least one value".to_string(),
        _ => return None,
    })
}

fn suppress_default_badge(arg: &ArgSpec, input_state: Option<&ArgInputState>) -> bool {
    let Some(input_state) = input_state else {
        return false;
    };
    if input_state.touched {
        return false;
    }

    match &input_state.value {
        ArgInput::Flag { present, source } => {
            arg.uses_toggle_semantics() && !present && *source == InputSource::Default
        }
        ArgInput::Count {
            occurrences,
            source,
        } => arg.uses_count_semantics() && *occurrences == 0 && *source == InputSource::Default,
        ArgInput::Values { .. } => false,
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

fn chip_spans(label: &str, style: Style) -> Vec<Span<'static>> {
    vec![Span::raw(" "), Span::styled(format!(" {label} "), style)]
}

#[cfg(test)]
mod tests {
    use clap::{Arg, ArgAction, Command};
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui::style::Modifier;

    use super::{
        FieldHelpContext, FieldWidget, compact_control_line, field_badges, field_help_text,
        populate_layout, text_input_is_truncated,
    };
    use crate::TuiConfig;
    use crate::frame_snapshot::FrameSnapshot;
    use crate::input::{ActiveTab, AppState, Focus, UiState};
    use crate::pipeline::EffectiveValueSource;
    use crate::query::form::{visible_args, visible_args_for_path, widget_for};
    use crate::spec::{ArgKind, ArgSpec, CommandSpec, ValueCardinality};
    use crate::ui::form::render_form;
    use crate::ui::screen::ScreenView;

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
            preview_argv: Vec::new(),
            validation: crate::pipeline::ValidationState::default(),
            effective_values: std::collections::BTreeMap::new(),
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
            preview_argv: Vec::new(),
            validation: crate::pipeline::ValidationState::default(),
            effective_values: std::collections::BTreeMap::new(),
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
            preview_argv: Vec::new(),
            validation: crate::pipeline::ValidationState::default(),
            effective_values: std::collections::BTreeMap::new(),
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
            preview_argv: Vec::new(),
            validation: crate::pipeline::ValidationState::default(),
            effective_values: std::collections::BTreeMap::new(),
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
    fn first_invalid_field_uses_summary_link_in_help_text() {
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
                validation_summary: Some("Missing required argument: --name"),
                field_error: Some("Required argument"),
                is_primary_invalid: true,
                effective_value: None,
            },
        )
        .expect("help text");

        assert!(help.contains("Missing required argument: --name"));
        assert!(help.contains("Required argument"));
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
                validation_summary: Some("Missing required argument: --name"),
                field_error: Some("Required argument"),
                is_primary_invalid: false,
                effective_value: None,
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
    fn metadata_badges_use_one_muted_label_style() {
        let config = TuiConfig::default();
        let selected_path = crate::spec::CommandPath::default();
        let badges = field_badges(
            &config,
            &option_arg("name", "--name"),
            &selected_path,
            Some(EffectiveValueSource::ConditionalDefault),
            None,
        );
        let rendered = badges
            .iter()
            .map(|span| span.content.to_string())
            .collect::<String>();

        assert!(rendered.contains("Conditional"));
        assert_eq!(badges[1].style.fg, Some(config.theme.metadata));
        assert!(!badges[1].style.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn global_badge_is_omitted_inside_global_section() {
        let config = TuiConfig::default();
        let mut arg = option_arg("profile", "--profile");
        arg.metadata.placement.global = true;
        arg.metadata.display.help_heading = Some("Global".to_string());
        let selected_path = crate::spec::CommandPath::default();

        let badges = field_badges(&config, &arg, &selected_path, None, None);
        let rendered = badges
            .iter()
            .map(|span| span.content.to_string())
            .collect::<String>();

        assert!(!rendered.contains("Global"));
    }

    #[test]
    fn inherited_badge_remains_visible_inside_global_section() {
        let config = TuiConfig::default();
        let mut arg = option_arg("profile", "--profile");
        arg.metadata.placement.global = true;
        arg.metadata.ownership.inherited_global = true;
        arg.metadata.ownership.owner_path = crate::spec::CommandPath::default();
        arg.metadata.display.help_heading = Some("Global".to_string());
        let selected_path = crate::spec::CommandPath::from(vec!["build".to_string()]);

        let badges = field_badges(&config, &arg, &selected_path, None, None);
        let rendered = badges
            .iter()
            .map(|span| span.content.to_string())
            .collect::<String>();

        assert!(rendered.contains("Inherited"));
        assert!(!rendered.contains("Global"));
    }

    #[test]
    fn layout_places_option_description_between_label_and_input() {
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
            preview_argv: Vec::new(),
            validation: crate::pipeline::ValidationState::default(),
            effective_values: std::collections::BTreeMap::new(),
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
        let description = field.description.expect("description rect");

        assert_eq!(description.y, label.y + label.height);
        assert_eq!(field.input.y, description.y + description.height);
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
            preview_argv: Vec::new(),
            validation: crate::pipeline::ValidationState::default(),
            effective_values: std::collections::BTreeMap::new(),
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
        assert!(rendered.contains("╭─ Inputs"));
        assert!(rendered.contains("-I, --include"));
    }

    #[test]
    fn section_heading_draws_rail_and_bottom_cap() {
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
            preview_argv: Vec::new(),
            validation: crate::pipeline::ValidationState::default(),
            effective_values: std::collections::BTreeMap::new(),
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

        assert!(first.section_rail.is_some());
        assert!(first.section_cap.is_none());
        assert!(second.section_rail.is_some());
        assert!(second.section_cap.is_some());
        assert_eq!(first.label.expect("label rect").x, 1);
        assert_eq!(second.input.x, 1);
    }

    #[test]
    fn section_rendering_shows_left_rail_and_bottom_cap() {
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
            preview_argv: Vec::new(),
            validation: crate::pipeline::ValidationState::default(),
            effective_values: std::collections::BTreeMap::new(),
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
        assert!(rendered.contains('│'));
        assert!(rendered.contains("╰──"));
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
                validation_summary: None,
                field_error: None,
                is_primary_invalid: false,
                effective_value: None,
            },
        )
        .expect("selected help text");

        assert!(help.contains("Include one or more paths"));
        assert!(help.contains("Expects: PATH"));
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
            preview_argv: Vec::new(),
            validation: crate::pipeline::ValidationState::default(),
            effective_values: std::collections::BTreeMap::new(),
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
    fn truncated_text_inputs_do_not_render_as_bordered_blocks() {
        let single = option_arg("target", "--target");
        let mut multi = option_arg("paths", "--paths");
        multi.value_cardinality = ValueCardinality::Many;

        assert!(text_input_is_truncated(
            &single,
            ratatui::layout::Rect::new(0, 0, 20, 1)
        ));
        assert!(!text_input_is_truncated(
            &single,
            ratatui::layout::Rect::new(0, 0, 20, 3)
        ));
        assert!(text_input_is_truncated(
            &multi,
            ratatui::layout::Rect::new(0, 0, 20, 4)
        ));
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
            preview_argv: Vec::new(),
            validation: crate::pipeline::ValidationState::default(),
            effective_values: std::collections::BTreeMap::new(),
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
            preview_argv: Vec::new(),
            validation: crate::pipeline::ValidationState::default(),
            effective_values: std::collections::BTreeMap::new(),
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
            preview_argv: derived.argv,
            validation: derived.validation,
            effective_values: derived.effective_values,
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
        assert!(rendered.contains("+/-"));
        assert!(!rendered.contains(" v "));
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
            preview_argv: derived.argv,
            validation: derived.validation,
            effective_values: derived.effective_values,
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
        assert!(rendered.contains("--verbose"));
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
            preview_argv: derived.argv,
            validation: derived.validation,
            effective_values: derived.effective_values,
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
            derived.argv,
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
                validation_summary: None,
                field_error: None,
                is_primary_invalid: false,
                effective_value: None,
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
            preview_argv: Vec::new(),
            validation: crate::pipeline::ValidationState::default(),
            effective_values: std::collections::BTreeMap::new(),
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
            preview_argv: derived.argv,
            validation: derived.validation,
            effective_values: derived.effective_values,
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
            preview_argv: derived.argv,
            validation: derived.validation,
            effective_values: derived.effective_values,
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
            preview_argv: Vec::new(),
            validation: crate::pipeline::ValidationState::default(),
            effective_values: std::collections::BTreeMap::new(),
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
