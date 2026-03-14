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
    let border_style = if hovered {
        Style::default()
            .fg(config.theme.accent)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(config.theme.divider)
    };
    let preview_bg = if hovered {
        config.theme.surface_raised
    } else {
        config.theme.preview_bg
    };
    let bar = Paragraph::new(command_preview_line(config, &vm.preview_argv, hovered))
        .style(Style::default().bg(preview_bg))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(border_style)
                .style(Style::default().bg(preview_bg)),
        );
    frame.render_widget(bar, area);
}

fn command_preview_line<'a>(config: &TuiConfig, argv: &'a [String], hovered: bool) -> Line<'a> {
    let hover_fg = config.theme.text;
    let prompt_style = if hovered {
        Style::default()
            .fg(hover_fg)
            .add_modifier(Modifier::BOLD)
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
            if hovered {
                Style::default()
                    .fg(config.theme.accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
                    .fg(config.theme.accent)
                    .add_modifier(Modifier::BOLD)
            }
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
