use crate::pipeline::{FieldInstanceId, FieldSemantics, ValidationState};
use crate::query::form::OrderedArg;
use ratatui::{layout::Rect, style::Style, text::Span};
use std::collections::BTreeMap;

pub(crate) fn wrap_validation_message(message: &str, width: u16) -> String {
    if width == 0 {
        return message.to_owned();
    }
    let mut lines = Vec::new();
    for line in message.lines() {
        let mut row = String::new();
        let mut used = 0;
        for word in line.split_whitespace() {
            let span = Span::raw(word);
            if used > 0 && used + 1 + span.width() > usize::from(width) {
                lines.push(std::mem::take(&mut row));
                used = 0;
            }
            if used > 0 {
                row.push(' ');
                used += 1;
            }
            for grapheme in span.styled_graphemes(Style::default()) {
                let cells = Span::raw(grapheme.symbol).width();
                if used > 0 && used + cells > usize::from(width) {
                    lines.push(std::mem::take(&mut row));
                    used = 0;
                }
                row.push_str(grapheme.symbol);
                used += cells;
            }
        }
        lines.push(row);
    }
    lines.join("\n")
}

pub(crate) fn validation_errors(
    args: &[OrderedArg<'_>],
    validation: &ValidationState,
    area: Rect,
    semantics: &BTreeMap<FieldInstanceId, FieldSemantics>,
) -> BTreeMap<String, String> {
    let label_width = super::preferred_label_column_width_with_semantics(args, semantics);
    let (_, _, _, width) = super::field_description_geometry(area, false, label_width);
    args.iter()
        .filter_map(|item| {
            validation
                .inline_error(&item.arg.id)
                .map(|message| (item.arg.id.clone(), wrap_validation_message(message, width)))
        })
        .collect()
}
