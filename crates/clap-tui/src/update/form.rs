use crate::controller::navigation;
use crate::form_editor::{self, EditResult};
use crate::frame_snapshot::FrameSnapshot;
use crate::input::{AppState, MouseSelection};
use crate::query::{
    form::{self, FieldWidget, OrderedArg},
    selectors,
};
use crate::runtime::{AppKeyCode, AppKeyEvent, AppMouseEvent};
use ratatui::layout::Rect;

use super::{Action, Effect};

const REPEATED_CONTROL_ROW_WIDTH: u16 = 8;
const REPEATED_ROW_HEIGHT: u16 = 3;

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
            EditResult::Ignored
        };
        if matches!(key.code, AppKeyCode::Up | AppKeyCode::Down)
            && !key.modifiers.alt
            && matches!(result, EditResult::Ignored)
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
        } else if matches!(key.code, AppKeyCode::Enter) && matches!(result, EditResult::Handled) {
            navigation::ensure_active_repeated_row_visible(state, frame_snapshot, &item.arg.id);
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
    let validation = state.derived_validation();
    if let Some(hit) = form_click_hit(
        &args,
        event,
        state,
        frame_snapshot,
        &validation.field_errors,
    ) {
        state.ui.set_selected_arg_index(hit.item.order_index);
        if matches!(hit.item.widget, FieldWidget::Counter) && hit.in_input {
            apply_counter_click(event, state, frame_snapshot, &hit.item.arg.id);
        } else if matches!(hit.item.widget, FieldWidget::RepeatedText) && hit.in_input {
            if apply_repeated_text_click(event, state, frame_snapshot, &hit.item.arg.id) {
                navigation::ensure_active_repeated_row_visible(
                    state,
                    frame_snapshot,
                    &hit.item.arg.id,
                );
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
            if let Some((row, col)) = frame_snapshot.input_position_from_point(
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
) -> bool {
    let Some(field) = frame_snapshot.form_field_layout(arg_id) else {
        return false;
    };
    let input = field.input;
    let Some(arg) = state.domain.arg_for_input(arg_id).cloned() else {
        return false;
    };
    let editor = form_editor::editor_for_render(
        &state.ui,
        arg.owner_path(),
        &arg,
        &form_editor::displayed_text(state, &arg),
    );
    let total_rows = editor.row_count().max(1);
    if total_rows <= 1 {
        match repeated_control_click_target(event.column, event.row, input, true, true) {
            Some(RepeatedControlClickTarget::Remove) => return true,
            Some(RepeatedControlClickTarget::Add) => {
                let _ = form_editor::insert_repeated_row(state, &arg);
                navigation::ensure_active_repeated_row_visible(state, frame_snapshot, &arg.id);
                return true;
            }
            None => return false,
        }
    }
    let visible_rows = usize::from(input.height / REPEATED_ROW_HEIGHT).min(total_rows);
    if visible_rows == 0 {
        return false;
    }
    let start_row = repeated_visible_start_row(visible_rows, total_rows, field.input_clip_top);
    let relative_y = event.row.saturating_sub(input.y);
    let row_index = start_row + usize::from(relative_y / REPEATED_ROW_HEIGHT);
    if row_index >= total_rows {
        return false;
    }
    let row = u16::try_from(row_index).unwrap_or(u16::MAX);
    let row_rect = repeated_row_rect(input, row_index.saturating_sub(start_row));
    let Some(row_rect) = row_rect else {
        return false;
    };
    form_editor::clear_selection(state, &arg);
    state.ui.clear_mouse_selection();
    let is_last_row = row_index + 1 == total_rows;
    match repeated_control_click_target(event.column, event.row, row_rect, true, is_last_row) {
        Some(RepeatedControlClickTarget::Remove) => {
            form_editor::set_cursor_from_click(state, &arg, row, 0);
            if total_rows > 1 {
                let _ = form_editor::remove_repeated_row(state, &arg);
            }
        }
        Some(RepeatedControlClickTarget::Add) => {
            form_editor::set_cursor_from_click(state, &arg, row, 0);
            let _ = form_editor::insert_repeated_row(state, &arg);
            navigation::ensure_active_repeated_row_visible(state, frame_snapshot, &arg.id);
        }
        None => {
            let textarea_rect = repeated_row_textarea_rect(row_rect, true, is_last_row);
            if let Some((_, col)) =
                input_position_from_rect(textarea_rect, event.column, event.row, false)
            {
                state.ui.set_mouse_selection(Some(MouseSelection {
                    arg_id: arg_id.to_string(),
                    anchor_row: row,
                    anchor_col: col,
                    active: false,
                }));
                form_editor::set_cursor_from_click(state, &arg, row, col);
            }
        }
    }

    true
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RepeatedControlClickTarget {
    Remove,
    Add,
}

fn repeated_control_click_target(
    x: u16,
    y: u16,
    row_rect: Rect,
    show_remove: bool,
    show_add: bool,
) -> Option<RepeatedControlClickTarget> {
    if show_remove
        && let Some(remove) = repeated_remove_rect(row_rect, show_remove, show_add)
        && contains(remove, x, y)
    {
        Some(RepeatedControlClickTarget::Remove)
    } else if show_add
        && let Some(add) = repeated_add_rect(row_rect)
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
    let Some(input) = frame_snapshot.form_input_rect(arg_id) else {
        return;
    };

    match counter_click_target(input, event.column) {
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
}

fn form_click_hit<'a>(
    args: &'a [OrderedArg<'a>],
    event: AppMouseEvent,
    state: &AppState,
    frame_snapshot: &FrameSnapshot,
    field_errors: &std::collections::BTreeMap<String, String>,
) -> Option<FormClickHit<'a>> {
    if let Some(hit) = frame_snapshot.form_field_at(event.column, event.row) {
        let item = args.iter().find(|item| item.arg.id == hit.arg_id)?;
        return Some(FormClickHit {
            item,
            in_input: hit.in_input,
            in_label: hit.in_label,
        });
    }

    let content_y =
        frame_snapshot.form_content_y(event.row, state.ui.form_scroll(frame_snapshot))?;
    let hit = form::hit_test_form_content_with_errors(args, content_y, field_errors)?;
    let item = args
        .iter()
        .find(|item| item.order_index == hit.order_index)?;
    Some(FormClickHit {
        item,
        in_input: hit.in_input,
        in_label: hit.in_label,
    })
}

fn repeated_row_rect(input: Rect, visible_index: usize) -> Option<Rect> {
    let y = input.y.saturating_add(
        u16::try_from(visible_index)
            .ok()?
            .saturating_mul(REPEATED_ROW_HEIGHT),
    );
    Some(Rect::new(input.x, y, input.width, REPEATED_ROW_HEIGHT))
}

fn repeated_visible_start_row(
    visible_rows: usize,
    total_rows: usize,
    input_clip_top: u16,
) -> usize {
    let clipped_rows = usize::from(
        input_clip_top.saturating_add(REPEATED_ROW_HEIGHT.saturating_sub(1)) / REPEATED_ROW_HEIGHT,
    );
    clipped_rows.min(total_rows.saturating_sub(visible_rows.min(total_rows)))
}

fn repeated_row_textarea_rect(row_rect: Rect, show_remove: bool, show_add: bool) -> Rect {
    let with_controls = show_remove || show_add;
    let width = if with_controls && row_rect.width > REPEATED_CONTROL_ROW_WIDTH {
        row_rect
            .width
            .saturating_sub(REPEATED_CONTROL_ROW_WIDTH)
            .saturating_sub(1)
    } else {
        row_rect.width
    };
    Rect::new(row_rect.x, row_rect.y, width, row_rect.height)
}

fn repeated_remove_rect(row_rect: Rect, show_remove: bool, show_add: bool) -> Option<Rect> {
    if !show_remove || row_rect.height < 2 {
        return None;
    }
    let strip_start = row_rect
        .x
        .saturating_add(row_rect.width.saturating_sub(REPEATED_CONTROL_ROW_WIDTH));
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

fn repeated_add_rect(row_rect: Rect) -> Option<Rect> {
    if row_rect.height < 2 {
        return None;
    }
    let x = row_rect.x.saturating_add(row_rect.width.saturating_sub(4));
    Some(Rect::new(
        x,
        row_rect
            .y
            .saturating_add(row_rect.height.saturating_sub(1).min(1)),
        3,
        1,
    ))
}

fn contains(area: Rect, x: u16, y: u16) -> bool {
    x >= area.x
        && x < area.x.saturating_add(area.width)
        && y >= area.y
        && y < area.y.saturating_add(area.height)
}

fn input_position_from_rect(rect: Rect, x: u16, y: u16, clamp: bool) -> Option<(u16, u16)> {
    let inner_x = rect.x.saturating_add(1);
    let inner_y = rect.y.saturating_add(1);
    let inner_w = rect.width.saturating_sub(2);
    let inner_h = rect.height.saturating_sub(2);
    if inner_w == 0 || inner_h == 0 {
        return None;
    }
    if !clamp && (x < inner_x || y < inner_y || x >= inner_x + inner_w || y >= inner_y + inner_h) {
        return None;
    }
    let clamped_x = if clamp {
        x.clamp(inner_x, inner_x + inner_w - 1)
    } else {
        x
    };
    let clamped_y = if clamp {
        y.clamp(inner_y, inner_y + inner_h - 1)
    } else {
        y
    };
    Some((
        clamped_y
            .saturating_sub(inner_y)
            .min(inner_h.saturating_sub(1)),
        clamped_x
            .saturating_sub(inner_x)
            .min(inner_w.saturating_sub(1)),
    ))
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
    use clap::{Arg, ArgAction, ArgGroup, Command};

    use super::super::{Action, Effect, apply_action};
    use super::*;
    use crate::frame_snapshot::{FormFieldLayout, FrameSnapshot};
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
            section_rail: None,
            section_right_rail: None,
            section_cap: None,
            label: Some(ratatui::layout::Rect::new(0, 2, 10, 1)),
            input: ratatui::layout::Rect::new(12, 2, 20, 1),
            input_clip_top: 0,
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
        let argv = crate::pipeline::build_command_line(&state);
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
            ratatui::layout::Rect::new(10, 1, 30, 1),
        );
        snapshot.layout.form_fields.push(FormFieldLayout {
            arg_id: "verbose".to_string(),
            heading: None,
            section_rail: None,
            section_right_rail: None,
            section_cap: None,
            label: Some(ratatui::layout::Rect::new(0, 1, 9, 1)),
            input: ratatui::layout::Rect::new(10, 1, 30, 1),
            input_clip_top: 0,
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
            crate::pipeline::build_command_line(&state),
            vec!["tool".to_string(), "-v".to_string()]
        );

        let effect = apply_action(
            &Action::ClickForm(AppMouseEvent {
                kind: crate::runtime::AppMouseEventKind::Down(crate::runtime::AppMouseButton::Left),
                column: 34,
                row: 1,
                modifiers: AppKeyModifiers::default(),
            }),
            &mut state,
            &snapshot,
        );

        assert_eq!(effect, Effect::None);
        assert_eq!(
            crate::pipeline::build_command_line(&state),
            vec!["tool".to_string()]
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
            section_rail: None,
            section_right_rail: None,
            section_cap: None,
            label: Some(ratatui::layout::Rect::new(0, 1, 9, 1)),
            input: ratatui::layout::Rect::new(10, 1, 30, 6),
            input_clip_top: 0,
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
            crate::pipeline::build_command_line(&state),
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
            section_rail: None,
            section_right_rail: None,
            section_cap: None,
            label: Some(ratatui::layout::Rect::new(0, 1, 9, 1)),
            input: ratatui::layout::Rect::new(10, 1, 30, 6),
            input_clip_top: 0,
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
            crate::pipeline::build_command_line(&state),
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
            section_rail: None,
            section_right_rail: None,
            section_cap: None,
            label: Some(ratatui::layout::Rect::new(0, 1, 9, 1)),
            input: ratatui::layout::Rect::new(10, 1, 30, 9),
            input_clip_top: 0,
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
            crate::pipeline::build_command_line(&state),
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
            section_rail: None,
            section_right_rail: None,
            section_cap: None,
            label: Some(ratatui::layout::Rect::new(0, 1, 9, 1)),
            input: ratatui::layout::Rect::new(10, 1, 30, 3),
            input_clip_top: REPEATED_ROW_HEIGHT,
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
            crate::pipeline::build_command_line(&state),
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
            section_rail: None,
            section_right_rail: None,
            section_cap: None,
            label: Some(ratatui::layout::Rect::new(0, 1, 9, 1)),
            input: ratatui::layout::Rect::new(10, 1, 30, 6),
            input_clip_top: 0,
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
            section_rail: None,
            section_right_rail: None,
            section_cap: None,
            label: Some(ratatui::layout::Rect::new(0, 1, 9, 1)),
            input: ratatui::layout::Rect::new(10, 1, 30, 3),
            input_clip_top: 0,
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
            section_rail: None,
            section_right_rail: None,
            section_cap: None,
            label: Some(ratatui::layout::Rect::new(0, 0, 9, 1)),
            input: ratatui::layout::Rect::new(10, 0, 30, 6),
            input_clip_top: 0,
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

        let argv = crate::pipeline::build_command_line(&state);
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
            section_rail: None,
            section_right_rail: None,
            section_cap: None,
            label: Some(ratatui::layout::Rect::new(0, 0, 9, 1)),
            input: ratatui::layout::Rect::new(10, 0, 30, 5),
            input_clip_top: 0,
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
            section_rail: None,
            section_right_rail: None,
            section_cap: None,
            label: Some(ratatui::layout::Rect::new(0, 2, 9, 1)),
            input: ratatui::layout::Rect::new(10, 2, 30, 5),
            input_clip_top: 0,
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
}
