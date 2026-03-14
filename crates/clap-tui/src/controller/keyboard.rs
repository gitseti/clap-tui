use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tui_textarea::TextArea;

use crate::config::TuiConfig;
use crate::input::{ActiveTab, AppState, ArgValue, Focus};
use crate::spec::ArgKind;
use crate::view::{argv, form};

use super::Action;
use super::navigation;

pub(crate) fn handle_key_event(
    key: KeyEvent,
    state: &mut AppState,
    config: &TuiConfig,
) -> Option<Action> {
    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
        return Some(Action::Exit);
    }

    if key.code == KeyCode::Enter && key.modifiers.contains(KeyModifiers::CONTROL) {
        return Some(Action::Run(argv::build_argv(state)));
    }

    if matches!(state.focus, Focus::Search) {
        handle_search_input(key, state);
        return None;
    }

    if let Some(active_enum) = state.enum_open.clone() {
        if handle_enum_input(key, state, &active_enum) {
            return None;
        }
    }

    if matches!(state.focus, Focus::Form) && handle_form_text_input(key, state, config) {
        return None;
    }

    match key.code {
        KeyCode::Tab => {
            state.focus = match state.focus {
                Focus::Sidebar => Focus::Form,
                _ => Focus::Sidebar,
            };
        }
        KeyCode::Char(c) if c == config.keymap.help => navigation::toggle_help_tab(state),
        KeyCode::F(1) => navigation::toggle_help_tab(state),
        KeyCode::BackTab => {
            if matches!(state.focus, Focus::Form) {
                navigation::cycle_tabs(state);
            }
        }
        KeyCode::Char(c) if c == config.keymap.search => {
            state.focus = Focus::Search;
        }
        KeyCode::Up => match state.focus {
            Focus::Sidebar => navigation::move_sidebar_selection(state, -1),
            Focus::Form => navigation::move_form_selection(state, -1),
            _ => {}
        },
        KeyCode::Down => match state.focus {
            Focus::Sidebar => navigation::move_sidebar_selection(state, 1),
            Focus::Form => navigation::move_form_selection(state, 1),
            _ => {}
        },
        KeyCode::Left => {
            if matches!(state.focus, Focus::Sidebar) {
                navigation::collapse_selected(state);
            }
        }
        KeyCode::Right => {
            if matches!(state.focus, Focus::Sidebar) {
                navigation::expand_selected(state);
            }
        }
        KeyCode::Enter => {
            if matches!(state.focus, Focus::Sidebar) {
                navigation::select_sidebar(state);
            } else if matches!(state.focus, Focus::Form) {
                navigation::activate_form_field(state);
            }
        }
        KeyCode::Char(' ') => {
            if matches!(state.focus, Focus::Form) {
                navigation::activate_form_field(state);
            }
        }
        _ => {}
    }

    None
}

fn handle_search_input(key: KeyEvent, state: &mut AppState) {
    match key.code {
        KeyCode::Esc | KeyCode::Enter => {
            state.focus = Focus::Sidebar;
        }
        KeyCode::Backspace => {
            state.search.pop();
        }
        KeyCode::Char(c) => {
            state.search.push(c);
        }
        _ => {}
    }
}

fn handle_form_text_input(key: KeyEvent, state: &mut AppState, config: &TuiConfig) -> bool {
    if matches!(state.active_tab, ActiveTab::Help) {
        return false;
    }
    let command = state.current_command().clone();
    let args = form::visible_args(&command, state.active_tab);
    let Some(item) = args
        .iter()
        .find(|item| item.order_index == state.selected_arg_index)
    else {
        return false;
    };
    if !matches!(item.arg.kind, ArgKind::Option | ArgKind::Positional) {
        return false;
    }

    match key.code {
        KeyCode::Tab | KeyCode::Up | KeyCode::Down | KeyCode::Enter => return false,
        KeyCode::Char(c) if c == config.keymap.search => return false,
        _ => {}
    }

    let arg_id = item.arg.id.clone();
    let current = state
        .current_inputs()
        .and_then(|inputs| inputs.values.get(&arg_id))
        .and_then(|value| match value {
            ArgValue::Text(text) => Some(text.clone()),
            _ => None,
        })
        .unwrap_or_default();
    let default_value = item.arg.default.clone();
    let has_default = default_value.is_some();
    let is_touched = state.is_touched(&arg_id);

    let textarea = state.textarea_for(&arg_id, &current);
    if has_default && !is_touched {
        match key.code {
            KeyCode::Char(_) | KeyCode::Backspace => {
                *textarea = TextArea::new(vec![String::new()]);
            }
            _ => {}
        }
    }
    let modified = textarea.input(key);
    if modified {
        let text = textarea.lines().join("\n");
        if text.is_empty() && default_value.is_some() {
            state.current_inputs_mut().values.remove(&arg_id);
            state.clear_touched(&arg_id);
        } else {
            state.set_text_value(&arg_id, text);
            state.mark_touched(&arg_id);
        }
    }
    true
}

fn handle_enum_input(key: KeyEvent, state: &mut AppState, arg_id: &str) -> bool {
    let command = state.current_command().clone();
    let Some(arg) = command.args.iter().find(|arg| arg.id == arg_id) else {
        state.enum_open = None;
        return true;
    };
    let arg_id = arg.id.clone();
    let len = arg.possible_values.len();

    match key.code {
        KeyCode::Esc => {
            state.enum_open = None;
            return true;
        }
        KeyCode::Up => {
            if len == 0 {
                return true;
            }
            let current = state
                .current_inputs()
                .and_then(|inputs| inputs.values.get(&arg_id))
                .and_then(|value| match value {
                    ArgValue::Enum(idx) => Some(*idx),
                    _ => None,
                })
                .unwrap_or(0);
            let next = if current == 0 { len - 1 } else { current - 1 };
            state
                .current_inputs_mut()
                .values
                .insert(arg_id.clone(), ArgValue::Enum(next));
            state.mark_touched(&arg_id);
            navigation::ensure_enum_visible(state, next, len);
            return true;
        }
        KeyCode::Down => {
            state.cycle_enum(&arg_id, len);
            let current = state
                .current_inputs()
                .and_then(|inputs| inputs.values.get(&arg_id))
                .and_then(|value| match value {
                    ArgValue::Enum(idx) => Some(*idx),
                    _ => None,
                })
                .unwrap_or(0);
            state.mark_touched(&arg_id);
            navigation::ensure_enum_visible(state, current, len);
            return true;
        }
        KeyCode::Enter | KeyCode::Char(' ') => {
            state.enum_open = None;
            return true;
        }
        _ => {}
    }
    false
}
