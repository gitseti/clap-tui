use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph, Widget};

use crate::argv_serializer::{RenderedCommand, RenderedShellToken, RenderedShellTokenKind};
use crate::config::TuiConfig;

use super::styles;

mod layout;
mod wrapping;

pub(crate) use layout::PreviewLayout;
use wrapping::wrapped_preview;

struct PreviewWidget<'a> {
    config: &'a TuiConfig,
    layout: PreviewLayout,
}

impl Widget for PreviewWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let surface = styles::surface(
            self.config,
            if self.layout.hovered {
                styles::Surface::ControlActive
            } else {
                styles::Surface::Result
            },
        );
        Block::default().style(surface).render(area, buf);
        Paragraph::new(self.layout.header)
            .style(surface)
            .render(self.layout.header_area, buf);
        Paragraph::new(self.layout.text)
            .style(surface)
            .scroll((self.layout.scroll, 0))
            .render(self.layout.content_area, buf);
        if let Some((area, line)) = self.layout.footer {
            Paragraph::new(line)
                .style(Style::default().fg(self.config.theme.metadata))
                .render(area, buf);
        }
    }
}

pub(crate) fn render_preview(frame: &mut Frame<'_>, config: &TuiConfig, layout: PreviewLayout) {
    let area = layout.area;
    frame.render_widget(PreviewWidget { config, layout }, area);
}

fn preview_header_line(
    config: &TuiConfig,
    hovered: bool,
    width: u16,
    hidden: bool,
) -> Line<'static> {
    let title = if hidden {
        "Command Preview · more ↓ Ctrl+P expand"
    } else {
        "Command Preview"
    };
    let hint = if hovered {
        "Click or Ctrl+Y copies"
    } else {
        "Click/Ctrl+Y copy"
    };
    let hint = if usize::from(width) < title.chars().count() + hint.chars().count() + 4 {
        "Ctrl+Y copy"
    } else {
        hint
    };
    let width = usize::from(width);
    let title_width = title.chars().count();
    let hint_width = hint.chars().count();
    let left_rule_width = if width > title_width.saturating_add(hint_width).saturating_add(8) {
        2
    } else {
        0
    };
    let separators = usize::from(left_rule_width > 0)
        .saturating_add(1)
        .saturating_add(1)
        .saturating_add(1);
    let middle_rule_width = width
        .saturating_sub(left_rule_width)
        .saturating_sub(title_width)
        .saturating_sub(hint_width)
        .saturating_sub(separators)
        .max(1);

    let mut spans = Vec::new();
    if left_rule_width > 0 {
        spans.push(Span::styled(
            "─".repeat(left_rule_width),
            preview_rule_style(config, hovered),
        ));
        spans.push(Span::raw(" "));
    }
    spans.push(Span::styled(title, preview_title_style(config, hovered)));
    spans.push(Span::raw(" "));
    spans.push(Span::styled(
        "─".repeat(middle_rule_width),
        preview_rule_style(config, hovered),
    ));
    spans.push(Span::raw(" "));
    spans.push(Span::styled(hint, preview_hint_style(config, hovered)));
    Line::from(spans)
}

fn preview_title_style(config: &TuiConfig, hovered: bool) -> Style {
    let style = Style::default().fg(config.theme.metadata);
    if hovered {
        style.fg(config.theme.metadata)
    } else {
        style
    }
}

fn preview_hint_style(config: &TuiConfig, hovered: bool) -> Style {
    if hovered {
        Style::default().fg(config.theme.metadata)
    } else {
        Style::default().fg(config.theme.dim)
    }
}

fn preview_rule_style(config: &TuiConfig, hovered: bool) -> Style {
    if hovered {
        Style::default().fg(config.theme.border)
    } else {
        Style::default().fg(config.theme.divider)
    }
}

fn preview_command_inset(width: u16) -> u16 {
    if width < 28 { 1 } else { 2 }
}

fn command_preview_line(
    config: &TuiConfig,
    command: Option<&RenderedCommand>,
    hovered: bool,
) -> Line<'static> {
    let prompt_style = if hovered {
        Style::default()
            .fg(config.theme.text)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(config.theme.result_accent)
            .add_modifier(Modifier::BOLD)
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
    let base_text = if hovered {
        config.theme.text
    } else {
        config.theme.focus
    };
    match kind {
        RenderedShellTokenKind::EntryPoint => Style::default()
            .fg(config.theme.result_accent)
            .add_modifier(Modifier::BOLD),
        RenderedShellTokenKind::SubcommandName
        | RenderedShellTokenKind::Value
        | RenderedShellTokenKind::DelimiterJoinedValue => {
            Style::default().fg(base_text).add_modifier(Modifier::BOLD)
        }
        RenderedShellTokenKind::OptionSpelling => Style::default()
            .fg(config.theme.accent)
            .add_modifier(Modifier::BOLD),
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
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use ratatui::style::Modifier;

    use super::{PreviewWidget, command_preview_line, preview_command_inset, preview_header_line};
    use crate::argv_serializer::{RenderedCommand, RenderedShellToken, RenderedShellTokenKind};
    use crate::config::TuiConfig;
    use ratatui::widgets::Widget;

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

    fn row_text(buf: &Buffer, area: Rect, row: u16) -> String {
        (area.x..area.x + area.width)
            .map(|x| buf[(x, area.y + row)].symbol())
            .collect::<String>()
    }

    #[test]
    fn preview_highlights_binary_name_and_flags() {
        let config = TuiConfig::default();
        let command = rendered_command(&[
            ("clap-features", RenderedShellTokenKind::EntryPoint),
            ("serve", RenderedShellTokenKind::SubcommandName),
            ("--feature=gzip", RenderedShellTokenKind::OptionSpelling),
            ("-literal", RenderedShellTokenKind::Value),
        ]);
        let line = command_preview_line(&config, Some(&command), false);

        assert_eq!(line.spans[1].content.as_ref(), "clap-features");
        assert_eq!(line.spans[1].style.fg, Some(config.theme.result_accent));
        assert!(line.spans[1].style.add_modifier.contains(Modifier::BOLD));
        assert_eq!(line.spans[3].content.as_ref(), "serve");
        assert_eq!(line.spans[3].style.fg, Some(config.theme.focus));
        assert!(line.spans[3].style.add_modifier.contains(Modifier::BOLD));
        assert_eq!(line.spans[5].content.as_ref(), "--feature=gzip");
        assert_eq!(line.spans[5].style.fg, Some(config.theme.accent));
        assert!(line.spans[5].style.add_modifier.contains(Modifier::BOLD));
        assert_eq!(line.spans[7].content.as_ref(), "-literal");
        assert_eq!(line.spans[7].style.fg, Some(config.theme.focus));
        assert!(line.spans[7].style.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn preview_hover_uses_contrasting_text_on_command_prompt() {
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
    fn preview_header_uses_single_label_and_secondary_copy_hint() {
        let config = TuiConfig::default();

        let line = preview_header_line(&config, false, 80, false);

        assert_eq!(line.spans[0].content.as_ref(), "──");
        assert_eq!(line.spans[0].style.fg, Some(config.theme.divider));
        assert_eq!(line.spans[2].content.as_ref(), "Command Preview");
        assert_eq!(line.spans[2].style.fg, Some(config.theme.metadata));
        assert!(!line.spans[2].style.add_modifier.contains(Modifier::BOLD));
        assert_eq!(line.spans[6].style.fg, Some(config.theme.dim));
        assert!(line.spans[6].content.as_ref().contains("copy"));
    }

    #[test]
    fn preview_command_prompt_and_content_are_emphasized_over_header_hint() {
        let config = TuiConfig::default();
        let command = rendered_command(&[
            ("tool", RenderedShellTokenKind::EntryPoint),
            ("value", RenderedShellTokenKind::Value),
        ]);

        let title = preview_header_line(&config, false, 80, false);
        let line = command_preview_line(&config, Some(&command), false);

        assert_eq!(line.spans[0].style.fg, Some(config.theme.result_accent));
        assert!(line.spans[0].style.add_modifier.contains(Modifier::BOLD));
        assert_eq!(line.spans[1].style.fg, Some(config.theme.result_accent));
        assert_eq!(line.spans[3].style.fg, Some(config.theme.focus));
        assert!(line.spans[3].style.add_modifier.contains(Modifier::BOLD));
        assert_eq!(title.spans[2].style.fg, Some(config.theme.metadata));
    }

    #[test]
    fn preview_hover_keeps_copy_hint_secondary() {
        let config = TuiConfig::default();

        let line = preview_header_line(&config, true, 80, false);

        assert_eq!(line.spans[2].style.fg, Some(config.theme.metadata));
        assert_eq!(line.spans[6].style.fg, Some(config.theme.metadata));
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

    #[test]
    fn roomy_dock_renders_header_padding_command_and_bottom_padding_rows() {
        let config = TuiConfig::default();
        let command = rendered_command(&[
            ("tool", RenderedShellTokenKind::EntryPoint),
            ("serve", RenderedShellTokenKind::SubcommandName),
        ]);
        let area = Rect::new(0, 0, 48, 4);
        let mut buf = Buffer::empty(area);

        Widget::render(
            PreviewWidget {
                config: &config,
                layout: super::PreviewLayout::dock(&config, Some(&command), false, area),
            },
            area,
            &mut buf,
        );

        let header = row_text(&buf, area, 0);
        let spacer = row_text(&buf, area, 1);
        let command_row = row_text(&buf, area, 2);
        let bottom = row_text(&buf, area, 3);

        assert!(header.contains("Command Preview"));
        assert!(header.contains("Click/Ctrl+Y copy"));
        assert!(header.find("Click/Ctrl+Y copy") > header.find("Command Preview"));
        assert_eq!(spacer.trim(), "");
        assert_eq!(preview_command_inset(area.width.saturating_sub(2)), 2);
        assert!(command_row.starts_with("   $ tool serve"));
        assert_eq!(bottom.trim(), "");
    }

    #[test]
    fn compact_dock_renders_header_and_command_rows() {
        let config = TuiConfig::default();
        let command = rendered_command(&[
            ("tool", RenderedShellTokenKind::EntryPoint),
            ("serve", RenderedShellTokenKind::SubcommandName),
        ]);
        let area = Rect::new(0, 0, 40, 2);
        let mut buf = Buffer::empty(area);

        Widget::render(
            PreviewWidget {
                config: &config,
                layout: super::PreviewLayout::dock(&config, Some(&command), false, area),
            },
            area,
            &mut buf,
        );

        let header = row_text(&buf, area, 0);
        let command_row = row_text(&buf, area, 1);

        assert!(header.contains("Command Preview"));
        assert!(header.contains("─"));
        assert!(header.contains("Click/Ctrl+Y copy"));
        assert_eq!(preview_command_inset(area.width.saturating_sub(2)), 2);
        assert!(command_row.starts_with("   $ tool serve"));
    }
}

#[cfg(test)]
mod wrapping_tests;
