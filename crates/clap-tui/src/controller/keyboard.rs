use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::config::TuiConfig;
use crate::form_editor::{self, EditResult};
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
    if matches!(state.ui.focus, Focus::Search) {
        handle_search_input(key, state);
        return None;
    }
    if let Some(active_choice) = state.ui.dropdown_open.clone() {
        if handle_choice_input(key, state, &active_choice) {
            return None;
        }
    }
    if matches!(state.ui.focus, Focus::Form) && handle_form_text_input(key, state, config) {
        return None;
    }

    match key.code {
        KeyCode::Tab => {
            state.ui.focus = match state.ui.focus {
                Focus::Sidebar => Focus::Form,
                _ => Focus::Sidebar,
            };
        }
        KeyCode::Char(c) if c == config.keymap.help => navigation::toggle_help_tab(state),
        KeyCode::F(1) => navigation::toggle_help_tab(state),
        KeyCode::BackTab => {
            if matches!(state.ui.focus, Focus::Form) {
                navigation::cycle_tabs(state);
            }
        }
        KeyCode::Char(c) if c == config.keymap.search => state.ui.focus = Focus::Search,
        KeyCode::Up => match state.ui.focus {
            Focus::Sidebar => navigation::move_sidebar_selection(state, -1),
            Focus::Form => navigation::move_form_selection(state, -1),
            Focus::Search => {}
        },
        KeyCode::Down => match state.ui.focus {
            Focus::Sidebar => navigation::move_sidebar_selection(state, 1),
            Focus::Form => navigation::move_form_selection(state, 1),
            Focus::Search => {}
        },
        KeyCode::Left => {
            if matches!(state.ui.focus, Focus::Sidebar) {
                navigation::collapse_selected(state);
            }
        }
        KeyCode::Right => {
            if matches!(state.ui.focus, Focus::Sidebar) {
                navigation::expand_selected(state);
            }
        }
        KeyCode::Enter => {
            if matches!(state.ui.focus, Focus::Sidebar) {
                navigation::select_sidebar(state);
            } else if matches!(state.ui.focus, Focus::Form) {
                navigation::activate_form_field(state);
            }
        }
        KeyCode::Char(' ') => {
            if matches!(state.ui.focus, Focus::Form) {
                navigation::activate_form_field(state);
            }
        }
        _ => {}
    }

    None
}

fn handle_search_input(key: KeyEvent, state: &mut AppState) {
    match key.code {
        KeyCode::Esc | KeyCode::Enter => state.ui.focus = Focus::Sidebar,
        KeyCode::Backspace => {
            state.ui.search_query.pop();
        }
        KeyCode::Char(c) => state.ui.search_query.push(c),
        _ => {}
    }
}

fn handle_form_text_input(key: KeyEvent, state: &mut AppState, config: &TuiConfig) -> bool {
    if matches!(state.ui.active_tab, ActiveTab::Help) {
        return false;
    }
    let command = state.current_command().clone();
    let args = form::visible_args(&command, state.ui.active_tab);
    let Some(item) = args
        .iter()
        .find(|item| item.order_index == state.ui.selected_arg_index)
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

    matches!(
        form_editor::apply_key_to_text_field(state, item.arg, key),
        EditResult::Handled
    )
}

fn handle_choice_input(key: KeyEvent, state: &mut AppState, arg_id: &str) -> bool {
    let command = state.current_command().clone();
    let Some(arg) = command.args.iter().find(|arg| arg.id == arg_id) else {
        state.ui.dropdown_open = None;
        return true;
    };
    let len = arg.choices.len();

    match key.code {
        KeyCode::Up => {
            if len == 0 {
                return true;
            }
            let current = state
                .current_inputs()
                .and_then(|inputs| inputs.values.get(&arg.id))
                .and_then(|value| match value {
                    ArgValue::Choice(selected) => {
                        arg.choices.iter().position(|choice| choice == selected)
                    }
                    _ => None,
                })
                .unwrap_or(0);
            let next = if current == 0 { len - 1 } else { current - 1 };
            state.set_choice_value(&arg.id, arg.choices[next].clone());
            state.mark_touched(&arg.id);
            navigation::ensure_enum_visible(state, next, len);
            true
        }
        KeyCode::Down => {
            state.cycle_choice(&arg.id, &arg.choices);
            let current = state
                .current_inputs()
                .and_then(|inputs| inputs.values.get(&arg.id))
                .and_then(|value| match value {
                    ArgValue::Choice(selected) => {
                        arg.choices.iter().position(|choice| choice == selected)
                    }
                    _ => None,
                })
                .unwrap_or(0);
            state.mark_touched(&arg.id);
            navigation::ensure_enum_visible(state, current, len);
            true
        }
        KeyCode::Esc | KeyCode::Enter | KeyCode::Char(' ') => {
            state.ui.dropdown_open = None;
            true
        }
        _ => false,
    }
}
