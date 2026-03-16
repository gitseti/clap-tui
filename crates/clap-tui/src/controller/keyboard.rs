use crate::config::TuiConfig;
use crate::frame_snapshot::FrameSnapshot;
use crate::input::{AppState, Focus};
use crate::query::form;
use crate::runtime::{AppKeyCode, AppKeyEvent};
use crate::update::Action;

pub(crate) fn handle_key_event(
    key: AppKeyEvent,
    state: &AppState,
    _frame_snapshot: &FrameSnapshot,
    config: &TuiConfig,
) -> Option<Action> {
    if key.code == AppKeyCode::Char('c') && key.modifiers.control {
        return Some(Action::Exit);
    }
    if key.code == AppKeyCode::Char('r') && key.modifiers.control {
        return Some(Action::Run);
    }
    if key.code == AppKeyCode::Enter && key.modifiers.control {
        return Some(Action::Run);
    }
    if key.code == AppKeyCode::Esc {
        return Some(Action::Escape);
    }
    if state.ui.help_open {
        if matches!(key.code, AppKeyCode::F(1))
            || matches!(key.code, AppKeyCode::Char(c) if c == config.keymap.help)
        {
            return Some(Action::ToggleHelp);
        }
        return match key.code {
            AppKeyCode::Up => Some(Action::ScrollForm(-1)),
            AppKeyCode::Down => Some(Action::ScrollForm(1)),
            AppKeyCode::PageUp => Some(Action::ScrollForm(-10)),
            AppKeyCode::PageDown => Some(Action::ScrollForm(10)),
            AppKeyCode::Tab => Some(Action::ToggleFocus),
            _ => None,
        };
    }
    if matches!(state.ui.focus, Focus::Search) {
        return Some(Action::SearchInput(key));
    }
    if let Some(active_choice) = state.ui.dropdown_open.as_ref() {
        if matches!(
            key.code,
            AppKeyCode::Up
                | AppKeyCode::Down
                | AppKeyCode::Esc
                | AppKeyCode::Enter
                | AppKeyCode::Char(' ')
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
        AppKeyCode::Tab => Some(Action::ToggleFocus),
        AppKeyCode::Char(c) if c == config.keymap.help => Some(Action::ToggleHelp),
        AppKeyCode::F(1) => Some(Action::ToggleHelp),
        AppKeyCode::BackTab => {
            if matches!(state.ui.focus, Focus::Form) {
                Some(Action::CycleTabs)
            } else {
                None
            }
        }
        AppKeyCode::Char(c) if c == config.keymap.search => Some(Action::FocusSearch),
        AppKeyCode::Up => match state.ui.focus {
            Focus::Sidebar => Some(Action::MoveSidebarSelection(-1)),
            Focus::Form => Some(Action::MoveFormSelection(-1)),
            Focus::Search => None,
        },
        AppKeyCode::Down => match state.ui.focus {
            Focus::Sidebar => Some(Action::MoveSidebarSelection(1)),
            Focus::Form => Some(Action::MoveFormSelection(1)),
            Focus::Search => None,
        },
        AppKeyCode::Left => {
            if matches!(state.ui.focus, Focus::Sidebar) {
                Some(Action::CollapseSelected)
            } else {
                None
            }
        }
        AppKeyCode::Right => {
            if matches!(state.ui.focus, Focus::Sidebar) {
                Some(Action::SidebarRight)
            } else {
                None
            }
        }
        AppKeyCode::Enter => {
            if matches!(state.ui.focus, Focus::Sidebar) {
                Some(Action::SelectSidebar)
            } else if matches!(state.ui.focus, Focus::Form) {
                Some(Action::ActivateFormField)
            } else {
                None
            }
        }
        AppKeyCode::Char(' ') => {
            if matches!(state.ui.focus, Focus::Form) {
                Some(Action::ActivateFormField)
            } else {
                None
            }
        }
        _ => None,
    }
}

fn is_form_text_input(key: AppKeyEvent, state: &AppState, config: &TuiConfig) -> bool {
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
        AppKeyCode::Tab
        | AppKeyCode::Up
        | AppKeyCode::Down
        | AppKeyCode::Enter
        | AppKeyCode::Esc => false,
        AppKeyCode::Char(c) if c == config.keymap.search => false,
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use crate::config::TuiConfig;
    use crate::frame_snapshot::FrameSnapshot;
    use crate::input::{AppState, Focus};
    use crate::runtime::{AppKeyCode, AppKeyEvent, AppKeyModifiers};
    use crate::spec::CommandSpec;
    use crate::update::Action;

    use super::handle_key_event;

    fn command() -> CommandSpec {
        CommandSpec {
            name: "tool".to_string(),
            version: None,
            about: None,
            help: String::new(),
            args: Vec::new(),
            subcommands: Vec::new(),
        }
    }

    fn key(code: AppKeyCode) -> AppKeyEvent {
        AppKeyEvent::new(code, AppKeyModifiers::default())
    }

    #[test]
    fn right_in_sidebar_emits_sidebar_right_action() {
        let state = AppState::new(command());

        let action = handle_key_event(
            key(AppKeyCode::Right),
            &state,
            &FrameSnapshot::default(),
            &TuiConfig::default(),
        );

        assert_eq!(action, Some(Action::SidebarRight));
    }

    #[test]
    fn right_outside_sidebar_preserves_existing_behavior() {
        let mut state = AppState::new(command());
        state.ui.focus = Focus::Form;

        let action = handle_key_event(
            key(AppKeyCode::Right),
            &state,
            &FrameSnapshot::default(),
            &TuiConfig::default(),
        );

        assert_eq!(action, None);
    }

    #[test]
    fn escape_emits_centralized_escape_action() {
        let mut state = AppState::new(command());
        state.ui.focus = Focus::Search;

        let action = handle_key_event(
            key(AppKeyCode::Esc),
            &state,
            &FrameSnapshot::default(),
            &TuiConfig::default(),
        );

        assert_eq!(action, Some(Action::Escape));
    }

    #[test]
    fn ctrl_r_emits_run_action() {
        let state = AppState::new(command());

        let action = handle_key_event(
            AppKeyEvent::new(
                AppKeyCode::Char('r'),
                AppKeyModifiers {
                    control: true,
                    alt: false,
                    shift: false,
                },
            ),
            &state,
            &FrameSnapshot::default(),
            &TuiConfig::default(),
        );

        assert_eq!(action, Some(Action::Run));
    }
}
