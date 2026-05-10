use std::collections::HashMap;

mod compact;
mod fields;
mod help;
mod optional_value;
mod repeated;
#[cfg(test)]
mod test_support;
mod text;

use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::{Scrollbar, ScrollbarOrientation, ScrollbarState};

use crate::config::TuiConfig;
use crate::frame_snapshot::FrameSnapshot;
use crate::input::{Focus, UiState};
use crate::layout::form as form_layout;
use crate::query::form::FieldWidget;
use crate::repeated_field::repeated_input_height;

use super::{screen::ScreenView, styles};

#[cfg(test)]
use crate::repeated_field::{repeated_add_rect, repeated_remove_rect, repeated_row_textarea_rect};
#[cfg(test)]
use help::{FieldHelpContext, field_help_text_for_test as field_help_text};

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
    let content_height = form_layout::measure_fields_height_with_layout_overrides_and_semantics(
        &vm.active_args,
        &vm.validation.field_errors,
        &input_height_overrides,
        &label_height_overrides,
        &vm.field_semantics,
    );
    let viewport_height = content_area.height;
    let form_scroll = ui.form_scroll(frame_snapshot);

    if ui.help_open {
        help::render_help_overlay(
            frame,
            config,
            area,
            ui.help_scroll(frame_snapshot),
            &vm.command.help,
        );
        return;
    }

    if content_area.width > 0 && viewport_height > 0 {
        let cursor = fields::render_fields(
            frame.buffer_mut(),
            ui,
            config,
            vm,
            &frame_snapshot.layout.form_fields,
            frame_snapshot.first_invalid_field_id(),
        );
        if let Some(cursor) = cursor {
            frame.set_cursor_position(cursor);
        }
    }

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
            let current_value = fields::effective_compatibility_value(vm, item.arg);
            let selected_values = fields::effective_selected_values(vm, item.arg);
            let value =
                fields::display_value(item.widget, current_value.as_ref(), &selected_values);
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

pub(super) fn blit(
    target: &mut Buffer,
    source: &Buffer,
    source_origin: (u16, u16),
    target_origin: (u16, u16),
    size: (u16, u16),
) {
    for dy in 0..size.1 {
        for dx in 0..size.0 {
            let Some(cell) = source.cell((
                source_origin.0.saturating_add(dx),
                source_origin.1.saturating_add(dy),
            )) else {
                continue;
            };
            if let Some(slot) = target.cell_mut((
                target_origin.0.saturating_add(dx),
                target_origin.1.saturating_add(dy),
            )) {
                *slot = cell.clone();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use clap::{Arg, ArgAction, Command, value_parser};
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    use super::{
        FieldHelpContext, FieldWidget, field_help_text, populate_layout, repeated_add_rect,
        repeated_remove_rect, repeated_row_textarea_rect,
    };
    use crate::TuiConfig;
    use crate::frame_snapshot::FrameSnapshot;
    use crate::input::{ActiveTab, AppState, Focus};
    use crate::query::form::{visible_args, visible_args_for_path, widget_for};
    use crate::spec::{CommandSpec, ValueCardinality};
    use crate::ui::form::render_form;
    use crate::ui::form::test_support::{
        buffer_text, cell_bg, cell_fg, choice_arg, command, option_arg, ui_state,
    };
    use crate::ui::screen::ScreenView;
    use ratatui::layout::Rect;

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

        assert_eq!(first.label.expect("label rect").x, 0);
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
    fn bottom_clipped_text_inputs_keep_their_visible_border_row() {
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
                < crate::layout::form::field_metrics(vm.active_args[0].arg).input_height
        );

        let mut terminal = Terminal::new(TestBackend::new(40, 2)).expect("terminal");
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
    fn bottom_clipped_compact_controls_render_visible_rows_without_overflow() {
        let command = CommandSpec::from_command(
            &Command::new("tool").arg(Arg::new("debug").long("debug").action(ArgAction::SetTrue)),
        );
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

        populate_layout(&ui, Rect::new(0, 0, 40, 1), &vm, &mut snapshot);
        let field = snapshot
            .layout
            .form_fields
            .first()
            .expect("clipped compact field layout");
        assert_eq!(field.input.height, 1);
        assert_eq!(field.input_clip_top, 0);

        let mut terminal = Terminal::new(TestBackend::new(40, 3)).expect("terminal");
        terminal
            .draw(|frame| {
                render_form(frame, &ui, &TuiConfig::default(), &vm, &snapshot);
            })
            .expect("draw");

        let rendered = buffer_text(terminal.backend());
        assert!(!rendered.contains("Disabled"));
        assert_eq!(
            terminal.backend().buffer()[(field.input.x, field.input.y)].symbol(),
            "╭"
        );
        assert_eq!(
            terminal.backend().buffer()[(field.input.x, field.input.y + 1)].symbol(),
            " "
        );
    }

    #[test]
    fn bottom_clipped_text_inputs_render_border_without_overflow() {
        let command = CommandSpec::from_command(
            &Command::new("tool").arg(
                Arg::new("allow_negative_integer")
                    .long("allow-negative-integer")
                    .default_value("0")
                    .allow_negative_numbers(true)
                    .value_parser(value_parser!(i32)),
            ),
        );
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

        populate_layout(&ui, Rect::new(0, 0, 50, 1), &vm, &mut snapshot);
        let field = snapshot
            .layout
            .form_fields
            .first()
            .expect("clipped text field layout");
        assert_eq!(field.input.height, 1);
        assert_eq!(field.input_clip_top, 0);

        let mut terminal = Terminal::new(TestBackend::new(50, 3)).expect("terminal");
        terminal
            .draw(|frame| {
                render_form(frame, &ui, &TuiConfig::default(), &vm, &snapshot);
            })
            .expect("draw");

        assert_eq!(
            terminal.backend().buffer()[(field.input.x, field.input.y)].symbol(),
            "╭"
        );
        assert_eq!(
            terminal.backend().buffer()[(field.input.x, field.input.y + 1)].symbol(),
            " "
        );
    }

    #[test]
    fn descriptions_render_when_their_content_row_enters_the_viewport() {
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
        let description = field.description.expect("visible description hit target");
        assert!(
            snapshot
                .form_field_at(description.x, description.y)
                .is_some_and(|hit| hit.arg_id == "include" && hit.in_description)
        );

        let mut terminal = Terminal::new(TestBackend::new(40, 4)).expect("terminal");
        terminal
            .draw(|frame| {
                render_form(frame, &ui, &TuiConfig::default(), &vm, &snapshot);
            })
            .expect("draw");

        let rendered = buffer_text(terminal.backend());
        assert!(rendered.contains("Multi-value path list"));
    }

    #[test]
    fn descriptions_render_when_input_is_fully_above_viewport() {
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
        ui.form_scroll = 8;

        populate_layout(&ui, Rect::new(0, 0, 40, 2), &vm, &mut snapshot);

        let field = snapshot
            .layout
            .form_fields
            .first()
            .expect("description-only field layout");
        assert_eq!(field.arg_id, "include");
        assert_eq!(field.input.height, 0);
        let description = field.description.expect("visible description");
        assert!(
            snapshot
                .form_field_at(description.x, description.y)
                .is_some_and(|hit| {
                    hit.arg_id == "include" && hit.in_description && !hit.in_input
                })
        );

        let mut terminal = Terminal::new(TestBackend::new(40, 2)).expect("terminal");
        terminal
            .draw(|frame| {
                render_form(frame, &ui, &TuiConfig::default(), &vm, &snapshot);
            })
            .expect("draw");

        let rendered = buffer_text(terminal.backend());
        assert!(rendered.contains("Multi-value path list"));
    }

    #[test]
    fn orphan_description_renders_adjacent_to_next_field_label() {
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
        ui.form_scroll = 3;

        populate_layout(&ui, Rect::new(0, 0, 40, 6), &vm, &mut snapshot);

        let define_layout = snapshot
            .layout
            .form_fields
            .iter()
            .find(|field| field.arg_id == "define")
            .expect("orphan description field layout");
        let include_layout = snapshot
            .layout
            .form_fields
            .iter()
            .find(|field| field.arg_id == "include")
            .expect("next visible field layout");
        let define_description = define_layout
            .description
            .expect("orphan description rect for define");
        let include_label = include_layout
            .label
            .expect("visible label rect for include");
        assert_eq!(define_layout.input.height, 0);
        assert!(
            define_description.y < include_label.y
                || define_description.y >= include_label.y + include_label.height,
            "orphan description should not overlap the next field's label row"
        );

        let mut terminal = Terminal::new(TestBackend::new(40, 6)).expect("terminal");
        terminal
            .draw(|frame| {
                render_form(frame, &ui, &TuiConfig::default(), &vm, &snapshot);
            })
            .expect("draw");

        let rendered = buffer_text(terminal.backend());
        assert!(rendered.contains("Key/value pairs"));
        assert!(rendered.contains("include"));
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

        let field = snapshot.layout.form_fields.first().expect("field layout");
        let first_row = Rect::new(field.input.x, field.input.y, field.input.width, 3);
        let second_row = Rect::new(field.input.x, field.input.y + 3, field.input.width, 3);
        let first_remove =
            repeated_remove_rect(first_row, true, false).expect("first remove control");
        let last_add = repeated_add_rect(second_row).expect("last add control");
        assert_eq!(
            cell_bg(terminal.backend(), first_remove.x, first_remove.y),
            TuiConfig::default().theme.accent
        );
        assert_eq!(
            cell_bg(terminal.backend(), last_add.x, last_add.y),
            TuiConfig::default().theme.accent
        );
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
    fn bottom_clipped_repeated_text_field_renders_only_visible_border_row() {
        let mut multi = option_arg("terminated-paths", "--terminated-paths");
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
        let ui = ui_state();

        populate_layout(&ui, Rect::new(0, 0, 60, 1), &vm, &mut snapshot);
        let field = snapshot
            .layout
            .form_fields
            .first()
            .expect("clipped repeated field layout");
        assert_eq!(field.input.height, 1);
        assert_eq!(field.input_clip_top, 0);

        let mut terminal = Terminal::new(TestBackend::new(60, 3)).expect("terminal");
        terminal
            .draw(|frame| {
                render_form(frame, &ui, &TuiConfig::default(), &vm, &snapshot);
            })
            .expect("draw");

        assert_eq!(
            terminal.backend().buffer()[(field.input.x, field.input.y)].symbol(),
            "╭"
        );
        assert!(!buffer_text(terminal.backend()).contains(" + "));
        assert_eq!(
            terminal.backend().buffer()[(field.input.x, field.input.y + 1)].symbol(),
            " "
        );
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
    fn repeated_text_cursor_on_clipped_row_is_not_placed_on_screen() {
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
        state.domain.set_text_value("include", "a\nb\nc");
        state.domain.mark_touched("include");
        let arg = state
            .domain
            .current_command()
            .args
            .iter()
            .find(|arg| arg.id == "include")
            .cloned()
            .expect("include arg");
        crate::form_editor::set_cursor_from_click(&mut state, &arg, 0, 1);
        let mut ui = ui_state();
        ui.editors = std::mem::take(&mut state.ui.editors);
        ui.form_scroll = 2;
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
        populate_layout(&ui, Rect::new(0, 0, 60, 3), &vm, &mut snapshot);
        let field = snapshot.layout.form_fields.first().expect("field layout");
        assert_eq!(field.input_clip_top, 2);

        let mut terminal = Terminal::new(TestBackend::new(60, 3)).expect("terminal");
        // Park the cursor outside the viewport so a "cursor never placed"
        // outcome is distinguishable from a "placed at (0, 0)" outcome.
        let sentinel = ratatui::layout::Position::new(59, 2);
        terminal.set_cursor_position(sentinel).expect("park cursor");
        terminal
            .draw(|frame| {
                render_form(frame, &ui, &TuiConfig::default(), &vm, &snapshot);
            })
            .expect("draw");

        let cursor = terminal.get_cursor_position().expect("cursor query");
        assert_eq!(
            cursor, sentinel,
            "cursor on a clipped repeated row must not be re-placed by the form renderer"
        );
        let _ = field;
    }

    #[test]
    fn repeated_text_cursor_on_visible_row_after_clip_lands_inside_viewport() {
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
        state.domain.set_text_value("include", "a\nb\nc");
        state.domain.mark_touched("include");
        let arg = state
            .domain
            .current_command()
            .args
            .iter()
            .find(|arg| arg.id == "include")
            .cloned()
            .expect("include arg");
        // Cursor on row 1 (the row whose content sits inside the viewport).
        crate::form_editor::set_cursor_from_click(&mut state, &arg, 1, 0);
        let mut ui = ui_state();
        ui.editors = std::mem::take(&mut state.ui.editors);
        ui.form_scroll = 2;
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
        populate_layout(&ui, Rect::new(0, 0, 60, 3), &vm, &mut snapshot);
        let field = snapshot.layout.form_fields.first().expect("field layout");

        let mut terminal = Terminal::new(TestBackend::new(60, 3)).expect("terminal");
        terminal
            .draw(|frame| {
                render_form(frame, &ui, &TuiConfig::default(), &vm, &snapshot);
            })
            .expect("draw");

        let cursor = terminal.get_cursor_position().expect("cursor placed");
        assert!(
            cursor.y >= field.input.y && cursor.y < field.input.y + field.input.height,
            "cursor for a visible row must land inside the viewport, got {cursor:?}"
        );
    }

    #[test]
    fn middle_clipped_repeated_text_renders_only_intersecting_rows() {
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
        state
            .domain
            .set_text_value("include", "alpha\nbeta\ngamma\ndelta\nepsilon");
        state.domain.mark_touched("include");
        let mut ui = ui_state();
        ui.editors = std::mem::take(&mut state.ui.editors);
        // start_row > 0 (clip_top above row boundary) and end_row < total_rows
        // (visible.bottom inside an interior row): partial clips on both ends.
        ui.form_scroll = 4;
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
        populate_layout(&ui, Rect::new(0, 0, 60, 5), &vm, &mut snapshot);

        let mut terminal = Terminal::new(TestBackend::new(60, 5)).expect("terminal");
        terminal
            .draw(|frame| {
                render_form(frame, &ui, &TuiConfig::default(), &vm, &snapshot);
            })
            .expect("draw");

        let rendered = buffer_text(terminal.backend());
        assert!(
            !rendered.contains("alpha"),
            "row above viewport must be clipped"
        );
        assert!(rendered.contains("beta"));
        assert!(rendered.contains("gamma"));
        assert!(!rendered.contains("delta") || !rendered.contains("epsilon"));
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
        ui.form_scroll = 3;
        populate_layout(&ui, Rect::new(0, 0, 60, 3), &vm, &mut snapshot);

        let mut terminal = Terminal::new(TestBackend::new(60, 3)).expect("terminal");
        terminal
            .draw(|frame| {
                render_form(frame, &ui, &TuiConfig::default(), &vm, &snapshot);
            })
            .expect("draw");

        let rendered = buffer_text(terminal.backend());
        assert!(!rendered.contains("alpha"));
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
