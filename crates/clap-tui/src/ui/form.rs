use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, BorderType, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
};
use tui_textarea::TextArea;

use crate::config::TuiConfig;
use crate::input::{ActiveTab, AppState, ArgValue, Focus, TabButtonLayout};
use crate::spec::{ArgKind, ArgSpec, enum_value_matches_default};
use crate::view::form::{self, field_metrics};

use super::{dropdown, screen::ScreenView, styles};

const TAB_CONTENT_TOP_PADDING: u16 = 1;

pub(crate) fn render_form(
    frame: &mut Frame<'_>,
    state: &mut AppState,
    config: &TuiConfig,
    area: Rect,
    vm: &ScreenView,
) {
    state.layout.form = Some(area);
    state.layout.dropdown = None;
    state.layout.form_inputs.clear();
    state.layout.form_tabs.clear();
    state.layout.form_view = Some(area);

    let show_tabs = vm.visible_tabs.len() > 1;
    let mut content_area = area;
    if show_tabs {
        let tabs_rect = Rect::new(area.x, area.y, area.width, 1);
        render_tab_bar(frame, state, config, tabs_rect, vm);
        let content_offset = 1 + TAB_CONTENT_TOP_PADDING;
        content_area = Rect::new(
            area.x,
            area.y.saturating_add(content_offset),
            area.width,
            area.height.saturating_sub(content_offset),
        );
    }
    state.layout.form_view = Some(content_area);

    let ordered_args = vm.ordered_active_args();
    let content_height = match state.active_tab {
        ActiveTab::Help => form::measure_help_height(&vm.command.help),
        _ => form::measure_fields_height(&ordered_args),
    };
    let viewport_height = content_area.height;
    state.form_scroll_max = content_height.saturating_sub(viewport_height);
    state.form_scroll = state.form_scroll.min(state.form_scroll_max);

    match state.active_tab {
        ActiveTab::Help => render_help(
            frame,
            config,
            content_area,
            state.form_scroll,
            &vm.command.help,
        ),
        _ => {
            let cursor_y = content_area.y as i32 - state.form_scroll as i32;
            render_fields(frame, state, config, content_area, cursor_y, vm);
        }
    }

    if content_height > viewport_height {
        let scroll_steps = state.form_scroll_max.saturating_add(1) as usize;
        let mut scrollbar_state = ScrollbarState::new(scroll_steps)
            .position(state.form_scroll as usize)
            .viewport_content_length(viewport_height as usize);
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .track_symbol(Some("┃"))
            .thumb_symbol("█")
            .thumb_style(
                Style::default()
                    .fg(config.theme.panel_focus_border)
                    .add_modifier(Modifier::BOLD),
            )
            .track_style(Style::default().fg(config.theme.dim));
        frame.render_stateful_widget(scrollbar, content_area, &mut scrollbar_state);
    }
}

fn render_tab_bar(
    frame: &mut Frame<'_>,
    state: &mut AppState,
    config: &TuiConfig,
    area: Rect,
    vm: &ScreenView,
) {
    let mut spans = Vec::new();
    let mut cursor_x = area.x;
    for tab in &vm.visible_tabs {
        let label = format!(" {} ", tab_label(*tab));
        let width = label.chars().count() as u16;
        let active = *tab == state.active_tab;
        let hovered = state.hover_tab == Some(*tab);
        let style = if active {
            Style::default()
                .fg(config.theme.panel_bg)
                .bg(config.theme.accent)
                .add_modifier(Modifier::BOLD)
        } else if hovered {
            Style::default()
                .fg(config.theme.text)
                .bg(config.theme.surface_raised)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .fg(config.theme.dim)
                .bg(config.theme.surface_raised)
        };
        spans.push(Span::styled(label.clone(), style));
        spans.push(Span::raw(" "));

        let rect = Rect::new(cursor_x, area.y, width, 1);
        state
            .layout
            .form_tabs
            .push(TabButtonLayout { tab: *tab, rect });
        cursor_x = cursor_x.saturating_add(width + 1);
    }
    let bar = Paragraph::new(Line::from(spans)).style(styles::panel(config));
    frame.render_widget(bar, area);
}

fn tab_label(tab: ActiveTab) -> &'static str {
    match tab {
        ActiveTab::Options => "Options",
        ActiveTab::Arguments => "Arguments",
        ActiveTab::Help => "Help",
    }
}

fn render_fields(
    frame: &mut Frame<'_>,
    state: &mut AppState,
    config: &TuiConfig,
    area: Rect,
    start_y: i32,
    vm: &ScreenView,
) {
    let mut y = start_y;
    for item in &vm.active_args {
        if y >= area.y as i32 + area.height as i32 {
            break;
        }
        let selected =
            item.order_index == state.selected_arg_index && matches!(state.focus, Focus::Form);
        let metrics = field_metrics(&item.arg);
        if y < 0 {
            y += i32::from(metrics.total_height);
            continue;
        }

        let mut input_y = y;
        if metrics.label_height > 0 {
            let label_rect = Rect::new(area.x, y as u16, area.width, metrics.label_height);
            if rect_visible(area, label_rect) {
                let mut spans = vec![Span::styled(
                    item.arg.name.clone(),
                    styles::label(config, selected),
                )];
                if item.arg.required {
                    spans.push(Span::raw(" "));
                    spans.push(Span::styled(
                        "*",
                        Style::default().fg(config.theme.accent),
                    ));
                }
                frame.render_widget(Paragraph::new(Line::from(spans)), label_rect);
            }
            input_y += i32::from(metrics.label_height);
        }

        let input_rect = Rect::new(
            area.x,
            input_y as u16,
            area.width,
            metrics.input_height,
        );
        state
            .layout
            .form_inputs
            .insert(item.arg.id.clone(), input_rect);
        let value = vm
            .inputs
            .as_ref()
            .and_then(|inputs| inputs.values.get(&item.arg.id))
            .map(|v| match v {
                ArgValue::Bool(v) => {
                    if *v {
                        "[x]".to_string()
                    } else {
                        "[ ]".to_string()
                    }
                }
                ArgValue::Text(v) => v.clone(),
                ArgValue::Enum(idx) => item
                    .arg
                    .possible_values
                    .get(*idx)
                    .cloned()
                    .unwrap_or_default(),
            })
            .unwrap_or_default();
        let current_value = vm
            .inputs
            .as_ref()
            .and_then(|inputs| inputs.values.get(&item.arg.id));
        let is_default =
            value_matches_default(&item.arg, current_value, state.is_touched(&item.arg.id));

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(if selected {
                Style::default().fg(config.theme.panel_focus_border)
            } else {
                Style::default().fg(config.theme.border)
            });
        let fill_style = styles::input(config, selected);
        let text_style = if is_default {
            styles::placeholder(config)
        } else {
            Style::default().fg(config.theme.text)
        };

        match item.arg.kind {
            ArgKind::Flag => {
                if rect_visible(area, input_rect) {
                    render_flag_toggle(
                        frame,
                        config,
                        input_rect,
                        selected,
                        &item.arg.name,
                        &value,
                        text_style,
                    );
                }
            }
            ArgKind::Enum => {
                if rect_visible(area, input_rect) {
                    let display = if value.is_empty() {
                        "Select..."
                    } else {
                        value.as_str()
                    };
                    let input = Paragraph::new(enum_display_line(
                        config,
                        display,
                        input_rect.width,
                        selected,
                        is_default,
                        state.enum_open.as_deref() == Some(&item.arg.id),
                    ))
                    .style(styles::compact_control(config, selected));
                    frame.render_widget(input, input_rect);
                }
                if state.enum_open.as_deref() == Some(&item.arg.id) {
                    state.layout.dropdown = state
                        .layout
                        .form_view
                        .and_then(|form_view| {
                            dropdown::dropdown_layout(
                                form_view,
                                input_rect,
                                item.arg.possible_values.len(),
                            )
                        })
                        .map(|layout| layout.rect);
                }
            }
            ArgKind::Option | ArgKind::Positional => {
                if selected {
                    let textarea = state.textarea_for(&item.arg.id, &value);
                    if textarea.lines().join("\n") != value {
                        *textarea = TextArea::new(vec![value.clone()]);
                    }
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
                    if rect_visible(area, input_rect) {
                        frame.render_widget(textarea.widget(), input_rect);
                        place_textarea_cursor(frame, textarea, input_rect);
                    }
                } else if rect_visible(area, input_rect) {
                    let input = Paragraph::new(value)
                        .block(block)
                        .style(fill_style.patch(text_style));
                    frame.render_widget(input, input_rect);
                }
            }
        }

        if let Some(help) = field_help_text(&item.arg) {
            let help_rect = Rect::new(
                area.x,
                input_rect.y.saturating_add(metrics.input_height),
                area.width,
                metrics.description_height.max(1),
            );
            if rect_visible(area, help_rect) {
                frame.render_widget(
                    Paragraph::new(Line::from(Span::raw(help))).style(styles::help(config)),
                    help_rect,
                );
            }
        }

        y += i32::from(metrics.total_height);
    }
}

fn rect_visible(area: Rect, rect: Rect) -> bool {
    rect.y < area.y + area.height && rect.y + rect.height > area.y
}

fn value_matches_default(arg: &ArgSpec, value: Option<&ArgValue>, is_touched: bool) -> bool {
    match value {
        Some(ArgValue::Enum(index)) => !is_touched && enum_value_matches_default(arg, *index),
        Some(ArgValue::Text(text)) => !is_touched && arg.default.as_deref() == Some(text.as_str()),
        Some(ArgValue::Bool(enabled)) => {
            !is_touched
                && matches!(
                    (arg.default.as_deref(), *enabled),
                    (Some("true"), true) | (Some("false"), false)
                )
        }
        None => false,
    }
}

fn render_flag_toggle(
    frame: &mut Frame<'_>,
    config: &TuiConfig,
    area: Rect,
    selected: bool,
    label: &str,
    value: &str,
    text_style: Style,
) {
    let enabled = value == "[x]";
    let line = Line::from(vec![
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
    ]);
    let toggle = Paragraph::new(line).style(styles::flag_toggle(config, selected));
    frame.render_widget(toggle, area);
}

fn enum_display_line(
    config: &TuiConfig,
    value: &str,
    inner_width: u16,
    selected: bool,
    is_default: bool,
    open: bool,
) -> Line<'static> {
    let value_style = styles::compact_control_value(config, selected, is_default);
    let affordance_style = styles::compact_control_affordance(config, selected, open);
    let affordance_width = 3;
    let available_value = inner_width.saturating_sub(affordance_width + 1);
    let value_width = value.chars().count() as u16;
    let padding = available_value.saturating_sub(value_width.saturating_add(1));
    Line::from(vec![
        Span::raw(" "),
        Span::styled(value.to_string(), value_style),
        Span::raw(" ".repeat(padding as usize)),
        Span::styled(" v ", affordance_style),
    ])
}

fn place_textarea_cursor(frame: &mut Frame<'_>, textarea: &TextArea<'_>, area: Rect) {
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
        .saturating_add(col as u16)
        .min(inner_x + inner_w - 1);
    let y = inner_y
        .saturating_add(row as u16)
        .min(inner_y + inner_h - 1);
    frame.set_cursor_position((x, y));
}

fn render_help(frame: &mut Frame<'_>, config: &TuiConfig, area: Rect, scroll: u16, help: &str) {
    let paragraph = Paragraph::new(help.to_string())
        .style(Style::default().fg(config.theme.text))
        .scroll((scroll, 0));
    frame.render_widget(paragraph, area);
}

fn field_help_text(arg: &ArgSpec) -> Option<String> {
    arg.help.clone().or_else(|| arg.value_hint.clone())
}
