use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Clear, Paragraph};

use crate::config::TuiConfig;
use crate::input::AppState;

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
    let icon = toast_icon(toast.is_error);
    let text = format!(" {icon} {} ", toast.message);
    let width = u16::try_from(text.chars().count())
        .unwrap_or(area.width)
        .min(area.width);
    let x = area.x.saturating_add(area.width.saturating_sub(width));
    let y = area
        .y
        .saturating_add(area.height.saturating_sub(height))
        .max(area.y);
    let toast_area = Rect::new(x, y, width, height);

    let background = if toast.is_error {
        config.theme.error
    } else {
        config.theme.success
    };
    let toast_style = Style::default()
        .fg(config.theme.shell_bg)
        .bg(background)
        .add_modifier(Modifier::BOLD);
    let text_y = if height >= 3 {
        toast_area.y.saturating_add(1)
    } else {
        toast_area.y
    };
    let text_area = Rect::new(toast_area.x, text_y, toast_area.width, 1);

    frame.render_widget(Clear, toast_area);
    frame.render_widget(Block::default().style(toast_style), toast_area);
    frame.render_widget(
        Paragraph::new(Line::from(text)).style(toast_style),
        text_area,
    );
}

fn toast_icon(is_error: bool) -> &'static str {
    if is_error { "✕" } else { "✓" }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    use crate::input::{AppState, Toast};
    use crate::spec::CommandSpec;

    use super::render_toast;
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
    fn error_toast_uses_solid_error_background() {
        let mut state = state_with_toast("Clipboard unavailable", true);
        let config = TuiConfig::default();
        let mut terminal = Terminal::new(TestBackend::new(50, 6)).expect("terminal");

        terminal
            .draw(|frame| render_toast(frame, &mut state, &config, frame.area()))
            .expect("draw");

        let buffer = terminal.backend().buffer();
        let fill_cell = &buffer[(27, 4)];
        let text_cell = &buffer[(29, 4)];

        assert_eq!(fill_cell.bg, config.theme.error);
        assert_eq!(text_cell.bg, config.theme.error);
        assert_eq!(text_cell.fg, config.theme.shell_bg);
    }

    #[test]
    fn success_toast_uses_solid_success_background() {
        let mut state = state_with_toast("Copied command to clipboard", false);
        let config = TuiConfig::default();
        let mut terminal = Terminal::new(TestBackend::new(60, 6)).expect("terminal");

        terminal
            .draw(|frame| render_toast(frame, &mut state, &config, frame.area()))
            .expect("draw");

        let buffer = terminal.backend().buffer();
        let fill_cell = &buffer[(30, 4)];
        let text_cell = &buffer[(32, 4)];

        assert_eq!(fill_cell.bg, config.theme.success);
        assert_eq!(text_cell.bg, config.theme.success);
        assert_eq!(text_cell.fg, config.theme.shell_bg);
    }

    #[test]
    fn toast_copy_uses_status_icon_without_text_label() {
        let config = TuiConfig::default();
        let mut state = state_with_toast("Copied command to clipboard", false);
        let mut terminal = Terminal::new(TestBackend::new(60, 6)).expect("terminal");

        terminal
            .draw(|frame| render_toast(frame, &mut state, &config, frame.area()))
            .expect("draw");
        let rendered = buffer_text(terminal.backend());

        assert!(rendered.contains("✓"));
        assert!(!rendered.contains("Success"));
        assert!(!rendered.contains("Error"));
        assert!(!rendered.contains("feedback"));
    }

    fn buffer_text(backend: &TestBackend) -> String {
        backend
            .buffer()
            .content
            .iter()
            .map(ratatui::buffer::Cell::symbol)
            .collect::<String>()
    }
}
