use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, BorderType, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Tabs,
};

use crate::config::TuiConfig;
use crate::form_editor;
use crate::frame_snapshot::{FormFieldLayout, FrameSnapshot, TabButtonLayout};
use crate::input::{ActiveTab, ArgValue, Focus, UiState};
use crate::spec::CommandPath;
use crate::spec::{ArgSpec, choice_value_matches_default};
use crate::view::form::{self, field_metrics};

use super::{dropdown, screen::ScreenView, styles};

const TAB_CONTENT_TOP_PADDING: u16 = 1;
const TAB_PADDING_LEFT: &str = " ";
const TAB_PADDING_RIGHT: &str = " ";
const TAB_DIVIDER: &str = "│";

pub(crate) fn populate_layout(
    ui: &UiState,
    area: Rect,
    vm: &ScreenView<'_>,
    frame_snapshot: &mut FrameSnapshot,
) {
    let show_tabs = vm.visible_tabs.len() > 1;
    let mut content_area = area;
    let content_height = match ui.active_tab {
        ActiveTab::Help => form::measure_help_height(&vm.command.help),
        _ => form::measure_fields_height(&vm.active_args),
    };
    let tab_layouts = if show_tabs {
        let tabs_rect = Rect::new(area.x, area.y, area.width, 1);
        let content_offset = 1 + TAB_CONTENT_TOP_PADDING;
        content_area = Rect::new(
            area.x,
            area.y.saturating_add(content_offset),
            area.width,
            area.height.saturating_sub(content_offset),
        );
        layout_tab_bar(tabs_rect, vm)
    } else {
        Vec::new()
    };
    let viewport_height = content_area.height;
    frame_snapshot.form_scroll_max = content_height.saturating_sub(viewport_height);
    let form_scroll = ui.form_scroll(frame_snapshot);
    let frame_layout = &mut frame_snapshot.layout;
    frame_layout.form = Some(area);
    frame_layout.dropdown = None;
    frame_layout.form_fields.clear();
    frame_layout.form_inputs.clear();
    frame_layout.form_tabs = tab_layouts;
    frame_layout.form_view = Some(content_area);

    if ui.active_tab == ActiveTab::Help {
        return;
    }

    let mut y = i32::from(content_area.y) - i32::from(form_scroll);
    for item in &vm.active_args {
        let metrics = field_metrics(item.arg);
        let item_bottom = y + i32::from(metrics.total_height);
        if y >= i32::from(content_area.y) + i32::from(content_area.height) {
            break;
        }
        if item_bottom <= i32::from(content_area.y) {
            y += i32::from(metrics.total_height);
            continue;
        }
        let label = if metrics.label_height > 0 {
            clipped_rect(area.x, area.width, y, metrics.label_height, content_area)
        } else {
            None
        };
        let input_y = y + i32::from(metrics.label_height);
        let Some(input) = clipped_rect(
            area.x,
            area.width,
            input_y,
            metrics.input_height,
            content_area,
        ) else {
            y += i32::from(metrics.total_height);
            continue;
        };
        let description = field_help_text(item.arg)
            .map(|_| {
                clipped_rect(
                    area.x,
                    area.width,
                    input_y + i32::from(metrics.input_height),
                    metrics.description_height.max(1),
                    content_area,
                )
            })
            .flatten();

        frame_layout.form_inputs.insert(item.arg.id.clone(), input);
        frame_layout.form_fields.push(FormFieldLayout {
            arg_id: item.arg.id.clone(),
            label,
            input,
            description,
        });

        if ui.dropdown_open.as_deref() == Some(&item.arg.id) {
            frame_layout.dropdown = frame_layout
                .form_view
                .and_then(|form_view| {
                    dropdown::dropdown_layout(form_view, input, item.arg.choices.len())
                })
                .map(|layout| layout.rect);
        }

        y += i32::from(metrics.total_height);
    }
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

pub(crate) fn render_form(
    frame: &mut Frame<'_>,
    ui: &UiState,
    selected_path: &CommandPath,
    config: &TuiConfig,
    vm: &ScreenView<'_>,
    frame_snapshot: &FrameSnapshot,
) {
    let Some(area) = frame_snapshot.layout.form else {
        return;
    };
    let frame_layout = &frame_snapshot.layout;
    let show_tabs = !frame_layout.form_tabs.is_empty();
    if show_tabs {
        render_tab_bar(
            frame,
            ui,
            config,
            Rect::new(area.x, area.y, area.width, 1),
            vm,
        );
    }

    let content_area = frame_layout.form_view.unwrap_or(area);
    let content_height = match ui.active_tab {
        ActiveTab::Help => form::measure_help_height(&vm.command.help),
        _ => form::measure_fields_height(&vm.active_args),
    };
    let viewport_height = content_area.height;
    let form_scroll = ui.form_scroll(frame_snapshot);

    match ui.active_tab {
        ActiveTab::Help => render_help(frame, config, content_area, form_scroll, &vm.command.help),
        _ => render_fields(frame, ui, selected_path, config, vm, frame_snapshot),
    }

    if content_height > viewport_height {
        let scroll_steps = usize::from(frame_snapshot.form_scroll_max.saturating_add(1));
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

fn layout_tab_bar(area: Rect, vm: &ScreenView<'_>) -> Vec<TabButtonLayout> {
    let mut cursor_x = area.x;
    let right = area.x.saturating_add(area.width);
    vm.visible_tabs
        .iter()
        .enumerate()
        .filter_map(|(index, tab)| {
            let width = tab_render_width(*tab);
            let layout = TabButtonLayout {
                tab: *tab,
                rect: intersect_rects(Rect::new(cursor_x, area.y, width, 1), area)?,
            };
            cursor_x = cursor_x.saturating_add(width);
            if index + 1 < vm.visible_tabs.len() {
                cursor_x = cursor_x.saturating_add(tab_divider_width());
            }
            if cursor_x >= right {
                return Some(layout);
            }
            Some(layout)
        })
        .collect()
}

fn render_tab_bar(
    frame: &mut Frame<'_>,
    ui: &UiState,
    config: &TuiConfig,
    area: Rect,
    vm: &ScreenView<'_>,
) {
    frame.render_widget(build_tabs(ui, config, vm), area);
}

fn tab_label(tab: ActiveTab) -> &'static str {
    match tab {
        ActiveTab::Options => "Options",
        ActiveTab::Arguments => "Arguments",
        ActiveTab::Help => "Help",
    }
}

fn tab_title(ui: &UiState, config: &TuiConfig, tab: ActiveTab) -> Line<'static> {
    let style = if ui.hover_tab == Some(tab) && ui.active_tab != tab {
        Style::default()
            .fg(config.theme.text)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    Line::from(Span::styled(tab_label(tab).to_string(), style))
}

fn selected_tab_index(ui: &UiState, vm: &ScreenView<'_>) -> usize {
    vm.visible_tabs
        .iter()
        .position(|tab| *tab == ui.active_tab)
        .unwrap_or_default()
}

fn build_tabs<'a>(ui: &UiState, config: &TuiConfig, vm: &'a ScreenView<'a>) -> Tabs<'static> {
    let titles = vm
        .visible_tabs
        .iter()
        .map(|tab| tab_title(ui, config, *tab))
        .collect::<Vec<_>>();
    Tabs::new(titles)
        .select(selected_tab_index(ui, vm))
        .style(Style::default().fg(config.theme.dim))
        .highlight_style(
            Style::default()
                .fg(config.theme.panel_bg)
                .bg(config.theme.accent)
                .add_modifier(Modifier::BOLD),
        )
        .divider(Span::raw(TAB_DIVIDER))
        .padding(TAB_PADDING_LEFT, TAB_PADDING_RIGHT)
}

fn tab_render_width(tab: ActiveTab) -> u16 {
    usize_to_u16(
        TAB_PADDING_LEFT.chars().count()
            + tab_label(tab).chars().count()
            + TAB_PADDING_RIGHT.chars().count(),
    )
}

fn tab_divider_width() -> u16 {
    usize_to_u16(TAB_DIVIDER.chars().count())
}

fn usize_to_u16(width: usize) -> u16 {
    u16::try_from(width).unwrap_or(u16::MAX)
}

#[allow(clippy::too_many_lines)]
fn render_fields(
    frame: &mut Frame<'_>,
    ui: &UiState,
    selected_path: &CommandPath,
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

        if let Some(label_rect) = field.label {
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
            render_flag_toggle(
                frame,
                config,
                field.input,
                selected,
                &item.arg.display_name,
                &value,
                text_style,
            );
        } else if item.arg.uses_choice_input() {
            let display = if value.is_empty() {
                "Select..."
            } else {
                value.as_str()
            };
            let input = Paragraph::new(enum_display_line(
                config,
                display,
                field.input.width,
                selected,
                is_default,
                ui.dropdown_open.as_deref() == Some(&item.arg.id),
            ))
            .style(styles::compact_control(config, selected));
            frame.render_widget(input, field.input);
        } else if input_is_truncated {
            frame.render_widget(
                Paragraph::new(value).style(fill_style.patch(text_style)),
                field.input,
            );
        } else if selected {
            let editor = form_editor::editor_for_render(ui, selected_path, item.arg, &value);
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
            frame.render_widget(textarea.widget(), field.input);
            place_textarea_cursor(frame, &textarea, field.input);
        } else {
            frame.render_widget(
                Paragraph::new(value)
                    .block(block)
                    .style(fill_style.patch(text_style)),
                field.input,
            );
        }

        if let (Some(help), Some(help_rect)) = (field_help_text(item.arg), field.description) {
            frame.render_widget(
                Paragraph::new(Line::from(Span::raw(help))).style(styles::help(config)),
                help_rect,
            );
        }
    }
}

fn text_input_is_truncated(arg: &ArgSpec, input: Rect) -> bool {
    arg.accepts_text_input() && input.height < field_metrics(arg).input_height
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
    frame.render_widget(
        Paragraph::new(line).style(styles::flag_toggle(config, selected)),
        area,
    );
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
    use ratatui::buffer::Buffer;
    use ratatui::style::{Modifier, Style};
    use ratatui::widgets::Widget;

    use super::{
        build_tabs, populate_layout, selected_tab_index, tab_render_width, tab_title,
        text_input_is_truncated,
    };
    use crate::config::TuiConfig;
    use crate::frame_snapshot::FrameSnapshot;
    use crate::input::{ActiveTab, Focus, UiState};
    use crate::spec::{ArgKind, ArgSpec, CommandSpec, ValueCardinality};
    use crate::ui::screen::ScreenView;
    use crate::view::form::visible_args;

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
        }
    }

    #[test]
    fn tab_titles_keep_expected_labels_and_order() {
        let command = command();
        let vm = ScreenView {
            command: &command,
            tree_items: Vec::new(),
            visible_tabs: [ActiveTab::Options, ActiveTab::Arguments, ActiveTab::Help],
            active_args: Vec::new(),
            preview_argv: Vec::new(),
            inputs: None,
        };

        let labels = vm
            .visible_tabs
            .iter()
            .map(|tab| tab_title(&ui_state(), &TuiConfig::default(), *tab).to_string())
            .collect::<Vec<_>>();

        assert_eq!(labels, vec!["Options", "Arguments", "Help"]);
    }

    #[test]
    fn selected_tab_index_follows_active_tab() {
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
        ui.active_tab = ActiveTab::Help;

        assert_eq!(selected_tab_index(&ui, &vm), 2);
    }

    #[test]
    fn hovered_unselected_tab_title_is_bold() {
        let mut ui = ui_state();
        ui.hover_tab = Some(ActiveTab::Arguments);

        let title = tab_title(&ui, &TuiConfig::default(), ActiveTab::Arguments);
        let span = title.spans.first().expect("tab title span");

        assert!(span.style.add_modifier.contains(Modifier::BOLD));
        assert_eq!(span.style.fg, Some(TuiConfig::default().theme.text));
    }

    #[test]
    fn hovered_selected_tab_title_uses_default_style() {
        let mut ui = ui_state();
        ui.hover_tab = Some(ActiveTab::Options);

        let title = tab_title(&ui, &TuiConfig::default(), ActiveTab::Options);
        let span = title.spans.first().expect("tab title span");

        assert_eq!(span.style, Style::default());
    }

    #[test]
    fn layout_phase_populates_tab_and_form_view_geometry() {
        let command = command();
        let vm = ScreenView {
            command: &command,
            tree_items: Vec::new(),
            visible_tabs: [ActiveTab::Options, ActiveTab::Arguments, ActiveTab::Help],
            active_args: Vec::new(),
            preview_argv: Vec::new(),
            inputs: None,
        };
        let mut snapshot = FrameSnapshot::default();

        populate_layout(
            &ui_state(),
            ratatui::layout::Rect::new(2, 3, 40, 12),
            &vm,
            &mut snapshot,
        );

        assert_eq!(snapshot.layout.form_tabs.len(), 3);
        assert_eq!(
            snapshot.layout.form_view,
            Some(ratatui::layout::Rect::new(2, 5, 40, 10))
        );
        assert_eq!(
            snapshot.layout.form_tabs[0].rect,
            ratatui::layout::Rect::new(2, 3, tab_render_width(ActiveTab::Options), 1)
        );
        assert_eq!(
            snapshot.layout.form_tabs[1].rect.x,
            2 + tab_render_width(ActiveTab::Options) + 1
        );
    }

    #[test]
    fn layout_phase_clips_tab_geometry_to_visible_area() {
        let command = command();
        let vm = ScreenView {
            command: &command,
            tree_items: Vec::new(),
            visible_tabs: [ActiveTab::Options, ActiveTab::Arguments, ActiveTab::Help],
            active_args: Vec::new(),
            preview_argv: Vec::new(),
            inputs: None,
        };
        let mut snapshot = FrameSnapshot::default();

        populate_layout(
            &ui_state(),
            ratatui::layout::Rect::new(2, 3, 12, 6),
            &vm,
            &mut snapshot,
        );

        assert_eq!(snapshot.layout.form_tabs.len(), 2);
        assert_eq!(
            snapshot.layout.form_tabs[0].rect,
            ratatui::layout::Rect::new(2, 3, 9, 1)
        );
        assert_eq!(
            snapshot.layout.form_tabs[1].rect,
            ratatui::layout::Rect::new(12, 3, 2, 1)
        );
    }

    #[test]
    fn tabs_render_with_stock_padding_and_dividers() {
        let command = command();
        let vm = ScreenView {
            command: &command,
            tree_items: Vec::new(),
            visible_tabs: [ActiveTab::Options, ActiveTab::Arguments, ActiveTab::Help],
            active_args: Vec::new(),
            preview_argv: Vec::new(),
            inputs: None,
        };
        let area = ratatui::layout::Rect::new(0, 0, 32, 1);
        let mut buffer = Buffer::empty(area);

        Widget::render(
            build_tabs(&ui_state(), &TuiConfig::default(), &vm),
            area,
            &mut buffer,
        );

        assert_eq!(buffer[(0, 0)].symbol(), " ");
        assert_eq!(buffer[(1, 0)].symbol(), "O");
        assert_eq!(buffer[(8, 0)].symbol(), " ");
        assert_eq!(buffer[(9, 0)].symbol(), "│");
        assert_eq!(buffer[(10, 0)].symbol(), " ");
        assert_eq!(buffer[(11, 0)].symbol(), "A");
    }

    #[test]
    fn layout_phase_clips_scrolled_fields_to_form_view() {
        let command = CommandSpec {
            name: "tool".to_string(),
            about: None,
            help: String::new(),
            args: vec![
                option_arg("target", "--target"),
                option_arg("output", "--output"),
                option_arg("mode", "--mode"),
            ],
            subcommands: Vec::new(),
        };
        let vm = ScreenView {
            command: &command,
            tree_items: Vec::new(),
            visible_tabs: [ActiveTab::Options, ActiveTab::Arguments, ActiveTab::Help],
            active_args: visible_args(&command, ActiveTab::Options),
            preview_argv: Vec::new(),
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
}
