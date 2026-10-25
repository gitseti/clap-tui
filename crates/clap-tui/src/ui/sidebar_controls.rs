use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    widgets::Paragraph,
};

use crate::{
    config::TuiConfig,
    frame_snapshot::{FrameLayout, FrameSnapshot},
    input::{HoverTarget, UiState},
};

pub(crate) fn populate_layout(sidebar: Rect, main: Rect, header: Rect, layout: &mut FrameLayout) {
    if sidebar.width > 0 && main.width > 0 && main.height > 0 {
        layout.sidebar_divider = Some(Rect::new(main.x, main.y, 1, main.height));
    }
    if header.width > 0 && header.height > 0 {
        let width = crate::sidebar::TOGGLE_WIDTH.min(header.width);
        layout.sidebar_toggle = Some(Rect::new(header.x, header.y, width, 1));
    }
}

pub(crate) fn render(
    frame: &mut Frame<'_>,
    ui: &UiState,
    config: &TuiConfig,
    snapshot: &FrameSnapshot,
) {
    if let Some(divider) = snapshot.layout.sidebar_divider {
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
    }
    if let Some(toggle) = snapshot.layout.sidebar_toggle {
        let hovered = ui.hover == Some(HoverTarget::SidebarToggle);
        frame.render_widget(
            Paragraph::new(if ui.sidebar.collapsed {
                " › "
            } else {
                " ‹ "
            })
            .style(if hovered {
                Style::default()
                    .fg(config.theme.accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(config.theme.metadata)
            }),
            toggle,
        );
    }
}
