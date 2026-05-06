use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::Paragraph;

use crate::config::TuiConfig;
use crate::form_editor;
use crate::input::UiState;

use super::fields::FieldRenderModel;
use super::{help, styles};

pub(super) fn render_text_field(
    frame: &mut Frame<'_>,
    ui: &UiState,
    area: Rect,
    config: &TuiConfig,
    model: &FieldRenderModel<'_>,
) {
    if model.input_is_truncated {
        frame.render_widget(
            Paragraph::new(display_lines(model)).style(model.fill_style.patch(model.text_style)),
            area,
        );
    } else if model.selected {
        render_textarea_field(
            frame,
            ui,
            model,
            (model.field_error.is_none())
                .then(|| help::required_empty_prompt(model.arg, model.widget, model.required))
                .flatten(),
            area,
            config,
        );
    } else {
        frame.render_widget(
            Paragraph::new(display_lines(model))
                .block(model.block.clone())
                .style(model.fill_style.patch(model.text_style)),
            area,
        );
    }
}

pub(super) fn render_textarea_field(
    frame: &mut Frame<'_>,
    ui: &UiState,
    model: &FieldRenderModel<'_>,
    placeholder: Option<String>,
    area: Rect,
    config: &TuiConfig,
) {
    render_textarea_value(frame, ui, model, &model.value, placeholder, area, config);
}

pub(super) fn render_textarea_value(
    frame: &mut Frame<'_>,
    ui: &UiState,
    model: &FieldRenderModel<'_>,
    value: &str,
    placeholder: Option<String>,
    area: Rect,
    config: &TuiConfig,
) {
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
    frame.render_widget(&textarea, area);
    place_textarea_cursor(frame, &textarea, area);
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

pub(super) fn place_textarea_cursor(
    frame: &mut Frame<'_>,
    textarea: &tui_textarea::TextArea<'_>,
    area: Rect,
) {
    if area.width < 3 || area.height < 3 {
        return;
    }
    let (row, col) = textarea.cursor();
    let inner_x = area.x.saturating_add(1);
    let inner_y = area.y.saturating_add(1);
    let inner_w = area.width.saturating_sub(2);
    let inner_h = area.height.saturating_sub(2);
    if inner_w == 0 || inner_h == 0 {
        return;
    }
    let x = inner_x
        .saturating_add(u16::try_from(col).unwrap_or(inner_w.saturating_sub(1)))
        .min(inner_x + inner_w - 1);
    let y = inner_y
        .saturating_add(u16::try_from(row).unwrap_or(inner_h.saturating_sub(1)))
        .min(inner_y + inner_h - 1);
    frame.set_cursor_position((x, y));
}
