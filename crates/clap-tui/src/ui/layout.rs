use ratatui::layout::{Constraint, Direction, Layout, Margin, Rect};

use crate::config::TuiConfig;
use crate::frame_snapshot::FrameSnapshot;
use crate::input::UiState;

use super::{footer, form, screen::ScreenView, sidebar};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LayoutMode {
    Roomy,
    Compact,
}

impl LayoutMode {
    pub(crate) fn for_size(size: Rect) -> Self {
        if size.height < 20 || size.width < 80 {
            Self::Compact
        } else {
            Self::Roomy
        }
    }

    pub(crate) fn is_compact(self) -> bool {
        matches!(self, Self::Compact)
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ScreenAreas {
    pub(crate) sidebar: Rect,
    pub(crate) main: Rect,
    pub(crate) header: Rect,
    pub(crate) form: Rect,
    pub(crate) preview: Rect,
    pub(crate) footer: Rect,
}

#[derive(Debug, Clone)]
pub(crate) struct ScreenLayout {
    pub(crate) areas: ScreenAreas,
    pub(crate) snapshot: FrameSnapshot,
}

pub(crate) fn build_screen_layout(
    ui: &UiState,
    config: &TuiConfig,
    size: Rect,
    vm: &ScreenView<'_>,
) -> ScreenLayout {
    let mode = LayoutMode::for_size(size);
    let mut sidebar_width =
        u16::try_from(u32::from(size.width) * u32::from(config.layout.sidebar_ratio) / 100)
            .unwrap_or(size.width);
    sidebar_width = if mode.is_compact() {
        sidebar_width.clamp(18, 24)
    } else {
        sidebar_width.clamp(22, 30)
    };
    let preview_height = if mode.is_compact() { 1 } else { 3 };

    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),
            Constraint::Length(preview_height),
            Constraint::Length(1),
        ])
        .split(size);

    let body_area = vertical[0];
    let preview_row_area = vertical[1];
    let footer_area = vertical[2];
    let root = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(
                sidebar_width
                    .max(if mode.is_compact() { 18 } else { 20 })
                    .min(body_area.width.saturating_sub(20)),
            ),
            Constraint::Min(20),
        ])
        .split(body_area);

    let sidebar_area = root[0];
    let main_area = root[1];
    let main_inner = main_area.inner(Margin {
        horizontal: 1,
        vertical: 0,
    });
    let preview_area = Rect::new(
        main_inner.x,
        preview_row_area.y,
        preview_row_area
            .x
            .saturating_add(preview_row_area.width)
            .saturating_sub(main_inner.x),
        preview_row_area.height,
    );
    let header_height = if super::header::has_header_content(vm.command) {
        super::header::header_height(vm.command, mode.is_compact())
    } else {
        0
    };
    let main_sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(header_height), Constraint::Min(0)])
        .split(main_inner);

    let areas = ScreenAreas {
        sidebar: sidebar_area,
        main: main_area,
        header: main_sections[0],
        form: Rect::new(
            main_inner.x,
            main_inner.y.saturating_add(header_height),
            main_inner.width,
            main_inner.height.saturating_sub(header_height),
        ),
        preview: preview_area,
        footer: footer_area,
    };

    let mut snapshot = FrameSnapshot::default();
    snapshot.layout.preview = Some(preview_area);
    snapshot.layout.footer = Some(footer_area);
    sidebar::populate_layout(sidebar_area, vm, &mut snapshot.layout);
    form::populate_layout(ui, areas.form, vm, &mut snapshot);
    footer::populate_layout(ui, footer_area, &mut snapshot.layout);

    ScreenLayout { areas, snapshot }
}

#[cfg(test)]
mod tests {
    use crate::input::{ActiveTab, Focus, UiState};
    use crate::pipeline::ValidationState;
    use crate::spec::{CommandPath, CommandSpec};

    use super::{LayoutMode, build_screen_layout};
    use crate::config::TuiConfig;
    use crate::ui::screen::ScreenView;
    use ratatui::layout::Rect;

    fn ui_state() -> UiState {
        UiState {
            focus: Focus::Sidebar,
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

    fn command(about: Option<&str>) -> CommandSpec {
        CommandSpec {
            name: "tool".to_string(),
            version: None,
            about: about.map(str::to_string),
            help: String::new(),
            args: Vec::new(),
            subcommands: Vec::new(),
            ..CommandSpec::default()
        }
    }

    fn view(command: &CommandSpec) -> ScreenView<'_> {
        ScreenView {
            command,
            root: command,
            selected_path: CommandPath::default(),
            tree_rows: Vec::new(),
            sidebar_scroll: 0,
            active_args: Vec::new(),
            authoritative_argv: vec!["tool".to_string()],
            rendered_command: None,
            validation: ValidationState::default(),
            effective_values: std::collections::BTreeMap::new(),
            field_semantics: std::collections::BTreeMap::new(),
            inputs: None,
        }
    }

    #[test]
    fn layout_mode_switches_at_compact_thresholds() {
        assert_eq!(
            LayoutMode::for_size(Rect::new(0, 0, 80, 20)),
            LayoutMode::Roomy
        );
        assert_eq!(
            LayoutMode::for_size(Rect::new(0, 0, 79, 20)),
            LayoutMode::Compact
        );
        assert_eq!(
            LayoutMode::for_size(Rect::new(0, 0, 80, 19)),
            LayoutMode::Compact
        );
    }

    #[test]
    fn compact_layout_preserves_header_but_keeps_preview_compact() {
        let command = command(Some("Run the selected tool"));
        let layout = build_screen_layout(
            &ui_state(),
            &TuiConfig::default(),
            Rect::new(0, 0, 70, 18),
            &view(&command),
        );

        assert_eq!(layout.areas.header.height, 2);
        assert_eq!(layout.areas.preview.height, 1);
        assert_eq!(layout.areas.preview.x, layout.areas.form.x);
        assert_eq!(layout.areas.preview.x + layout.areas.preview.width, 70);
        assert_eq!(layout.areas.footer.height, 1);
    }

    #[test]
    fn roomy_layout_keeps_header_when_description_is_available() {
        let command = command(Some("Run the selected tool"));
        let layout = build_screen_layout(
            &ui_state(),
            &TuiConfig::default(),
            Rect::new(0, 0, 100, 24),
            &view(&command),
        );

        assert_eq!(layout.areas.header.height, 3);
        assert_eq!(layout.areas.preview.height, 3);
        assert_eq!(layout.areas.preview.x, layout.areas.form.x);
        assert_eq!(layout.areas.preview.x + layout.areas.preview.width, 100);
        assert_eq!(layout.areas.footer.height, 1);
    }
}
