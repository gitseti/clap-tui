use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    widgets::{Block, Paragraph},
};

use crate::{
    config::TuiConfig,
    frame_snapshot::{FrameLayout, FrameSnapshot},
    input::{HoverTarget, UiState},
};

pub(crate) fn populate_layout(sidebar: Rect, main: Rect, layout: &mut FrameLayout) {
    if main.width == 0 || main.height == 0 {
        return;
    }
    layout.sidebar_divider = Some(Rect::new(main.x, main.y, 1, main.height));
    let toggle_width = crate::sidebar::TOGGLE_WIDTH.min(main.x.saturating_sub(sidebar.x));
    layout.sidebar_toggle = Some(Rect::new(main.x - toggle_width, main.y, toggle_width, 1));
}

pub(crate) fn render(
    frame: &mut Frame<'_>,
    ui: &UiState,
    config: &TuiConfig,
    snapshot: &FrameSnapshot,
) {
    let Some(divider) = snapshot.layout.sidebar_divider else {
        return;
    };
    if ui.sidebar.collapsed
        && let Some(toggle) = snapshot.layout.sidebar_toggle
    {
        frame.render_widget(
            Block::default().style(super::styles::surface(
                config,
                super::styles::Surface::Workspace,
            )),
            Rect::new(toggle.x, divider.y, toggle.width, divider.height),
        );
    }
    let active = ui.sidebar.is_resizing() || ui.hover == Some(HoverTarget::SidebarDivider);
    frame.render_widget(
        Paragraph::new(
            (0..divider.height)
                .map(|_| "│")
                .collect::<Vec<_>>()
                .join("\n"),
        )
        .style(
            Style::default()
                .bg(config.theme.workspace_bg)
                .fg(if active {
                    config.theme.accent
                } else {
                    config.theme.divider
                }),
        ),
        divider,
    );
    if let Some(toggle) = snapshot.layout.sidebar_toggle {
        let hovered = ui.hover == Some(HoverTarget::SidebarToggle);
        frame.render_widget(
            Paragraph::new(if ui.sidebar.collapsed {
                " › "
            } else {
                " ‹ "
            })
            .style(super::styles::compact_control_affordance(
                config, hovered, false,
            )),
            toggle,
        );
    }
}
