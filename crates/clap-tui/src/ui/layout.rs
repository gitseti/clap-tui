use ratatui::layout::{Constraint, Direction, Layout, Margin, Rect};

use crate::config::TuiConfig;
use crate::frame_snapshot::{self, FrameSnapshot};
use crate::input::UiState;

use super::{footer, screen::ScreenView, sidebar};

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
    let mut sidebar_width =
        u16::try_from(u32::from(size.width) * u32::from(config.layout.sidebar_ratio) / 100)
            .unwrap_or(size.width);
    sidebar_width = sidebar_width.clamp(22, 30);

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
    let main_inner = main_area.inner(Margin {
        horizontal: 1,
        vertical: 1,
    });
    let header_height = main_inner.height.min(2);
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
    frame_snapshot::populate_form_layout(
        ui,
        areas.form,
        &vm.active_args,
        &vm.command.help,
        &vm.validation,
        &mut snapshot,
    );
    footer::populate_layout(ui, footer_area, &mut snapshot.layout);

    ScreenLayout { areas, snapshot }
}
