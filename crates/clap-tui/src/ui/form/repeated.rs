use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph, Widget};

use crate::config::TuiConfig;
use crate::form_editor;
use crate::input::UiState;
use crate::layout::form::FormFieldLayout;
use crate::query::form::FieldWidget;
use crate::repeated_field::{
    REPEATED_ROW_HEIGHT, repeated_add_rect, repeated_remove_rect, repeated_row_textarea_rect,
};

use super::fields::FieldRenderModel;
use super::{help, styles, text};

const REPEATED_CONTROL_REMOVE: &str = " - ";
const REPEATED_CONTROL_ADD: &str = " + ";

pub(super) fn render_repeated_text_field(
    buffer: &mut Buffer,
    ui: &UiState,
    field: &FormFieldLayout,
    config: &TuiConfig,
    model: &FieldRenderModel<'_>,
) -> Option<(u16, u16)> {
    let editor =
        form_editor::editor_view_for_render(ui, model.arg.owner_path(), model.arg, &model.value);
    let total_rows = editor.row_count().max(1);
    let current_row = editor.current_row();
    let visible = field.input;
    let clip_top = field.input_clip_top;
    let clip_bottom = clip_top.saturating_add(visible.height);
    let start_row = usize::from(clip_top / REPEATED_ROW_HEIGHT);
    let end_row =
        usize::from(clip_bottom.saturating_add(REPEATED_ROW_HEIGHT - 1) / REPEATED_ROW_HEIGHT)
            .min(total_rows);

    // The renderer reads `full_input_height` from layout. If the value layout
    // computed disagrees with what the row count implies, the slow path would
    // silently render a different geometry — catch that in debug builds.
    debug_assert_eq!(
        field.full_input_height,
        u16::try_from(total_rows)
            .unwrap_or(u16::MAX)
            .saturating_mul(REPEATED_ROW_HEIGHT),
        "layout/render disagree on repeated field height for {}",
        field.arg_id,
    );

    let row_local = Rect::new(visible.x, 0, visible.width, REPEATED_ROW_HEIGHT);
    let mut row_buffer = Buffer::empty(row_local);
    let surface_style = styles::surface(config, styles::Surface::Workspace);

    let mut cursor = None;
    for row_index in start_row..end_row {
        let row_top = u16::try_from(row_index)
            .unwrap_or(u16::MAX)
            .saturating_mul(REPEATED_ROW_HEIGHT);
        let is_last = row_index + 1 == total_rows;
        let active = model.selected && row_index == current_row;
        row_buffer.reset();
        row_buffer.set_style(row_local, surface_style);

        let line = editor.line(row_index);
        let placeholder = (model.field_error.is_none() && row_index == 0 && line.is_empty())
            .then(|| {
                help::required_empty_prompt(model.arg, FieldWidget::RepeatedText, model.required)
            })
            .flatten();
        let row_cursor = render_repeated_row_textarea(
            &mut row_buffer,
            config,
            repeated_row_textarea_rect(row_local, true, is_last),
            line.as_ref(),
            placeholder,
            model.text_style,
            active.then_some(u16::try_from(editor.cursor().col).unwrap_or(u16::MAX)),
        );
        render_repeated_row_controls(
            config,
            &mut row_buffer,
            row_local,
            active,
            total_rows > 1,
            is_last,
        );

        // Map the row's full-input y-range onto the visible viewport.
        let local_skip = clip_top.saturating_sub(row_top);
        let frame_y = visible.y.saturating_add(row_top.saturating_sub(clip_top));
        let copy_height = REPEATED_ROW_HEIGHT.saturating_sub(local_skip).min(
            visible
                .y
                .saturating_add(visible.height)
                .saturating_sub(frame_y),
        );
        super::blit(
            buffer,
            &row_buffer,
            (visible.x, local_skip),
            (visible.x, frame_y),
            (visible.width, copy_height),
        );

        if cursor.is_none()
            && let Some((cx, cy)) = row_cursor
            && cy >= local_skip
            && cy < local_skip.saturating_add(copy_height)
        {
            cursor = Some((cx, frame_y.saturating_add(cy.saturating_sub(local_skip))));
        }
    }
    cursor
}

fn render_repeated_row_textarea(
    buffer: &mut Buffer,
    config: &TuiConfig,
    area: Rect,
    value: &str,
    placeholder: Option<String>,
    text_style: Style,
    active_cursor_col: Option<u16>,
) -> Option<(u16, u16)> {
    let active = active_cursor_col.is_some();
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(styles::field_border(config, active, false));
    if let Some(cursor_col) = active_cursor_col {
        let mut textarea = ratatui_textarea::TextArea::new(vec![value.to_string()]);
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
        textarea.move_cursor(ratatui_textarea::CursorMove::Jump(0, cursor_col));
        (&textarea).render(area, buffer);
        text::textarea_cursor_position(&textarea, area)
    } else {
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
        })
        .render(area, buffer);
        None
    }
}

fn render_repeated_row_controls(
    config: &TuiConfig,
    buffer: &mut Buffer,
    row: Rect,
    active: bool,
    can_remove: bool,
    show_add: bool,
) {
    if let Some(remove_rect) = repeated_remove_rect(row, true, show_add) {
        Paragraph::new(REPEATED_CONTROL_REMOVE)
            .style(styles::compact_control_affordance(
                config, active, can_remove,
            ))
            .render(remove_rect, buffer);
    }
    if show_add && let Some(add_rect) = repeated_add_rect(row) {
        Paragraph::new(REPEATED_CONTROL_ADD)
            .style(styles::compact_control_affordance(config, active, true))
            .render(add_rect, buffer);
    }
}
