use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, BorderType, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
    Widget,
};

use crate::config::TuiConfig;
use crate::frame_snapshot::{FrameLayout, FrameSnapshot, TabButtonLayout};
use crate::form_editor;
use crate::input::{ActiveTab, ArgValue, Focus, UiState};
use crate::spec::CommandPath;
use crate::spec::{ArgSpec, choice_value_matches_default};
use crate::view::form::{self, field_metrics};

use super::{dropdown, screen::ScreenView, styles};

const TAB_CONTENT_TOP_PADDING: u16 = 1;

pub(crate) fn render_form(
    frame: &mut Frame<'_>,
    ui: &mut UiState,
    selected_path: &CommandPath,
    config: &TuiConfig,
    area: Rect,
    vm: &ScreenView<'_>,
    frame_snapshot: &mut FrameSnapshot,
) {
    let frame_layout = &mut frame_snapshot.layout;
    frame_layout.form = Some(area);
    frame_layout.dropdown = None;
    frame_layout.form_inputs.clear();
    frame_layout.form_tabs.clear();
    frame_layout.form_view = Some(area);

    let show_tabs = vm.visible_tabs.len() > 1;
    let mut content_area = area;
    if show_tabs {
        let tabs_rect = Rect::new(area.x, area.y, area.width, 1);
        render_tab_bar(frame, ui, config, tabs_rect, vm, frame_layout);
        let content_offset = 1 + TAB_CONTENT_TOP_PADDING;
        content_area = Rect::new(
            area.x,
            area.y.saturating_add(content_offset),
            area.width,
            area.height.saturating_sub(content_offset),
        );
    }
    frame_layout.form_view = Some(content_area);

    let content_height = match ui.active_tab {
        ActiveTab::Help => form::measure_help_height(&vm.command.help),
        _ => form::measure_fields_height(&vm.active_args),
    };
    let viewport_height = content_area.height;
    let form_scroll_max = content_height.saturating_sub(viewport_height);
    frame_snapshot.form_scroll_max = form_scroll_max;
    let form_scroll = ui.form_scroll.min(frame_snapshot.form_scroll_max);

    match ui.active_tab {
        ActiveTab::Help => render_help(frame, config, content_area, form_scroll, &vm.command.help),
        _ => {
            let cursor_y = content_area.y as i32 - i32::from(form_scroll);
            render_fields(frame, ui, selected_path, config, content_area, cursor_y, vm, frame_layout);
        }
    }

    if content_height > viewport_height {
        let scroll_steps = usize::from(form_scroll_max.saturating_add(1));
        let mut scrollbar_state = ScrollbarState::new(scroll_steps)
            .position(usize::from(form_scroll))
            .viewport_content_length(usize::from(viewport_height));
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
    ui: &UiState,
    config: &TuiConfig,
    area: Rect,
    vm: &ScreenView<'_>,
    frame_layout: &mut FrameLayout,
) {
    let mut cursor_x = area.x;
    let view = build_tab_bar_view(ui, config, vm);
    for item in &view.items {
        let width = u16::try_from(item.label.chars().count()).unwrap_or(area.width);
        frame_layout.form_tabs.push(TabButtonLayout {
            tab: item.tab,
            rect: Rect::new(cursor_x, area.y, width, 1),
        });
        cursor_x = cursor_x.saturating_add(width + 1);
    }
    frame.render_widget(TabBarWidget { view: &view }, area);
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TabBarItem {
    tab: ActiveTab,
    label: String,
    style: Style,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TabBarView {
    items: Vec<TabBarItem>,
}

struct TabBarWidget<'a> {
    view: &'a TabBarView,
}

impl<'a> Widget for TabBarWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let line = Line::from(
            self.view
                .items
                .iter()
                .enumerate()
                .flat_map(|(index, item)| {
                    let mut spans = Vec::with_capacity(2);
                    spans.push(Span::styled(item.label.clone(), item.style));
                    if index + 1 < self.view.items.len() {
                        spans.push(Span::raw(" "));
                    }
                    spans
                })
                .collect::<Vec<_>>(),
        );
        Widget::render(Paragraph::new(line), area, buf);
    }
}

fn build_tab_bar_view(ui: &UiState, config: &TuiConfig, vm: &ScreenView<'_>) -> TabBarView {
    TabBarView {
        items: vm
            .visible_tabs
            .iter()
            .map(|tab| {
                let active = *tab == ui.active_tab;
                let hovered = ui.hover_tab == Some(*tab);
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
                TabBarItem {
                    tab: *tab,
                    label: format!(" {} ", tab_label(*tab)),
                    style,
                }
            })
            .collect(),
    }
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
    ui: &mut UiState,
    selected_path: &CommandPath,
    config: &TuiConfig,
    area: Rect,
    start_y: i32,
    vm: &ScreenView<'_>,
    frame_layout: &mut FrameLayout,
) {
    let mut y = start_y;
    for item in &vm.active_args {
        if y >= area.y as i32 + area.height as i32 {
            break;
        }
        let selected = item.order_index == ui.selected_arg_index && matches!(ui.focus, Focus::Form);
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
                    item.arg.display_name.clone(),
                    styles::label(config, selected),
                )];
                if item.arg.required {
                    spans.push(Span::raw(" "));
                    spans.push(Span::styled("*", Style::default().fg(config.theme.accent)));
                }
                frame.render_widget(Paragraph::new(Line::from(spans)), label_rect);
            }
            input_y += i32::from(metrics.label_height);
        }

        let input_rect = Rect::new(area.x, input_y as u16, area.width, metrics.input_height);
        frame_layout
            .form_inputs
            .insert(item.arg.id.clone(), input_rect);
        let value = vm
            .inputs
            .as_ref()
            .and_then(|inputs| inputs.values.get(&item.arg.id))
            .map(|arg_value| match arg_value {
                ArgValue::Bool(enabled) => {
                    if *enabled {
                        "[x]".to_string()
                    } else {
                        "[ ]".to_string()
                    }
                }
                ArgValue::Text(text) => text.clone(),
                ArgValue::Choice(value) => value.clone(),
            })
            .unwrap_or_default();
        let current_value = vm
            .inputs
            .as_ref()
            .and_then(|inputs| inputs.values.get(&item.arg.id));
        let is_default = value_matches_default(
            item.arg,
            current_value,
            vm.inputs
                .is_some_and(|inputs| inputs.touched.contains(&item.arg.id)),
        );

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

        if item.arg.is_flag() {
            if rect_visible(area, input_rect) {
                render_flag_toggle(
                    frame,
                    config,
                    input_rect,
                    selected,
                    &item.arg.display_name,
                    &value,
                    text_style,
                );
            }
        } else if item.arg.uses_choice_input() {
            if rect_visible(area, input_rect) {
                let display = if value.is_empty() { "Select..." } else { value.as_str() };
                let input = Paragraph::new(enum_display_line(
                    config,
                    display,
                    input_rect.width,
                    selected,
                    is_default,
                    ui.dropdown_open.as_deref() == Some(&item.arg.id),
                ))
                .style(styles::compact_control(config, selected));
                frame.render_widget(input, input_rect);
            }
            if ui.dropdown_open.as_deref() == Some(&item.arg.id) {
                frame_layout.dropdown = frame_layout
                    .form_view
                    .and_then(|form_view| {
                        dropdown::dropdown_layout(form_view, input_rect, item.arg.choices.len())
                    })
                    .map(|layout| layout.rect);
            }
        } else if selected {
            let textarea = form_editor::ensure_editor(ui, selected_path, item.arg, &value);
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
            frame.render_widget(
                Paragraph::new(value).block(block).style(fill_style.patch(text_style)),
                input_rect,
            );
        }

        if let Some(help) = field_help_text(item.arg) {
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
    frame.render_widget(Paragraph::new(line).style(styles::flag_toggle(config, selected)), area);
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
    let value_width = u16::try_from(value.chars().count()).unwrap_or(available_value);
    let padding = available_value.saturating_sub(value_width.saturating_add(1));
    Line::from(vec![
        Span::raw(" "),
        Span::styled(value.to_string(), value_style),
        Span::raw(" ".repeat(usize::from(padding))),
        Span::styled(" v ", affordance_style),
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

fn render_help(frame: &mut Frame<'_>, config: &TuiConfig, area: Rect, scroll: u16, help: &str) {
    frame.render_widget(
        Paragraph::new(help.to_string())
            .style(Style::default().fg(config.theme.text))
            .scroll((scroll, 0)),
        area,
    );
}

fn field_help_text(arg: &ArgSpec) -> Option<String> {
    arg.help.clone().or_else(|| arg.value_hint.clone())
}

#[cfg(test)]
mod tests {
    use ratatui::style::Modifier;

    use super::build_tab_bar_view;
    use crate::config::TuiConfig;
    use crate::input::{ActiveTab, Focus, UiState};
    use crate::spec::CommandSpec;
    use crate::ui::screen::ScreenView;

    fn command() -> CommandSpec {
        CommandSpec {
            name: "tool".to_string(),
            about: None,
            help: String::new(),
            args: Vec::new(),
            subcommands: Vec::new(),
        }
    }

    fn ui_state() -> UiState {
        UiState {
            focus: Focus::Form,
            active_tab: ActiveTab::Options,
            last_non_help_tab: ActiveTab::Options,
            selected_arg_index: 0,
            search_query: String::new(),
            editors: crate::editor_state::EditorState::default(),
            dropdown_open: None,
            dropdown_scroll: 0,
            form_scroll: 0,
            hover: None,
            hover_tab: None,
            mouse_select: None,
        }
    }

    #[test]
    fn tab_bar_view_keeps_expected_labels_and_order() {
        let command = command();
        let vm = ScreenView {
            command: &command,
            tree_items: Vec::new(),
            visible_tabs: [ActiveTab::Options, ActiveTab::Arguments, ActiveTab::Help],
            active_args: Vec::new(),
            preview_argv: Vec::new(),
            inputs: None,
        };

        let view = build_tab_bar_view(&ui_state(), &TuiConfig::default(), &vm);
        let labels = view.items.iter().map(|item| item.label.as_str()).collect::<Vec<_>>();

        assert_eq!(labels, vec![" Options ", " Arguments ", " Help "]);
    }

    #[test]
    fn tab_bar_view_maps_active_and_hovered_styles() {
        let command = command();
        let vm = ScreenView {
            command: &command,
            tree_items: Vec::new(),
            visible_tabs: [ActiveTab::Options, ActiveTab::Arguments, ActiveTab::Help],
            active_args: Vec::new(),
            preview_argv: Vec::new(),
            inputs: None,
        };
        let mut ui = ui_state();
        ui.active_tab = ActiveTab::Arguments;
        ui.hover_tab = Some(ActiveTab::Help);
        let config = TuiConfig::default();

        let view = build_tab_bar_view(&ui, &config, &vm);

        assert_eq!(view.items[1].style.fg, Some(config.theme.panel_bg));
        assert_eq!(view.items[1].style.bg, Some(config.theme.accent));
        assert!(view.items[1].style.add_modifier.contains(Modifier::BOLD));
        assert_eq!(view.items[2].style.fg, Some(config.theme.text));
        assert_eq!(view.items[2].style.bg, Some(config.theme.surface_raised));
        assert!(view.items[2].style.add_modifier.contains(Modifier::BOLD));
        assert_eq!(view.items[0].style.fg, Some(config.theme.dim));
        assert_eq!(view.items[0].style.bg, Some(config.theme.surface_raised));
    }
}
