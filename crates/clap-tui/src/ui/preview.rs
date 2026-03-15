use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph, Widget};

use crate::config::TuiConfig;
use crate::input::{HoverTarget, UiState};

use super::screen::ScreenView;
use super::styles;

struct PreviewWidget<'a> {
    config: &'a TuiConfig,
    argv: &'a [String],
    hovered: bool,
}

impl Widget for PreviewWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let border_style = if self.hovered {
            Style::default()
                .fg(self.config.theme.accent)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(self.config.theme.divider)
        };
        let preview_bg = if self.hovered {
            self.config.theme.surface_raised
        } else {
            self.config.theme.preview_bg
        };
        let bar = Paragraph::new(command_preview_line(self.config, self.argv, self.hovered))
            .style(Style::default().bg(preview_bg))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(border_style)
                    .style(Style::default().bg(preview_bg)),
            );
        Widget::render(bar, area, buf);
    }
}

pub(crate) fn render_preview(
    frame: &mut Frame<'_>,
    ui: &UiState,
    config: &TuiConfig,
    area: Rect,
    vm: &ScreenView<'_>,
) {
    let hovered = ui.hover == Some(HoverTarget::Preview);
    frame.render_widget(
        PreviewWidget {
            config,
            argv: &vm.preview_argv,
            hovered,
        },
        area,
    );
}

fn command_preview_line<'a>(config: &TuiConfig, argv: &'a [String], hovered: bool) -> Line<'a> {
    let hover_fg = config.theme.text;
    let prompt_style = if hovered {
        Style::default().fg(hover_fg).add_modifier(Modifier::BOLD)
    } else {
        styles::help(config)
    };
    let mut spans = vec![Span::styled("$ ", prompt_style)];
    let mut seen_flag = false;
    for (index, token) in argv.iter().enumerate() {
        if index > 0 {
            spans.push(Span::raw(" "));
        }
        let style = if token.starts_with('-') {
            seen_flag = true;
            Style::default()
                .fg(config.theme.accent)
                .add_modifier(Modifier::BOLD)
        } else if !seen_flag {
            Style::default()
                .fg(if hovered { hover_fg } else { config.theme.text })
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(if hovered { hover_fg } else { config.theme.text })
        };
        spans.push(Span::styled(token.as_str(), style));
    }
    Line::from(spans)
}

#[cfg(test)]
mod tests {
    use ratatui::style::{Color, Modifier};

    use super::command_preview_line;
    use crate::config::TuiConfig;

    #[test]
    fn preview_highlights_flags_without_accenting_command_name() {
        let config = TuiConfig::default();
        let argv = vec![
            "tool".to_string(),
            "serve".to_string(),
            "--port".to_string(),
            "3000".to_string(),
        ];

        let line = command_preview_line(&config, &argv, false);

        assert_eq!(line.spans[1].content.as_ref(), "tool");
        assert_eq!(line.spans[1].style.fg, Some(config.theme.text));
        assert!(line.spans[1].style.add_modifier.contains(Modifier::BOLD));
        assert_eq!(line.spans[5].content.as_ref(), "--port");
        assert_eq!(line.spans[5].style.fg, Some(config.theme.accent));
        assert_eq!(line.spans[7].style.fg, Some(config.theme.text));
        assert_ne!(line.spans[5].style.fg, Some(Color::Reset));
    }

    #[test]
    fn preview_hover_uses_contrasting_text_on_accent_background() {
        let config = TuiConfig::default();
        let argv = vec!["tool".to_string(), "--verbose".to_string()];

        let line = command_preview_line(&config, &argv, true);

        assert_eq!(line.spans[1].style.fg, Some(config.theme.text));
        assert_eq!(line.spans[1].style.bg, None);
        assert_eq!(line.spans[3].style.fg, Some(config.theme.accent));
        assert_eq!(line.spans[3].style.bg, None);
    }
}
