use ratatui::{Frame, style::Style, widgets::Paragraph};

use crate::{
    config::TuiConfig,
    frame_snapshot::FrameSnapshot,
    input::{HoverTarget, UiState},
};

pub(crate) fn render(
    frame: &mut Frame<'_>,
    ui: &UiState,
    config: &TuiConfig,
    snapshot: &FrameSnapshot,
) {
    let Some(divider) = snapshot.layout.sidebar_divider else {
        return;
    };
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
