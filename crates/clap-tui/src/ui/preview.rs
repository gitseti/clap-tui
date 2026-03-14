use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};

use crate::config::TuiConfig;
use crate::input::AppState;

use super::screen::ScreenView;
use super::styles;

pub(crate) fn render_preview(
    frame: &mut Frame<'_>,
    state: &mut AppState,
    config: &TuiConfig,
    area: Rect,
    vm: &ScreenView,
) {
    let hovered = state.hover == Some(crate::input::HoverTarget::Preview);
    let command_style = if hovered {
        Style::default()
            .fg(config.theme.accent)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(config.theme.text)
    };
    let command_line = Line::from(vec![
        Span::styled("$ ", styles::help(config)),
        Span::styled(vm.preview_argv.join(" "), command_style),
    ]);
    let bar = Paragraph::new(vec![command_line])
        .style(if hovered {
            Style::default().bg(config.theme.input_bg)
        } else {
            styles::panel(config)
        })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(styles::panel_border(config, hovered)),
        );
    frame.render_widget(bar, area);
}
