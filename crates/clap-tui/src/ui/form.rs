use ratatui::Frame;
use ratatui::layout::{Margin, Rect};
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
    let form_focused = matches!(state.focus, Focus::Form);
    let form_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(styles::panel_border(config, form_focused))
        .title(Line::from(Span::styled(
            "Form",
            styles::panel_title(config, form_focused),
        )))
        .style(styles::panel(config));
    frame.render_widget(form_block, area);
    state.layout.form = Some(area);
    state.layout.dropdown = None;
    state.layout.form_inputs.clear();
    state.layout.form_tabs.clear();
    state.layout.form_view = Some(area);

    let inner = area.inner(Margin {
        horizontal: 1,
        vertical: 1,
    });
    let show_tabs = vm.visible_tabs.len() > 1;
    let mut content_area = inner;
    if show_tabs {
        let tabs_rect = Rect::new(inner.x, inner.y, inner.width, 1);
        render_tab_bar(frame, state, config, tabs_rect, vm);
        let content_offset = 1 + TAB_CONTENT_TOP_PADDING;
        content_area = Rect::new(
            inner.x,
            inner.y.saturating_add(content_offset),
            inner.width,
            inner.height.saturating_sub(content_offset),
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
                    .fg(config.theme.accent)
                    .add_modifier(Modifier::BOLD),
            )
            .track_style(Style::default().fg(config.theme.text));
        frame.render_stateful_widget(scrollbar, area, &mut scrollbar_state);
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
        let style = if active || hovered {
            Style::default()
                .fg(config.theme.panel_bg)
                .bg(config.theme.accent)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .fg(config.theme.accent)
                .bg(config.theme.pill_bg)
                .add_modifier(Modifier::BOLD)
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

        let mut label = item.arg.name.clone();
        if item.arg.required {
            label.push_str(" *");
        }
        let label_rect = Rect::new(area.x, y as u16, area.width, metrics.label_height);
        if y >= area.y as i32 && y < area.y as i32 + area.height as i32 {
            frame.render_widget(
                Paragraph::new(Line::from(vec![
                    Span::styled(
                        if item.arg.required { "✓" } else { "•" },
                        Style::default().fg(config.theme.accent),
                    ),
                    Span::raw(" "),
                    Span::styled(label, styles::label(config, selected)),
                ])),
                label_rect,
            );
        }

        let description = item
            .arg
            .help
            .clone()
            .or_else(|| item.arg.value_hint.clone());
        if let Some(description) = description.as_ref() {
            let description_rect = Rect::new(
                area.x,
                (y + i32::from(metrics.label_height)) as u16,
                area.width,
                metrics.description_height.max(1),
            );
            if rect_visible(area, description_rect) {
                frame.render_widget(
                    Paragraph::new(Line::from(Span::raw(description.clone())))
                        .style(styles::help(config)),
                    description_rect,
                );
            }
        }

        let input_rect = Rect::new(
            area.x,
            (y + i32::from(metrics.label_height + metrics.description_height)) as u16,
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
                Style::default().fg(config.theme.accent)
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
                    render_flag_toggle(frame, config, input_rect, selected, &value, text_style);
                }
            }
            ArgKind::Enum => {
                if rect_visible(area, input_rect) {
                    let display = if value.is_empty() {
                        "Select…"
                    } else {
                        value.as_str()
                    };
                    let chevron_style = if state.enum_open.as_deref() == Some(&item.arg.id) {
                        Style::default().fg(config.theme.accent)
                    } else {
                        styles::placeholder(config)
                    };
                    let input = Paragraph::new(enum_display_line(
                        display,
                        input_rect.width.saturating_sub(2),
                        text_style,
                        chevron_style,
                    ))
                    .block(block)
                    .style(fill_style);
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
                        .bg(config.theme.input_bg);
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
                            .bg(config.theme.input_bg)
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
    value: &str,
    text_style: Style,
) {
    let enabled = value == "[x]";
    let line = Line::from(vec![
        Span::styled(if enabled { "[x]" } else { "[ ]" }, text_style),
        Span::raw(" "),
        Span::styled(if enabled { "Enabled" } else { "Disabled" }, text_style),
    ]);
    let toggle = Paragraph::new(line).style(styles::flag_toggle(config, selected));
    frame.render_widget(toggle, area);
}

fn enum_display_line(
    value: &str,
    inner_width: u16,
    value_style: Style,
    chevron_style: Style,
) -> Line<'static> {
    let value_width = value.chars().count() as u16;
    let padding = inner_width.saturating_sub(value_width.saturating_add(1));
    Line::from(vec![
        Span::styled(value.to_string(), value_style),
        Span::raw(" ".repeat(padding as usize)),
        Span::styled("▾", chevron_style),
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
