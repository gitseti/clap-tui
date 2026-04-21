use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget, Wrap};

use crate::argv_serializer::{RenderedCommand, RenderedShellToken, RenderedShellTokenKind};
use crate::config::TuiConfig;
use crate::input::{HoverTarget, UiState};

use super::styles;
use super::{layout::LayoutMode, screen::ScreenView};

struct PreviewWidget<'a> {
    config: &'a TuiConfig,
    command: Option<&'a RenderedCommand>,
    hovered: bool,
    mode: LayoutMode,
}

impl Widget for PreviewWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let surface = if self.hovered {
            styles::surface(self.config, styles::Surface::ControlActive)
        } else {
            styles::surface(self.config, styles::Surface::Result)
        };
        if area.height <= 1 || self.mode.is_compact() {
            let compact = Paragraph::new(compact_preview_line(
                self.config,
                self.command,
                self.hovered,
            ))
            .style(
                surface
                    .fg(self.config.theme.text)
                    .bg(self.config.theme.header_bg),
            );
            Widget::render(compact, area, buf);
            return;
        }

        let title_bar = Rect::new(area.x, area.y, area.width, 1);
        Widget::render(
            Paragraph::new(preview_title_line(self.config, self.hovered))
                .style(Style::default().bg(self.config.theme.header_bg)),
            title_bar,
            buf,
        );
        let content_area = Rect::new(
            area.x,
            area.y.saturating_add(1),
            area.width,
            area.height.saturating_sub(1),
        );
        let bar = Paragraph::new(command_preview_line(
            self.config,
            self.command,
            self.hovered,
        ))
        .style(surface.fg(self.config.theme.text))
        .wrap(Wrap { trim: false });
        Widget::render(bar, content_area, buf);
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
            command: vm.rendered_command.as_ref(),
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
        Span::styled(" Command Preview ", styles::preview_title(config)),
        Span::raw(" "),
        Span::styled(
            hint.to_string(),
            styles::help(config).bg(config.theme.header_bg),
        ),
    ])
}

fn command_preview_line(
    config: &TuiConfig,
    command: Option<&RenderedCommand>,
    hovered: bool,
) -> Line<'static> {
    let hover_fg = config.theme.text;
    let prompt_style = if hovered {
        Style::default().fg(hover_fg).add_modifier(Modifier::BOLD)
    } else {
        styles::help(config)
    };
    let mut spans = vec![Span::styled("$ ", prompt_style)];
    let Some(command) = command else {
        spans.push(Span::styled(
            "serialization blocked",
            Style::default()
                .fg(config.theme.error)
                .add_modifier(Modifier::BOLD),
        ));
        return Line::from(spans);
    };

    spans.extend(rendered_token_spans(config, &command.tokens, hovered));
    Line::from(spans)
}

fn compact_preview_line(
    config: &TuiConfig,
    command: Option<&RenderedCommand>,
    hovered: bool,
) -> Line<'static> {
    let mut spans = vec![
        Span::styled(
            "Preview",
            Style::default()
                .fg(if hovered {
                    config.theme.text
                } else {
                    config.theme.result_accent
                })
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
    ];
    spans.extend(command_preview_line(config, command, hovered).spans);
    spans.push(Span::raw(" "));
    spans.push(Span::styled(
        "Ctrl+Y copy",
        Style::default().fg(config.theme.dim),
    ));
    Line::from(spans)
}

fn rendered_token_spans(
    config: &TuiConfig,
    tokens: &[RenderedShellToken],
    hovered: bool,
) -> Vec<Span<'static>> {
    let mut spans = Vec::with_capacity(tokens.len().saturating_mul(2));
    for (index, token) in tokens.iter().enumerate() {
        if index > 0 {
            spans.push(Span::raw(" "));
        }
        spans.push(Span::styled(
            token.text.clone(),
            preview_token_style(config, token.kind, hovered),
        ));
    }
    spans
}

fn preview_token_style(config: &TuiConfig, kind: RenderedShellTokenKind, hovered: bool) -> Style {
    let _ = hovered;
    let text = config.theme.text;
    match kind {
        RenderedShellTokenKind::EntryPoint => Style::default()
            .fg(config.theme.result_accent)
            .add_modifier(Modifier::BOLD),
        RenderedShellTokenKind::SubcommandName => {
            Style::default().fg(text).add_modifier(Modifier::BOLD)
        }
        RenderedShellTokenKind::OptionSpelling => Style::default()
            .fg(config.theme.accent)
            .add_modifier(Modifier::BOLD),
        RenderedShellTokenKind::Value | RenderedShellTokenKind::DelimiterJoinedValue => {
            Style::default().fg(text)
        }
        RenderedShellTokenKind::RawBoundary => Style::default()
            .fg(config.theme.info)
            .add_modifier(Modifier::BOLD),
        RenderedShellTokenKind::Terminator => Style::default()
            .fg(config.theme.warning)
            .add_modifier(Modifier::BOLD),
        RenderedShellTokenKind::PreservedExternalToken => {
            Style::default().fg(config.theme.metadata)
        }
    }
}

#[cfg(test)]
mod tests {
    use ratatui::style::Modifier;

    use super::{command_preview_line, compact_preview_line, preview_title_line};
    use crate::argv_serializer::{RenderedCommand, RenderedShellToken, RenderedShellTokenKind};
    use crate::config::TuiConfig;

    fn rendered_command(tokens: &[(&str, RenderedShellTokenKind)]) -> RenderedCommand {
        let tokens = tokens
            .iter()
            .map(|(text, kind)| RenderedShellToken {
                text: (*text).to_string(),
                kind: *kind,
            })
            .collect::<Vec<_>>();
        let text = tokens
            .iter()
            .map(|token| token.text.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        RenderedCommand { text, tokens }
    }

    #[test]
    fn preview_highlights_binary_name_and_flags() {
        let config = TuiConfig::default();
        let command = rendered_command(&[
            ("kitchen-sink", RenderedShellTokenKind::EntryPoint),
            ("serve", RenderedShellTokenKind::SubcommandName),
            ("--feature=gzip", RenderedShellTokenKind::OptionSpelling),
            ("-literal", RenderedShellTokenKind::Value),
        ]);
        let line = command_preview_line(&config, Some(&command), false);

        assert_eq!(line.spans[1].content.as_ref(), "kitchen-sink");
        assert_eq!(line.spans[1].style.fg, Some(config.theme.result_accent));
        assert!(line.spans[1].style.add_modifier.contains(Modifier::BOLD));
        assert_eq!(line.spans[3].content.as_ref(), "serve");
        assert!(line.spans[3].style.add_modifier.contains(Modifier::BOLD));
        assert_eq!(line.spans[5].content.as_ref(), "--feature=gzip");
        assert_eq!(line.spans[5].style.fg, Some(config.theme.accent));
        assert!(line.spans[5].style.add_modifier.contains(Modifier::BOLD));
        assert_eq!(line.spans[7].content.as_ref(), "-literal");
        assert_eq!(line.spans[7].style.fg, Some(config.theme.text));
        assert!(!line.spans[7].style.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn preview_hover_uses_contrasting_text_on_accent_background() {
        let config = TuiConfig::default();
        let command = rendered_command(&[
            ("tool", RenderedShellTokenKind::EntryPoint),
            ("--verbose", RenderedShellTokenKind::OptionSpelling),
        ]);
        let line = command_preview_line(&config, Some(&command), true);

        assert_eq!(line.spans[0].style.fg, Some(config.theme.text));
        assert_eq!(line.spans[1].style.bg, None);
    }

    #[test]
    fn compact_preview_line_advertises_keyboard_copy_path() {
        let config = TuiConfig::default();
        let command = rendered_command(&[
            ("tool", RenderedShellTokenKind::EntryPoint),
            ("serve", RenderedShellTokenKind::SubcommandName),
        ]);
        let line = compact_preview_line(&config, Some(&command), false);
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

    #[test]
    fn preview_styles_boundaries_and_preserved_external_tokens() {
        let config = TuiConfig::default();
        let command = rendered_command(&[
            ("tool", RenderedShellTokenKind::EntryPoint),
            ("--", RenderedShellTokenKind::RawBoundary),
            (";", RenderedShellTokenKind::Terminator),
            (
                "'external arg'",
                RenderedShellTokenKind::PreservedExternalToken,
            ),
        ]);
        let line = command_preview_line(&config, Some(&command), false);

        assert_eq!(line.spans[3].content.as_ref(), "--");
        assert_eq!(line.spans[3].style.fg, Some(config.theme.info));
        assert_eq!(line.spans[5].content.as_ref(), ";");
        assert_eq!(line.spans[5].style.fg, Some(config.theme.warning));
        assert_eq!(line.spans[7].content.as_ref(), "'external arg'");
        assert_eq!(line.spans[7].style.fg, Some(config.theme.metadata));
    }
}
