use crate::controller::navigation;
use crate::form_editor::{self, EditResult};
use crate::frame_snapshot::FrameSnapshot;
use crate::input::{AppState, MouseSelection};
use crate::query::form::{self, FieldWidget};
use crate::runtime::{AppKeyCode, AppKeyEvent, AppMouseEvent};

use super::{Action, Effect};

pub(crate) fn apply(
    action: &Action,
    state: &mut AppState,
    frame_snapshot: &FrameSnapshot,
) -> Effect {
    match action {
        Action::ChoiceInput { arg_id, key } => {
            apply_choice_input(*key, state, frame_snapshot, arg_id);
        }
        Action::FormTextInput(key) => apply_form_text_input(*key, state),
        Action::FormWidgetInput(key) => apply_form_widget_input(*key, state),
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

pub(crate) fn apply_paste_text(state: &mut AppState, text: &str) {
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
    if !item.widget.accepts_text_input() {
        return;
    }

    let _ = matches!(
        form_editor::apply_paste_to_text_field(state, item.arg, text),
        EditResult::Handled
    );
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
    if !item.widget.accepts_text_input() {
        return;
    }

    let _ = matches!(
        form_editor::apply_key_to_text_field(state, item.arg, key),
        EditResult::Handled
    );
}

fn apply_form_widget_input(key: AppKeyEvent, state: &mut AppState) {
    let command = state.domain.current_command().clone();
    let args = form::visible_args(&command, state.ui.active_tab);
    let Some(item) = args
        .iter()
        .find(|item| item.order_index == state.ui.selected_arg_index)
    else {
        return;
    };

    if matches!(item.widget, FieldWidget::Counter) {
        match key.code {
            AppKeyCode::Right | AppKeyCode::Char('+' | '=') => {
                state.domain.increment_counter(&item.arg.id);
            }
            AppKeyCode::Left | AppKeyCode::Char('-') | AppKeyCode::Backspace => {
                state.domain.decrement_counter(&item.arg.id);
            }
            _ => {}
        }
    } else if matches!(item.widget, FieldWidget::RepeatedText) {
        let _ = if matches!(key.code, AppKeyCode::Enter) {
            form_editor::insert_repeated_row(state, item.arg)
        } else if matches!(key.code, AppKeyCode::Up) && key.modifiers.alt {
            form_editor::move_repeated_row_up(state, item.arg)
        } else if matches!(key.code, AppKeyCode::Down) && key.modifiers.alt {
            form_editor::move_repeated_row_down(state, item.arg)
        } else if matches!(key.code, AppKeyCode::Delete | AppKeyCode::Backspace)
            && key.modifiers.control
        {
            form_editor::remove_repeated_row(state, item.arg)
        } else {
            EditResult::Ignored
        };
    } else if matches!(item.widget, FieldWidget::OptionalValue) {
        match key.code {
            AppKeyCode::Right => state.domain.toggle_optional_value_flag(&item.arg.id, true),
            AppKeyCode::Left | AppKeyCode::Delete | AppKeyCode::Backspace => {
                state.domain.clear_value_and_untouch(&item.arg.id);
            }
            _ => {}
        }
    }
}

fn apply_choice_input(
    key: AppKeyEvent,
    state: &mut AppState,
    frame_snapshot: &FrameSnapshot,
    arg_id: &str,
) {
    let Some(arg) = state.domain.arg_for_input(arg_id).cloned() else {
        state.ui.close_dropdown();
        return;
    };
    let len = arg.choices.len();
    let is_multi = matches!(form::widget_for(&arg), FieldWidget::MultiChoice);

    match key.code {
        AppKeyCode::Up => {
            if len == 0 {
                return;
            }
            let current = state.ui.dropdown_cursor(len);
            let next = if current == 0 { len - 1 } else { current - 1 };
            state.ui.set_dropdown_cursor(next);
            navigation::ensure_enum_visible(state, frame_snapshot, next, len);
        }
        AppKeyCode::Down => {
            if len == 0 {
                return;
            }
            let current = state.ui.dropdown_cursor(len);
            let next = (current + 1) % len;
            state.ui.set_dropdown_cursor(next);
            navigation::ensure_enum_visible(state, frame_snapshot, next, len);
        }
        AppKeyCode::Char(' ') => {
            let index = state.ui.dropdown_cursor(len);
            let Some(choice) = arg.choices.get(index) else {
                return;
            };
            if is_multi {
                state.domain.toggle_choice_value_touched(&arg.id, choice);
            } else {
                state
                    .domain
                    .set_choice_value_touched(&arg.id, choice.clone());
                state.ui.close_dropdown();
            }
        }
        AppKeyCode::Enter => {
            if is_multi {
                state.ui.close_dropdown();
            } else {
                let index = state.ui.dropdown_cursor(len);
                if let Some(choice) = arg.choices.get(index) {
                    state
                        .domain
                        .set_choice_value_touched(&arg.id, choice.clone());
                }
                state.ui.close_dropdown();
            }
        }
        AppKeyCode::Esc => {
            state.ui.close_dropdown();
        }
        _ => {}
    }
}

fn apply_form_click(event: AppMouseEvent, state: &mut AppState, frame_snapshot: &FrameSnapshot) {
    if state.ui.help_open {
        return;
    }
    state.ui.close_dropdown();
    let Some(content_y) =
        frame_snapshot.form_content_y(event.row, state.ui.form_scroll(frame_snapshot))
    else {
        return;
    };
    let command = state.domain.current_command().clone();
    let args = form::visible_args(&command, state.ui.active_tab);
    let validation = crate::pipeline::derive(state).validation;
    if let Some(hit) =
        form::hit_test_form_content_with_errors(&args, content_y, &validation.field_errors)
    {
        state.ui.set_selected_arg_index(hit.order_index);
        state.ui.focus_form();
        if matches!(
            hit.widget,
            FieldWidget::Toggle | FieldWidget::Counter | FieldWidget::OptionalValue
        ) && hit.in_input
        {
            navigation::activate_form_field(state, frame_snapshot);
        }
        if hit.widget.uses_choice_popup() && hit.in_input {
            let total = state
                .domain
                .arg_for_input(&hit.arg_id)
                .map_or(0, |arg| arg.choices.len());
            navigation::open_enum_dropdown(state, frame_snapshot, &hit.arg_id, total);
        }
        if hit.widget.accepts_text_input()
            && let Some(arg) = state.domain.arg_for_input(&hit.arg_id).cloned()
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
    let Some(arg) = state.domain.arg_for_input(arg_id).cloned() else {
        return;
    };
    let visible_rows = frame_snapshot.dropdown_visible_rows().unwrap_or(0);
    let scroll = state.ui.dropdown_scroll(arg.choices.len(), visible_rows);
    if let Some(index) = frame_snapshot.dropdown_choice_index(row, scroll)
        && let Some(choice) = arg.choices.get(index)
    {
        state.ui.set_dropdown_cursor(index);
        if matches!(form::widget_for(&arg), FieldWidget::MultiChoice) {
            state.domain.toggle_choice_value_touched(&arg.id, choice);
        } else {
            state
                .domain
                .set_choice_value_touched(&arg.id, choice.clone());
            state.ui.close_dropdown();
        }
    }
}

#[cfg(test)]
mod tests {
    use clap::{Arg, ArgAction, Command};

    use super::super::{Action, Effect, apply_action};
    use super::*;
    use crate::frame_snapshot::FrameSnapshot;
    use crate::runtime::AppKeyModifiers;

    fn key(code: AppKeyCode) -> AppKeyEvent {
        AppKeyEvent::new(code, AppKeyModifiers::default())
    }

    #[test]
    fn multi_select_dropdown_toggles_current_choice_without_closing() {
        let mut state = AppState::from_command(
            &Command::new("tool").arg(
                Arg::new("color")
                    .long("color")
                    .action(ArgAction::Append)
                    .num_args(1)
                    .value_parser(["red", "green", "blue"]),
            ),
        );
        state.ui.dropdown_open = Some("color".to_string());
        state.ui.dropdown_cursor = 1;

        let effect = apply_action(
            &Action::ChoiceInput {
                arg_id: "color".to_string(),
                key: key(AppKeyCode::Char(' ')),
            },
            &mut state,
            &FrameSnapshot::default(),
        );

        assert_eq!(effect, Effect::None);
        assert_eq!(state.ui.dropdown_open.as_deref(), Some("color"));
        let arg = state
            .domain
            .current_command()
            .args
            .iter()
            .find(|arg| arg.id == "color")
            .expect("color arg");
        assert_eq!(
            state
                .domain
                .current_form()
                .map(|form| form.selected_values(arg))
                .unwrap_or_default(),
            vec!["green".to_string()]
        );
    }

    #[test]
    fn inherited_choice_input_updates_global_value_from_descendant_form() {
        let mut state = AppState::from_command(
            &Command::new("tool")
                .arg(
                    Arg::new("color")
                        .long("color")
                        .global(true)
                        .value_parser(["red", "green", "blue"]),
                )
                .subcommand(Command::new("admin")),
        );
        state
            .select_command_path(&["admin".to_string()])
            .expect("valid path");
        state.ui.dropdown_open = Some("color".to_string());
        state.ui.dropdown_cursor = 1;

        let effect = apply_action(
            &Action::ChoiceInput {
                arg_id: "color".to_string(),
                key: key(AppKeyCode::Enter),
            },
            &mut state,
            &FrameSnapshot::default(),
        );

        assert_eq!(effect, Effect::None);
        assert!(state.ui.dropdown_open.is_none());
        let arg = state.domain.arg_for_input("color").expect("color arg");
        assert_eq!(
            state
                .domain
                .current_form()
                .map(|form| form.selected_values(arg))
                .unwrap_or_default(),
            vec!["green".to_string()]
        );
    }

    #[test]
    fn optional_value_widget_clear_disables_the_field() {
        let mut state = AppState::from_command(
            &Command::new("tool").arg(
                Arg::new("color")
                    .long("color")
                    .action(ArgAction::SetTrue)
                    .num_args(0..=1),
            ),
        );
        state.domain.set_text_value("color", "blue");
        state.domain.mark_touched("color");

        let effect = apply_action(
            &Action::FormWidgetInput(key(AppKeyCode::Left)),
            &mut state,
            &FrameSnapshot::default(),
        );

        assert_eq!(effect, Effect::None);
        let argv = crate::pipeline::build_command_line(&state);
        assert_eq!(argv, vec!["tool".to_string()]);
    }

    #[test]
    fn optional_value_text_input_keeps_appending_after_enable() {
        let mut state = AppState::from_command(
            &Command::new("tool").arg(
                Arg::new("color")
                    .long("color")
                    .action(ArgAction::Set)
                    .num_args(0..=1)
                    .require_equals(true)
                    .default_missing_value("always"),
            ),
        );

        let effect = apply_action(
            &Action::FormWidgetInput(key(AppKeyCode::Right)),
            &mut state,
            &FrameSnapshot::default(),
        );
        assert_eq!(effect, Effect::None);

        let effect = apply_action(
            &Action::FormTextInput(key(AppKeyCode::Char('n'))),
            &mut state,
            &FrameSnapshot::default(),
        );
        assert_eq!(effect, Effect::None);

        let effect = apply_action(
            &Action::FormTextInput(key(AppKeyCode::Char('e'))),
            &mut state,
            &FrameSnapshot::default(),
        );
        assert_eq!(effect, Effect::None);

        let argv = crate::pipeline::build_command_line(&state);
        assert_eq!(argv, vec!["tool".to_string(), "--color=ne".to_string()]);
    }

    #[test]
    fn optional_value_with_choices_keeps_appending_partial_text() {
        let mut state = AppState::from_command(
            &Command::new("tool").arg(
                Arg::new("color")
                    .long("color")
                    .num_args(0..=1)
                    .require_equals(true)
                    .default_value("auto")
                    .default_missing_value("always")
                    .value_parser(["auto", "always", "never"]),
            ),
        );

        let effect = apply_action(
            &Action::FormWidgetInput(key(AppKeyCode::Right)),
            &mut state,
            &FrameSnapshot::default(),
        );
        assert_eq!(effect, Effect::None);

        let effect = apply_action(
            &Action::FormTextInput(key(AppKeyCode::Char('n'))),
            &mut state,
            &FrameSnapshot::default(),
        );
        assert_eq!(effect, Effect::None);

        let effect = apply_action(
            &Action::FormTextInput(key(AppKeyCode::Char('e'))),
            &mut state,
            &FrameSnapshot::default(),
        );
        assert_eq!(effect, Effect::None);

        let arg = state
            .domain
            .current_command()
            .args
            .iter()
            .find(|arg| arg.id == "color")
            .expect("color arg");
        assert_eq!(
            state
                .domain
                .current_form()
                .and_then(|form| form.compatibility_value(arg)),
            Some(crate::input::ArgValue::Text("ne".to_string()))
        );
        let argv = crate::pipeline::build_command_line(&state);
        assert_eq!(argv, vec!["tool".to_string(), "--color=ne".to_string()]);
    }

    #[test]
    fn trailing_argv_text_input_does_not_write_into_previous_positional() {
        let mut state = AppState::from_command(
            &Command::new("tool")
                .arg(Arg::new("program").required(true).index(1))
                .arg(
                    Arg::new("argv")
                        .index(2)
                        .action(ArgAction::Append)
                        .num_args(1..)
                        .trailing_var_arg(true)
                        .allow_hyphen_values(true),
                ),
        );
        state.ui.selected_arg_index = 1;

        let effect = apply_action(
            &Action::FormTextInput(key(AppKeyCode::Char('a'))),
            &mut state,
            &FrameSnapshot::default(),
        );
        assert_eq!(effect, Effect::None);

        let program = state
            .domain
            .current_command()
            .args
            .iter()
            .find(|arg| arg.id == "program")
            .expect("program arg");
        let argv_arg = state
            .domain
            .current_command()
            .args
            .iter()
            .find(|arg| arg.id == "argv")
            .expect("argv arg");

        let form = state.domain.current_form().expect("form state");
        assert_eq!(form.compatibility_value(program), None);
        assert_eq!(
            form.compatibility_value(argv_arg),
            Some(crate::input::ArgValue::Text("a".to_string()))
        );
    }

    #[test]
    fn repeated_row_shortcuts_reorder_occurrences() {
        let mut state = AppState::from_command(
            &Command::new("tool").arg(
                Arg::new("include")
                    .long("include")
                    .action(ArgAction::Append)
                    .num_args(1),
            ),
        );
        state.domain.set_text_value("include", "alpha\nbeta\ngamma");
        let arg = state
            .domain
            .current_command()
            .args
            .iter()
            .find(|arg| arg.id == "include")
            .cloned()
            .expect("include arg");
        crate::form_editor::set_cursor_from_click(&mut state, &arg, 1, 0);

        let effect = apply_action(
            &Action::FormWidgetInput(AppKeyEvent::new(
                AppKeyCode::Down,
                AppKeyModifiers {
                    alt: true,
                    ..AppKeyModifiers::default()
                },
            )),
            &mut state,
            &FrameSnapshot::default(),
        );

        assert_eq!(effect, Effect::None);
        let argv = crate::pipeline::build_command_line(&state);
        assert_eq!(
            argv,
            vec![
                "tool".to_string(),
                "--include".to_string(),
                "alpha".to_string(),
                "--include".to_string(),
                "gamma".to_string(),
                "--include".to_string(),
                "beta".to_string(),
            ]
        );
    }

    #[test]
    fn repeated_row_shortcuts_remove_the_current_occurrence() {
        let mut state = AppState::from_command(
            &Command::new("tool").arg(
                Arg::new("include")
                    .long("include")
                    .action(ArgAction::Append)
                    .num_args(1),
            ),
        );
        state.domain.set_text_value("include", "alpha\nbeta\ngamma");
        let arg = state
            .domain
            .current_command()
            .args
            .iter()
            .find(|arg| arg.id == "include")
            .cloned()
            .expect("include arg");
        crate::form_editor::set_cursor_from_click(&mut state, &arg, 1, 0);

        let effect = apply_action(
            &Action::FormWidgetInput(AppKeyEvent::new(
                AppKeyCode::Delete,
                AppKeyModifiers {
                    control: true,
                    ..AppKeyModifiers::default()
                },
            )),
            &mut state,
            &FrameSnapshot::default(),
        );

        assert_eq!(effect, Effect::None);
        let argv = crate::pipeline::build_command_line(&state);
        assert_eq!(
            argv,
            vec![
                "tool".to_string(),
                "--include".to_string(),
                "alpha".to_string(),
                "--include".to_string(),
                "gamma".to_string(),
            ]
        );
    }

    #[test]
    fn repeated_row_insert_keeps_an_editable_blank_row() {
        let mut state = AppState::from_command(
            &Command::new("tool").arg(
                Arg::new("include")
                    .long("include")
                    .action(ArgAction::Append)
                    .num_args(1),
            ),
        );
        state.domain.set_text_value("include", "alpha\nbeta");
        let arg = state
            .domain
            .current_command()
            .args
            .iter()
            .find(|arg| arg.id == "include")
            .cloned()
            .expect("include arg");
        crate::form_editor::set_cursor_from_click(&mut state, &arg, 0, 0);

        let effect = apply_action(
            &Action::FormWidgetInput(key(AppKeyCode::Enter)),
            &mut state,
            &FrameSnapshot::default(),
        );

        assert_eq!(effect, Effect::None);

        let editor = crate::form_editor::editor_for_render(
            &state.ui,
            arg.owner_path(),
            &arg,
            &crate::form_editor::displayed_text(&state, &arg),
        );
        assert_eq!(editor.text(), "alpha\n\nbeta");

        apply_action(
            &Action::FormTextInput(key(AppKeyCode::Char('x'))),
            &mut state,
            &FrameSnapshot::default(),
        );

        let argv = crate::pipeline::build_command_line(&state);
        assert_eq!(
            argv,
            vec![
                "tool".to_string(),
                "--include".to_string(),
                "alpha".to_string(),
                "--include".to_string(),
                "x".to_string(),
                "--include".to_string(),
                "beta".to_string(),
            ]
        );
    }
}
