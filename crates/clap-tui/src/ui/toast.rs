use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph};

use crate::config::TuiConfig;
use crate::input::AppState;

use super::styles;

pub(crate) fn render_toast(
    frame: &mut Frame<'_>,
    state: &mut AppState,
    config: &TuiConfig,
    area: Rect,
) {
    let Some(toast) = state.notifications.toast.as_ref() else {
        return;
    };
    if area.width < 4 || area.height == 0 {
        return;
    }

    let height = area.height.min(3);
    let label = toast_label(toast.is_error);
    let text = if height >= 3 {
        format!(" {} ", toast.message)
    } else {
        format!(" {label} · {} ", toast.message)
    };
    let width = (u16::try_from(text.chars().count()).unwrap_or(area.width) + 2)
        .min(area.width.saturating_sub(2));
    let x = area
        .x
        .saturating_add(area.width.saturating_sub(width.saturating_add(1)));
    let y = area
        .y
        .saturating_add(area.height.saturating_sub(height))
        .max(area.y);
    let toast_area = Rect::new(x, y, width, height);

    let border = if toast.is_error {
        config.theme.error
    } else {
        config.theme.success
    };
    let text_style = if toast.is_error {
        Style::default()
            .fg(config.theme.error)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(config.theme.success)
            .add_modifier(Modifier::BOLD)
    };

    frame.render_widget(Clear, toast_area);
    if height >= 3 {
        frame.render_widget(
            Paragraph::new(Line::from(text))
                .style(text_style.bg(config.theme.overlay_bg))
                .block(
                    Block::default()
                        .title(toast_title(config, toast.is_error))
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(Style::default().fg(border)),
                ),
            toast_area,
        );
    } else {
        frame.render_widget(
            Paragraph::new(Line::from(text)).style(text_style.bg(config.theme.overlay_bg)),
            toast_area,
        );
    }
}

fn toast_label(is_error: bool) -> &'static str {
    if is_error { "Error" } else { "Success" }
}

fn toast_title(config: &TuiConfig, is_error: bool) -> Line<'static> {
    let label = toast_label(is_error);
    let label_style = if is_error {
        styles::status_chip(config, false)
    } else {
        styles::success_chip(config, false)
    };
    Line::from(vec![
        Span::styled(format!(" {label} "), label_style),
        Span::raw(" "),
        Span::styled("feedback", styles::help(config)),
    ])
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui::style::Modifier;

    use crate::input::{AppState, Toast};
    use crate::spec::CommandSpec;

    use super::{render_toast, toast_title};
    use crate::config::TuiConfig;

    fn state_with_toast(message: &str, is_error: bool) -> AppState {
        let mut state = AppState::new(CommandSpec {
            name: "tool".to_string(),
            version: None,
            about: None,
            help: String::new(),
            args: Vec::new(),
            subcommands: Vec::new(),
            ..CommandSpec::default()
        });
        state.notifications.toast = Some(Toast {
            message: message.to_string(),
            expires_at: Instant::now() + Duration::from_secs(30),
            is_error,
        });
        state
    }

    #[test]
    fn error_toast_uses_error_border_and_bold_error_text() {
        let mut state = state_with_toast("Clipboard unavailable", true);
        let config = TuiConfig::default();
        let mut terminal = Terminal::new(TestBackend::new(50, 6)).expect("terminal");

        terminal
            .draw(|frame| render_toast(frame, &mut state, &config, frame.area()))
            .expect("draw");

        let buffer = terminal.backend().buffer();
        let border_cell = &buffer[(27, 3)];
        let text_cell = &buffer[(29, 4)];

        assert_eq!(border_cell.fg, config.theme.error);
        assert_eq!(text_cell.fg, config.theme.error);
        assert!(text_cell.modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn success_toast_uses_success_border_and_bold_success_text() {
        let mut state = state_with_toast("Copied command to clipboard", false);
        let config = TuiConfig::default();
        let mut terminal = Terminal::new(TestBackend::new(60, 6)).expect("terminal");

        terminal
            .draw(|frame| render_toast(frame, &mut state, &config, frame.area()))
            .expect("draw");

        let buffer = terminal.backend().buffer();
        let border_cell = &buffer[(30, 3)];
        let text_cell = &buffer[(32, 4)];

        assert_eq!(border_cell.fg, config.theme.success);
        assert_eq!(text_cell.fg, config.theme.success);
        assert!(text_cell.modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn toast_title_explains_success_and_error_feedback() {
        let config = TuiConfig::default();

        let error_title = toast_title(&config, true);
        let success_title = toast_title(&config, false);

        assert!(error_title.spans[0].content.as_ref().contains("Error"));
        assert!(success_title.spans[0].content.as_ref().contains("Success"));
    }
}
