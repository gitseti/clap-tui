use ratatui::layout::Rect;

use crate::form_editor;
use crate::input::UiState;
use crate::spec::ArgSpec;

pub(crate) const REPEATED_CONTROL_WIDTH: u16 = 8;
pub(crate) const REPEATED_CONTROL_ROW_WIDTH: u16 = 8;
pub(crate) const REPEATED_ROW_HEIGHT: u16 = 3;
const FIELD_GAP_HEIGHT: u16 = 1;

#[derive(Debug, Clone)]
pub(crate) struct RepeatedFieldProjection {
    pub(crate) input: Rect,
    pub(crate) rows: Vec<Rect>,
    pub(crate) description: Option<Rect>,
    #[allow(dead_code)]
    pub(crate) total_height: u16,
}

impl RepeatedFieldProjection {
    pub(crate) fn row(&self, index: usize) -> Option<Rect> {
        self.rows.get(index).copied()
    }
}

pub(crate) fn repeated_input_height(ui: &UiState, arg: &ArgSpec, value: &str) -> u16 {
    let editor = form_editor::editor_view_for_render(ui, arg.owner_path(), arg, value);
    let rows = editor.row_count().max(1);
    if rows <= 1 {
        REPEATED_ROW_HEIGHT
    } else {
        u16::try_from(rows)
            .unwrap_or(u16::MAX)
            .saturating_mul(REPEATED_ROW_HEIGHT)
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn project_repeated_field(
    ui: &UiState,
    arg: &ArgSpec,
    value: &str,
    field_top: u16,
    input_x: u16,
    input_width: u16,
    show_description: bool,
    label_height: u16,
) -> RepeatedFieldProjection {
    project_repeated_field_with_input_height(
        repeated_input_height(ui, arg, value),
        field_top,
        input_x,
        input_width,
        show_description,
        label_height,
    )
}

pub(crate) fn project_repeated_field_with_input_height(
    input_height: u16,
    field_top: u16,
    input_x: u16,
    input_width: u16,
    show_description: bool,
    label_height: u16,
) -> RepeatedFieldProjection {
    let input_height = input_height.max(REPEATED_ROW_HEIGHT);
    let input = Rect::new(input_x, field_top, input_width, input_height);
    let row_count =
        usize::from(input_height.saturating_add(REPEATED_ROW_HEIGHT - 1) / REPEATED_ROW_HEIGHT)
            .max(1);
    let rows = (0..row_count)
        .filter_map(|index| {
            let y = input.y.saturating_add(
                u16::try_from(index)
                    .ok()?
                    .saturating_mul(REPEATED_ROW_HEIGHT),
            );
            Some(Rect::new(input.x, y, input.width, REPEATED_ROW_HEIGHT))
        })
        .collect();
    let content_height = input_height.max(label_height);
    let description = show_description.then(|| {
        Rect::new(
            input_x,
            field_top.saturating_add(content_height),
            input_width,
            1,
        )
    });
    let total_height = content_height
        .saturating_add(u16::from(show_description))
        .saturating_add(FIELD_GAP_HEIGHT);

    RepeatedFieldProjection {
        input,
        rows,
        description,
        total_height,
    }
}

pub(crate) fn repeated_row_textarea_rect(
    row_rect: Rect,
    show_remove: bool,
    show_add: bool,
) -> Rect {
    let with_controls = show_remove || show_add;
    let width = if with_controls && row_rect.width > REPEATED_CONTROL_WIDTH {
        row_rect
            .width
            .saturating_sub(REPEATED_CONTROL_WIDTH)
            .saturating_sub(1)
    } else {
        row_rect.width
    };
    Rect::new(row_rect.x, row_rect.y, width, row_rect.height)
}

pub(crate) fn repeated_remove_rect(
    row_rect: Rect,
    show_remove: bool,
    show_add: bool,
) -> Option<Rect> {
    if !show_remove || row_rect.height < 2 {
        return None;
    }
    let strip_start = row_rect
        .x
        .saturating_add(row_rect.width.saturating_sub(REPEATED_CONTROL_WIDTH));
    let x = if show_add {
        strip_start
    } else {
        strip_start.saturating_add(2)
    };
    Some(Rect::new(
        x,
        row_rect
            .y
            .saturating_add(row_rect.height.saturating_sub(1).min(1)),
        3,
        1,
    ))
}

pub(crate) fn repeated_add_rect(row_rect: Rect) -> Option<Rect> {
    if row_rect.height < 2 {
        return None;
    }
    Some(Rect::new(
        row_rect.x.saturating_add(
            row_rect
                .width
                .saturating_sub(REPEATED_CONTROL_ROW_WIDTH)
                .saturating_add(4),
        ),
        row_rect
            .y
            .saturating_add(row_rect.height.saturating_sub(1).min(1)),
        3,
        1,
    ))
}

#[cfg(test)]
mod tests {
    use clap::{Arg, ArgAction, Command};

    use super::*;
    use crate::input::AppState;

    #[test]
    fn repeated_projection_places_rows_and_description_from_shared_geometry() {
        let mut state = AppState::from_command(
            &Command::new("tool").arg(
                Arg::new("include")
                    .long("include")
                    .action(ArgAction::Append)
                    .num_args(1)
                    .help("Include path"),
            ),
        );
        state.domain.set_text_value("include", "alpha\nbeta\ngamma");
        let arg = state.domain.arg_for_input("include").expect("include arg");

        let projection =
            project_repeated_field(&state.ui, arg, "alpha\nbeta\ngamma", 7, 12, 24, true, 1);

        assert_eq!(projection.input, Rect::new(12, 7, 24, 9));
        assert_eq!(projection.row(0), Some(Rect::new(12, 7, 24, 3)));
        assert_eq!(projection.row(1), Some(Rect::new(12, 10, 24, 3)));
        assert_eq!(projection.row(2), Some(Rect::new(12, 13, 24, 3)));
        assert_eq!(projection.description, Some(Rect::new(12, 16, 24, 1)));
        assert_eq!(projection.total_height, 11);
    }
}
