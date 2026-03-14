use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Margin, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, BorderType, Borders, List, ListItem, ListState, Paragraph, Scrollbar,
    ScrollbarOrientation, ScrollbarState,
};
use tui_textarea::TextArea;

use crate::config::TuiConfig;
use crate::input::{ActiveTab, AppState, Focus, FooterButtonLayout, HoverTarget, TabButtonLayout};
use crate::spec::CommandSpec;

#[derive(Debug, Clone)]
pub struct TreeItem {
    pub label: String,
    pub path: Vec<String>,
    pub has_children: bool,
    pub indent: usize,
    pub expanded: bool,
}

fn style_panel(config: &TuiConfig) -> Style {
    Style::default().bg(config.theme.panel_bg)
}

fn style_header(config: &TuiConfig) -> Style {
    Style::default().bg(config.theme.header_bg)
}

fn style_input(config: &TuiConfig, selected: bool) -> Style {
    if selected {
        Style::default().bg(config.theme.focus_bg)
    } else {
        Style::default().bg(config.theme.input_bg)
    }
}

fn style_label(config: &TuiConfig, selected: bool) -> Style {
    if selected {
        Style::default()
            .fg(config.theme.accent)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(config.theme.dim)
    }
}

fn style_help(config: &TuiConfig) -> Style {
    Style::default().fg(config.theme.dim)
}

fn style_placeholder(config: &TuiConfig) -> Style {
    Style::default().fg(config.theme.dim)
}

fn style_list_highlight(config: &TuiConfig) -> Style {
    Style::default()
        .fg(config.theme.accent)
        .bg(config.theme.focus_bg)
        .add_modifier(Modifier::BOLD)
}

pub fn render(frame: &mut Frame<'_>, state: &mut AppState, config: &TuiConfig) {
    let size = frame.area();
    let sidebar_width = (size.width as u32 * config.layout.sidebar_ratio as u32 / 100) as u16;

    // Background card
    let background = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(config.theme.border))
        .style(style_panel(config));
    frame.render_widget(background, size);
    let inner_size = size.inner(Margin {
        horizontal: 1,
        vertical: 1,
    });

    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(inner_size);

    let body_area = vertical[0];
    let preview_area = vertical[1];
    let footer_area = vertical[2];

    let root = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(
                sidebar_width
                    .max(20)
                    .min(body_area.width.saturating_sub(20)),
            ),
            Constraint::Min(20),
        ])
        .split(body_area);

    let sidebar_area = root[0];
    let main_area = root[1];

    render_sidebar(frame, state, config, sidebar_area);
    render_main(frame, state, config, main_area);
    render_dropdown_overlay(frame, state, config);
    render_preview_bar(frame, state, config, preview_area);
    render_footer(frame, state, config, footer_area);

    state.layout.sidebar = Some(sidebar_area);
    state.layout.footer = Some(footer_area);
}

pub fn tree_items(state: &AppState) -> Vec<TreeItem> {
    build_tree_items(&state.root, &state.expanded, &state.search)
}

fn render_sidebar(frame: &mut Frame<'_>, state: &mut AppState, config: &TuiConfig, area: Rect) {
    let sidebar = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    let mut search_style = if state.search.is_empty() {
        style_placeholder(config)
    } else {
        Style::default().fg(config.theme.text)
    };
    if matches!(state.focus, Focus::Search) {
        search_style = search_style.add_modifier(Modifier::BOLD);
        if !state.search.is_empty() {
            search_style = search_style.fg(config.theme.accent);
        }
    }

    let search = Paragraph::new(format!(
        "🔍 {}",
        if state.search.is_empty() {
            "/ to search".to_string()
        } else {
            state.search.clone()
        }
    ))
    .style(search_style)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title("Search")
            .style(style_panel(config)),
    );
    frame.render_widget(search, sidebar[0]);
    state.layout.search = Some(sidebar[0]);

    let items = build_tree_items(&state.root, &state.expanded, &state.search);
    state.layout.sidebar_items.clear();
    let mut list_state = ListState::default();
    let selected_index = items
        .iter()
        .position(|item| item.path == state.selected_path)
        .unwrap_or(0);
    list_state.select(Some(selected_index));

    let list_items = items
        .iter()
        .map(|item| ListItem::new(Line::from(item.label.clone())))
        .collect::<Vec<_>>();

    let list = List::new(list_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title("Commands")
                .style(style_panel(config)),
        )
        .style(
            Style::default()
                .fg(config.theme.dim)
                .bg(config.theme.panel_bg),
        )
        .highlight_style(style_list_highlight(config))
        .highlight_symbol("> ");

    frame.render_stateful_widget(list, sidebar[1], &mut list_state);
    state.layout.sidebar = Some(sidebar[1]);

    // Build hit-test rects for visible rows
    let list_area = sidebar[1];
    let content_y = list_area.y.saturating_add(1);
    let content_x = list_area.x.saturating_add(1);
    let content_height = list_area.height.saturating_sub(2) as usize;
    for (index, item) in items.iter().take(content_height).enumerate() {
        let row_y = content_y.saturating_add(index as u16);
        let row_rect = Rect::new(content_x, row_y, list_area.width.saturating_sub(2), 1);
        let caret = if item.has_children {
            let caret_x = content_x.saturating_add(item.indent as u16);
            Some(Rect::new(caret_x, row_y, 1, 1))
        } else {
            None
        };
        state
            .layout
            .sidebar_items
            .push(crate::input::SidebarItemLayout {
                path: item.path.clone(),
                row: row_rect,
                caret,
                has_children: item.has_children,
            });
    }
}

fn render_main(frame: &mut Frame<'_>, state: &mut AppState, config: &TuiConfig, area: Rect) {
    let main = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(8)])
        .split(area);

    render_header(frame, state, config, main[0]);
    render_form(frame, state, config, main[1]);
}

fn render_header(frame: &mut Frame<'_>, state: &mut AppState, config: &TuiConfig, area: Rect) {
    let cmd = state.current_command();
    let breadcrumb = if state.selected_path.is_empty() {
        cmd.name.clone()
    } else {
        let mut parts = vec![cmd.name.clone()];
        parts.extend(state.selected_path.iter().cloned());
        parts.join(" > ")
    };
    let title = Span::styled(
        cmd.name.clone(),
        Style::default()
            .fg(config.theme.accent)
            .add_modifier(Modifier::BOLD),
    );
    let desc = Span::styled(
        cmd.about.clone().unwrap_or_default(),
        Style::default().fg(config.theme.dim),
    );
    let crumb = Span::styled(
        format!("  |  {breadcrumb}"),
        Style::default().fg(config.theme.dim),
    );
    let line = Line::from(vec![title, Span::raw("  "), desc, crumb]);
    let header = Paragraph::new(line).style(style_header(config)).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(config.theme.border))
            .style(style_header(config)),
    );
    frame.render_widget(header, area);
}

fn render_footer(frame: &mut Frame<'_>, _state: &mut AppState, config: &TuiConfig, area: Rect) {
    let chips = vec![
        (HoverTarget::Run, "⌃↩ Run"),
        (HoverTarget::Exit, "⌃C Exit"),
        (HoverTarget::Search, "/ Search"),
        (HoverTarget::Focus, "Tab Focus"),
    ];
    let mut spans = Vec::new();
    _state.layout.footer_buttons.clear();
    let mut cursor_x = area.x;
    for (target, chip) in chips {
        let label = format!(" {chip} ");
        let width = label.chars().count() as u16;
        let hovered = _state.hover == Some(target);
        let style = if hovered {
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
        _state
            .layout
            .footer_buttons
            .push(FooterButtonLayout { target, rect });
        cursor_x = cursor_x.saturating_add(width + 1);
    }
    let line = Line::from(spans);
    let footer = Paragraph::new(line).style(style_panel(config));
    frame.render_widget(footer, area);
}

fn render_form(frame: &mut Frame<'_>, state: &mut AppState, config: &TuiConfig, area: Rect) {
    let form_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title("Form")
        .style(style_panel(config));
    frame.render_widget(form_block, area);
    state.layout.form = Some(area);

    state.ensure_defaults();
    let inputs = state.current_inputs().cloned();
    let args = state.current_command().args.clone();
    let ordered = order_args(&args);

    state.layout.dropdown = None;
    state.layout.form_inputs.clear();
    state.layout.form_tabs.clear();
    state.layout.form_view = Some(area);

    let positionals = ordered
        .iter()
        .enumerate()
        .filter(|(_, (_, arg))| matches!(arg.kind, crate::spec::ArgKind::Positional))
        .map(|(order_idx, (_, arg))| (order_idx, *arg))
        .collect::<Vec<_>>();
    let others = ordered
        .iter()
        .enumerate()
        .filter(|(_, (_, arg))| !matches!(arg.kind, crate::spec::ArgKind::Positional))
        .map(|(order_idx, (_, arg))| (order_idx, *arg))
        .collect::<Vec<_>>();

    state.ensure_active_tab_visible();
    if state.active_tab != ActiveTab::Help {
        state.ensure_selected_arg_visible();
    }

    let inner = area.inner(Margin {
        horizontal: 1,
        vertical: 1,
    });
    let visible_tabs = state.visible_tabs();
    let show_tabs = visible_tabs.len() > 1;
    let mut content_area = inner;
    if show_tabs {
        let tabs_rect = Rect::new(inner.x, inner.y, inner.width, 1);
        render_tab_bar(frame, state, config, tabs_rect, &visible_tabs);
        content_area = Rect::new(
            inner.x,
            inner.y + 1,
            inner.width,
            inner.height.saturating_sub(1),
        );
    }
    state.layout.form_view = Some(content_area);

    let content_height = match state.active_tab {
        ActiveTab::Options => measure_fields_height(&others, inputs.as_ref()),
        ActiveTab::Arguments => measure_fields_height(&positionals, inputs.as_ref()),
        ActiveTab::Help => measure_help_height(&state.current_command().help),
    };
    let viewport_height = content_area.height;
    state.form_scroll_max = content_height.saturating_sub(viewport_height);
    state.form_scroll = state.form_scroll.min(state.form_scroll_max);

    match state.active_tab {
        ActiveTab::Options => {
            let cursor_y = content_area.y as i32 - state.form_scroll as i32;
            render_fields(
                frame,
                state,
                config,
                content_area,
                cursor_y,
                &others,
                inputs.as_ref(),
            );
        }
        ActiveTab::Arguments => {
            let cursor_y = content_area.y as i32 - state.form_scroll as i32;
            render_fields(
                frame,
                state,
                config,
                content_area,
                cursor_y,
                &positionals,
                inputs.as_ref(),
            );
        }
        ActiveTab::Help => {
            render_help(frame, state, config, content_area);
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

fn order_args(args: &Vec<crate::spec::ArgSpec>) -> Vec<(usize, &crate::spec::ArgSpec)> {
    let mut positionals = args
        .iter()
        .enumerate()
        .filter(|(_, arg)| matches!(arg.kind, crate::spec::ArgKind::Positional))
        .filter(|(_, arg)| !is_help_arg(arg))
        .collect::<Vec<_>>();
    positionals.sort_by_key(|(_, arg)| arg.positional_index.unwrap_or(usize::MAX));

    let mut others = args
        .iter()
        .enumerate()
        .filter(|(_, arg)| !matches!(arg.kind, crate::spec::ArgKind::Positional))
        .filter(|(_, arg)| !is_help_arg(arg))
        .collect::<Vec<_>>();
    others.sort_by_key(|(_, arg)| arg.name.clone());

    positionals.extend(others);
    positionals
        .into_iter()
        .enumerate()
        .map(|(order_idx, (_, arg))| (order_idx, arg))
        .collect()
}

fn is_help_arg(arg: &crate::spec::ArgSpec) -> bool {
    arg.id == "help" || arg.name == "--help" || arg.name == "-h"
}

fn render_tab_bar(
    frame: &mut Frame<'_>,
    state: &mut AppState,
    config: &TuiConfig,
    area: Rect,
    tabs: &[ActiveTab],
) {
    let mut spans = Vec::new();
    let mut cursor_x = area.x;
    for tab in tabs {
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
    let line = Line::from(spans);
    let bar = Paragraph::new(line).style(style_panel(config));
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
    items: &[(usize, &crate::spec::ArgSpec)],
    inputs: Option<&crate::input::CommandInputs>,
) -> i32 {
    let mut y = start_y;
    for (order_index, arg) in items {
        if y >= area.y as i32 + area.height as i32 {
            break;
        }
        let selected =
            *order_index == state.selected_arg_index && matches!(state.focus, Focus::Form);
        let mut label = format!("{}", arg.name);
        if arg.required {
            label.push_str(" *");
        }
        let label_style = style_label(config, selected);
        let icon = if arg.required { "✓" } else { "•" };
        let input_height = if arg.is_multi { 5 } else { 3 };
        let help_text = arg.help.clone().or_else(|| arg.value_hint.clone());
        let help_height = if help_text.is_some() { 1 } else { 0 };
        let field_total = 1 + input_height + 1 + help_height;
        if y < 0 {
            y += field_total as i32;
            continue;
        }
        let label_rect = Rect::new(area.x, y as u16, area.width, 1);
        if y >= area.y as i32 && y < area.y as i32 + area.height as i32 {
            frame.render_widget(
                Paragraph::new(Line::from(vec![
                    Span::styled(icon, Style::default().fg(config.theme.accent)),
                    Span::raw(" "),
                    Span::styled(label, label_style),
                ])),
                label_rect,
            );
        }

        let input_rect = Rect::new(
            area.x + 1,
            (y + 1) as u16,
            area.width.saturating_sub(2),
            input_height,
        );
        state.layout.form_inputs.insert(arg.id.clone(), input_rect);
        let value = inputs
            .and_then(|i| i.values.get(&arg.id))
            .map(|v| match v {
                crate::input::ArgValue::Bool(v) => if *v { "[x]" } else { "[ ]" }.to_string(),
                crate::input::ArgValue::Text(v) => v.clone(),
                crate::input::ArgValue::Enum(idx) => {
                    arg.possible_values.get(*idx).cloned().unwrap_or_default()
                }
            })
            .unwrap_or_default();
        let is_default = !state.is_touched(&arg.id)
            && match (&arg.default, inputs.and_then(|i| i.values.get(&arg.id))) {
                (Some(def), Some(crate::input::ArgValue::Text(v))) => v == def,
                (Some(def), Some(crate::input::ArgValue::Enum(idx))) => arg
                    .possible_values
                    .get(*idx)
                    .map(|v| v == def)
                    .unwrap_or(false),
                (Some(def), Some(crate::input::ArgValue::Bool(v))) => {
                    (def == "true" && *v) || (def == "false" && !*v)
                }
                _ => false,
            };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(if selected {
                Style::default().fg(config.theme.accent)
            } else {
                Style::default().fg(config.theme.border)
            });
        let fill_style = style_input(config, selected);
        let text_style = if is_default {
            style_placeholder(config)
        } else {
            Style::default().fg(config.theme.text)
        };

        match arg.kind {
            crate::spec::ArgKind::Flag => {
                if rect_visible(area, input_rect) {
                    let checkbox = if value.is_empty() {
                        "[ ]".to_string()
                    } else {
                        value
                    };
                    let input = Paragraph::new(checkbox)
                        .block(block)
                        .style(fill_style.patch(text_style));
                    frame.render_widget(input, input_rect);
                }
            }
            crate::spec::ArgKind::Enum => {
                if rect_visible(area, input_rect) {
                    let display = if value.is_empty() {
                        "Select…".to_string()
                    } else {
                        format!("{value}  ▾")
                    };
                    let input = Paragraph::new(display)
                        .block(block)
                        .style(fill_style.patch(text_style));
                    frame.render_widget(input, input_rect);
                }
                if state.enum_open.as_deref() == Some(&arg.id) {
                    let dropdown_height = arg.possible_values.len().min(6) as u16;
                    let dropdown_rect = Rect::new(
                        area.x,
                        input_rect.y + input_rect.height,
                        area.width,
                        dropdown_height + 2,
                    );
                    state.layout.dropdown = Some(dropdown_rect);
                }
            }
            crate::spec::ArgKind::Option | crate::spec::ArgKind::Positional => {
                if selected {
                    let textarea = state.textarea_for(&arg.id, &value);
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
                } else {
                    if rect_visible(area, input_rect) {
                        let input = Paragraph::new(value)
                            .block(block)
                            .style(fill_style.patch(text_style));
                        frame.render_widget(input, input_rect);
                    }
                }
            }
        }

        // left focus bar
        if selected && rect_visible(area, input_rect) {
            let bar = Rect::new(area.x, input_rect.y, 1, input_rect.height);
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(
                    "│",
                    Style::default().fg(config.theme.accent),
                ))),
                bar,
            );
        }

        y += input_height as i32 + 1;
        let mut help_rect = None;
        if let Some(help) = help_text {
            if y >= area.y as i32 + area.height as i32 {
                break;
            }
            let help_line = Line::from(Span::raw(help));
            let rect = Rect::new(area.x, y as u16, area.width, 1);
            if y >= area.y as i32 && y < area.y as i32 + area.height as i32 {
                frame.render_widget(Paragraph::new(help_line).style(style_help(config)), rect);
            }
            help_rect = Some(rect);
            y += 1;
        }

        let _ = help_rect;
    }
    y
}

fn rect_visible(area: Rect, rect: Rect) -> bool {
    rect.y < area.y + area.height && rect.y + rect.height > area.y
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

fn measure_form_height(
    items: &[(usize, &crate::spec::ArgSpec)],
    inputs: Option<&crate::input::CommandInputs>,
) -> u16 {
    let mut height: u16 = 0;
    for (_, arg) in items {
        let input_height = if arg.is_multi { 5 } else { 3 };
        height += 1; // label
        height += input_height;
        height += 1; // gap
        let help_text = arg.help.clone().or_else(|| arg.value_hint.clone());
        if help_text.is_some() {
            height += 1;
        }
    }
    let _ = inputs;
    height
}

fn measure_fields_height(
    items: &[(usize, &crate::spec::ArgSpec)],
    inputs: Option<&crate::input::CommandInputs>,
) -> u16 {
    measure_form_height(items, inputs)
}

fn measure_help_height(help: &str) -> u16 {
    u16::try_from(help.lines().count()).unwrap_or(u16::MAX)
}

fn render_help(frame: &mut Frame<'_>, state: &AppState, config: &TuiConfig, area: Rect) {
    let help = state.current_command().help.clone();
    let paragraph = Paragraph::new(help)
        .style(Style::default().fg(config.theme.text))
        .scroll((state.form_scroll, 0));
    frame.render_widget(paragraph, area);
}

fn render_dropdown_overlay(frame: &mut Frame<'_>, state: &mut AppState, config: &TuiConfig) {
    let Some(arg_id) = state.enum_open.clone() else {
        return;
    };
    let Some(rect) = state.layout.dropdown else {
        return;
    };
    let Some(arg) = state.current_command().args.iter().find(|a| a.id == arg_id) else {
        return;
    };
    let total = arg.possible_values.len();
    let visible = rect.height.saturating_sub(2) as usize;
    let start = state.enum_scroll.min(total.saturating_sub(visible));
    let end = (start + visible).min(total);
    let items = arg
        .possible_values
        .iter()
        .skip(start)
        .take(visible)
        .map(|v| {
            let line = Line::from(Span::styled(
                v.clone(),
                Style::default().fg(config.theme.text),
            ));
            ListItem::new(line)
        })
        .collect::<Vec<_>>();
    let mut list_state = ListState::default();
    let current_idx = state
        .current_inputs()
        .and_then(|i| i.values.get(&arg.id))
        .and_then(|v| match v {
            crate::input::ArgValue::Enum(idx) => Some(*idx),
            _ => None,
        })
        .unwrap_or(0);
    if current_idx >= start && current_idx < end {
        list_state.select(Some(current_idx - start));
    }
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title("Select"),
        )
        .highlight_style(
            Style::default()
                .fg(config.theme.text)
                .bg(config.theme.focus_bg)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ")
        .style(Style::default().bg(config.theme.input_bg));
    frame.render_stateful_widget(list, rect, &mut list_state);

    if total > visible && visible > 0 {
        let scroll_steps = total.saturating_sub(visible).saturating_add(1);
        let mut scrollbar_state = ScrollbarState::new(scroll_steps)
            .position(state.enum_scroll)
            .viewport_content_length(visible);
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .track_symbol(Some("┃"))
            .thumb_symbol("█")
            .thumb_style(
                Style::default()
                    .fg(config.theme.accent)
                    .add_modifier(Modifier::BOLD),
            )
            .track_style(Style::default().fg(config.theme.text));
        frame.render_stateful_widget(scrollbar, rect, &mut scrollbar_state);
    }
}

fn render_preview_bar(frame: &mut Frame<'_>, state: &mut AppState, config: &TuiConfig, area: Rect) {
    let argv = build_preview_argv(state);
    let _missing = missing_required(state);
    let command_line = Line::from(vec![
        Span::styled("$ ", style_help(config)),
        Span::styled(argv.join(" "), Style::default().fg(config.theme.text)),
    ]);
    let bar = Paragraph::new(vec![command_line])
        .style(style_panel(config))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(config.theme.border)),
        );
    frame.render_widget(bar, area);
}

fn build_preview_argv(state: &AppState) -> Vec<String> {
    let mut argv = Vec::new();
    argv.push(state.root.name.clone());
    argv.extend(state.selected_path.iter().cloned());

    let inputs = state.current_inputs();
    let args = state.current_command().args.iter();
    let mut positionals: Vec<(usize, usize, String)> = Vec::new();
    let mut pos_seq: usize = 0;

    for arg in args {
        let is_touched = state.is_touched(&arg.id);
        if arg.default.is_some() && !is_touched {
            continue;
        }
        match inputs.and_then(|i| i.values.get(&arg.id)) {
            Some(crate::input::ArgValue::Bool(true)) => {
                argv.push(arg.name.clone());
            }
            Some(crate::input::ArgValue::Text(value)) if !value.is_empty() => {
                if arg.kind == crate::spec::ArgKind::Positional {
                    if let Some(idx) = arg.positional_index {
                        if arg.is_multi {
                            for part in value.lines().filter(|s| !s.trim().is_empty()) {
                                positionals.push((idx, pos_seq, part.to_string()));
                                pos_seq += 1;
                            }
                        } else {
                            positionals.push((idx, pos_seq, value.clone()));
                            pos_seq += 1;
                        }
                    }
                } else if arg.is_multi {
                    for part in value.lines().filter(|s| !s.trim().is_empty()) {
                        argv.push(arg.name.clone());
                        argv.push(part.to_string());
                    }
                } else {
                    argv.push(arg.name.clone());
                    argv.push(value.clone());
                }
            }
            Some(crate::input::ArgValue::Enum(idx)) => {
                if let Some(val) = arg.possible_values.get(*idx) {
                    argv.push(arg.name.clone());
                    argv.push(val.clone());
                }
            }
            _ => {}
        }
    }

    positionals.sort_by_key(|(idx, seq, _)| (*idx, *seq));
    for (_, _, value) in positionals {
        argv.push(value);
    }

    argv
}

fn missing_required(state: &AppState) -> Vec<String> {
    let inputs = state.current_inputs();
    state
        .current_command()
        .args
        .iter()
        .filter(|arg| arg.required)
        .filter_map(|arg| match inputs.and_then(|i| i.values.get(&arg.id)) {
            Some(crate::input::ArgValue::Bool(true)) => None,
            Some(crate::input::ArgValue::Text(value)) if !value.is_empty() => None,
            Some(crate::input::ArgValue::Enum(_)) => None,
            _ => Some(arg.name.clone()),
        })
        .collect()
}

fn build_tree_items(
    root: &CommandSpec,
    expanded: &std::collections::HashSet<String>,
    search: &str,
) -> Vec<TreeItem> {
    let mut items = Vec::new();
    let filter = search.trim().to_lowercase();
    build_tree_items_inner(root, expanded, &filter, &mut Vec::new(), 0, &mut items);
    items
}

fn build_tree_items_inner(
    cmd: &CommandSpec,
    expanded: &std::collections::HashSet<String>,
    filter: &str,
    path: &mut Vec<String>,
    depth: usize,
    items: &mut Vec<TreeItem>,
) -> bool {
    let matches = filter.is_empty()
        || cmd.name.to_lowercase().contains(filter)
        || cmd
            .about
            .as_deref()
            .unwrap_or("")
            .to_lowercase()
            .contains(filter);

    let mut any_child_matches = false;
    path.push(cmd.name.clone());
    let key = path.join("::");

    let is_expanded = expanded.contains(&key);
    let mut child_items = Vec::new();
    for sub in &cmd.subcommands {
        let mut child_path = path.clone();
        let child_matches = build_tree_items_inner(
            sub,
            expanded,
            filter,
            &mut child_path,
            depth + 1,
            &mut child_items,
        );
        if child_matches {
            any_child_matches = true;
        }
    }

    let include = matches || any_child_matches;
    if include {
        let indent = "  ".repeat(depth);
        let caret = if cmd.subcommands.is_empty() {
            " "
        } else if is_expanded {
            "-"
        } else {
            "+"
        };
        let label = format!("{indent}{caret} {}", cmd.name);
        let display_path = if depth == 0 {
            Vec::new()
        } else {
            path[1..].to_vec()
        };
        items.push(TreeItem {
            label,
            path: display_path,
            has_children: !cmd.subcommands.is_empty(),
            indent: indent.len(),
            expanded: is_expanded,
        });
        let show_children = if filter.is_empty() {
            is_expanded
        } else {
            any_child_matches
        };
        if show_children {
            items.extend(child_items);
        }
    }

    path.pop();
    include
}
