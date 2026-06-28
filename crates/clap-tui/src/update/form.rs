use crate::controller::navigation;
use crate::form_editor::{self, EditResult, RepeatedRowEditResult};
use crate::frame_snapshot::FrameSnapshot;
use crate::input::{AppState, MouseSelection};
use crate::layout::form as form_layout;
use crate::query::{
    form::{self, FieldWidget, OrderedArg},
    selectors,
};
use crate::repeated_field::repeated_input_height;
use crate::runtime::{AppKeyCode, AppKeyEvent, AppMouseEvent};
use ratatui::layout::Rect;

use super::field_hit::{self, RepeatedInputTarget};
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
        Action::FormWidgetInput(key) => apply_form_widget_input(*key, state, frame_snapshot),
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
    let root = state.domain.root.clone();
    let selected_path = state.domain.selected_path().clone();
    let args = selectors::visible_form_args(&root, &selected_path, state.ui.active_tab);
    let Some(item) = selectors::active_form_field(&args, state.ui.selected_arg_index) else {
        return;
    };
    if !item.widget.accepts_text_input() {
        return;
    }
    if !state.field_can_edit(item.arg) {
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
    let root = state.domain.root.clone();
    let selected_path = state.domain.selected_path().clone();
    let args = selectors::visible_form_args(&root, &selected_path, state.ui.active_tab);
    let Some(item) = selectors::active_form_field(&args, state.ui.selected_arg_index) else {
        return;
    };
    if !item.widget.accepts_text_input() {
        return;
    }
    if !state.field_can_edit(item.arg) {
        return;
    }

    let _ = matches!(
        form_editor::apply_key_to_text_field(state, item.arg, key),
        EditResult::Handled
    );
}

fn apply_form_widget_input(key: AppKeyEvent, state: &mut AppState, frame_snapshot: &FrameSnapshot) {
    let root = state.domain.root.clone();
    let selected_path = state.domain.selected_path().clone();
    let args = selectors::visible_form_args(&root, &selected_path, state.ui.active_tab);
    let Some(item) = selectors::active_form_field(&args, state.ui.selected_arg_index) else {
        return;
    };
    if !state.field_can_edit(item.arg) {
        return;
    }

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
        let result = if matches!(key.code, AppKeyCode::Enter) {
            form_editor::activate_repeated_row(state, item.arg)
        } else if matches!(key.code, AppKeyCode::Up) && !key.modifiers.alt {
            form_editor::navigate_repeated_row(state, item.arg, -1)
        } else if matches!(key.code, AppKeyCode::Down) && !key.modifiers.alt {
            form_editor::navigate_repeated_row(state, item.arg, 1)
        } else if matches!(key.code, AppKeyCode::Up) && key.modifiers.alt {
            form_editor::move_repeated_row_up(state, item.arg)
        } else if matches!(key.code, AppKeyCode::Down) && key.modifiers.alt {
            form_editor::move_repeated_row_down(state, item.arg)
        } else if matches!(key.code, AppKeyCode::Delete | AppKeyCode::Backspace)
            && key.modifiers.control
        {
            form_editor::remove_repeated_row(state, item.arg)
        } else {
            RepeatedRowEditResult::Ignored
        };
        if matches!(key.code, AppKeyCode::Up | AppKeyCode::Down)
            && !key.modifiers.alt
            && matches!(result, RepeatedRowEditResult::Ignored)
        {
            navigation::move_form_selection(
                state,
                frame_snapshot,
                if matches!(key.code, AppKeyCode::Up) {
                    -1
                } else {
                    1
                },
            );
        } else {
            handle_repeated_row_result(state, frame_snapshot, &item.arg.id, result);
        }
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

fn handle_repeated_row_result(
    state: &mut AppState,
    frame_snapshot: &FrameSnapshot,
    arg_id: &str,
    result: RepeatedRowEditResult,
) {
    if result.should_recompute_visibility() {
        navigation::ensure_active_repeated_row_visible(
            state,
            frame_snapshot,
            arg_id,
            result.active_row_index(),
        );
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
    if !state.field_can_edit(&arg) {
        state.ui.close_dropdown();
        return;
    }
    let widget = form::widget_for(&arg);
    let required = state.field_required(&arg);
    let has_clear_row = form::single_choice_has_clear_row(&arg, widget, required);
    let clear_offset = usize::from(has_clear_row);
    let len = arg.choices.len().saturating_add(clear_offset);
    let is_multi = matches!(widget, FieldWidget::MultiChoice);

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
            let row = state.ui.dropdown_cursor(len);
            if has_clear_row && row == 0 {
                state.domain.clear_value_and_untouch(&arg.id);
                state.ui.close_dropdown();
                return;
            }
            let Some(choice) = arg.choices.get(row - clear_offset) else {
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
                let row = state.ui.dropdown_cursor(len);
                if has_clear_row && row == 0 {
                    state.domain.clear_value_and_untouch(&arg.id);
                } else if let Some(choice) = arg.choices.get(row - clear_offset) {
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
    if frame_snapshot
        .form_content_y(event.row, state.ui.form_scroll(frame_snapshot))
        .is_none()
    {
        return;
    }
    state.ui.focus_form();
    let root = state.domain.root.clone();
    let selected_path = state.domain.selected_path().clone();
    let args = selectors::visible_form_args(&root, &selected_path, state.ui.active_tab);
    let derived = state.derived().clone();
    let input_height_overrides = repeated_input_height_overrides(&args, state);
    if let Some(hit) = form_click_hit(
        &args,
        event,
        state,
        frame_snapshot,
        &derived.validation.field_errors,
        &input_height_overrides,
        &derived.field_semantics,
    ) {
        state.ui.set_selected_arg_index(hit.item.order_index);
        if !hit.in_input && !hit.in_label && hit.in_description {
            // Orphan-description hit: the input is fully off-screen above the
            // viewport but the description row is still visible. Selecting the
            // field and scrolling it back into view is the contract.
            navigation::ensure_form_visible(state, frame_snapshot);
            return;
        }
        let can_edit = state.field_can_edit(hit.item.arg);
        if !can_edit {
            navigation::ensure_form_visible(state, frame_snapshot);
            return;
        }
        if matches!(hit.item.widget, FieldWidget::Counter) && hit.in_input {
            apply_counter_click(event, state, frame_snapshot, &hit.item.arg.id);
        } else if matches!(hit.item.widget, FieldWidget::RepeatedText) && hit.in_input {
            let result = apply_repeated_text_click(event, state, frame_snapshot, &hit.item.arg.id);
            handle_repeated_row_result(state, frame_snapshot, &hit.item.arg.id, result);
            if !matches!(result, RepeatedRowEditResult::Ignored) {
                return;
            }
        } else if matches!(
            hit.item.widget,
            FieldWidget::Toggle | FieldWidget::OptionalValue
        ) && hit.in_input
        {
            navigation::activate_form_field(state, frame_snapshot);
        }
        if hit.item.widget.uses_choice_popup() && hit.in_input {
            let total = state
                .domain
                .arg_for_input(&hit.item.arg.id)
                .map_or(0, |arg| arg.choices.len());
            navigation::open_enum_dropdown(state, frame_snapshot, &hit.item.arg.id, total);
        }
        if hit.item.widget.accepts_text_input()
            && let Some(arg) = state.domain.arg_for_input(&hit.item.arg.id).cloned()
        {
            form_editor::clear_selection(state, &arg);
            state.ui.clear_mouse_selection();
            if let Some((row, col)) = text_input_position_from_point(
                frame_snapshot,
                &hit.item.arg.id,
                event.column,
                event.row,
                false,
            ) {
                let (row, col) = form_editor::click_cursor_position(state, &arg, row, col);
                state.ui.set_mouse_selection(Some(MouseSelection {
                    arg_id: hit.item.arg.id.clone(),
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

fn apply_repeated_text_click(
    event: AppMouseEvent,
    state: &mut AppState,
    frame_snapshot: &FrameSnapshot,
    arg_id: &str,
) -> RepeatedRowEditResult {
    let Some(field) = frame_snapshot.form_field_layout(arg_id) else {
        return RepeatedRowEditResult::Ignored;
    };
    let Some(arg) = state.domain.arg_for_input(arg_id).cloned() else {
        return RepeatedRowEditResult::Ignored;
    };
    let editor = form_editor::editor_for_render(
        &state.ui,
        arg.owner_path(),
        &arg,
        &form_editor::displayed_text(state, &arg),
    );
    let active_row_before = editor.current_row();
    let total_rows = editor.row_count().max(1);
    let Some(target) =
        field_hit::repeated_input_target_from_point(field, total_rows, event.column, event.row)
    else {
        return RepeatedRowEditResult::Ignored;
    };
    form_editor::clear_selection(state, &arg);
    state.ui.clear_mouse_selection();
    match target {
        RepeatedInputTarget::Remove { row } => {
            form_editor::set_cursor_from_click(state, &arg, row, 0);
            if total_rows > 1 {
                form_editor::remove_repeated_row(state, &arg)
            } else {
                RepeatedRowEditResult::HandledNoFocusChange
            }
        }
        RepeatedInputTarget::Add { row } => {
            form_editor::set_cursor_from_click(state, &arg, row, 0);
            form_editor::insert_repeated_row(state, &arg)
        }
        RepeatedInputTarget::Text { row, col } => {
            let row_index = usize::from(row);
            state.ui.set_mouse_selection(Some(MouseSelection {
                arg_id: arg_id.to_string(),
                anchor_row: row,
                anchor_col: col,
                active: false,
            }));
            form_editor::set_cursor_from_click(state, &arg, row, col);
            if row_index == active_row_before {
                RepeatedRowEditResult::HandledNoFocusChange
            } else {
                RepeatedRowEditResult::Handled {
                    focus_changed: true,
                    structure_changed: false,
                    active_row_index: Some(row_index),
                }
            }
        }
    }
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RepeatedControlClickTarget {
    Remove,
    Add,
}

#[cfg(test)]
fn repeated_control_click_target(
    x: u16,
    y: u16,
    row_rect: Rect,
    show_remove: bool,
    show_add: bool,
) -> Option<RepeatedControlClickTarget> {
    if show_remove
        && let Some(remove) =
            crate::repeated_field::repeated_remove_rect(row_rect, show_remove, show_add)
        && contains(remove, x, y)
    {
        Some(RepeatedControlClickTarget::Remove)
    } else if show_add
        && let Some(add) = crate::repeated_field::repeated_add_rect(row_rect)
        && contains(add, x, y)
    {
        Some(RepeatedControlClickTarget::Add)
    } else {
        None
    }
}

fn apply_counter_click(
    event: AppMouseEvent,
    state: &mut AppState,
    frame_snapshot: &FrameSnapshot,
    arg_id: &str,
) {
    let Some(field) = frame_snapshot.form_field_layout(arg_id) else {
        return;
    };
    if !field_hit::compact_control_content_row_contains(field, event.row) {
        return;
    }

    match counter_click_target(field.input, event.column) {
        Some(CounterClickTarget::Decrement) => state.domain.decrement_counter(arg_id),
        Some(CounterClickTarget::Increment) => state.domain.increment_counter(arg_id),
        None => {}
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CounterClickTarget {
    Decrement,
    Increment,
}

fn counter_click_target(input: Rect, column: u16) -> Option<CounterClickTarget> {
    if column < input.x || column >= input.x.saturating_add(input.width) || input.width == 0 {
        return None;
    }

    let relative = column.saturating_sub(input.x);
    if input.width < 7 {
        return if relative < input.width / 2 {
            Some(CounterClickTarget::Decrement)
        } else {
            Some(CounterClickTarget::Increment)
        };
    }

    let minus_start = input.width.saturating_sub(7);
    let plus_start = input.width.saturating_sub(3);
    if relative >= minus_start && relative < minus_start.saturating_add(3) {
        Some(CounterClickTarget::Decrement)
    } else if relative >= plus_start && relative < input.width {
        Some(CounterClickTarget::Increment)
    } else {
        None
    }
}

struct FormClickHit<'a> {
    item: &'a OrderedArg<'a>,
    in_input: bool,
    in_label: bool,
    in_description: bool,
}

fn form_click_hit<'a>(
    args: &'a [OrderedArg<'a>],
    event: AppMouseEvent,
    state: &AppState,
    frame_snapshot: &FrameSnapshot,
    field_errors: &std::collections::BTreeMap<String, String>,
    input_height_overrides: &std::collections::HashMap<String, u16>,
    field_semantics: &std::collections::BTreeMap<
        crate::pipeline::FieldInstanceId,
        crate::pipeline::FieldSemantics,
    >,
) -> Option<FormClickHit<'a>> {
    if let Some(hit) = frame_snapshot.form_field_at(event.column, event.row) {
        let item = args.iter().find(|item| item.arg.id == hit.arg_id)?;
        return Some(FormClickHit {
            item,
            in_input: hit.in_input,
            in_label: hit.in_label,
            in_description: hit.in_description,
        });
    }

    let content_y =
        frame_snapshot.form_content_y(event.row, state.ui.form_scroll(frame_snapshot))?;
    let hit = form_layout::hit_test_form_content_with_layout_overrides_and_semantics(
        args,
        content_y,
        field_errors,
        input_height_overrides,
        &std::collections::HashMap::new(),
        field_semantics,
    )?;
    let item = args
        .iter()
        .find(|item| item.order_index == hit.order_index)?;
    Some(FormClickHit {
        item,
        in_input: hit.in_input,
        in_label: hit.in_label,
        in_description: hit.in_description,
    })
}

fn repeated_input_height_overrides(
    args: &[OrderedArg<'_>],
    state: &AppState,
) -> std::collections::HashMap<String, u16> {
    args.iter()
        .filter(|item| matches!(item.widget, FieldWidget::RepeatedText))
        .map(|item| {
            let value = form_editor::displayed_text(state, item.arg);
            (
                item.arg.id.clone(),
                repeated_input_height(&state.ui, item.arg, &value),
            )
        })
        .collect()
}

fn text_input_position_from_point(
    frame_snapshot: &FrameSnapshot,
    arg_id: &str,
    x: u16,
    y: u16,
    clamp: bool,
) -> Option<(u16, u16)> {
    frame_snapshot
        .form_field_layout(arg_id)
        .and_then(|field| field_hit::text_input_position_from_point(field, x, y, clamp))
        .or_else(|| frame_snapshot.input_position_from_point(arg_id, x, y, clamp))
}

#[cfg(test)]
fn contains(area: Rect, x: u16, y: u16) -> bool {
    x >= area.x
        && x < area.x.saturating_add(area.width)
        && y >= area.y
        && y < area.y.saturating_add(area.height)
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
    let widget = form::widget_for(&arg);
    let required = state.field_required(&arg);
    let has_clear_row = form::single_choice_has_clear_row(&arg, widget, required);
    let clear_offset = usize::from(has_clear_row);
    let total_rows = arg.choices.len().saturating_add(clear_offset);
    let visible_rows = frame_snapshot.dropdown_visible_rows().unwrap_or(0);
    let scroll = state.ui.dropdown_scroll(total_rows, visible_rows);
    let Some(index) = frame_snapshot.dropdown_choice_index(row, scroll) else {
        return;
    };
    if index >= total_rows {
        return;
    }
    state.ui.set_dropdown_cursor(index);
    if has_clear_row && index == 0 {
        state.domain.clear_value_and_untouch(&arg.id);
        state.ui.close_dropdown();
        return;
    }
    let Some(choice) = arg.choices.get(index - clear_offset) else {
        return;
    };
    if matches!(widget, FieldWidget::MultiChoice) {
        state.domain.toggle_choice_value_touched(&arg.id, choice);
    } else {
        state
            .domain
            .set_choice_value_touched(&arg.id, choice.clone());
        state.ui.close_dropdown();
    }
}

#[cfg(test)]
mod tests {
    use clap::{Arg, ArgAction, ArgGroup, Command};

    use super::super::{Action, Effect, apply_action};
    use super::*;
    use crate::frame_snapshot::FrameSnapshot;
    use crate::layout::form::FormFieldLayout;
    use crate::repeated_field::{REPEATED_ROW_HEIGHT, repeated_add_rect, repeated_remove_rect};
    use crate::runtime::AppKeyModifiers;

    fn key(code: AppKeyCode) -> AppKeyEvent {
        AppKeyEvent::new(code, AppKeyModifiers::default())
    }

    fn repeated_navigation_state() -> AppState {
        let mut state = AppState::from_command(
            &Command::new("tool")
                .arg(Arg::new("prefix").long("prefix").action(ArgAction::Set))
                .arg(
                    Arg::new("include")
                        .long("include")
                        .action(ArgAction::Append)
                        .num_args(1),
                )
                .arg(Arg::new("suffix").long("suffix").action(ArgAction::Set)),
        );
        state.domain.set_text_value("include", "alpha\nbeta");
        state.domain.mark_touched("include");
        state.ui.focus_form();
        state
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
        // Row 0 is the synthetic "(none)" entry for non-required choices.
        // "red" -> 1, "green" -> 2, "blue" -> 3.
        state.ui.dropdown_cursor = 2;

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
    fn clicking_blank_form_space_still_moves_focus_to_form() {
        let mut state = AppState::from_command(
            &Command::new("tool")
                .arg(Arg::new("path").long("path"))
                .arg(Arg::new("name").long("name")),
        );
        state.ui.focus_sidebar();
        let mut snapshot = FrameSnapshot::default();
        snapshot.layout.form = Some(ratatui::layout::Rect::new(0, 0, 40, 10));
        snapshot.layout.form_view = Some(ratatui::layout::Rect::new(0, 0, 40, 10));

        let effect = apply_action(
            &Action::ClickForm(AppMouseEvent {
                kind: crate::runtime::AppMouseEventKind::Down(crate::runtime::AppMouseButton::Left),
                column: 1,
                row: 9,
                modifiers: AppKeyModifiers::default(),
            }),
            &mut state,
            &snapshot,
        );

        assert_eq!(effect, Effect::None);
        assert!(matches!(state.ui.focus, crate::input::Focus::Form));
    }

    #[test]
    fn fallback_form_hit_testing_uses_dynamic_repeated_heights() {
        let mut state = AppState::from_command(
            &Command::new("tool")
                .arg(Arg::new("prefix").long("prefix"))
                .arg(
                    Arg::new("include")
                        .long("include")
                        .action(ArgAction::Append)
                        .num_args(1),
                )
                .arg(Arg::new("suffix").long("suffix")),
        );
        state.domain.set_text_value("include", "alpha\nbeta\ngamma");
        state.domain.mark_touched("include");
        let mut snapshot = FrameSnapshot::default();
        snapshot.layout.form = Some(ratatui::layout::Rect::new(0, 0, 80, 20));
        snapshot.layout.form_view = Some(ratatui::layout::Rect::new(0, 0, 80, 20));

        let effect = apply_action(
            &Action::ClickForm(AppMouseEvent {
                kind: crate::runtime::AppMouseEventKind::Down(crate::runtime::AppMouseButton::Left),
                column: 1,
                row: 11,
                modifiers: AppKeyModifiers::default(),
            }),
            &mut state,
            &snapshot,
        );

        assert_eq!(effect, Effect::None);
        assert_eq!(state.ui.selected_arg_index, 1);
    }

    #[test]
    fn form_clicks_use_visible_snapshot_geometry_for_scrolled_error_fields() {
        let mut state = AppState::from_command(
            &Command::new("tool")
                .group(ArgGroup::new("mode").args(["fast", "safe"]))
                .arg(
                    Arg::new("fast")
                        .long("fast")
                        .action(ArgAction::SetTrue)
                        .group("mode"),
                )
                .arg(
                    Arg::new("safe")
                        .long("safe")
                        .action(ArgAction::SetTrue)
                        .group("mode"),
                ),
        );
        state.domain.toggle_flag_touched("fast");
        state.domain.toggle_flag_touched("safe");
        assert!(state.derived_validation().field_errors.contains_key("fast"));

        state.ui.set_form_scroll(99);
        let mut snapshot = FrameSnapshot {
            form_scroll_max: 99,
            ..FrameSnapshot::default()
        };
        snapshot.layout.form = Some(ratatui::layout::Rect::new(0, 0, 80, 10));
        snapshot.layout.form_view = Some(ratatui::layout::Rect::new(0, 0, 80, 10));
        snapshot.layout.form_fields.push(FormFieldLayout {
            arg_id: "fast".to_string(),
            heading: None,
            label: Some(ratatui::layout::Rect::new(0, 2, 10, 1)),
            input: ratatui::layout::Rect::new(12, 2, 20, 1),
            input_clip_top: 0,
            full_input_height: 1,
            description: Some(ratatui::layout::Rect::new(12, 3, 40, 1)),
        });

        let effect = apply_action(
            &Action::ClickForm(AppMouseEvent {
                kind: crate::runtime::AppMouseEventKind::Down(crate::runtime::AppMouseButton::Left),
                column: 13,
                row: 2,
                modifiers: AppKeyModifiers::default(),
            }),
            &mut state,
            &snapshot,
        );

        assert_eq!(effect, Effect::None);
        let argv = crate::pipeline::build_authoritative_command_line(&state);
        assert_eq!(argv, vec!["tool".to_string(), "--safe".to_string()]);
    }

    #[test]
    fn counter_mouse_buttons_increment_and_decrement() {
        let mut state = AppState::from_command(
            &Command::new("tool").arg(Arg::new("verbose").short('v').action(ArgAction::Count)),
        );
        let mut snapshot = FrameSnapshot::default();
        snapshot.layout.form = Some(ratatui::layout::Rect::new(0, 0, 80, 6));
        snapshot.layout.form_view = Some(ratatui::layout::Rect::new(0, 0, 80, 6));
        snapshot.layout.form_inputs.insert(
            "verbose".to_string(),
            ratatui::layout::Rect::new(10, 1, 30, 3),
        );
        snapshot.layout.form_fields.push(FormFieldLayout {
            arg_id: "verbose".to_string(),
            heading: None,
            label: Some(ratatui::layout::Rect::new(0, 1, 9, 1)),
            input: ratatui::layout::Rect::new(10, 1, 30, 3),
            input_clip_top: 0,
            full_input_height: 3,
            description: None,
        });

        let effect = apply_action(
            &Action::ClickForm(AppMouseEvent {
                kind: crate::runtime::AppMouseEventKind::Down(crate::runtime::AppMouseButton::Left),
                column: 38,
                row: 2,
                modifiers: AppKeyModifiers::default(),
            }),
            &mut state,
            &snapshot,
        );

        assert_eq!(effect, Effect::None);
        assert_eq!(
            crate::pipeline::build_authoritative_command_line(&state),
            vec!["tool".to_string(), "-v".to_string()]
        );

        let effect = apply_action(
            &Action::ClickForm(AppMouseEvent {
                kind: crate::runtime::AppMouseEventKind::Down(crate::runtime::AppMouseButton::Left),
                column: 34,
                row: 2,
                modifiers: AppKeyModifiers::default(),
            }),
            &mut state,
            &snapshot,
        );

        assert_eq!(effect, Effect::None);
        assert_eq!(
            crate::pipeline::build_authoritative_command_line(&state),
            vec!["tool".to_string()]
        );
    }

    #[test]
    fn clipped_counter_border_only_click_does_not_change_value() {
        let mut state = AppState::from_command(
            &Command::new("tool").arg(Arg::new("verbose").short('v').action(ArgAction::Count)),
        );
        let mut snapshot = FrameSnapshot::default();
        snapshot.layout.form = Some(ratatui::layout::Rect::new(0, 0, 80, 3));
        snapshot.layout.form_view = Some(ratatui::layout::Rect::new(0, 0, 80, 3));
        snapshot.layout.form_inputs.insert(
            "verbose".to_string(),
            ratatui::layout::Rect::new(10, 1, 30, 1),
        );
        snapshot.layout.form_fields.push(FormFieldLayout {
            arg_id: "verbose".to_string(),
            heading: None,
            label: Some(ratatui::layout::Rect::new(0, 1, 9, 1)),
            input: ratatui::layout::Rect::new(10, 1, 30, 1),
            input_clip_top: 0,
            full_input_height: 1,
            description: None,
        });

        let effect = apply_action(
            &Action::ClickForm(AppMouseEvent {
                kind: crate::runtime::AppMouseEventKind::Down(crate::runtime::AppMouseButton::Left),
                column: 38,
                row: 1,
                modifiers: AppKeyModifiers::default(),
            }),
            &mut state,
            &snapshot,
        );

        assert_eq!(effect, Effect::None);
        assert_eq!(
            crate::pipeline::build_authoritative_command_line(&state),
            vec!["tool".to_string()]
        );
    }

    #[test]
    fn top_clipped_counter_content_row_still_clicks_visible_controls() {
        let mut state = AppState::from_command(
            &Command::new("tool").arg(Arg::new("verbose").short('v').action(ArgAction::Count)),
        );
        let mut snapshot = FrameSnapshot::default();
        snapshot.layout.form = Some(ratatui::layout::Rect::new(0, 0, 80, 3));
        snapshot.layout.form_view = Some(ratatui::layout::Rect::new(0, 0, 80, 3));
        snapshot.layout.form_inputs.insert(
            "verbose".to_string(),
            ratatui::layout::Rect::new(10, 1, 30, 2),
        );
        snapshot.layout.form_fields.push(FormFieldLayout {
            arg_id: "verbose".to_string(),
            heading: None,
            label: Some(ratatui::layout::Rect::new(0, 1, 9, 1)),
            input: ratatui::layout::Rect::new(10, 1, 30, 2),
            input_clip_top: 1,
            full_input_height: 3,
            description: None,
        });

        let effect = apply_action(
            &Action::ClickForm(AppMouseEvent {
                kind: crate::runtime::AppMouseEventKind::Down(crate::runtime::AppMouseButton::Left),
                column: 38,
                row: 1,
                modifiers: AppKeyModifiers::default(),
            }),
            &mut state,
            &snapshot,
        );

        assert_eq!(effect, Effect::None);
        assert_eq!(
            crate::pipeline::build_authoritative_command_line(&state),
            vec!["tool".to_string(), "-v".to_string()]
        );
    }

    #[test]
    fn repeated_text_remove_chip_removes_the_clicked_row() {
        let mut state = AppState::from_command(
            &Command::new("tool").arg(
                Arg::new("include")
                    .long("include")
                    .action(ArgAction::Append)
                    .num_args(1),
            ),
        );
        state.domain.set_text_value("include", "alpha\nbeta");
        state.domain.mark_touched("include");
        let mut snapshot = FrameSnapshot::default();
        snapshot.layout.form = Some(ratatui::layout::Rect::new(0, 0, 80, 8));
        snapshot.layout.form_view = Some(ratatui::layout::Rect::new(0, 0, 80, 8));
        snapshot.layout.form_inputs.insert(
            "include".to_string(),
            ratatui::layout::Rect::new(10, 1, 30, 6),
        );
        snapshot.layout.form_fields.push(FormFieldLayout {
            arg_id: "include".to_string(),
            heading: None,
            label: Some(ratatui::layout::Rect::new(0, 1, 9, 1)),
            input: ratatui::layout::Rect::new(10, 1, 30, 6),
            input_clip_top: 0,
            full_input_height: 6,
            description: None,
        });

        assert_eq!(
            repeated_control_click_target(
                34,
                5,
                ratatui::layout::Rect::new(10, 4, 30, 3),
                true,
                true,
            ),
            Some(RepeatedControlClickTarget::Remove)
        );

        let effect = apply_action(
            &Action::ClickForm(AppMouseEvent {
                kind: crate::runtime::AppMouseEventKind::Down(crate::runtime::AppMouseButton::Left),
                column: 34,
                row: 5,
                modifiers: AppKeyModifiers::default(),
            }),
            &mut state,
            &snapshot,
        );

        assert_eq!(effect, Effect::None);
        assert_eq!(
            crate::pipeline::build_authoritative_command_line(&state),
            vec![
                "tool".to_string(),
                "--include".to_string(),
                "alpha".to_string()
            ]
        );
    }

    #[test]
    fn repeated_text_remove_chip_on_the_first_row_removes_that_row() {
        let mut state = AppState::from_command(
            &Command::new("tool").arg(
                Arg::new("include")
                    .long("include")
                    .action(ArgAction::Append)
                    .num_args(1),
            ),
        );
        state.domain.set_text_value("include", "alpha\nbeta");
        state.domain.mark_touched("include");
        let mut snapshot = FrameSnapshot::default();
        snapshot.layout.form = Some(ratatui::layout::Rect::new(0, 0, 80, 8));
        snapshot.layout.form_view = Some(ratatui::layout::Rect::new(0, 0, 80, 8));
        snapshot.layout.form_inputs.insert(
            "include".to_string(),
            ratatui::layout::Rect::new(10, 1, 30, 6),
        );
        snapshot.layout.form_fields.push(FormFieldLayout {
            arg_id: "include".to_string(),
            heading: None,
            label: Some(ratatui::layout::Rect::new(0, 1, 9, 1)),
            input: ratatui::layout::Rect::new(10, 1, 30, 6),
            input_clip_top: 0,
            full_input_height: 6,
            description: None,
        });

        assert_eq!(
            repeated_control_click_target(
                36,
                2,
                ratatui::layout::Rect::new(10, 1, 30, 3),
                true,
                false,
            ),
            Some(RepeatedControlClickTarget::Remove)
        );

        let effect = apply_action(
            &Action::ClickForm(AppMouseEvent {
                kind: crate::runtime::AppMouseEventKind::Down(crate::runtime::AppMouseButton::Left),
                column: 36,
                row: 2,
                modifiers: AppKeyModifiers::default(),
            }),
            &mut state,
            &snapshot,
        );

        assert_eq!(effect, Effect::None);
        assert_eq!(
            crate::pipeline::build_authoritative_command_line(&state),
            vec![
                "tool".to_string(),
                "--include".to_string(),
                "beta".to_string()
            ]
        );
    }

    #[test]
    fn repeated_text_remove_chip_in_the_middle_row_uses_the_external_gutter() {
        let mut state = AppState::from_command(
            &Command::new("tool").arg(
                Arg::new("include")
                    .long("include")
                    .action(ArgAction::Append)
                    .num_args(1),
            ),
        );
        state.domain.set_text_value("include", "alpha\nbeta\ngamma");
        state.domain.mark_touched("include");
        let mut snapshot = FrameSnapshot::default();
        snapshot.layout.form = Some(ratatui::layout::Rect::new(0, 0, 80, 11));
        snapshot.layout.form_view = Some(ratatui::layout::Rect::new(0, 0, 80, 11));
        snapshot.layout.form_inputs.insert(
            "include".to_string(),
            ratatui::layout::Rect::new(10, 1, 30, 9),
        );
        snapshot.layout.form_fields.push(FormFieldLayout {
            arg_id: "include".to_string(),
            heading: None,
            label: Some(ratatui::layout::Rect::new(0, 1, 9, 1)),
            input: ratatui::layout::Rect::new(10, 1, 30, 9),
            input_clip_top: 0,
            full_input_height: 9,
            description: None,
        });
        let arg = state
            .domain
            .arg_for_input("include")
            .cloned()
            .expect("include arg");
        crate::form_editor::set_cursor_from_click(&mut state, &arg, 1, 0);

        assert_eq!(
            repeated_control_click_target(
                36,
                5,
                ratatui::layout::Rect::new(10, 4, 30, 3),
                true,
                false,
            ),
            Some(RepeatedControlClickTarget::Remove)
        );

        let effect = apply_action(
            &Action::ClickForm(AppMouseEvent {
                kind: crate::runtime::AppMouseEventKind::Down(crate::runtime::AppMouseButton::Left),
                column: 36,
                row: 5,
                modifiers: AppKeyModifiers::default(),
            }),
            &mut state,
            &snapshot,
        );

        assert_eq!(effect, Effect::None);
        assert_eq!(
            crate::pipeline::build_authoritative_command_line(&state),
            vec![
                "tool".to_string(),
                "--include".to_string(),
                "alpha".to_string(),
                "--include".to_string(),
                "gamma".to_string()
            ]
        );
    }

    #[test]
    fn clipped_single_row_repeated_controls_do_not_receive_hits_below_the_visible_row() {
        let row_rect = ratatui::layout::Rect::new(10, 1, 30, 1);

        assert_eq!(repeated_remove_rect(row_rect, true, true), None);
        assert_eq!(repeated_add_rect(row_rect), None);
        assert_eq!(
            repeated_control_click_target(34, 2, row_rect, true, true),
            None
        );
    }

    #[test]
    fn repeated_text_add_chip_leaves_the_far_right_column_inactive() {
        let row_rect = ratatui::layout::Rect::new(10, 1, 30, 3);

        assert_eq!(
            repeated_add_rect(row_rect),
            Some(ratatui::layout::Rect::new(36, 2, 3, 1))
        );
        assert_eq!(
            repeated_control_click_target(39, 2, row_rect, true, true),
            None
        );
        assert_eq!(
            repeated_control_click_target(38, 2, row_rect, true, true),
            Some(RepeatedControlClickTarget::Add)
        );
    }

    #[test]
    fn repeated_text_buttons_keep_a_gap_between_remove_and_add() {
        let row_rect = ratatui::layout::Rect::new(10, 1, 30, 3);

        assert_eq!(
            repeated_remove_rect(row_rect, true, true),
            Some(ratatui::layout::Rect::new(32, 2, 3, 1))
        );
        assert_eq!(
            repeated_add_rect(row_rect),
            Some(ratatui::layout::Rect::new(36, 2, 3, 1))
        );
        assert_eq!(
            repeated_control_click_target(35, 2, row_rect, true, true),
            None
        );
    }

    #[test]
    fn repeated_text_clicks_follow_outer_scroll_row_order_when_clipped() {
        let mut state = AppState::from_command(
            &Command::new("tool").arg(
                Arg::new("include")
                    .long("include")
                    .action(ArgAction::Append)
                    .num_args(1),
            ),
        );
        state.domain.set_text_value("include", "alpha\nbeta\ngamma");
        state.domain.mark_touched("include");
        let mut snapshot = FrameSnapshot::default();
        snapshot.layout.form = Some(ratatui::layout::Rect::new(0, 0, 80, 5));
        snapshot.layout.form_view = Some(ratatui::layout::Rect::new(0, 0, 80, 5));
        snapshot.layout.form_inputs.insert(
            "include".to_string(),
            ratatui::layout::Rect::new(10, 1, 30, 3),
        );
        snapshot.layout.form_fields.push(FormFieldLayout {
            arg_id: "include".to_string(),
            heading: None,
            label: Some(ratatui::layout::Rect::new(0, 1, 9, 1)),
            input: ratatui::layout::Rect::new(10, 1, 30, 3),
            input_clip_top: REPEATED_ROW_HEIGHT,
            full_input_height: REPEATED_ROW_HEIGHT + 3,
            description: None,
        });
        let arg = state
            .domain
            .arg_for_input("include")
            .cloned()
            .expect("include arg");
        crate::form_editor::set_cursor_from_click(&mut state, &arg, 2, 0);

        let effect = apply_action(
            &Action::ClickForm(AppMouseEvent {
                kind: crate::runtime::AppMouseEventKind::Down(crate::runtime::AppMouseButton::Left),
                column: 36,
                row: 2,
                modifiers: AppKeyModifiers::default(),
            }),
            &mut state,
            &snapshot,
        );

        assert_eq!(effect, Effect::None);
        assert_eq!(
            crate::pipeline::build_authoritative_command_line(&state),
            vec![
                "tool".to_string(),
                "--include".to_string(),
                "alpha".to_string(),
                "--include".to_string(),
                "gamma".to_string()
            ]
        );
    }

    #[test]
    fn repeated_text_clicks_use_content_rows_when_partially_clipped() {
        let mut state = AppState::from_command(
            &Command::new("tool").arg(
                Arg::new("include")
                    .long("include")
                    .action(ArgAction::Append)
                    .num_args(1),
            ),
        );
        state.domain.set_text_value("include", "alpha\nbeta\ngamma");
        state.domain.mark_touched("include");
        let mut snapshot = FrameSnapshot::default();
        snapshot.layout.form = Some(ratatui::layout::Rect::new(0, 0, 80, 5));
        snapshot.layout.form_view = Some(ratatui::layout::Rect::new(0, 0, 80, 5));
        snapshot.layout.form_inputs.insert(
            "include".to_string(),
            ratatui::layout::Rect::new(10, 1, 30, 3),
        );
        snapshot.layout.form_fields.push(FormFieldLayout {
            arg_id: "include".to_string(),
            heading: None,
            label: Some(ratatui::layout::Rect::new(0, 1, 9, 1)),
            input: ratatui::layout::Rect::new(10, 1, 30, 3),
            input_clip_top: 1,
            full_input_height: 4,
            description: None,
        });
        let arg = state
            .domain
            .arg_for_input("include")
            .cloned()
            .expect("include arg");
        crate::form_editor::set_cursor_from_click(&mut state, &arg, 2, 0);

        let effect = apply_action(
            &Action::ClickForm(AppMouseEvent {
                kind: crate::runtime::AppMouseEventKind::Down(crate::runtime::AppMouseButton::Left),
                column: 35,
                row: 1,
                modifiers: AppKeyModifiers::default(),
            }),
            &mut state,
            &snapshot,
        );

        assert_eq!(effect, Effect::None);
        assert_eq!(
            crate::pipeline::build_authoritative_command_line(&state),
            vec![
                "tool".to_string(),
                "--include".to_string(),
                "beta".to_string(),
                "--include".to_string(),
                "gamma".to_string()
            ]
        );
    }

    #[test]
    fn repeated_text_add_chip_creates_another_editor_row() {
        let mut state = AppState::from_command(
            &Command::new("tool").arg(
                Arg::new("include")
                    .long("include")
                    .action(ArgAction::Append)
                    .num_args(1),
            ),
        );
        state.domain.set_text_value("include", "alpha\nbeta");
        state.domain.mark_touched("include");
        let mut snapshot = FrameSnapshot::default();
        snapshot.layout.form = Some(ratatui::layout::Rect::new(0, 0, 80, 8));
        snapshot.layout.form_view = Some(ratatui::layout::Rect::new(0, 0, 80, 8));
        snapshot.layout.form_inputs.insert(
            "include".to_string(),
            ratatui::layout::Rect::new(10, 1, 30, 6),
        );
        snapshot.layout.form_fields.push(FormFieldLayout {
            arg_id: "include".to_string(),
            heading: None,
            label: Some(ratatui::layout::Rect::new(0, 1, 9, 1)),
            input: ratatui::layout::Rect::new(10, 1, 30, 6),
            input_clip_top: 0,
            full_input_height: 6,
            description: None,
        });

        let effect = apply_action(
            &Action::ClickForm(AppMouseEvent {
                kind: crate::runtime::AppMouseEventKind::Down(crate::runtime::AppMouseButton::Left),
                column: 37,
                row: 5,
                modifiers: AppKeyModifiers::default(),
            }),
            &mut state,
            &snapshot,
        );

        assert_eq!(effect, Effect::None);
        let arg = state.domain.arg_for_input("include").expect("include arg");
        let editor = crate::form_editor::editor_for_render(
            &state.ui,
            arg.owner_path(),
            arg,
            &crate::form_editor::displayed_text(&state, arg),
        );
        assert_eq!(editor.row_count(), 3);
    }

    #[test]
    fn repeated_text_single_row_add_chip_creates_another_editor_row() {
        let mut state = AppState::from_command(
            &Command::new("tool").arg(
                Arg::new("tag")
                    .long("tag")
                    .action(ArgAction::Append)
                    .num_args(1),
            ),
        );
        let mut snapshot = FrameSnapshot::default();
        snapshot.layout.form = Some(ratatui::layout::Rect::new(0, 0, 80, 6));
        snapshot.layout.form_view = Some(ratatui::layout::Rect::new(0, 0, 80, 6));
        snapshot
            .layout
            .form_inputs
            .insert("tag".to_string(), ratatui::layout::Rect::new(10, 1, 30, 3));
        snapshot.layout.form_fields.push(FormFieldLayout {
            arg_id: "tag".to_string(),
            heading: None,
            label: Some(ratatui::layout::Rect::new(0, 1, 9, 1)),
            input: ratatui::layout::Rect::new(10, 1, 30, 3),
            input_clip_top: 0,
            full_input_height: 3,
            description: None,
        });

        let effect = apply_action(
            &Action::ClickForm(AppMouseEvent {
                kind: crate::runtime::AppMouseEventKind::Down(crate::runtime::AppMouseButton::Left),
                column: 37,
                row: 2,
                modifiers: AppKeyModifiers::default(),
            }),
            &mut state,
            &snapshot,
        );

        assert_eq!(effect, Effect::None);
        let arg = state.domain.arg_for_input("tag").expect("tag arg");
        let editor = crate::form_editor::editor_for_render(
            &state.ui,
            arg.owner_path(),
            arg,
            &crate::form_editor::displayed_text(&state, arg),
        );
        assert_eq!(editor.row_count(), 2);
    }

    #[test]
    fn repeated_text_add_chip_scrolls_to_reveal_the_new_row() {
        let mut state = AppState::from_command(
            &Command::new("tool").arg(
                Arg::new("include")
                    .long("include")
                    .action(ArgAction::Append)
                    .num_args(1),
            ),
        );
        state.domain.set_text_value("include", "alpha\nbeta");
        state.domain.mark_touched("include");
        let mut snapshot = FrameSnapshot::default();
        snapshot.layout.form = Some(ratatui::layout::Rect::new(0, 0, 80, 6));
        snapshot.layout.form_view = Some(ratatui::layout::Rect::new(0, 0, 80, 6));
        snapshot.layout.form_inputs.insert(
            "include".to_string(),
            ratatui::layout::Rect::new(10, 0, 30, 6),
        );
        snapshot.layout.form_fields.push(FormFieldLayout {
            arg_id: "include".to_string(),
            heading: None,
            label: Some(ratatui::layout::Rect::new(0, 0, 9, 1)),
            input: ratatui::layout::Rect::new(10, 0, 30, 6),
            input_clip_top: 0,
            full_input_height: 6,
            description: None,
        });

        let effect = apply_action(
            &Action::ClickForm(AppMouseEvent {
                kind: crate::runtime::AppMouseEventKind::Down(crate::runtime::AppMouseButton::Left),
                column: 37,
                row: 4,
                modifiers: AppKeyModifiers::default(),
            }),
            &mut state,
            &snapshot,
        );

        assert_eq!(effect, Effect::None);
        assert_eq!(state.ui.form_scroll, 3);
    }

    #[test]
    fn clicking_an_already_visible_repeated_row_does_not_scroll() {
        let mut state = AppState::from_command(
            &Command::new("tool").arg(
                Arg::new("include")
                    .long("include")
                    .action(ArgAction::Append)
                    .num_args(1),
            ),
        );
        state.domain.set_text_value("include", "alpha\nbeta");
        state.domain.mark_touched("include");
        let mut snapshot = FrameSnapshot::default();
        snapshot.layout.form = Some(ratatui::layout::Rect::new(0, 0, 80, 8));
        snapshot.layout.form_view = Some(ratatui::layout::Rect::new(0, 0, 80, 8));
        snapshot.layout.form_inputs.insert(
            "include".to_string(),
            ratatui::layout::Rect::new(10, 1, 30, 6),
        );
        snapshot.layout.form_fields.push(FormFieldLayout {
            arg_id: "include".to_string(),
            heading: None,
            label: Some(ratatui::layout::Rect::new(0, 1, 9, 1)),
            input: ratatui::layout::Rect::new(10, 1, 30, 6),
            input_clip_top: 0,
            full_input_height: 6,
            description: None,
        });

        let effect = apply_action(
            &Action::ClickForm(AppMouseEvent {
                kind: crate::runtime::AppMouseEventKind::Down(crate::runtime::AppMouseButton::Left),
                column: 12,
                row: 5,
                modifiers: AppKeyModifiers::default(),
            }),
            &mut state,
            &snapshot,
        );

        assert_eq!(effect, Effect::None);
        assert_eq!(state.ui.form_scroll, 0);
        let arg = state.domain.arg_for_input("include").expect("include arg");
        let editor = crate::form_editor::editor_for_render(
            &state.ui,
            arg.owner_path(),
            arg,
            &crate::form_editor::displayed_text(&state, arg),
        );
        assert_eq!(editor.cursor().row, 1);
    }

    #[test]
    fn clicking_add_only_scrolls_when_the_resolved_target_is_clipped() {
        let mut state = AppState::from_command(
            &Command::new("tool").arg(
                Arg::new("include")
                    .long("include")
                    .action(ArgAction::Append)
                    .num_args(1),
            ),
        );
        state.domain.set_text_value("include", "alpha\nbeta");
        state.domain.mark_touched("include");
        let mut snapshot = FrameSnapshot::default();
        snapshot.layout.form = Some(ratatui::layout::Rect::new(0, 0, 80, 10));
        snapshot.layout.form_view = Some(ratatui::layout::Rect::new(0, 0, 80, 10));
        snapshot.layout.form_inputs.insert(
            "include".to_string(),
            ratatui::layout::Rect::new(10, 1, 30, 6),
        );
        snapshot.layout.form_fields.push(FormFieldLayout {
            arg_id: "include".to_string(),
            heading: None,
            label: Some(ratatui::layout::Rect::new(0, 1, 9, 1)),
            input: ratatui::layout::Rect::new(10, 1, 30, 6),
            input_clip_top: 0,
            full_input_height: 6,
            description: None,
        });

        let effect = apply_action(
            &Action::ClickForm(AppMouseEvent {
                kind: crate::runtime::AppMouseEventKind::Down(crate::runtime::AppMouseButton::Left),
                column: 37,
                row: 5,
                modifiers: AppKeyModifiers::default(),
            }),
            &mut state,
            &snapshot,
        );

        assert_eq!(effect, Effect::None);
        assert_eq!(state.ui.form_scroll, 0);
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
        let argv = crate::pipeline::build_authoritative_command_line(&state);
        assert_eq!(argv, vec!["tool".to_string()]);
    }

    #[test]
    fn clicking_untouched_default_text_input_starts_at_the_beginning() {
        let mut state = AppState::from_command(
            &Command::new("tool").arg(Arg::new("config").long("config").default_value("prod.toml")),
        );
        let mut snapshot = FrameSnapshot::default();
        snapshot.layout.form = Some(ratatui::layout::Rect::new(0, 0, 40, 6));
        snapshot.layout.form_view = Some(ratatui::layout::Rect::new(0, 0, 40, 6));
        snapshot.layout.form_inputs.insert(
            "config".to_string(),
            ratatui::layout::Rect::new(0, 0, 20, 3),
        );

        let effect = apply_action(
            &Action::ClickForm(AppMouseEvent {
                kind: crate::runtime::AppMouseEventKind::Down(crate::runtime::AppMouseButton::Left),
                column: 6,
                row: 1,
                modifiers: AppKeyModifiers::default(),
            }),
            &mut state,
            &snapshot,
        );

        assert_eq!(effect, Effect::None);
        let arg = state.domain.arg_for_input("config").expect("config arg");
        let editor = crate::form_editor::editor_for_render(
            &state.ui,
            arg.owner_path(),
            arg,
            &crate::form_editor::displayed_text(&state, arg),
        );
        assert_eq!(
            editor.cursor(),
            crate::editor_state::TextPosition::default()
        );
        assert_eq!(
            state
                .ui
                .mouse_select
                .as_ref()
                .map(|selection| (selection.anchor_row, selection.anchor_col)),
            Some((0, 0))
        );
    }

    #[test]
    fn top_clipped_text_input_click_maps_to_visible_content_row() {
        let mut state =
            AppState::from_command(&Command::new("tool").arg(Arg::new("config").long("config")));
        state.domain.set_text_value("config", "abcdef");
        state.domain.mark_touched("config");
        let mut snapshot = FrameSnapshot::default();
        snapshot.layout.form = Some(ratatui::layout::Rect::new(0, 0, 40, 3));
        snapshot.layout.form_view = Some(ratatui::layout::Rect::new(0, 0, 40, 3));
        snapshot.layout.form_inputs.insert(
            "config".to_string(),
            ratatui::layout::Rect::new(10, 1, 20, 2),
        );
        snapshot.layout.form_fields.push(FormFieldLayout {
            arg_id: "config".to_string(),
            heading: None,
            label: Some(ratatui::layout::Rect::new(0, 1, 9, 1)),
            input: ratatui::layout::Rect::new(10, 1, 20, 2),
            input_clip_top: 1,
            full_input_height: 3,
            description: None,
        });

        let effect = apply_action(
            &Action::ClickForm(AppMouseEvent {
                kind: crate::runtime::AppMouseEventKind::Down(crate::runtime::AppMouseButton::Left),
                column: 13,
                row: 1,
                modifiers: AppKeyModifiers::default(),
            }),
            &mut state,
            &snapshot,
        );

        assert_eq!(effect, Effect::None);
        let arg = state.domain.arg_for_input("config").expect("config arg");
        let editor = crate::form_editor::editor_for_render(
            &state.ui,
            arg.owner_path(),
            arg,
            &crate::form_editor::displayed_text(&state, arg),
        );
        assert_eq!(
            editor.cursor(),
            crate::editor_state::TextPosition { row: 0, col: 2 }
        );
    }

    #[test]
    fn top_clipped_optional_value_text_click_maps_to_visible_content_row() {
        let mut state = AppState::from_command(
            &Command::new("tool").arg(
                Arg::new("color")
                    .long("color")
                    .action(ArgAction::Set)
                    .num_args(0..=1),
            ),
        );
        state.domain.set_text_value("color", "blue");
        state.domain.mark_touched("color");
        let mut snapshot = FrameSnapshot::default();
        snapshot.layout.form = Some(ratatui::layout::Rect::new(0, 0, 40, 3));
        snapshot.layout.form_view = Some(ratatui::layout::Rect::new(0, 0, 40, 3));
        snapshot.layout.form_inputs.insert(
            "color".to_string(),
            ratatui::layout::Rect::new(10, 1, 20, 2),
        );
        snapshot.layout.form_fields.push(FormFieldLayout {
            arg_id: "color".to_string(),
            heading: None,
            label: Some(ratatui::layout::Rect::new(0, 1, 9, 1)),
            input: ratatui::layout::Rect::new(10, 1, 20, 2),
            input_clip_top: 1,
            full_input_height: 3,
            description: None,
        });

        let effect = apply_action(
            &Action::ClickForm(AppMouseEvent {
                kind: crate::runtime::AppMouseEventKind::Down(crate::runtime::AppMouseButton::Left),
                column: 13,
                row: 1,
                modifiers: AppKeyModifiers::default(),
            }),
            &mut state,
            &snapshot,
        );

        assert_eq!(effect, Effect::None);
        let arg = state.domain.arg_for_input("color").expect("color arg");
        let editor = crate::form_editor::editor_for_render(
            &state.ui,
            arg.owner_path(),
            arg,
            &crate::form_editor::displayed_text(&state, arg),
        );
        assert_eq!(
            editor.cursor(),
            crate::editor_state::TextPosition { row: 0, col: 2 }
        );
    }

    #[test]
    fn bottom_clipped_text_border_click_does_not_start_text_selection() {
        let mut state =
            AppState::from_command(&Command::new("tool").arg(Arg::new("config").long("config")));
        state.domain.set_text_value("config", "abcdef");
        state.domain.mark_touched("config");
        let arg = state
            .domain
            .arg_for_input("config")
            .cloned()
            .expect("config arg");
        crate::form_editor::set_cursor_from_click(&mut state, &arg, 0, 0);
        let mut snapshot = FrameSnapshot::default();
        snapshot.layout.form = Some(ratatui::layout::Rect::new(0, 0, 40, 3));
        snapshot.layout.form_view = Some(ratatui::layout::Rect::new(0, 0, 40, 3));
        snapshot.layout.form_inputs.insert(
            "config".to_string(),
            ratatui::layout::Rect::new(10, 1, 20, 1),
        );
        snapshot.layout.form_fields.push(FormFieldLayout {
            arg_id: "config".to_string(),
            heading: None,
            label: Some(ratatui::layout::Rect::new(0, 1, 9, 1)),
            input: ratatui::layout::Rect::new(10, 1, 20, 1),
            input_clip_top: 2,
            full_input_height: 3,
            description: None,
        });

        let effect = apply_action(
            &Action::ClickForm(AppMouseEvent {
                kind: crate::runtime::AppMouseEventKind::Down(crate::runtime::AppMouseButton::Left),
                column: 13,
                row: 1,
                modifiers: AppKeyModifiers::default(),
            }),
            &mut state,
            &snapshot,
        );

        assert_eq!(effect, Effect::None);
        assert!(state.ui.mouse_select.is_none());
        let editor = crate::form_editor::editor_for_render(
            &state.ui,
            arg.owner_path(),
            &arg,
            &crate::form_editor::displayed_text(&state, &arg),
        );
        assert_eq!(
            editor.cursor(),
            crate::editor_state::TextPosition { row: 0, col: 0 }
        );
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

        let argv = crate::pipeline::build_authoritative_command_line(&state);
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
        let argv = crate::pipeline::build_authoritative_command_line(&state);
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
        let argv = crate::pipeline::build_authoritative_command_line(&state);
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
    fn repeated_row_up_on_the_first_row_moves_to_the_previous_field() {
        let mut state = repeated_navigation_state();
        let mut snapshot = FrameSnapshot::default();
        snapshot.layout.form = Some(ratatui::layout::Rect::new(0, 0, 80, 11));
        snapshot.layout.form_view = Some(ratatui::layout::Rect::new(0, 0, 80, 11));
        state.ui.set_selected_arg_index(1);
        let arg = state.domain.current_command().args[1].clone();
        crate::form_editor::set_cursor_from_click(&mut state, &arg, 0, 0);

        let effect = apply_action(
            &Action::FormWidgetInput(AppKeyEvent::new(AppKeyCode::Up, AppKeyModifiers::default())),
            &mut state,
            &snapshot,
        );

        assert_eq!(effect, Effect::None);
        assert_eq!(state.ui.selected_arg_index, 0);
    }

    #[test]
    fn repeated_row_down_on_the_last_row_moves_to_the_next_field() {
        let mut state = repeated_navigation_state();
        let mut snapshot = FrameSnapshot::default();
        snapshot.layout.form = Some(ratatui::layout::Rect::new(0, 0, 80, 11));
        snapshot.layout.form_view = Some(ratatui::layout::Rect::new(0, 0, 80, 11));
        state.ui.set_selected_arg_index(1);
        let arg = state.domain.current_command().args[1].clone();
        crate::form_editor::set_cursor_from_click(&mut state, &arg, 1, 0);

        let effect = apply_action(
            &Action::FormWidgetInput(AppKeyEvent::new(
                AppKeyCode::Down,
                AppKeyModifiers::default(),
            )),
            &mut state,
            &snapshot,
        );

        assert_eq!(effect, Effect::None);
        assert_eq!(state.ui.selected_arg_index, 2);
    }

    #[test]
    fn repeated_row_down_on_a_non_last_row_stays_inside_the_editor() {
        let mut state = repeated_navigation_state();
        let mut snapshot = FrameSnapshot::default();
        snapshot.layout.form = Some(ratatui::layout::Rect::new(0, 0, 80, 11));
        snapshot.layout.form_view = Some(ratatui::layout::Rect::new(0, 0, 80, 11));
        state.ui.set_selected_arg_index(1);
        let arg = state.domain.current_command().args[1].clone();
        crate::form_editor::set_cursor_from_click(&mut state, &arg, 0, 0);

        let effect = apply_action(
            &Action::FormWidgetInput(AppKeyEvent::new(
                AppKeyCode::Down,
                AppKeyModifiers::default(),
            )),
            &mut state,
            &snapshot,
        );

        assert_eq!(effect, Effect::None);
        assert_eq!(state.ui.selected_arg_index, 1);
        let editor = crate::form_editor::editor_for_render(
            &state.ui,
            arg.owner_path(),
            &arg,
            &crate::form_editor::displayed_text(&state, &arg),
        );
        assert_eq!(editor.cursor().row, 1);
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
        let argv = crate::pipeline::build_authoritative_command_line(&state);
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
    fn repeated_row_enter_moves_to_the_next_existing_row() {
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
        assert_eq!(editor.text(), "alpha\nbeta");
        assert_eq!(editor.cursor().row, 1);

        apply_action(
            &Action::FormTextInput(key(AppKeyCode::Char('x'))),
            &mut state,
            &FrameSnapshot::default(),
        );

        let argv = crate::pipeline::build_authoritative_command_line(&state);
        assert_eq!(
            argv,
            vec![
                "tool".to_string(),
                "--include".to_string(),
                "alpha".to_string(),
                "--include".to_string(),
                "xbeta".to_string(),
            ]
        );
    }

    #[test]
    fn repeated_row_enter_scrolls_to_reveal_a_newly_inserted_row() {
        let mut state = AppState::from_command(
            &Command::new("tool").arg(
                Arg::new("include")
                    .long("include")
                    .action(ArgAction::Append)
                    .num_args(1),
            ),
        );
        state.domain.set_text_value("include", "alpha\nbeta");
        state.domain.mark_touched("include");
        let arg = state
            .domain
            .current_command()
            .args
            .iter()
            .find(|arg| arg.id == "include")
            .cloned()
            .expect("include arg");
        crate::form_editor::set_cursor_from_click(&mut state, &arg, 1, 0);
        let mut snapshot = FrameSnapshot::default();
        snapshot.layout.form = Some(ratatui::layout::Rect::new(0, 0, 80, 5));
        snapshot.layout.form_view = Some(ratatui::layout::Rect::new(0, 0, 80, 5));
        snapshot.layout.form_inputs.insert(
            "include".to_string(),
            ratatui::layout::Rect::new(10, 0, 30, 5),
        );
        snapshot.layout.form_fields.push(FormFieldLayout {
            arg_id: "include".to_string(),
            heading: None,
            label: Some(ratatui::layout::Rect::new(0, 0, 9, 1)),
            input: ratatui::layout::Rect::new(10, 0, 30, 5),
            input_clip_top: 0,
            full_input_height: 5,
            description: None,
        });

        let effect = apply_action(
            &Action::FormWidgetInput(key(AppKeyCode::Enter)),
            &mut state,
            &snapshot,
        );

        assert_eq!(effect, Effect::None);
        assert_eq!(state.ui.form_scroll, 4);
        let editor = crate::form_editor::editor_for_render(
            &state.ui,
            arg.owner_path(),
            &arg,
            &crate::form_editor::displayed_text(&state, &arg),
        );
        assert_eq!(editor.cursor().row, 2);
    }

    #[test]
    fn repeated_row_enter_scrolls_far_enough_when_the_input_starts_lower_in_the_form() {
        let mut state = AppState::from_command(
            &Command::new("tool").arg(
                Arg::new("include")
                    .long("include")
                    .action(ArgAction::Append)
                    .num_args(1),
            ),
        );
        state.domain.set_text_value("include", "alpha\nbeta");
        state.domain.mark_touched("include");
        let arg = state
            .domain
            .current_command()
            .args
            .iter()
            .find(|arg| arg.id == "include")
            .cloned()
            .expect("include arg");
        crate::form_editor::set_cursor_from_click(&mut state, &arg, 1, 0);
        let mut snapshot = FrameSnapshot::default();
        snapshot.layout.form = Some(ratatui::layout::Rect::new(0, 0, 80, 8));
        snapshot.layout.form_view = Some(ratatui::layout::Rect::new(0, 0, 80, 8));
        snapshot.layout.form_inputs.insert(
            "include".to_string(),
            ratatui::layout::Rect::new(10, 2, 30, 5),
        );
        snapshot.layout.form_fields.push(FormFieldLayout {
            arg_id: "include".to_string(),
            heading: None,
            label: Some(ratatui::layout::Rect::new(0, 2, 9, 1)),
            input: ratatui::layout::Rect::new(10, 2, 30, 5),
            input_clip_top: 0,
            full_input_height: 5,
            description: None,
        });

        let effect = apply_action(
            &Action::FormWidgetInput(key(AppKeyCode::Enter)),
            &mut state,
            &snapshot,
        );

        assert_eq!(effect, Effect::None);
        assert_eq!(state.ui.form_scroll, 1);
        let editor = crate::form_editor::editor_for_render(
            &state.ui,
            arg.owner_path(),
            &arg,
            &crate::form_editor::displayed_text(&state, &arg),
        );
        assert_eq!(editor.cursor().row, 2);
    }

    #[test]
    fn repeated_row_insert_does_not_surface_an_orphaned_neighbor_description() {
        let mut state = AppState::from_command(
            &Command::new("tool")
                .arg(
                    Arg::new("define")
                        .long("define")
                        .help("Key/value pairs")
                        .action(ArgAction::Append)
                        .num_args(1),
                )
                .arg(
                    Arg::new("include")
                        .long("include")
                        .action(ArgAction::Append)
                        .num_args(1),
                ),
        );
        state.domain.set_text_value("define", "A=1\nB=2");
        state.domain.mark_touched("define");
        state.domain.set_text_value("include", "alpha\nbeta");
        state.domain.mark_touched("include");
        state.ui.focus_form();
        state.ui.selected_arg_index = 1;
        state.ui.form_scroll = 5;
        let include_arg = state
            .domain
            .current_command()
            .args
            .iter()
            .find(|arg| arg.id == "include")
            .cloned()
            .expect("include arg");
        crate::form_editor::set_cursor_from_click(&mut state, &include_arg, 1, 0);

        let mut snapshot = FrameSnapshot {
            form_scroll_max: 99,
            ..FrameSnapshot::default()
        };
        snapshot.layout.form = Some(ratatui::layout::Rect::new(0, 0, 80, 6));
        snapshot.layout.form_view = Some(ratatui::layout::Rect::new(0, 0, 80, 6));

        let effect = apply_action(
            &Action::FormWidgetInput(key(AppKeyCode::Enter)),
            &mut state,
            &snapshot,
        );

        assert_eq!(effect, Effect::None);

        let root = state.domain.root.clone();
        let selected_path = state.domain.selected_path().clone();
        let active_args = selectors::visible_form_args(&root, &selected_path, state.ui.active_tab);
        let derived = state.derived().clone();
        let mut post_action_snapshot = FrameSnapshot::default();
        let input_height_overrides = active_args
            .iter()
            .filter(|item| matches!(item.widget, FieldWidget::RepeatedText))
            .map(|item| {
                let displayed = crate::form_editor::displayed_text(&state, item.arg);
                (
                    item.arg.id.clone(),
                    crate::repeated_field::repeated_input_height(&state.ui, item.arg, &displayed),
                )
            })
            .collect();

        crate::frame_snapshot::populate_form_layout(
            &state.ui,
            ratatui::layout::Rect::new(0, 0, 80, 6),
            &active_args,
            "",
            &derived.validation,
            &derived.field_semantics,
            &input_height_overrides,
            &std::collections::HashMap::new(),
            &mut post_action_snapshot,
        );

        let define = post_action_snapshot
            .layout
            .form_fields
            .iter()
            .find(|field| field.arg_id == "define");
        assert!(define.is_none_or(|field| field.description.is_none()));
        assert!(define.is_none_or(|field| field.label.is_none()));
    }

    #[test]
    fn single_choice_clear_row_resets_value_via_space() {
        let mut state = AppState::from_command(
            &Command::new("tool").arg(Arg::new("mode").long("mode").value_parser(["fast", "safe"])),
        );
        state
            .domain
            .set_choice_value_touched("mode", "fast".to_string());
        state.ui.dropdown_open = Some("mode".to_string());
        state.ui.dropdown_cursor = 0;

        let effect = apply_action(
            &Action::ChoiceInput {
                arg_id: "mode".to_string(),
                key: key(AppKeyCode::Char(' ')),
            },
            &mut state,
            &FrameSnapshot::default(),
        );

        assert_eq!(effect, Effect::None);
        assert!(state.ui.dropdown_open.is_none());
        let argv = crate::pipeline::build_authoritative_command_line(&state);
        assert_eq!(argv, vec!["tool".to_string()]);
    }

    #[test]
    fn single_choice_clear_row_offsets_real_choice_selection() {
        let mut state = AppState::from_command(
            &Command::new("tool").arg(Arg::new("mode").long("mode").value_parser(["fast", "safe"])),
        );
        state.ui.dropdown_open = Some("mode".to_string());
        // Row 0 is "(none)", row 1 is "fast", row 2 is "safe".
        state.ui.dropdown_cursor = 1;

        let effect = apply_action(
            &Action::ChoiceInput {
                arg_id: "mode".to_string(),
                key: key(AppKeyCode::Char(' ')),
            },
            &mut state,
            &FrameSnapshot::default(),
        );

        assert_eq!(effect, Effect::None);
        let arg = state.domain.arg_for_input("mode").expect("mode arg");
        assert_eq!(
            state
                .domain
                .current_form()
                .map(|form| form.selected_values(arg))
                .unwrap_or_default(),
            vec!["fast".to_string()]
        );
    }

    #[test]
    fn single_choice_required_field_has_no_clear_row_offset() {
        let mut state = AppState::from_command(
            &Command::new("tool").arg(
                Arg::new("mode")
                    .long("mode")
                    .required(true)
                    .value_parser(["fast", "safe"]),
            ),
        );
        state.ui.dropdown_open = Some("mode".to_string());
        state.ui.dropdown_cursor = 0;

        let effect = apply_action(
            &Action::ChoiceInput {
                arg_id: "mode".to_string(),
                key: key(AppKeyCode::Enter),
            },
            &mut state,
            &FrameSnapshot::default(),
        );

        assert_eq!(effect, Effect::None);
        let arg = state.domain.arg_for_input("mode").expect("mode arg");
        assert_eq!(
            state
                .domain
                .current_form()
                .map(|form| form.selected_values(arg))
                .unwrap_or_default(),
            vec!["fast".to_string()]
        );
    }

    #[test]
    fn clicking_clear_row_resets_single_choice() {
        let mut state = AppState::from_command(
            &Command::new("tool").arg(Arg::new("mode").long("mode").value_parser(["fast", "safe"])),
        );
        state
            .domain
            .set_choice_value_touched("mode", "fast".to_string());
        state.ui.dropdown_open = Some("mode".to_string());
        let mut snapshot = FrameSnapshot::default();
        snapshot.layout.dropdown = Some(ratatui::layout::Rect::new(0, 0, 20, 5));

        // Click the first content row (just inside the border).
        let effect = apply_action(
            &Action::ClickDropdownChoice {
                arg_id: "mode".to_string(),
                row: 1,
            },
            &mut state,
            &snapshot,
        );

        assert_eq!(effect, Effect::None);
        assert!(state.ui.dropdown_open.is_none());
        let argv = crate::pipeline::build_authoritative_command_line(&state);
        assert_eq!(argv, vec!["tool".to_string()]);
    }
}
