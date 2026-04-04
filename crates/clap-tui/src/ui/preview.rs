use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph, Widget};

use crate::config::TuiConfig;
use crate::input::{HoverTarget, UiState};

use super::styles;
use super::{layout::LayoutMode, screen::ScreenView};

struct PreviewWidget<'a> {
    config: &'a TuiConfig,
    argv: &'a [String],
    hovered: bool,
    mode: LayoutMode,
}

impl Widget for PreviewWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let surface = if self.hovered {
            styles::surface(self.config, styles::Surface::Raised)
        } else {
            styles::surface(self.config, styles::Surface::Result)
        };
        if area.height <= 1 || self.mode.is_compact() {
            let compact =
                Paragraph::new(compact_preview_line(self.config, self.argv, self.hovered))
                    .style(surface.fg(self.config.theme.text));
            Widget::render(compact, area, buf);
            return;
        }

        let border_style = styles::preview_border(self.config, self.hovered);
        let bar = Paragraph::new(command_preview_line(self.config, self.argv, self.hovered))
            .style(surface.fg(self.config.theme.text))
            .block(
                Block::default()
                    .title(preview_title_line(self.config, self.hovered))
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(border_style)
                    .style(surface),
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
    let mode = LayoutMode::for_size(frame.area());
    frame.render_widget(
        PreviewWidget {
            config,
            argv: &vm.preview_argv,
            hovered,
            mode,
        },
        area,
    );
}

fn preview_title_line(config: &TuiConfig, hovered: bool) -> Line<'static> {
    let hint = if hovered {
        "Click or Ctrl+Y copies"
    } else {
        "Click/Ctrl+Y copy"
    };
    Line::from(vec![
        Span::styled(" Preview ", styles::preview_title(config)),
        Span::raw(" "),
        Span::styled(hint.to_string(), styles::help(config)),
    ])
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
        let style = if index == 0 {
            Style::default()
                .fg(config.theme.accent)
                .add_modifier(Modifier::BOLD | Modifier::ITALIC)
        } else if token.starts_with('-') {
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

fn compact_preview_line<'a>(config: &TuiConfig, argv: &'a [String], hovered: bool) -> Line<'a> {
    let mut spans = vec![
        Span::styled(
            "Preview",
            Style::default()
                .fg(if hovered {
                    config.theme.text
                } else {
                    config.theme.accent
                })
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
    ];
    spans.extend(command_preview_line(config, argv, hovered).spans);
    spans.push(Span::raw(" "));
    spans.push(Span::styled(
        "Ctrl+Y copy",
        Style::default().fg(config.theme.dim),
    ));
    Line::from(spans)
}

#[cfg(test)]
mod tests {
    use ratatui::style::{Color, Modifier};

    use super::{command_preview_line, compact_preview_line, preview_title_line};
    use crate::config::TuiConfig;

    #[test]
    fn preview_highlights_binary_name_and_flags() {
        let config = TuiConfig::default();
        let argv = vec![
            "tool".to_string(),
            "serve".to_string(),
            "--port".to_string(),
            "3000".to_string(),
        ];

        let line = command_preview_line(&config, &argv, false);

        assert_eq!(line.spans[1].content.as_ref(), "tool");
        assert_eq!(line.spans[1].style.fg, Some(config.theme.accent));
        assert!(line.spans[1].style.add_modifier.contains(Modifier::BOLD));
        assert!(line.spans[1].style.add_modifier.contains(Modifier::ITALIC));
        assert_eq!(line.spans[3].style.fg, Some(config.theme.text));
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

        assert_eq!(line.spans[1].style.fg, Some(config.theme.accent));
        assert_eq!(line.spans[1].style.bg, None);
        assert!(line.spans[1].style.add_modifier.contains(Modifier::ITALIC));
        assert_eq!(line.spans[3].style.fg, Some(config.theme.accent));
        assert_eq!(line.spans[3].style.bg, None);
    }

    #[test]
    fn compact_preview_line_advertises_keyboard_copy_path() {
        let config = TuiConfig::default();
        let argv = vec!["tool".to_string(), "serve".to_string()];

        let line = compact_preview_line(&config, &argv, false);
        let rendered = line
            .spans
            .iter()
            .map(|span| span.content.to_string())
            .collect::<String>();

        assert!(rendered.contains("Preview"));
        assert!(rendered.contains("Ctrl+Y copy"));
        assert!(rendered.contains("$ "));
    }

    #[test]
    fn preview_title_highlights_result_surface_and_copy_hint() {
        let config = TuiConfig::default();

        let line = preview_title_line(&config, false);

        assert!(line.spans[0].content.as_ref().contains("Preview"));
        assert_eq!(line.spans[0].style.fg, Some(config.theme.text));
        assert!(line.spans[0].style.add_modifier.contains(Modifier::BOLD));
        assert!(line.spans[2].content.as_ref().contains("copy"));
    }
}
