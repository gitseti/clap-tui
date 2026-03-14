use crossterm::event::{self, MouseButton, MouseEventKind};
use tui_textarea::{CursorMove, TextArea};

use crate::config::TuiConfig;
use crate::input::{AppState, ArgValue, Focus, MouseSelection};
use crate::spec::ArgKind;
use crate::view::{argv, command_tree, form};

use super::Action;
use super::navigation;

pub(crate) fn handle_mouse_event(
    event: event::MouseEvent,
    state: &mut AppState,
    _config: &TuiConfig,
) -> Option<Action> {
    let layout = state.layout.clone();
    if let MouseEventKind::Moved = event.kind {
        if update_mouse_selection(state, event) {
            return None;
        }
        update_hover(state, event.column, event.row);
    }
    if let MouseEventKind::Drag(MouseButton::Left) = event.kind {
        update_mouse_selection(state, event);
        return None;
    }
    if let MouseEventKind::Up(MouseButton::Left) = event.kind {
        state.mouse_select = None;
    }
    if let MouseEventKind::Down(MouseButton::Left) = event.kind {
        if let Some(dropdown) = layout.dropdown {
            if state.enum_open.is_some() {
                if contains(dropdown, event.column, event.row) {
                    if let Some(active) = state.enum_open.clone() {
                        handle_dropdown_click(event, state, &active);
                    }
                } else {
                    state.enum_open = None;
                }
                return None;
            }
        }
        if let Some(footer_area) = layout.footer {
            if contains(footer_area, event.column, event.row) {
                return handle_footer_click(event, state);
            }
        }
        if let Some(preview_area) = layout.preview {
            if contains(preview_area, event.column, event.row) {
                return Some(handle_preview_click(state));
            }
        }
        if let Some(search_area) = layout.search {
            if contains(search_area, event.column, event.row) {
                state.focus = Focus::Search;
                return None;
            }
        }
        if let Some(sidebar_area) = layout.sidebar {
            if contains(sidebar_area, event.column, event.row) {
                handle_sidebar_click(event, state);
                return None;
            }
        }
        if !state.layout.form_tabs.is_empty() && handle_tabs_click(event, state) {
            return None;
        }
        if let Some(form_area) = layout.form {
            if contains(form_area, event.column, event.row) {
                handle_form_click(event, state);
                return None;
            }
        }
    }
    if let Some(dropdown) = state.layout.dropdown {
        if contains(dropdown, event.column, event.row) && state.enum_open.is_some() {
            match event.kind {
                MouseEventKind::ScrollDown => {
                    navigation::scroll_enum(state, 1);
                    return None;
                }
                MouseEventKind::ScrollUp => {
                    navigation::scroll_enum(state, -1);
                    return None;
                }
                _ => {}
            }
        }
    }
    if let MouseEventKind::ScrollDown = event.kind {
        navigation::scroll_form(state, 2);
    }
    if let MouseEventKind::ScrollUp = event.kind {
        navigation::scroll_form(state, -2);
    }
    None
}

fn handle_preview_click(state: &AppState) -> Action {
    Action::CopyCommand(argv::build_argv(state).join(" "))
}

fn update_mouse_selection(state: &mut AppState, event: event::MouseEvent) -> bool {
    let Some(mut selection) = state.mouse_select.take() else {
        return false;
    };
    if let Some((row, col)) = input_position_from_event(state, &selection.arg_id, event, true) {
        let textarea = ensure_textarea_for_displayed(state, &selection.arg_id);
        if !selection.active {
            textarea.move_cursor(CursorMove::Jump(selection.anchor_row, selection.anchor_col));
            textarea.start_selection();
            selection.active = true;
        }
        textarea.move_cursor(CursorMove::Jump(row, col));
    }
    state.mouse_select = Some(selection);
    true
}

fn handle_sidebar_click(event: event::MouseEvent, state: &mut AppState) {
    let Some((path, caret, has_children)) = state
        .layout
        .sidebar_items
        .iter()
        .find(|item| contains(item.row, event.column, event.row))
        .map(|item| (item.path.clone(), item.caret, item.has_children))
    else {
        return;
    };

    if state.selected_path != path {
        state.selected_path = path.clone();
        state.focus_first_tab();
    }
    if let Some(caret) = caret {
        if contains(caret, event.column, event.row) && has_children {
            let items = command_tree::tree_items(&state.root, &state.expanded, &state.search);
            if let Some(item) = items.iter().find(|item| item.path == path) {
                if item.expanded {
                    navigation::collapse_selected(state);
                } else {
                    navigation::expand_selected(state);
                }
            }
        }
    }
    state.focus = Focus::Sidebar;
}

fn handle_form_click(event: event::MouseEvent, state: &mut AppState) {
    if matches!(state.active_tab, crate::input::ActiveTab::Help) {
        return;
    }
    let Some(form_view) = state.layout.form_view else {
        return;
    };
    if event.row < form_view.y || event.row >= form_view.y + form_view.height {
        return;
    }
    let content_y = event
        .row
        .saturating_sub(form_view.y)
        .saturating_add(state.form_scroll);
    let command = state.current_command().clone();
    let args = form::visible_args(&command, state.active_tab);
    if let Some(hit) = form::hit_test_form_content(&args, content_y) {
        state.selected_arg_index = hit.order_index;
        state.focus = Focus::Form;
        if hit.kind == ArgKind::Flag && (hit.in_input || hit.in_label) {
            state.toggle_flag(&hit.arg_id);
            state.mark_touched(&hit.arg_id);
        }
        if hit.kind == ArgKind::Enum && hit.in_input {
            let total = command
                .args
                .iter()
                .find(|arg| arg.id == hit.arg_id)
                .map(|arg| arg.possible_values.len())
                .unwrap_or(0);
            navigation::open_enum_dropdown(state, &hit.arg_id, total);
        }
        if matches!(hit.kind, ArgKind::Option | ArgKind::Positional) {
            let textarea = ensure_textarea_for_displayed(state, &hit.arg_id);
            textarea.cancel_selection();
            state.mouse_select = None;
            if let Some((row, col)) = input_position_from_event(state, &hit.arg_id, event, false) {
                state.mouse_select = Some(MouseSelection {
                    arg_id: hit.arg_id.clone(),
                    anchor_row: row,
                    anchor_col: col,
                    active: false,
                });
            }
            set_textarea_cursor_from_click(state, &hit.arg_id, event, hit.in_label);
        }
        navigation::ensure_form_visible(state);
    }
}

fn set_textarea_cursor_from_click(
    state: &mut AppState,
    arg_id: &str,
    event: event::MouseEvent,
    in_label: bool,
) {
    if let Some((row, col)) = input_position_from_event(state, arg_id, event, false) {
        set_textarea_cursor(state, arg_id, row, col);
    } else if in_label {
        set_textarea_cursor(state, arg_id, 0, 0);
    }
}

fn set_textarea_cursor(state: &mut AppState, arg_id: &str, row: u16, col: u16) {
    let command = state.current_command().clone();
    let default_value = command
        .args
        .iter()
        .find(|arg| arg.id == arg_id)
        .and_then(|arg| arg.default.clone());
    if default_value.is_some() && !state.is_touched(arg_id) {
        let textarea = state.textarea_for(arg_id, default_value.as_deref().unwrap_or(""));
        textarea.move_cursor(CursorMove::Jump(0, 0));
        return;
    }
    let textarea = ensure_textarea_for_displayed(state, arg_id);
    textarea.move_cursor(CursorMove::Jump(row, col));
}

fn displayed_text_for_arg(state: &AppState, arg_id: &str) -> String {
    if let Some(inputs) = state.current_inputs() {
        if let Some(ArgValue::Text(text)) = inputs.values.get(arg_id) {
            return text.clone();
        }
    }
    let command = state.current_command().clone();
    let default_value = command
        .args
        .iter()
        .find(|arg| arg.id == arg_id)
        .and_then(|arg| arg.default.clone());
    if default_value.is_some() && !state.is_touched(arg_id) {
        return default_value.unwrap_or_default();
    }
    String::new()
}

fn ensure_textarea_for_displayed<'a>(
    state: &'a mut AppState,
    arg_id: &str,
) -> &'a mut TextArea<'static> {
    let displayed = displayed_text_for_arg(state, arg_id);
    let textarea = state.textarea_for(arg_id, &displayed);
    if textarea.lines().join("\n") != displayed {
        *textarea = TextArea::new(vec![displayed]);
    }
    textarea
}

fn input_position_from_event(
    state: &AppState,
    arg_id: &str,
    event: event::MouseEvent,
    clamp: bool,
) -> Option<(u16, u16)> {
    let input_rect = state.layout.form_inputs.get(arg_id).copied()?;
    let inner_x = input_rect.x.saturating_add(1);
    let inner_y = input_rect.y.saturating_add(1);
    let inner_w = input_rect.width.saturating_sub(2);
    let inner_h = input_rect.height.saturating_sub(2);
    if inner_w == 0 || inner_h == 0 {
        return None;
    }
    if !clamp
        && (event.column < inner_x
            || event.row < inner_y
            || event.column >= inner_x + inner_w
            || event.row >= inner_y + inner_h)
    {
        return None;
    }
    let x = if clamp {
        event.column.clamp(inner_x, inner_x + inner_w - 1)
    } else {
        event.column
    };
    let y = if clamp {
        event.row.clamp(inner_y, inner_y + inner_h - 1)
    } else {
        event.row
    };
    let col = x.saturating_sub(inner_x).min(inner_w.saturating_sub(1));
    let row = y.saturating_sub(inner_y).min(inner_h.saturating_sub(1));
    Some((row, col))
}

fn handle_dropdown_click(event: event::MouseEvent, state: &mut AppState, arg_id: &str) {
    let command = state.current_command().clone();
    let Some(arg) = command.args.iter().find(|arg| arg.id == arg_id) else {
        return;
    };
    if let Some(area) = state.layout.dropdown {
        if event.row <= area.y || event.row >= area.y + area.height - 1 {
            return;
        }
        let index = event.row.saturating_sub(area.y + 1) as usize + state.enum_scroll;
        if index < arg.possible_values.len() {
            state
                .current_inputs_mut()
                .values
                .insert(arg.id.clone(), ArgValue::Enum(index));
            state.enum_open = None;
            state.mark_touched(&arg.id);
        }
    }
}

fn handle_tabs_click(event: event::MouseEvent, state: &mut AppState) -> bool {
    if let Some(tab) = state
        .layout
        .form_tabs
        .iter()
        .find(|tab| contains(tab.rect, event.column, event.row))
        .map(|tab| tab.tab)
    {
        navigation::switch_tab(state, tab);
        return true;
    }
    false
}

fn handle_footer_click(event: event::MouseEvent, state: &mut AppState) -> Option<Action> {
    for button in &state.layout.footer_buttons {
        if contains(button.rect, event.column, event.row) {
            match button.target {
                crate::input::HoverTarget::Run => {
                    return Some(Action::Run(argv::build_argv(state)));
                }
                crate::input::HoverTarget::Exit => return Some(Action::Exit),
                crate::input::HoverTarget::Search => {
                    state.focus = Focus::Search;
                    return None;
                }
                crate::input::HoverTarget::Focus => {
                    state.focus = Focus::Sidebar;
                    return None;
                }
                crate::input::HoverTarget::Help => {
                    navigation::toggle_help_tab(state);
                    return None;
                }
                crate::input::HoverTarget::Preview => return None,
            }
        }
    }
    None
}

fn update_hover(state: &mut AppState, x: u16, y: u16) {
    state.hover = state
        .layout
        .footer_buttons
        .iter()
        .find(|button| contains(button.rect, x, y))
        .map(|button| button.target);
    if state.hover.is_none()
        && state
            .layout
            .preview
            .is_some_and(|area| contains(area, x, y))
    {
        state.hover = Some(crate::input::HoverTarget::Preview);
    }

    state.hover_tab = state
        .layout
        .form_tabs
        .iter()
        .find(|tab| contains(tab.rect, x, y))
        .map(|tab| tab.tab);
}

fn contains(area: ratatui::layout::Rect, x: u16, y: u16) -> bool {
    x >= area.x && x < area.x + area.width && y >= area.y && y < area.y + area.height
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
    use ratatui::layout::Rect;

    use super::handle_form_click;
    use crate::input::{ActiveTab, AppState, ArgValue, Focus};
    use crate::spec::{ArgKind, ArgSpec, CommandSpec};

    fn arg(id: &str, name: &str, kind: ArgKind) -> ArgSpec {
        ArgSpec {
            id: id.to_string(),
            name: name.to_string(),
            help: None,
            required: false,
            kind,
            default: None,
            possible_values: Vec::new(),
            positional_index: None,
            is_multi: false,
            value_hint: None,
        }
    }

    fn command(args: Vec<ArgSpec>) -> CommandSpec {
        CommandSpec {
            name: "tool".to_string(),
            about: None,
            help: String::new(),
            args,
            subcommands: Vec::new(),
        }
    }

    fn click(column: u16, row: u16) -> MouseEvent {
        MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column,
            row,
            modifiers: KeyModifiers::empty(),
        }
    }

    #[test]
    fn flag_description_click_selects_without_toggling() {
        let mut verbose = arg("verbose", "--verbose", ArgKind::Flag);
        verbose.help = Some("Enable verbose output".to_string());
        let mut state = AppState::new(command(vec![verbose]));
        state.active_tab = ActiveTab::Options;
        state.layout.form = Some(Rect::new(0, 0, 30, 10));
        state.layout.form_view = Some(Rect::new(0, 0, 30, 10));

        handle_form_click(click(1, 1), &mut state);

        assert_eq!(state.selected_arg_index, 0);
        assert!(matches!(state.focus, Focus::Form));
        assert!(!state.is_touched("verbose"));
        assert!(
            state
                .current_inputs()
                .and_then(|inputs| inputs.values.get("verbose"))
                .is_none()
        );

        handle_form_click(click(1, 0), &mut state);

        assert!(state.is_touched("verbose"));
        assert!(matches!(
            state
                .current_inputs()
                .and_then(|inputs| inputs.values.get("verbose")),
            Some(ArgValue::Bool(true))
        ));
    }
}
