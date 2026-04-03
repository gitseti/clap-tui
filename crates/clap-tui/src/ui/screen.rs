use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders};
use std::collections::HashSet;

use crate::config::TuiConfig;
use crate::frame_snapshot::FrameSnapshot;
use crate::input::{ActiveTab, AppState, CommandFormState, Focus, UiState};
use crate::pipeline::{self, EffectiveArgValue, ValidationState};
use crate::query::{
    form,
    tree::{self, TreeRow},
};
use crate::spec::CommandSpec;

use super::{dropdown, footer, form as form_ui, header, layout, preview, sidebar, styles, toast};

#[derive(Debug, Clone)]
pub(crate) struct ScreenView<'a> {
    pub(crate) command: &'a CommandSpec,
    pub(crate) root: &'a CommandSpec,
    pub(crate) tree_rows: Vec<TreeRow>,
    pub(crate) active_args: Vec<form::OrderedArg<'a>>,
    pub(crate) preview_argv: Vec<String>,
    pub(crate) validation: ValidationState,
    pub(crate) effective_values: std::collections::BTreeMap<String, EffectiveArgValue>,
    pub(crate) inputs: Option<CommandFormState>,
}

impl<'a> ScreenView<'a> {
    pub(crate) fn build(
        command: &'a CommandSpec,
        root: &'a CommandSpec,
        expanded: &HashSet<String>,
        search_query: &str,
        active_tab: ActiveTab,
        inputs: Option<CommandFormState>,
        derived: pipeline::DerivedState,
    ) -> Self {
        Self {
            command,
            root,
            tree_rows: tree::tree_rows(root, expanded, search_query),
            active_args: form::visible_args(command, active_tab),
            preview_argv: derived.argv,
            validation: derived.validation,
            effective_values: derived.effective_values,
            inputs,
        }
    }
}

pub(crate) fn render(
    frame: &mut Frame<'_>,
    state: &mut AppState,
    config: &TuiConfig,
) -> FrameSnapshot {
    let size = frame.area();

    let background = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(styles::panel_border(config, false))
        .style(styles::panel(config));
    frame.render_widget(background, size);
    let derived = state.derived().clone();
    let selected_path = state.domain.selected_path().clone();
    let vm = ScreenView::build(
        state.domain.current_command(),
        &state.domain.root,
        &state.domain.expanded,
        &state.ui.search_query,
        state.ui.active_tab,
        state.domain.current_form(),
        derived,
    );
    let screen_layout = layout::build_screen_layout(&state.ui, config, size, &vm);
    let frame_snapshot = screen_layout.snapshot.clone();
    render_main(
        frame,
        &state.ui,
        config,
        screen_layout.areas.main,
        screen_layout.areas.header,
        &vm,
        &frame_snapshot,
    );
    sidebar::render_sidebar(
        frame,
        &state.ui,
        &selected_path,
        config,
        screen_layout.areas.sidebar,
        &vm,
        &frame_snapshot.layout,
    );
    dropdown::render_dropdown(
        frame,
        &state.ui,
        &frame_snapshot,
        &state.domain,
        config,
        Rect::default(),
        &vm,
    );
    preview::render_preview(frame, &state.ui, config, screen_layout.areas.preview, &vm);
    footer::render_footer(
        frame,
        &state.ui,
        config,
        screen_layout.areas.footer,
        &vm,
        &frame_snapshot.layout,
    );
    toast::render_toast(frame, state, config, size);
    frame_snapshot
}

#[allow(clippy::too_many_arguments)]
fn render_main(
    frame: &mut Frame<'_>,
    ui: &UiState,
    config: &TuiConfig,
    area: Rect,
    header_area: Rect,
    vm: &ScreenView<'_>,
    frame_snapshot: &FrameSnapshot,
) {
    let workspace_focused = matches!(ui.focus, Focus::Form);
    let workspace = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(styles::panel_border(config, workspace_focused))
        .title(workspace_title(config, vm.command))
        .style(styles::panel(config));
    frame.render_widget(workspace, area);
    if header_area.height == 0 || header_area.width == 0 {
        return;
    }

    header::render_header(frame, config, header_area, vm);
    form_ui::render_form(frame, ui, config, vm, frame_snapshot);
}

fn workspace_title(config: &TuiConfig, command: &CommandSpec) -> Line<'static> {
    Line::from(vec![
        Span::raw(" "),
        Span::styled(command.name.clone(), Style::default().fg(config.theme.text)),
        Span::raw(" "),
    ])
}

#[cfg(test)]
mod tests {
    use clap::{Arg, ArgAction, Command, builder::ArgPredicate};
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui::layout::{Position, Rect};

    use super::render;
    use crate::TuiConfig;
    use crate::frame_snapshot::FrameSnapshot;
    use crate::input::AppState;
    use crate::pipeline;
    use crate::runtime::{AppKeyCode, AppKeyEvent, AppKeyModifiers};

    fn render_app(state: &mut AppState) -> (TestBackend, FrameSnapshot) {
        let mut terminal = Terminal::new(TestBackend::new(100, 24)).expect("terminal");
        let mut snapshot = None;
        terminal
            .draw(|frame| {
                snapshot = Some(render(frame, state, &TuiConfig::default()));
            })
            .expect("draw");
        (
            terminal.backend().clone(),
            snapshot.expect("frame snapshot"),
        )
    }

    fn buffer_text(backend: &TestBackend) -> String {
        backend
            .buffer()
            .content
            .iter()
            .map(ratatui::buffer::Cell::symbol)
            .collect::<String>()
    }

    fn count_occurrences(haystack: &str, needle: &str) -> usize {
        haystack.match_indices(needle).count()
    }

    fn rect_text(backend: &TestBackend, area: Rect) -> String {
        let mut lines = Vec::new();
        for y in area.y..area.y + area.height {
            let mut line = String::new();
            for x in area.x..area.x + area.width {
                line.push_str(backend.buffer()[(x, y)].symbol());
            }
            lines.push(line);
        }
        lines.join("\n")
    }

    fn cell_fg(backend: &TestBackend, x: u16, y: u16) -> ratatui::style::Color {
        backend.buffer()[(x, y)].fg
    }

    fn key(code: AppKeyCode) -> AppKeyEvent {
        AppKeyEvent::new(code, AppKeyModifiers::default())
    }

    #[test]
    fn required_field_error_renders_inline_and_footer_summary_stays_visible() {
        let mut state = AppState::from_command(
            &Command::new("tool").arg(Arg::new("name").long("name").required(true)),
        );

        let (backend, snapshot) = render_app(&mut state);
        let rendered = buffer_text(&backend);
        let field = snapshot
            .layout
            .form_fields
            .first()
            .expect("required field layout");
        let config = TuiConfig::default();

        assert!(rendered.contains("Required argument"));
        assert!(rendered.contains("Missing required argument: --name"));
        let label = field.label.expect("label rect");
        assert_eq!(cell_fg(&backend, label.x, label.y), config.theme.error);
        assert_eq!(
            cell_fg(&backend, field.input.x, field.input.y),
            config.theme.error
        );
    }

    #[test]
    fn conflict_errors_render_inline_for_both_fields_and_match_footer_summary() {
        let mut state = AppState::from_command(
            &Command::new("tool")
                .arg(
                    Arg::new("debug")
                        .long("debug")
                        .action(ArgAction::SetTrue)
                        .conflicts_with("quiet"),
                )
                .arg(Arg::new("quiet").long("quiet").action(ArgAction::SetTrue)),
        );
        state.domain.toggle_flag_touched("debug");
        state.domain.toggle_flag_touched("quiet");

        let (backend, snapshot) = render_app(&mut state);
        let rendered = buffer_text(&backend);
        let summary = "Conflicting arguments: --debug, --quiet";

        assert_eq!(count_occurrences(&rendered, summary), 3);
        for field in &snapshot.layout.form_fields {
            assert!(field.description.is_some());
            assert!(rendered.contains(summary));
        }
    }

    #[test]
    fn invalid_value_error_renders_inline_and_matches_footer_summary() {
        let mut state = AppState::from_command(
            &Command::new("tool").arg(
                Arg::new("color")
                    .long("color")
                    .action(ArgAction::Set)
                    .value_parser(["red", "green"]),
            ),
        );
        state.domain.set_text_value("color", "orange");

        let (backend, snapshot) = render_app(&mut state);
        let rendered = buffer_text(&backend);
        let field = snapshot
            .layout
            .form_fields
            .first()
            .expect("color field layout");
        let summary = "Invalid value for --color: orange";

        assert_eq!(count_occurrences(&rendered, summary), 2);
        assert!(field.description.is_some());
    }

    #[test]
    fn default_missing_source_renders_without_inventing_preview_tokens() {
        let mut state = AppState::from_command(
            &Command::new("tool").arg(
                Arg::new("color")
                    .long("color")
                    .action(ArgAction::Set)
                    .default_value("auto")
                    .num_args(0..=1)
                    .require_equals(true)
                    .default_missing_value("always"),
            ),
        );
        state.domain.toggle_optional_value_flag("color", true);

        let (backend, snapshot) = render_app(&mut state);
        let rendered = buffer_text(&backend);
        let preview = rect_text(
            &backend,
            snapshot
                .layout
                .preview
                .expect("preview area should be present"),
        );

        assert!(rendered.contains("Default-missing"));
        assert!(rendered.contains("implicit: always"));
        assert!(preview.contains("$ tool --color"));
        assert!(!preview.contains("always"));
    }

    #[test]
    fn conditional_default_source_renders_without_inventing_preview_tokens() {
        let mut state = AppState::from_command(
            &Command::new("tool")
                .arg(Arg::new("flag").long("flag").action(ArgAction::SetTrue))
                .arg(Arg::new("mode").long("mode").default_value_if(
                    "flag",
                    ArgPredicate::IsPresent,
                    Some("auto"),
                )),
        );
        state.domain.toggle_flag_touched("flag");

        let (backend, snapshot) = render_app(&mut state);
        let rendered = buffer_text(&backend);
        let preview = rect_text(
            &backend,
            snapshot
                .layout
                .preview
                .expect("preview area should be present"),
        );

        assert!(rendered.contains("Conditional"));
        assert!(rendered.contains("auto"));
        assert!(preview.contains("$ tool --flag"));
        assert!(!preview.contains("--mode"));
    }

    #[test]
    fn selected_text_input_places_terminal_cursor_after_typed_text() {
        let mut state =
            AppState::from_command(&Command::new("tool").arg(Arg::new("config").long("config")));
        state.ui.focus_form();

        let arg = state
            .domain
            .current_command()
            .args
            .first()
            .cloned()
            .expect("config arg");
        crate::form_editor::apply_key_to_text_field(&mut state, &arg, key(AppKeyCode::Char('a')));
        crate::form_editor::apply_key_to_text_field(&mut state, &arg, key(AppKeyCode::Char('b')));

        let (mut backend, snapshot) = render_app(&mut state);
        let input = snapshot
            .layout
            .form_fields
            .first()
            .expect("config field")
            .input;

        backend.assert_cursor_position(Position::new(input.x + 3, input.y + 1));
    }

    #[test]
    fn inherited_text_input_keeps_cursor_position_when_rendered_from_descendant_command() {
        let mut state = AppState::from_command(
            &Command::new("tool")
                .arg(Arg::new("config").long("config").global(true))
                .subcommand(Command::new("admin")),
        );
        state
            .select_command_path(&["admin".to_string()])
            .expect("valid admin path");
        state.ui.focus_form();

        let arg = state
            .domain
            .current_command()
            .args
            .iter()
            .find(|arg| arg.id == "config")
            .cloned()
            .expect("inherited config arg");
        crate::form_editor::apply_key_to_text_field(&mut state, &arg, key(AppKeyCode::Char('a')));
        crate::form_editor::apply_key_to_text_field(&mut state, &arg, key(AppKeyCode::Char('b')));

        let (mut backend, snapshot) = render_app(&mut state);
        let input = snapshot
            .layout
            .form_fields
            .iter()
            .find(|field| field.arg_id == "config")
            .expect("config field")
            .input;

        backend.assert_cursor_position(Position::new(input.x + 3, input.y + 1));
    }

    #[test]
    fn redraw_only_changes_reuse_cached_derived_state() {
        pipeline::reset_validation_call_count();

        let mut state = AppState::from_command(
            &Command::new("tool").arg(Arg::new("name").long("name").required(true)),
        );

        let _ = render_app(&mut state);
        assert_eq!(pipeline::validation_call_count(), 1);

        state.ui.focus_search();
        let _ = render_app(&mut state);
        assert_eq!(pipeline::validation_call_count(), 1);

        state.domain.set_text_value("name", "codex");
        let _ = render_app(&mut state);
        assert_eq!(pipeline::validation_call_count(), 2);
    }

    #[test]
    fn trailing_argv_render_does_not_paint_previous_positional() {
        let mut state = AppState::from_command(
            &Command::new("tool")
                .arg(Arg::new("program").required(true).index(1))
                .arg(
                    Arg::new("argv")
                        .index(2)
                        .action(ArgAction::Append)
                        .num_args(1..)
                        .trailing_var_arg(true)
                        .allow_hyphen_values(true),
                ),
        );
        state.ui.focus_form();
        state.ui.selected_arg_index = 1;

        let argv_arg = state
            .domain
            .current_command()
            .args
            .iter()
            .find(|arg| arg.id == "argv")
            .cloned()
            .expect("argv arg");
        crate::form_editor::apply_key_to_text_field(
            &mut state,
            &argv_arg,
            key(AppKeyCode::Char('a')),
        );

        let (backend, snapshot) = render_app(&mut state);
        let program_input = snapshot
            .layout
            .form_fields
            .iter()
            .find(|field| field.arg_id == "program")
            .expect("program field")
            .input;
        let argv_input = snapshot
            .layout
            .form_fields
            .iter()
            .find(|field| field.arg_id == "argv")
            .expect("argv field")
            .input;

        let program_text = rect_text(&backend, program_input);
        let argv_text = rect_text(&backend, argv_input);

        assert!(!program_text.contains('a'));
        assert!(argv_text.contains('a'));
    }
}
