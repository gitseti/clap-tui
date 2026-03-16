use crate::controller::navigation;
use crate::form_editor::{self, EditResult};
use crate::frame_snapshot::FrameSnapshot;
use crate::input::{AppState, ArgValue, MouseSelection};
use crate::query::form;
use crate::runtime::{AppKeyCode, AppKeyEvent, AppMouseEvent};

use super::{Action, Effect};

pub(crate) fn apply(action: &Action, state: &mut AppState, frame_snapshot: &FrameSnapshot) -> Effect {
    match action {
        Action::ChoiceInput { arg_id, key } => {
            apply_choice_input(*key, state, frame_snapshot, arg_id);
        }
        Action::FormTextInput(key) => apply_form_text_input(*key, state),
        Action::MoveFormSelection(delta) => {
            navigation::move_form_selection(state, frame_snapshot, *delta);
        }
        Action::ActivateFormField => navigation::activate_form_field(state, frame_snapshot),
        Action::ClickDropdownChoice { arg_id, row } => {
            apply_dropdown_click(*row, state, frame_snapshot, arg_id);
        }
        Action::ClickForm(event) => apply_form_click(*event, state, frame_snapshot),
        Action::ScrollDropdown(delta) => navigation::scroll_enum(state, frame_snapshot, *delta),
        Action::ScrollForm(delta) => navigation::scroll_form(state, frame_snapshot, *delta),
        _ => {}
    }
    Effect::None
}

fn apply_form_text_input(key: AppKeyEvent, state: &mut AppState) {
    if state.ui.help_open {
        return;
    }
    let command = state.domain.current_command().clone();
    let args = form::visible_args(&command, state.ui.active_tab);
    let Some(item) = args
        .iter()
        .find(|item| item.order_index == state.ui.selected_arg_index)
    else {
        return;
    };
    if !item.arg.accepts_text_input() {
        return;
    }

    let _ = matches!(
        form_editor::apply_key_to_text_field(state, item.arg, key),
        EditResult::Handled
    );
}

fn apply_choice_input(
    key: AppKeyEvent,
    state: &mut AppState,
    frame_snapshot: &FrameSnapshot,
    arg_id: &str,
) {
    let command = state.domain.current_command().clone();
    let Some(arg) = command.args.iter().find(|arg| arg.id == arg_id) else {
        state.ui.close_dropdown();
        return;
    };
    let len = arg.choices.len();

    match key.code {
        AppKeyCode::Up => {
            if len == 0 {
                return;
            }
            let current = state
                .domain
                .current_form()
                .and_then(|inputs| inputs.values.get(&arg.id))
                .and_then(|value| match value {
                    ArgValue::Choice(selected) => {
                        arg.choices.iter().position(|choice| choice == selected)
                    }
                    _ => None,
                })
                .unwrap_or(0);
            let next = if current == 0 { len - 1 } else { current - 1 };
            state
                .domain
                .set_choice_value_touched(&arg.id, arg.choices[next].clone());
            navigation::ensure_enum_visible(state, frame_snapshot, next, len);
        }
        AppKeyCode::Down => {
            state.domain.cycle_choice_touched(&arg.id, &arg.choices);
            let current = state
                .domain
                .current_form()
                .and_then(|inputs| inputs.values.get(&arg.id))
                .and_then(|value| match value {
                    ArgValue::Choice(selected) => {
                        arg.choices.iter().position(|choice| choice == selected)
                    }
                    _ => None,
                })
                .unwrap_or(0);
            navigation::ensure_enum_visible(state, frame_snapshot, current, len);
        }
        AppKeyCode::Esc | AppKeyCode::Enter | AppKeyCode::Char(' ') => {
            state.ui.close_dropdown();
        }
        _ => {}
    }
}

fn apply_form_click(event: AppMouseEvent, state: &mut AppState, frame_snapshot: &FrameSnapshot) {
    if state.ui.help_open {
        return;
    }
    let Some(content_y) =
        frame_snapshot.form_content_y(event.row, state.ui.form_scroll(frame_snapshot))
    else {
        return;
    };
    let command = state.domain.current_command().clone();
    let args = form::visible_args(&command, state.ui.active_tab);
    if let Some(hit) = form::hit_test_form_content(&args, content_y) {
        state.ui.set_selected_arg_index(hit.order_index);
        state.ui.focus_form();
        if hit.is_flag && hit.in_input {
            state.domain.toggle_flag_touched(&hit.arg_id);
        }
        if hit.uses_choice_input && hit.in_input {
            let total = command
                .args
                .iter()
                .find(|arg| arg.id == hit.arg_id)
                .map_or(0, |arg| arg.choices.len());
            navigation::open_enum_dropdown(state, frame_snapshot, &hit.arg_id, total);
        }
        if hit.accepts_text_input
            && let Some(arg) = command
                .args
                .iter()
                .find(|arg| arg.id == hit.arg_id)
                .cloned()
        {
            form_editor::clear_selection(state, &arg);
            state.ui.clear_mouse_selection();
            if let Some((row, col)) = frame_snapshot.input_position_from_point(
                &hit.arg_id,
                event.column,
                event.row,
                false,
            ) {
                state.ui.set_mouse_selection(Some(MouseSelection {
                    arg_id: hit.arg_id.clone(),
                    anchor_row: row,
                    anchor_col: col,
                    active: false,
                }));
                form_editor::set_cursor_from_click(state, &arg, row, col);
            } else if hit.in_label {
                form_editor::set_cursor_from_click(state, &arg, 0, 0);
            }
        }
        navigation::ensure_form_visible(state, frame_snapshot);
    }
}

fn apply_dropdown_click(
    row: u16,
    state: &mut AppState,
    frame_snapshot: &FrameSnapshot,
    arg_id: &str,
) {
    let command = state.domain.current_command().clone();
    let Some(arg) = command.args.iter().find(|arg| arg.id == arg_id) else {
        return;
    };
    let visible_rows = frame_snapshot.dropdown_visible_rows().unwrap_or(0);
    let scroll = state.ui.dropdown_scroll(arg.choices.len(), visible_rows);
    if let Some(index) = frame_snapshot.dropdown_choice_index(row, scroll)
        && let Some(choice) = arg.choices.get(index)
    {
        state
            .domain
            .set_choice_value_touched(&arg.id, choice.clone());
        state.ui.close_dropdown();
    }
}
