use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, Widget};

use crate::config::TuiConfig;
use crate::form_editor;
use crate::input::UiState;

use super::fields::FieldRenderModel;
use super::{help, styles};

pub(super) fn render_text_field(
    buffer: &mut Buffer,
    ui: &UiState,
    area: Rect,
    config: &TuiConfig,
    model: &FieldRenderModel<'_>,
) -> Option<(u16, u16)> {
    if model.selected {
        render_textarea_field(
            buffer,
            ui,
            model,
            (model.field_error.is_none())
                .then(|| help::required_empty_prompt(model.arg, model.widget, model.required))
                .flatten(),
            area,
            config,
        )
    } else {
        Paragraph::new(display_lines(model))
            .block(model.block.clone())
            .style(model.fill_style.patch(model.text_style))
            .render(area, buffer);
        None
    }
}

pub(super) fn render_textarea_field(
    buffer: &mut Buffer,
    ui: &UiState,
    model: &FieldRenderModel<'_>,
    placeholder: Option<String>,
    area: Rect,
    config: &TuiConfig,
) -> Option<(u16, u16)> {
    render_textarea_value(buffer, ui, model, &model.value, placeholder, area, config)
}

pub(super) fn render_textarea_value(
    buffer: &mut Buffer,
    ui: &UiState,
    model: &FieldRenderModel<'_>,
    value: &str,
    placeholder: Option<String>,
    area: Rect,
    config: &TuiConfig,
) -> Option<(u16, u16)> {
    let textarea = textarea_value(ui, model, value, placeholder, config);
    (&textarea).render(area, buffer);
    textarea_cursor_position(&textarea, area)
}

fn textarea_value(
    ui: &UiState,
    model: &FieldRenderModel<'_>,
    value: &str,
    placeholder: Option<String>,
    config: &TuiConfig,
) -> ratatui_textarea::TextArea<'static> {
    let editor = form_editor::editor_for_render(ui, model.arg.owner_path(), model.arg, value);
    let mut textarea = editor.to_textarea(editor.selection_anchor());
    textarea.set_block(model.block.clone().style(styles::input(config, true)));
    let base_style = Style::default()
        .fg(model.text_style.fg.unwrap_or(config.theme.text))
        .bg(config.theme.surface_raised);
    textarea.set_style(base_style);
    textarea.set_cursor_line_style(base_style);
    textarea.set_cursor_style(
        Style::default()
            .bg(config.theme.accent)
            .add_modifier(Modifier::BOLD),
    );
    textarea.set_selection_style(
        Style::default()
            .fg(config.theme.text)
            .add_modifier(Modifier::REVERSED),
    );
    if let Some(placeholder) = placeholder {
        textarea.set_placeholder_text(placeholder);
        textarea.set_placeholder_style(styles::placeholder(config));
    }
    textarea
}

pub(super) fn display_lines(model: &FieldRenderModel<'_>) -> Vec<Line<'static>> {
    if model.value.is_empty() {
        return help::required_empty_prompt(model.arg, model.widget, model.required)
            .map_or_else(Vec::new, |placeholder| vec![Line::from(placeholder)]);
    }

    model
        .value
        .lines()
        .map(|line| Line::from(line.to_string()))
        .collect()
}

pub(super) fn textarea_cursor_position(
    textarea: &ratatui_textarea::TextArea<'_>,
    area: Rect,
) -> Option<(u16, u16)> {
    if area.width < 3 || area.height < 3 {
        return None;
    }
    let cursor = textarea.cursor();
    let (row, col) = (cursor.0, cursor.1);
    let inner_w = area.width.saturating_sub(2);
    let inner_h = area.height.saturating_sub(2);
    if inner_w == 0 || inner_h == 0 {
        return None;
    }
    let cursor_row = u16::try_from(row).unwrap_or(u16::MAX);
    let cursor_col = u16::try_from(col).unwrap_or(u16::MAX);
    let top_row = scroll_origin_for_cursor(cursor_row, inner_h);
    let top_col = scroll_origin_for_cursor(cursor_col, inner_w);
    let visible_row = cursor_row.saturating_sub(top_row);
    let visible_col = cursor_col.saturating_sub(top_col);
    if visible_row >= inner_h || visible_col >= inner_w {
        return None;
    }
    let x = area.x.saturating_add(1).saturating_add(visible_col);
    let y = area.y.saturating_add(1).saturating_add(visible_row);
    Some((x, y))
}

fn scroll_origin_for_cursor(cursor: u16, len: u16) -> u16 {
    if len == 0 {
        0
    } else if cursor >= len {
        cursor.saturating_add(1).saturating_sub(len)
    } else {
        0
    }
}
