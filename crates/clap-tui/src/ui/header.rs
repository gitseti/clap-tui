use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::config::TuiConfig;
use crate::input::AppState;

use super::{screen::ScreenView, styles};

pub(crate) fn render_header(
    frame: &mut Frame<'_>,
    state: &mut AppState,
    config: &TuiConfig,
    area: Rect,
    vm: &ScreenView,
) {
    let breadcrumb = if state.command.selected_path.is_empty() {
        vm.command.name.clone()
    } else {
        let mut parts = vec![vm.command.name.clone()];
        parts.extend(state.command.selected_path.iter().cloned());
        parts.join(" > ")
    };
    let mut lines = Vec::with_capacity(2);
    let mut title_line = vec![Span::styled(
        vm.command.name.clone(),
        Style::default()
            .fg(config.theme.text)
            .add_modifier(Modifier::BOLD),
    )];
    if let Some(about) = vm.command.about.as_ref().filter(|about| !about.is_empty()) {
        title_line.push(Span::raw("  "));
        title_line.push(Span::styled(
            about.clone(),
            Style::default().fg(config.theme.dim),
        ));
    }
    lines.push(Line::from(title_line));
    lines.push(Line::from(Span::styled(
        breadcrumb,
        Style::default().fg(config.theme.dim),
    )));

    frame.render_widget(Paragraph::new(lines).style(styles::header(config)), area);
}
