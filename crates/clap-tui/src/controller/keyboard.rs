use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tui_textarea::TextArea;

use crate::config::TuiConfig;
use crate::input::{ActiveTab, AppState, ArgValue, Focus};
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

    if matches!(state.interaction.focus, Focus::Search) {
        handle_search_input(key, state);
        return None;
    }

    if let Some(active_enum) = state.interaction.enum_open.clone() {
        if handle_enum_input(key, state, &active_enum) {
            return None;
        }
    }

    if matches!(state.interaction.focus, Focus::Form) && handle_form_text_input(key, state, config)
    {
        return None;
    }

    match key.code {
        KeyCode::Tab => {
            state.interaction.focus = match state.interaction.focus {
                Focus::Sidebar => Focus::Form,
                _ => Focus::Sidebar,
            };
        }
        KeyCode::Char(c) if c == config.keymap.help => navigation::toggle_help_tab(state),
        KeyCode::F(1) => navigation::toggle_help_tab(state),
        KeyCode::BackTab => {
            if matches!(state.interaction.focus, Focus::Form) {
                navigation::cycle_tabs(state);
            }
        }
        KeyCode::Char(c) if c == config.keymap.search => {
            state.interaction.focus = Focus::Search;
        }
        KeyCode::Up => match state.interaction.focus {
            Focus::Sidebar => navigation::move_sidebar_selection(state, -1),
            Focus::Form => navigation::move_form_selection(state, -1),
            Focus::Search => {}
        },
        KeyCode::Down => match state.interaction.focus {
            Focus::Sidebar => navigation::move_sidebar_selection(state, 1),
            Focus::Form => navigation::move_form_selection(state, 1),
            Focus::Search => {}
        },
        KeyCode::Left => {
            if matches!(state.interaction.focus, Focus::Sidebar) {
                navigation::collapse_selected(state);
            }
        }
        KeyCode::Right => {
            if matches!(state.interaction.focus, Focus::Sidebar) {
                navigation::expand_selected(state);
            }
        }
        KeyCode::Enter => {
            if matches!(state.interaction.focus, Focus::Sidebar) {
                navigation::select_sidebar(state);
            } else if matches!(state.interaction.focus, Focus::Form) {
                navigation::activate_form_field(state);
            }
        }
        KeyCode::Char(' ') => {
            if matches!(state.interaction.focus, Focus::Form) {
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
            state.interaction.focus = Focus::Sidebar;
        }
        KeyCode::Backspace => {
            state.command.search.pop();
        }
        KeyCode::Char(c) => {
            state.command.search.push(c);
        }
        _ => {}
    }
}

fn handle_form_text_input(key: KeyEvent, state: &mut AppState, config: &TuiConfig) -> bool {
    if matches!(state.command.active_tab, ActiveTab::Help) {
        return false;
    }
    let command = state.current_command().clone();
    let args = form::visible_args(&command, state.command.active_tab);
    let Some(item) = args
        .iter()
        .find(|item| item.order_index == state.command.selected_arg_index)
    else {
        return false;
    };
    if !item.arg.accepts_text_input() {
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
        state.interaction.enum_open = None;
        return true;
    };
    let len = arg.possible_values.len();

    match key.code {
        KeyCode::Up => {
            if len == 0 {
                return true;
            }
            let current = state
                .current_inputs()
                .and_then(|inputs| inputs.values.get(&arg.id))
                .and_then(|value| match value {
                    ArgValue::Enum(idx) => Some(*idx),
                    _ => None,
                })
                .unwrap_or(0);
            let next = if current == 0 { len - 1 } else { current - 1 };
            state
                .current_inputs_mut()
                .values
                .insert(arg.id.clone(), ArgValue::Enum(next));
            state.mark_touched(&arg.id);
            navigation::ensure_enum_visible(state, next, len);
            true
        }
        KeyCode::Down => {
            state.cycle_enum(&arg.id, len);
            let current = state
                .current_inputs()
                .and_then(|inputs| inputs.values.get(&arg.id))
                .and_then(|value| match value {
                    ArgValue::Enum(idx) => Some(*idx),
                    _ => None,
                })
                .unwrap_or(0);
            state.mark_touched(&arg.id);
            navigation::ensure_enum_visible(state, current, len);
            true
        }
        KeyCode::Esc | KeyCode::Enter | KeyCode::Char(' ') => {
            state.interaction.enum_open = None;
            true
        }
        _ => false,
    }
}
