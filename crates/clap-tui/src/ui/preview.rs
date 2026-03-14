use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};

use crate::config::TuiConfig;
use crate::input::AppState;

use super::screen::ScreenView;
use super::styles;

pub(crate) fn render_preview(
    frame: &mut Frame<'_>,
    _state: &mut AppState,
    config: &TuiConfig,
    area: Rect,
    vm: &ScreenView,
) {
    let command_line = Line::from(vec![
        Span::styled("$ ", styles::help(config)),
        Span::styled(
            vm.preview_argv.join(" "),
            Style::default().fg(config.theme.text),
        ),
    ]);
    let bar = Paragraph::new(vec![command_line])
        .style(styles::panel(config))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(styles::panel_border(config, false)),
        );
    frame.render_widget(bar, area);
}
