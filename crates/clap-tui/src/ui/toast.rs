use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph};

use crate::config::TuiConfig;
use crate::input::AppState;

pub(crate) fn render_toast(
    frame: &mut Frame<'_>,
    state: &mut AppState,
    config: &TuiConfig,
    area: Rect,
) {
    let Some(toast) = state.toast.as_ref() else {
        return;
    };

    let text = format!(" {} ", toast.message);
    let width = (text.chars().count() as u16 + 2).min(area.width.saturating_sub(2));
    let x = area
        .x
        .saturating_add(area.width.saturating_sub(width.saturating_add(1)));
    let y = area
        .y
        .saturating_add(area.height.saturating_sub(3))
        .max(area.y);
    let toast_area = Rect::new(x, y, width, 3);

    let border = if toast.is_error {
        config.theme.border
    } else {
        config.theme.accent
    };
    let text_style = if toast.is_error {
        Style::default().fg(config.theme.text)
    } else {
        Style::default()
            .fg(config.theme.accent)
            .add_modifier(Modifier::BOLD)
    };

    frame.render_widget(Clear, toast_area);
    frame.render_widget(
        Paragraph::new(Line::from(text))
            .style(text_style.bg(config.theme.panel_bg))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(border)),
            ),
        toast_area,
    );
}
