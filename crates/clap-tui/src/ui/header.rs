use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};

use crate::config::TuiConfig;
use crate::input::AppState;

use super::screen::ScreenView;
use super::styles;

pub(crate) fn render_header(
    frame: &mut Frame<'_>,
    state: &mut AppState,
    config: &TuiConfig,
    area: Rect,
    vm: &ScreenView,
) {
    let breadcrumb = if state.selected_path.is_empty() {
        vm.command.name.clone()
    } else {
        let mut parts = vec![vm.command.name.clone()];
        parts.extend(state.selected_path.iter().cloned());
        parts.join(" > ")
    };
    let title = Span::styled(
        vm.command.name.clone(),
        Style::default()
            .fg(config.theme.accent)
            .add_modifier(Modifier::BOLD),
    );
    let desc = Span::styled(
        vm.command.about.clone().unwrap_or_default(),
        Style::default().fg(config.theme.dim),
    );
    let crumb = Span::styled(
        format!("  |  {breadcrumb}"),
        Style::default().fg(config.theme.dim),
    );
    let line = Line::from(vec![title, Span::raw("  "), desc, crumb]);
    let header = Paragraph::new(line).style(styles::header(config)).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(config.theme.border))
            .style(styles::header(config)),
    );
    frame.render_widget(header, area);
}
