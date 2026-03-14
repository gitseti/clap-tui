use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::config::TuiConfig;
use crate::input::{AppState, FooterButtonLayout, HoverTarget};

use super::screen::ScreenView;
use super::styles;

pub(crate) fn render_footer(
    frame: &mut Frame<'_>,
    state: &mut AppState,
    config: &TuiConfig,
    area: Rect,
    _vm: &ScreenView,
) {
    let chips = vec![
        (HoverTarget::Run, "⌃↩ Run"),
        (HoverTarget::Exit, "⌃C Exit"),
        (HoverTarget::Search, "/ Search"),
        (HoverTarget::Focus, "Tab Focus"),
    ];
    let mut spans = Vec::new();
    state.layout.footer_buttons.clear();
    let mut cursor_x = area.x;
    for (target, chip) in chips {
        let label = format!(" {chip} ");
        let width = label.chars().count() as u16;
        let hovered = state.hover == Some(target);
        let style = if hovered {
            Style::default()
                .fg(config.theme.panel_bg)
                .bg(config.theme.accent)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .fg(config.theme.accent)
                .bg(config.theme.pill_bg)
                .add_modifier(Modifier::BOLD)
        };
        spans.push(Span::styled(label.clone(), style));
        spans.push(Span::raw(" "));

        let rect = Rect::new(cursor_x, area.y, width, 1);
        state
            .layout
            .footer_buttons
            .push(FooterButtonLayout { target, rect });
        cursor_x = cursor_x.saturating_add(width + 1);
    }
    let footer = Paragraph::new(Line::from(spans)).style(styles::panel(config));
    frame.render_widget(footer, area);
}
