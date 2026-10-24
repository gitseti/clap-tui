use super::command_preview_line;
use crate::argv_serializer::RenderedCommand;
use crate::config::TuiConfig;
use ratatui::style::Style;
use ratatui::text::{Line, Span, Text};

// Control pictures keep embedded newlines/tabs visible without changing the
// serialized command used for copying or execution. Wrap complete styled runs,
// allocating per run/line rather than per grapheme.
pub(super) fn wrapped_preview(
    config: &TuiConfig,
    command: Option<&RenderedCommand>,
    hovered: bool,
    width: u16,
) -> Text<'static> {
    if width == 0 {
        return Text::default();
    }
    let source = command_preview_line(config, command, hovered);
    let mut lines = Vec::new();
    let mut spans = Vec::new();
    let mut used = 0;
    for span in source.spans {
        let mut visible = String::new();
        for ch in span.content.chars() {
            match ch {
                '\0'..='\u{1f}' => visible
                    .push(char::from_u32(0x2400 + u32::from(ch)).expect("ASCII control picture")),
                '\u{7f}' => visible.push('␡'),
                ch if ch.is_control() => visible.extend(ch.escape_default()),
                ch => visible.push(ch),
            }
        }
        let visible = Span::styled(visible, span.style);
        let mut run = String::new();
        for grapheme in visible.styled_graphemes(Style::default()) {
            let cells = Span::raw(grapheme.symbol).width();
            if used > 0 && used + cells > usize::from(width) {
                if !run.is_empty() {
                    spans.push(Span::styled(std::mem::take(&mut run), span.style));
                }
                lines.push(Line::from(std::mem::take(&mut spans)));
                used = 0;
            }
            run.push_str(grapheme.symbol);
            used += cells;
        }
        if !run.is_empty() {
            spans.push(Span::styled(run, span.style));
        }
    }
    lines.push(Line::from(spans));
    Text::from(lines)
}
