use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, BorderType, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
};

use crate::config::TuiConfig;
use crate::form_editor;
use crate::frame_snapshot::{FormFieldLayout, FrameSnapshot, dropdown_geometry};
use crate::input::{ArgValue, Focus, UiState};
use crate::query::form::{self, field_description_offset, field_input_offset, field_metrics};
use crate::spec::CommandPath;
use crate::spec::{ArgSpec, choice_value_matches_default};

use super::{screen::ScreenView, styles};

pub(crate) fn populate_layout(
    ui: &UiState,
    area: Rect,
    vm: &ScreenView<'_>,
    frame_snapshot: &mut FrameSnapshot,
) {
    let content_area = area;
    let content_height = form::measure_fields_height(&vm.active_args);
    let help_height = form::measure_help_height(&vm.command.help);
    let viewport_height = content_area.height;
    let help_viewport_height = viewport_height.saturating_sub(4);
    frame_snapshot.form_scroll_max = content_height.saturating_sub(viewport_height);
    frame_snapshot.help_scroll_max = help_height.saturating_sub(help_viewport_height);
    let form_scroll = ui.form_scroll(frame_snapshot);
    let frame_layout = &mut frame_snapshot.layout;
    frame_layout.form = Some(area);
    frame_layout.dropdown = None;
    frame_layout.form_fields.clear();
    frame_layout.form_inputs.clear();
    frame_layout.form_tabs.clear();
    frame_layout.form_view = Some(content_area);

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
        let input_y = y + i32::from(field_input_offset(item.arg));
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
            .and_then(|_| {
                let description_y = y + i32::from(field_description_offset(item.arg)?);
                clipped_rect(
                    area.x,
                    area.width,
                    description_y,
                    metrics.description_height.max(1),
                    content_area,
                )
            });

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
                .and_then(|form_view| dropdown_geometry(form_view, input, item.arg.choices.len()))
                .map(|geometry| geometry.rect);
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
    let content_area = frame_layout.form_view.unwrap_or(area);
    let content_height = form::measure_fields_height(&vm.active_args);
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

    render_fields(frame, ui, selected_path, config, vm, frame_snapshot);

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
            .thumb_style(
                Style::default()
                    .fg(config.theme.panel_focus_border)
                    .add_modifier(Modifier::BOLD),
            )
            .track_style(Style::default().fg(config.theme.dim));
        frame.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
    }
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
        let shows_choice_placeholder = item.arg.uses_choice_input() && current_value.is_none();
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
        let text_style = if is_default || shows_choice_placeholder {
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
                is_default || shows_choice_placeholder,
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

fn render_help_overlay(
    frame: &mut Frame<'_>,
    config: &TuiConfig,
    area: Rect,
    scroll: u16,
    help: &str,
) {
    let popup = area.inner(ratatui::layout::Margin {
        horizontal: 1,
        vertical: 1,
    });
    let block = Block::default()
        .title(Line::from(vec![
            Span::raw(" "),
            Span::raw("Help"),
            Span::raw(" "),
        ]))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(config.theme.panel_focus_border))
        .style(styles::panel(config));
    let inner = popup.inner(ratatui::layout::Margin {
        horizontal: 1,
        vertical: 1,
    });
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
            .thumb_style(
                Style::default()
                    .fg(config.theme.panel_focus_border)
                    .add_modifier(Modifier::BOLD),
            )
            .track_style(Style::default().fg(config.theme.dim));
        frame.render_stateful_widget(scrollbar, inner, &mut scrollbar_state);
    }
}

fn field_help_text(arg: &ArgSpec) -> Option<String> {
    arg.help.clone().or_else(|| arg.value_hint.clone())
}

#[cfg(test)]
mod tests {
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    use super::{populate_layout, text_input_is_truncated};
    use crate::frame_snapshot::FrameSnapshot;
    use crate::input::{ActiveTab, Focus, UiState};
    use crate::spec::{ArgKind, ArgSpec, CommandSpec, ValueCardinality};
    use crate::ui::form::render_form;
    use crate::ui::screen::ScreenView;
    use crate::query::form::visible_args;
    use crate::TuiConfig;

    fn command() -> CommandSpec {
        CommandSpec {
            name: "tool".to_string(),
            version: None,
            about: None,
            help: String::new(),
            args: Vec::new(),
            subcommands: Vec::new(),
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
            tree_items: Vec::new(),
            active_args: Vec::new(),
            preview_argv: Vec::new(),
            validation: crate::pipeline::ValidationState::default(),
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
            tree_items: Vec::new(),
            active_args: Vec::new(),
            preview_argv: Vec::new(),
            validation: crate::pipeline::ValidationState::default(),
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
        };
        let vm = ScreenView {
            command: &command,
            root: &command,
            tree_items: Vec::new(),
            active_args: Vec::new(),
            preview_argv: Vec::new(),
            validation: crate::pipeline::ValidationState::default(),
            inputs: None,
        };
        let mut snapshot = FrameSnapshot::default();

        populate_layout(
            &ui_state(),
            ratatui::layout::Rect::new(2, 3, 40, 8),
            &vm,
            &mut snapshot,
        );

        assert_eq!(snapshot.help_scroll_max, 6);
    }

    #[test]
    fn help_overlay_skips_rendering_fields_underneath() {
        let mut command = command();
        command.help = "Command help".to_string();
        command.args = vec![option_arg("config", "--config")];
        let vm = ScreenView {
            command: &command,
            root: &command,
            tree_items: Vec::new(),
            active_args: visible_args(&command, ActiveTab::Inputs),
            preview_argv: Vec::new(),
            validation: crate::pipeline::ValidationState::default(),
            inputs: None,
        };
        let mut snapshot = FrameSnapshot::default();
        let mut ui = ui_state();
        ui.help_open = true;

        populate_layout(&ui, ratatui::layout::Rect::new(0, 0, 40, 8), &vm, &mut snapshot);

        let mut terminal = Terminal::new(TestBackend::new(40, 8)).expect("terminal");
        terminal
            .draw(|frame| {
                render_form(
                    frame,
                    &ui,
                    &Vec::<String>::new().into(),
                    &TuiConfig::default(),
                    &vm,
                    &snapshot,
                );
            })
            .expect("draw");

        let rendered = buffer_text(terminal.backend());
        assert!(rendered.contains("Command help"));
        assert!(!rendered.contains("--config"));
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
        };
        let vm = ScreenView {
            command: &command,
            root: &command,
            tree_items: Vec::new(),
            active_args: visible_args(&command, ActiveTab::Inputs),
            preview_argv: Vec::new(),
            validation: crate::pipeline::ValidationState::default(),
            inputs: None,
        };
        let mut snapshot = FrameSnapshot::default();

        populate_layout(
            &ui_state(),
            ratatui::layout::Rect::new(0, 0, 40, 8),
            &vm,
            &mut snapshot,
        );

        let field = snapshot
            .layout
            .form_fields
            .first()
            .expect("field layout");
        let label = field.label.expect("label rect");
        let description = field.description.expect("description rect");

        assert_eq!(description.y, label.y + label.height);
        assert_eq!(field.input.y, description.y + description.height);
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
        };
        let vm = ScreenView {
            command: &command,
            root: &command,
            tree_items: Vec::new(),
            active_args: visible_args(&command, ActiveTab::Inputs),
            preview_argv: Vec::new(),
            validation: crate::pipeline::ValidationState::default(),
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
        };
        let vm = ScreenView {
            command: &command,
            root: &command,
            tree_items: Vec::new(),
            active_args: visible_args(&command, ActiveTab::Inputs),
            preview_argv: Vec::new(),
            validation: crate::pipeline::ValidationState::default(),
            inputs: None,
        };
        let mut snapshot = FrameSnapshot::default();
        let ui = ui_state();

        populate_layout(&ui, ratatui::layout::Rect::new(0, 0, 40, 8), &vm, &mut snapshot);

        let mut terminal = Terminal::new(TestBackend::new(40, 8)).expect("terminal");
        terminal
            .draw(|frame| {
                render_form(
                    frame,
                    &ui,
                    &Vec::<String>::new().into(),
                    &TuiConfig::default(),
                    &vm,
                    &snapshot,
                );
            })
            .expect("draw");

        assert!(buffer_text(terminal.backend()).contains("Select..."));
    }
}
