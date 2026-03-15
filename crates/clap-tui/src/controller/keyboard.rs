use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::config::TuiConfig;
use crate::frame_snapshot::FrameSnapshot;
use crate::input::{AppState, Focus};
use crate::update::Action;
use crate::view::form;

pub(crate) fn handle_key_event(
    key: KeyEvent,
    state: &AppState,
    _frame_snapshot: &FrameSnapshot,
    config: &TuiConfig,
) -> Option<Action> {
    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
        return Some(Action::Exit);
    }
    if key.code == KeyCode::Enter && key.modifiers.contains(KeyModifiers::CONTROL) {
        return Some(Action::Run);
    }
    if matches!(state.ui.focus, Focus::Search) {
        return Some(Action::SearchInput(key));
    }
    if let Some(active_choice) = state.ui.dropdown_open.as_ref() {
        if matches!(
            key.code,
            KeyCode::Up | KeyCode::Down | KeyCode::Esc | KeyCode::Enter | KeyCode::Char(' ')
        ) {
            return Some(Action::ChoiceInput {
                arg_id: active_choice.clone(),
                key,
            });
        }
    }
    if matches!(state.ui.focus, Focus::Form) && is_form_text_input(key, state, config) {
        return Some(Action::FormTextInput(key));
    }

    match key.code {
        KeyCode::Tab => Some(Action::ToggleFocus),
        KeyCode::Char(c) if c == config.keymap.help => Some(Action::ToggleHelp),
        KeyCode::F(1) => Some(Action::ToggleHelp),
        KeyCode::BackTab => {
            if matches!(state.ui.focus, Focus::Form) {
                Some(Action::CycleTabs)
            } else {
                None
            }
        }
        KeyCode::Char(c) if c == config.keymap.search => Some(Action::FocusSearch),
        KeyCode::Up => match state.ui.focus {
            Focus::Sidebar => Some(Action::MoveSidebarSelection(-1)),
            Focus::Form => Some(Action::MoveFormSelection(-1)),
            Focus::Search => None,
        },
        KeyCode::Down => match state.ui.focus {
            Focus::Sidebar => Some(Action::MoveSidebarSelection(1)),
            Focus::Form => Some(Action::MoveFormSelection(1)),
            Focus::Search => None,
        },
        KeyCode::Left => {
            if matches!(state.ui.focus, Focus::Sidebar) {
                Some(Action::CollapseSelected)
            } else {
                None
            }
        }
        KeyCode::Right => {
            if matches!(state.ui.focus, Focus::Sidebar) {
                Some(Action::ExpandSelected)
            } else {
                None
            }
        }
        KeyCode::Enter => {
            if matches!(state.ui.focus, Focus::Sidebar) {
                Some(Action::SelectSidebar)
            } else if matches!(state.ui.focus, Focus::Form) {
                Some(Action::ActivateFormField)
            } else {
                None
            }
        }
        KeyCode::Char(' ') => {
            if matches!(state.ui.focus, Focus::Form) {
                Some(Action::ActivateFormField)
            } else {
                None
            }
        }
        _ => None,
    }
}

fn is_form_text_input(key: KeyEvent, state: &AppState, config: &TuiConfig) -> bool {
    let command = state.domain.current_command().clone();
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
        KeyCode::Tab | KeyCode::Up | KeyCode::Down | KeyCode::Enter => false,
        KeyCode::Char(c) if c == config.keymap.search => return false,
        _ => true,
    }
}
