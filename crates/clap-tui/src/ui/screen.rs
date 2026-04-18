use crate::config::TuiConfig;
use crate::frame_snapshot::FrameSnapshot;
use crate::input::{AppState, CommandFormState, Focus, UiState};
use crate::pipeline::{self, EffectiveArgValue, ValidationState};
use crate::query::{form, selectors, tree::TreeRow};
use crate::spec::{CommandPath, CommandSpec};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Paragraph};

use super::{dropdown, footer, form as form_ui, header, layout, preview, sidebar, styles, toast};

#[derive(Debug, Clone)]
pub(crate) struct ScreenView<'a> {
    pub(crate) command: &'a CommandSpec,
    pub(crate) root: &'a CommandSpec,
    pub(crate) selected_path: CommandPath,
    pub(crate) tree_rows: Vec<TreeRow>,
    pub(crate) sidebar_scroll: usize,
    pub(crate) active_args: Vec<form::OrderedArg<'a>>,
    pub(crate) preview_argv: Vec<String>,
    pub(crate) validation: ValidationState,
    pub(crate) effective_values: std::collections::BTreeMap<String, EffectiveArgValue>,
    pub(crate) inputs: Option<CommandFormState>,
}

impl<'a> ScreenView<'a> {
    pub(crate) fn from_state(state: &'a AppState, derived: pipeline::DerivedState) -> Self {
        let command = state.domain.current_command();
        let root = &state.domain.root;
        let tree_rows =
            selectors::visible_sidebar_rows(root, &state.domain.expanded, &state.ui.search_query);
        Self {
            command,
            root,
            selected_path: state.domain.selected_path().clone(),
            sidebar_scroll: 0,
            tree_rows,
            active_args: selectors::visible_form_args(
                root,
                state.domain.selected_path(),
                state.ui.active_tab,
            ),
            preview_argv: derived.argv,
            validation: derived.validation,
            effective_values: derived.effective_values,
            inputs: state.domain.current_form(),
        }
    }
}

pub(crate) fn render(
    frame: &mut Frame<'_>,
    state: &mut AppState,
    config: &TuiConfig,
) -> FrameSnapshot {
    let size = frame.area();

    let background = Block::default().style(styles::surface(config, styles::Surface::Shell));
    frame.render_widget(background, size);
    let derived = state.derived().clone();
    let mut vm = ScreenView::from_state(state, derived);
    vm.sidebar_scroll = state.ui.sidebar_scroll;
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
        &vm.selected_path,
        config,
        screen_layout.areas.sidebar,
        &vm,
        &frame_snapshot.layout,
    );
    render_preview_band_background(frame, config, size, screen_layout.areas.preview);
    if screen_layout.areas.form.x > 0 {
        let divider_height = screen_layout
            .areas
            .footer
            .y
            .saturating_sub(screen_layout.areas.sidebar.y);
        let divider_area = Rect::new(
            screen_layout.areas.form.x.saturating_sub(1),
            screen_layout.areas.sidebar.y,
            1,
            divider_height,
        );
        frame.render_widget(
            Paragraph::new(
                (0..divider_area.height)
                    .map(|_| "│")
                    .collect::<Vec<_>>()
                    .join("\n"),
            )
            .style(styles::surface(config, styles::Surface::Workspace).fg(config.theme.divider)),
            divider_area,
        );
    }
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

fn render_preview_band_background(
    frame: &mut Frame<'_>,
    config: &TuiConfig,
    size: Rect,
    preview_area: Rect,
) {
    if preview_area.height == 0 {
        return;
    }
    let band = Rect::new(0, preview_area.y, size.width, preview_area.height);
    frame.render_widget(
        Block::default().style(styles::surface(config, styles::Surface::Workspace)),
        band,
    );
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
    let workspace = Block::default().style(styles::surface(config, styles::Surface::Workspace));
    frame.render_widget(workspace, area);
    if header_area.height > 0 && header_area.width > 0 {
        header::render_header(frame, config, header_area, workspace_focused, vm);
    }
    form_ui::render_form(frame, ui, config, vm, frame_snapshot);
}

#[cfg(test)]
mod tests {
    use clap::{Arg, ArgAction, ArgGroup, Command, builder::ArgPredicate};
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui::layout::{Position, Rect};

    use super::render;
    use crate::TuiConfig;
    use crate::controller;
    use crate::frame_snapshot::FrameSnapshot;
    use crate::input::AppState;
    use crate::pipeline;
    use crate::runtime::{
        AppKeyCode, AppKeyEvent, AppKeyModifiers, AppMouseButton, AppMouseEvent, AppMouseEventKind,
    };
    use crate::update::{Effect, apply_action};

    fn render_app(state: &mut AppState) -> (TestBackend, FrameSnapshot) {
        render_app_with_size(state, 100, 24)
    }

    fn render_app_with_size(
        state: &mut AppState,
        width: u16,
        height: u16,
    ) -> (TestBackend, FrameSnapshot) {
        let mut terminal = Terminal::new(TestBackend::new(width, height)).expect("terminal");
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

    fn cell_bg(backend: &TestBackend, x: u16, y: u16) -> ratatui::style::Color {
        backend.buffer()[(x, y)].bg
    }

    fn key(code: AppKeyCode) -> AppKeyEvent {
        AppKeyEvent::new(code, AppKeyModifiers::default())
    }

    fn click(column: u16, row: u16) -> AppMouseEvent {
        AppMouseEvent {
            kind: AppMouseEventKind::Down(AppMouseButton::Left),
            column,
            row,
            modifiers: AppKeyModifiers::default(),
        }
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
    fn mixed_validation_errors_render_inline_for_all_fields() {
        let mut state = AppState::from_command(
            &Command::new("tool")
                .group(ArgGroup::new("mode").args(["debug", "quiet"]))
                .arg(Arg::new("name").long("name").required(true))
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
        assert_eq!(count_occurrences(&rendered, "Required argument"), 1);
        assert_eq!(
            snapshot
                .layout
                .form_fields
                .iter()
                .filter(|field| field.description.is_some())
                .count(),
            3
        );
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

        assert!(rendered.contains("implicit: always"));
        assert!(preview.contains("$ tool --color"));
        assert!(!preview.contains("always"));
    }

    #[test]
    fn untouched_set_true_flag_does_not_render_default_badge() {
        let mut state = AppState::from_command(
            &Command::new("tool").arg(Arg::new("upload").long("upload").action(ArgAction::SetTrue)),
        );

        let (backend, _) = render_app(&mut state);
        let rendered = buffer_text(&backend);

        assert!(!rendered.contains("Default"));
    }

    #[test]
    fn untouched_grouped_flag_does_not_render_default_badge_after_sibling_is_selected() {
        let mut state = AppState::from_command(
            &Command::new("tool")
                .arg(
                    Arg::new("fast")
                        .long("fast")
                        .action(ArgAction::SetTrue)
                        .group("mode"),
                )
                .arg(
                    Arg::new("safe")
                        .long("safe")
                        .action(ArgAction::SetTrue)
                        .group("mode"),
                ),
        );
        state.domain.toggle_flag_touched("safe");

        let (backend, _) = render_app(&mut state);
        let rendered = buffer_text(&backend);

        assert!(!rendered.contains("Default"));
    }

    #[test]
    fn untouched_count_flag_does_not_render_default_badge() {
        let mut state = AppState::from_command(
            &Command::new("tool").arg(Arg::new("verbose").short('v').action(ArgAction::Count)),
        );

        let (backend, _) = render_app(&mut state);
        let rendered = buffer_text(&backend);

        assert!(!rendered.contains("Default"));
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

        assert!(rendered.contains("auto"));
        assert!(preview.contains("$ tool --flag"));
        assert!(!preview.contains("--mode"));
    }

    #[test]
    fn sidebar_and_workspace_share_one_panel_fill_with_a_vertical_divider() {
        let mut state = AppState::from_command(
            &Command::new("tool")
                .about("Run tool")
                .arg(Arg::new("config").long("config"))
                .subcommand(Command::new("serve")),
        );
        let config = TuiConfig::default();

        state.ui.focus_sidebar();
        let (sidebar_backend, sidebar_snapshot) = render_app(&mut state);
        let sidebar_area = sidebar_snapshot.layout.sidebar.expect("sidebar area");
        let form_area = sidebar_snapshot.layout.form.expect("form area");
        let preview_area = sidebar_snapshot.layout.preview.expect("preview area");
        let sidebar_probe = (sidebar_area.x + 2, sidebar_area.y + sidebar_area.height - 2);
        let form_probe = (form_area.x + 2, form_area.y + form_area.height - 2);
        assert_eq!(
            cell_bg(&sidebar_backend, sidebar_probe.0, sidebar_probe.1),
            config.theme.workspace_bg
        );
        assert_eq!(
            cell_bg(&sidebar_backend, form_probe.0, form_probe.1),
            config.theme.workspace_bg
        );
        assert_eq!(
            sidebar_backend.buffer()[(form_area.x - 1, sidebar_area.y + 1)].symbol(),
            "│"
        );
        assert_eq!(
            cell_bg(&sidebar_backend, sidebar_area.x + 1, preview_area.y),
            config.theme.workspace_bg
        );
        assert_eq!(
            cell_bg(
                &sidebar_backend,
                preview_area
                    .x
                    .saturating_add(preview_area.width)
                    .saturating_sub(1),
                preview_area.y
            ),
            config.theme.header_bg
        );

        state.ui.focus_form();
        let (form_backend, form_snapshot) = render_app(&mut state);
        let sidebar_area = form_snapshot.layout.sidebar.expect("sidebar area");
        let form_area = form_snapshot.layout.form.expect("form area");
        let preview_area = form_snapshot.layout.preview.expect("preview area");
        let sidebar_probe = (sidebar_area.x + 2, sidebar_area.y + sidebar_area.height - 2);
        let form_probe = (form_area.x + 2, form_area.y + form_area.height - 2);
        assert_eq!(
            cell_bg(&form_backend, sidebar_probe.0, sidebar_probe.1),
            config.theme.workspace_bg
        );
        assert_eq!(
            cell_bg(&form_backend, form_probe.0, form_probe.1),
            config.theme.workspace_bg
        );
        assert_eq!(
            form_backend.buffer()[(form_area.x - 1, sidebar_area.y + 1)].symbol(),
            "│"
        );
        assert_eq!(
            cell_bg(&form_backend, sidebar_area.x + 1, preview_area.y),
            config.theme.workspace_bg
        );
        assert_eq!(
            cell_bg(
                &form_backend,
                preview_area
                    .x
                    .saturating_add(preview_area.width)
                    .saturating_sub(1),
                preview_area.y
            ),
            config.theme.header_bg
        );
    }

    #[test]
    fn grouped_untouched_false_flags_share_muted_idle_label_treatment() {
        let mut state = AppState::from_command(
            &Command::new("tool")
                .arg(
                    Arg::new("fast")
                        .long("fast")
                        .action(ArgAction::SetTrue)
                        .group("mode"),
                )
                .arg(
                    Arg::new("safe")
                        .long("safe")
                        .action(ArgAction::SetTrue)
                        .group("mode"),
                ),
        );
        let config = TuiConfig::default();

        let (backend, snapshot) = render_app(&mut state);
        let fast = snapshot.layout.form_fields.first().expect("fast field");
        let safe = snapshot.layout.form_fields.get(1).expect("safe field");

        assert_eq!(
            cell_fg(&backend, fast.input.x + 6, fast.input.y),
            config.theme.metadata
        );
        assert_eq!(
            cell_fg(&backend, safe.input.x + 6, safe.input.y),
            config.theme.metadata
        );
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
        let (before_backend, before_snapshot) = render_app(&mut state);
        let program_input_before = before_snapshot
            .layout
            .form_fields
            .iter()
            .find(|field| field.arg_id == "program")
            .expect("program field")
            .input;
        let program_text_before = rect_text(&before_backend, program_input_before);

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
            key(AppKeyCode::Char('z')),
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

        assert_eq!(program_text, program_text_before);
        assert!(argv_text.contains('z'));
    }

    #[test]
    fn scrolled_sidebar_snapshot_uses_windowed_rows_for_hit_testing() {
        let mut subcommands = Vec::new();
        for index in 0..18 {
            subcommands.push(crate::spec::CommandSpec {
                name: format!("cmd-{index}"),
                version: None,
                about: None,
                help: String::new(),
                args: Vec::new(),
                subcommands: Vec::new(),
                ..crate::spec::CommandSpec::default()
            });
        }
        let mut state = AppState::new(crate::spec::CommandSpec {
            name: "tool".to_string(),
            version: None,
            about: None,
            help: String::new(),
            args: Vec::new(),
            subcommands,
            ..crate::spec::CommandSpec::default()
        });
        state.ui.sidebar_scroll = 6;

        let (_, snapshot) = render_app_with_size(&mut state, 80, 16);
        let first_row = snapshot
            .layout
            .sidebar_items
            .first()
            .expect("visible sidebar row")
            .clone();

        assert_eq!(first_row.path.as_slice(), &["cmd-6".to_string()]);

        let action = controller::handle_mouse_event(
            click(first_row.row.x, first_row.row.y),
            &state,
            &snapshot,
            &TuiConfig::default(),
        )
        .expect("sidebar click action");
        let effect = apply_action(&action, &mut state, &snapshot);

        assert_eq!(effect, Effect::None);
        assert_eq!(
            state.domain.selected_path().as_slice(),
            &["cmd-6".to_string()]
        );
    }
}
