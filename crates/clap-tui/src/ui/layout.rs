use ratatui::layout::{Constraint, Direction, Layout, Margin, Rect};

use super::preview::PreviewLayout;
use crate::config::TuiConfig;
use crate::frame_snapshot::FrameSnapshot;
use crate::input::{HoverTarget, UiState};

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
    pub(crate) footer: Rect,
}

#[derive(Debug, Clone)]
pub(crate) struct ScreenLayout {
    // Expanded previews have no underlying workspace areas.
    pub(crate) areas: Option<ScreenAreas>,
    pub(crate) snapshot: FrameSnapshot,
    pub(crate) preview: PreviewLayout,
}

pub(crate) fn build_screen_layout(
    ui: &UiState,
    config: &TuiConfig,
    size: Rect,
    vm: &ScreenView<'_>,
) -> ScreenLayout {
    if ui.preview_open {
        let preview = PreviewLayout::expanded(
            config,
            vm.rendered_command.as_ref(),
            ui.preview_scroll,
            size,
        );
        let mut snapshot = FrameSnapshot::default();
        preview.populate_snapshot(&mut snapshot);
        return ScreenLayout {
            areas: None,
            snapshot,
            preview,
        };
    }
    let mode = LayoutMode::for_size(size);
    let mut sidebar_width =
        u16::try_from(u32::from(size.width) * u32::from(config.layout.sidebar_ratio) / 100)
            .unwrap_or(size.width);
    sidebar_width = if mode.is_compact() {
        sidebar_width.clamp(18, 24)
    } else {
        sidebar_width.clamp(22, 30)
    };
    let preview_height = if mode.is_compact() { 2 } else { 4 };

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
    let sidebar_width = ui.sidebar.width(body_area.width, sidebar_width);
    let sidebar_area = Rect::new(body_area.x, body_area.y, sidebar_width, body_area.height);
    let main_area = Rect::new(
        body_area.x.saturating_add(sidebar_width),
        body_area.y,
        body_area.width.saturating_sub(sidebar_width),
        body_area.height,
    );
    let main_inner = main_area.inner(Margin {
        horizontal: 1,
        vertical: 0,
    });
    let preview_area = preview_row_area;
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
        footer: footer_area,
    };

    let mut snapshot = FrameSnapshot::default();
    let preview = PreviewLayout::dock(
        config,
        vm.rendered_command.as_ref(),
        ui.hover == Some(HoverTarget::Preview),
        preview_area,
    );
    preview.populate_snapshot(&mut snapshot);
    snapshot.layout.footer = Some(footer_area);
    snapshot.layout.workspace = Some(body_area);
    super::sidebar_controls::populate_layout(
        sidebar_area,
        main_area,
        areas.header,
        &mut snapshot.layout,
    );
    if sidebar_area.width > 0 {
        sidebar::populate_layout(sidebar_area, vm, &mut snapshot.layout);
    }
    form::populate_layout(ui, areas.form, vm, &mut snapshot);
    footer::populate_layout(ui, footer_area, &vm.validation, &mut snapshot.layout);

    ScreenLayout {
        areas: Some(areas),
        snapshot,
        preview,
    }
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
            preview_open: false,
            preview_scroll: 0,
            selected_arg_index: 0,
            search_query: String::new(),
            editors: crate::editor_state::EditorState::default(),
            dropdown_open: None,
            dropdown_scroll: 0,
            dropdown_cursor: 0,
            sidebar: crate::sidebar::SidebarState::default(),
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

        let areas = layout.areas.expect("workspace layout");
        let preview = layout.snapshot.layout.preview.expect("preview layout");
        assert_eq!(areas.header.x, areas.form.x);
        assert_eq!(areas.header.height, 2);
        assert_eq!(preview.height, 2);
        assert_eq!(preview.x, 0);
        assert_eq!(preview.right(), 70);
        assert_eq!(areas.footer.height, 1);
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

        let areas = layout.areas.expect("workspace layout");
        let preview = layout.snapshot.layout.preview.expect("preview layout");
        assert_eq!(areas.header.x, areas.form.x);
        assert_eq!(areas.header.height, 3);
        assert_eq!(preview.height, 4);
        assert_eq!(preview.x, 0);
        assert_eq!(preview.right(), 100);
        assert_eq!(areas.footer.height, 1);
    }

    #[test]
    fn roomy_threshold_and_slightly_larger_sizes_keep_full_width_dock_geometry() {
        for size in [Rect::new(0, 0, 80, 20), Rect::new(0, 0, 80, 21)] {
            let layout = build_screen_layout(
                &ui_state(),
                &TuiConfig::default(),
                size,
                &view(&command(Some("Run the selected tool"))),
            );

            let preview = layout.snapshot.layout.preview.expect("preview layout");
            assert_eq!(LayoutMode::for_size(size), LayoutMode::Roomy);
            assert_eq!(preview.height, 4);
            assert_eq!(preview.x, 0);
            assert_eq!(preview.width, size.width);
        }
    }

    #[test]
    fn layout_snapshot_uses_live_validation_for_footer_status_hit_targets() {
        let command = command(Some("Run the selected tool"));
        let mut screen_view = view(&command);
        screen_view.validation = ValidationState {
            issues: vec![crate::pipeline::ValidationIssue::arguments(
                "Provide --name",
                Vec::new(),
            )],
        };

        let layout = build_screen_layout(
            &ui_state(),
            &TuiConfig::default(),
            Rect::new(0, 0, 100, 24),
            &screen_view,
        );

        assert!(
            layout
                .snapshot
                .layout
                .footer_buttons
                .iter()
                .any(|button| button.target == crate::input::HoverTarget::FooterStatus)
        );
    }
}
