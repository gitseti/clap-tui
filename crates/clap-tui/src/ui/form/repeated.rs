use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};

use crate::config::TuiConfig;
use crate::form_editor;
use crate::input::UiState;
use crate::query::form::FieldWidget;
use crate::repeated_field::{
    project_repeated_field, repeated_add_rect, repeated_remove_rect, repeated_row_textarea_rect,
};

use super::fields::FieldRenderModel;
use super::{help, styles, text};

const REPEATED_CONTROL_REMOVE: &str = " - ";
const REPEATED_CONTROL_ADD: &str = " + ";

pub(super) fn render_repeated_text_field(
    frame: &mut Frame<'_>,
    ui: &UiState,
    area: Rect,
    input_clip_top: u16,
    config: &TuiConfig,
    model: &FieldRenderModel<'_>,
) {
    let editor =
        form_editor::editor_for_render(ui, model.arg.owner_path(), model.arg, &model.value);
    let total_rows = editor.row_count().max(1);
    let current_row = editor.current_row();
    if total_rows <= 1 {
        let textarea_rect = repeated_row_textarea_rect(area, true, true);
        let placeholder = (model.field_error.is_none() && model.value.is_empty())
            .then(|| {
                help::required_empty_prompt(model.arg, FieldWidget::RepeatedText, model.required)
            })
            .flatten();
        let cursor_col =
            (current_row == 0).then_some(u16::try_from(editor.cursor().col).unwrap_or(u16::MAX));
        render_repeated_row_textarea(
            frame,
            config,
            textarea_rect,
            &model.value,
            placeholder,
            model.text_style,
            model.selected,
            model.selected,
            cursor_col,
        );
        render_repeated_row_controls(config, frame, area, model.selected, false, true, true);
        return;
    }
    let projection =
        project_repeated_field(ui, model.arg, &model.value, 0, area.x, area.width, false, 1);
    let visible_rows =
        usize::from(area.height / crate::repeated_field::REPEATED_ROW_HEIGHT).min(total_rows);
    if visible_rows == 0 {
        return;
    }
    let start_row = repeated_visible_start_row(visible_rows, projection.rows.len(), input_clip_top);

    for visible_index in 0..visible_rows {
        let row_index = start_row + visible_index;
        let Some(row) = projection.row(row_index) else {
            continue;
        };
        let row_rect = Rect::new(
            area.x,
            area.y.saturating_add(
                u16::try_from(visible_index)
                    .unwrap_or(u16::MAX)
                    .saturating_mul(crate::repeated_field::REPEATED_ROW_HEIGHT),
            ),
            row.width,
            row.height,
        );
        let is_last_row = row_index + 1 == total_rows;
        let textarea_rect = repeated_row_textarea_rect(row_rect, true, is_last_row);
        let active_row = model.selected && row_index == current_row;
        let line = editor.lines().get(row_index).cloned().unwrap_or_default();
        let placeholder = (model.field_error.is_none() && row_index == 0 && line.is_empty())
            .then(|| {
                help::required_empty_prompt(model.arg, FieldWidget::RepeatedText, model.required)
            })
            .flatten();

        render_repeated_row_textarea(
            frame,
            config,
            textarea_rect,
            line.as_str(),
            placeholder,
            model.text_style,
            active_row,
            active_row && model.selected,
            active_row.then_some(u16::try_from(editor.cursor().col).unwrap_or(u16::MAX)),
        );

        render_repeated_row_controls(
            config,
            frame,
            row_rect,
            active_row,
            total_rows > 1,
            true,
            is_last_row,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn render_repeated_row_textarea(
    frame: &mut Frame<'_>,
    config: &TuiConfig,
    area: Rect,
    value: &str,
    placeholder: Option<String>,
    text_style: Style,
    selected_row: bool,
    place_cursor: bool,
    cursor_col: Option<u16>,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(styles::field_border(config, selected_row, false));
    if selected_row {
        let mut textarea = tui_textarea::TextArea::new(vec![value.to_string()]);
        textarea.set_block(block.style(styles::input(config, true)));
        let base_style = Style::default()
            .fg(text_style.fg.unwrap_or(config.theme.text))
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
        if let Some(cursor_col) = cursor_col {
            textarea.move_cursor(tui_textarea::CursorMove::Jump(0, cursor_col));
        }
        frame.render_widget(&textarea, area);
        if place_cursor {
            text::place_textarea_cursor(frame, &textarea, area);
        }
    } else {
        frame.render_widget(
            Paragraph::new(if value.is_empty() {
                placeholder.unwrap_or_default()
            } else {
                value.to_string()
            })
            .block(block.style(styles::input(config, false)))
            .style(if value.is_empty() {
                styles::placeholder(config)
            } else {
                text_style
            }),
            area,
        );
    }
}

#[allow(clippy::fn_params_excessive_bools)]
fn render_repeated_row_controls(
    config: &TuiConfig,
    frame: &mut Frame<'_>,
    row_rect: Rect,
    active: bool,
    can_remove: bool,
    show_remove: bool,
    show_add: bool,
) {
    if show_remove && let Some(remove_rect) = repeated_remove_rect(row_rect, show_remove, show_add)
    {
        frame.render_widget(
            Paragraph::new(REPEATED_CONTROL_REMOVE).style(styles::compact_control_affordance(
                config, active, can_remove,
            )),
            remove_rect,
        );
    }
    if show_add && let Some(add_rect) = repeated_add_rect(row_rect) {
        frame.render_widget(
            Paragraph::new(REPEATED_CONTROL_ADD)
                .style(styles::compact_control_affordance(config, active, true)),
            add_rect,
        );
    }
}

fn repeated_visible_start_row(
    visible_rows: usize,
    total_rows: usize,
    input_clip_top: u16,
) -> usize {
    let clipped_rows = usize::from(
        input_clip_top.saturating_add(crate::repeated_field::REPEATED_ROW_HEIGHT.saturating_sub(1))
            / crate::repeated_field::REPEATED_ROW_HEIGHT,
    );
    clipped_rows.min(total_rows.saturating_sub(visible_rows.min(total_rows)))
}
