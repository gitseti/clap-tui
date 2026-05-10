use crate::form_editor;
use crate::frame_snapshot::FrameSnapshot;
use crate::input::AppState;
use crate::query::form::{self, FieldWidget};
use crate::runtime::AppMouseEvent;

use super::field_hit;
use super::{Action, Effect};

pub(crate) fn apply(
    action: &Action,
    state: &mut AppState,
    frame_snapshot: &FrameSnapshot,
) -> Effect {
    match action {
        Action::UpdateHover { x, y } => apply_hover_update(state, frame_snapshot, *x, *y),
        Action::UpdateMouseSelection(event) => {
            apply_mouse_selection(state, frame_snapshot, *event);
        }
        Action::ClearMouseSelection => state.ui.clear_mouse_selection(),
        _ => {}
    }
    Effect::None
}

fn apply_mouse_selection(
    state: &mut AppState,
    frame_snapshot: &FrameSnapshot,
    event: AppMouseEvent,
) {
    let Some(mut selection) = state.ui.mouse_select.take() else {
        return;
    };
    let arg = state.domain.arg_for_input(&selection.arg_id).cloned();
    if let Some(arg) = arg {
        let position = match form::widget_for(&arg) {
            FieldWidget::RepeatedText => {
                let total_rows = form_editor::editor_for_render(
                    &state.ui,
                    arg.owner_path(),
                    &arg,
                    &form_editor::displayed_text(state, &arg),
                )
                .row_count()
                .max(1);
                frame_snapshot
                    .form_field_layout(&selection.arg_id)
                    .and_then(|field| {
                        field_hit::repeated_text_position_from_point(
                            field,
                            total_rows,
                            event.column,
                            event.row,
                            true,
                        )
                    })
            }
            _ => frame_snapshot
                .form_field_layout(&selection.arg_id)
                .and_then(|field| {
                    field_hit::text_input_position_from_point(field, event.column, event.row, true)
                })
                .or_else(|| {
                    frame_snapshot.input_position_from_point(
                        &selection.arg_id,
                        event.column,
                        event.row,
                        true,
                    )
                }),
        };
        if let Some((row, col)) = position {
            if !selection.active {
                form_editor::start_selection(
                    state,
                    &arg,
                    selection.anchor_row,
                    selection.anchor_col,
                );
                selection.active = true;
            }
            form_editor::set_cursor_from_click(state, &arg, row, col);
        }
    }
    state.ui.set_mouse_selection(Some(selection));
}

fn apply_hover_update(state: &mut AppState, frame_snapshot: &FrameSnapshot, x: u16, y: u16) {
    let hover = if frame_snapshot.preview_contains(x, y) {
        Some(crate::input::HoverTarget::Preview)
    } else {
        frame_snapshot.footer_target_at(x, y)
    };
    state.ui.set_hover(hover);
    state.ui.set_hover_tab(frame_snapshot.tab_at(x, y));
}

#[cfg(test)]
mod tests {
    use clap::{Arg, ArgAction, Command};
    use ratatui::layout::Rect;

    use crate::frame_snapshot::FrameSnapshot;
    use crate::input::MouseSelection;
    use crate::layout::form::FormFieldLayout;
    use crate::runtime::{AppKeyModifiers, AppMouseButton, AppMouseEvent, AppMouseEventKind};
    use crate::spec::{
        ArgKind, ArgSpec, CommandSpec, EXTERNAL_SUBCOMMAND_NAME_ID, ValueCardinality,
    };

    use super::super::{Action, Effect, apply_action};

    fn command(args: Vec<ArgSpec>) -> CommandSpec {
        CommandSpec {
            name: "tool".to_string(),
            version: None,
            about: None,
            help: String::new(),
            args,
            subcommands: Vec::new(),
            ..CommandSpec::default()
        }
    }

    fn arg(id: &str, name: &str, kind: ArgKind) -> ArgSpec {
        ArgSpec {
            id: id.to_string(),
            display_name: name.to_string(),
            help: None,
            required: false,
            kind,
            default_values: Vec::new(),
            choices: Vec::new(),
            position: None,
            value_cardinality: ValueCardinality::One,
            value_hint: None,
            ..ArgSpec::default()
        }
    }

    #[test]
    fn mouse_selection_reducer_starts_editor_selection_on_drag() {
        let mut path = arg("path", "path", ArgKind::Positional);
        path.position = Some(1);
        let mut state = crate::input::AppState::new(command(vec![path]));
        state.ui.set_mouse_selection(Some(MouseSelection {
            arg_id: "path".to_string(),
            anchor_row: 0,
            anchor_col: 0,
            active: false,
        }));
        let mut snapshot = FrameSnapshot::default();
        snapshot
            .layout
            .form_inputs
            .insert("path".to_string(), ratatui::layout::Rect::new(0, 0, 12, 3));

        let action = Action::UpdateMouseSelection(AppMouseEvent {
            kind: AppMouseEventKind::Drag(AppMouseButton::Left),
            column: 2,
            row: 1,
            modifiers: AppKeyModifiers::default(),
        });
        let effect = apply_action(&action, &mut state, &snapshot);

        assert_eq!(effect, Effect::None);
        assert!(
            state
                .ui
                .mouse_select
                .as_ref()
                .is_some_and(|selection| selection.active)
        );
        let editor = state
            .ui
            .editors
            .editor(state.domain.selected_path(), "path")
            .expect("editor");
        assert_eq!(
            editor.selection_anchor(),
            Some(crate::editor_state::TextPosition::default())
        );
    }

    #[test]
    fn mouse_selection_reducer_supports_inherited_global_text_fields() {
        let mut state = crate::input::AppState::from_command(
            &Command::new("tool")
                .arg(Arg::new("config").long("config").global(true))
                .subcommand(Command::new("admin")),
        );
        state
            .select_command_path(&["admin".to_string()])
            .expect("valid path");
        state.ui.set_mouse_selection(Some(MouseSelection {
            arg_id: "config".to_string(),
            anchor_row: 0,
            anchor_col: 0,
            active: false,
        }));
        let mut snapshot = FrameSnapshot::default();
        snapshot
            .layout
            .form_inputs
            .insert("config".to_string(), Rect::new(0, 0, 12, 3));

        let action = Action::UpdateMouseSelection(AppMouseEvent {
            kind: AppMouseEventKind::Drag(AppMouseButton::Left),
            column: 3,
            row: 1,
            modifiers: AppKeyModifiers::default(),
        });
        let effect = apply_action(&action, &mut state, &snapshot);

        assert_eq!(effect, Effect::None);
        let arg = state.domain.arg_for_input("config").expect("config arg");
        let editor = crate::form_editor::editor_for_render(
            &state.ui,
            arg.owner_path(),
            arg,
            &crate::form_editor::displayed_text(&state, arg),
        );
        assert_eq!(
            editor.selection_anchor(),
            Some(crate::editor_state::TextPosition::default())
        );
        assert_eq!(
            editor.cursor(),
            crate::editor_state::TextPosition { row: 0, col: 2 }
        );
    }

    #[test]
    fn mouse_selection_reducer_supports_virtual_external_subcommand_fields() {
        let mut state = crate::input::AppState::from_command(
            &Command::new("tool").allow_external_subcommands(true),
        );
        state.ui.set_mouse_selection(Some(MouseSelection {
            arg_id: EXTERNAL_SUBCOMMAND_NAME_ID.to_string(),
            anchor_row: 0,
            anchor_col: 0,
            active: false,
        }));
        let mut snapshot = FrameSnapshot::default();
        snapshot.layout.form_inputs.insert(
            EXTERNAL_SUBCOMMAND_NAME_ID.to_string(),
            Rect::new(0, 0, 16, 3),
        );

        let action = Action::UpdateMouseSelection(AppMouseEvent {
            kind: AppMouseEventKind::Drag(AppMouseButton::Left),
            column: 4,
            row: 1,
            modifiers: AppKeyModifiers::default(),
        });
        let effect = apply_action(&action, &mut state, &snapshot);

        assert_eq!(effect, Effect::None);
        let arg = state
            .domain
            .arg_for_input(EXTERNAL_SUBCOMMAND_NAME_ID)
            .expect("virtual arg");
        let editor = crate::form_editor::editor_for_render(
            &state.ui,
            arg.owner_path(),
            arg,
            &crate::form_editor::displayed_text(&state, arg),
        );
        assert_eq!(
            editor.selection_anchor(),
            Some(crate::editor_state::TextPosition::default())
        );
        assert_eq!(
            editor.cursor(),
            crate::editor_state::TextPosition { row: 0, col: 3 }
        );
    }

    #[test]
    fn repeated_mouse_selection_uses_content_rows_when_partially_clipped() {
        let mut state = crate::input::AppState::from_command(
            &Command::new("tool").arg(
                Arg::new("include")
                    .long("include")
                    .action(ArgAction::Append)
                    .num_args(1),
            ),
        );
        state.domain.set_text_value("include", "alpha\nbeta\ngamma");
        state.domain.mark_touched("include");
        state.ui.set_mouse_selection(Some(MouseSelection {
            arg_id: "include".to_string(),
            anchor_row: 0,
            anchor_col: 0,
            active: false,
        }));
        let mut snapshot = FrameSnapshot::default();
        snapshot.layout.form = Some(Rect::new(0, 0, 80, 6));
        snapshot.layout.form_view = Some(Rect::new(0, 0, 80, 6));
        snapshot
            .layout
            .form_inputs
            .insert("include".to_string(), Rect::new(10, 1, 30, 5));
        snapshot.layout.form_fields.push(FormFieldLayout {
            arg_id: "include".to_string(),
            heading: None,
            label: Some(Rect::new(0, 1, 9, 1)),
            input: Rect::new(10, 1, 30, 5),
            input_clip_top: 2,
            full_input_height: 7,
            description: None,
        });

        let action = Action::UpdateMouseSelection(AppMouseEvent {
            kind: AppMouseEventKind::Drag(AppMouseButton::Left),
            column: 13,
            row: 4,
            modifiers: AppKeyModifiers::default(),
        });
        let effect = apply_action(&action, &mut state, &snapshot);

        assert_eq!(effect, Effect::None);
        let arg = state.domain.arg_for_input("include").expect("include arg");
        let editor = crate::form_editor::editor_for_render(
            &state.ui,
            arg.owner_path(),
            arg,
            &crate::form_editor::displayed_text(&state, arg),
        );
        assert_eq!(
            editor.cursor(),
            crate::editor_state::TextPosition { row: 1, col: 2 }
        );
    }
}
