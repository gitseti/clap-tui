use crate::config::TuiConfig;
use crate::frame_snapshot::FrameSnapshot;
use crate::input::{AppState, ArgValue, Focus};
use crate::query::form::{self, FieldWidget};
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
        return handle_help_key_event(key, config);
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
    if matches!(state.ui.focus, Focus::Form)
        && let Some(action) = handle_form_widget_key_event(key, state, config)
    {
        return Some(action);
    }
    if matches!(state.ui.focus, Focus::Form) && is_form_text_input(key, state) {
        return Some(Action::FormTextInput(key));
    }

    handle_focused_key_event(key, state, config)
}

fn handle_form_widget_key_event(
    key: AppKeyEvent,
    state: &AppState,
    config: &TuiConfig,
) -> Option<Action> {
    let command = state.domain.current_command().clone();
    let args = form::visible_args(&command, state.ui.active_tab);
    let item = args
        .iter()
        .find(|item| item.order_index == state.ui.selected_arg_index)?;

    match item.widget {
        FieldWidget::Counter => match key.code {
            AppKeyCode::Right
            | AppKeyCode::Char('+' | '=' | '-')
            | AppKeyCode::Left
            | AppKeyCode::Backspace => Some(Action::FormWidgetInput(key)),
            _ => None,
        },
        FieldWidget::RepeatedText
            if key.code == AppKeyCode::Enter || repeated_row_shortcut(key) =>
        {
            Some(Action::FormWidgetInput(key))
        }
        FieldWidget::OptionalValue => match key.code {
            AppKeyCode::Left | AppKeyCode::Delete | AppKeyCode::Backspace | AppKeyCode::Right => {
                Some(Action::FormWidgetInput(key))
            }
            AppKeyCode::Char(c) if c == config.keymap.search => None,
            _ => None,
        },
        _ => None,
    }
}

fn repeated_row_shortcut(key: AppKeyEvent) -> bool {
    matches!(key.code, AppKeyCode::Up | AppKeyCode::Down) && key.modifiers.alt
        || matches!(key.code, AppKeyCode::Delete | AppKeyCode::Backspace) && key.modifiers.control
}

fn handle_help_key_event(key: AppKeyEvent, config: &TuiConfig) -> Option<Action> {
    if matches!(key.code, AppKeyCode::F(1))
        || matches!(key.code, AppKeyCode::Char(c) if c == config.keymap.help)
    {
        return Some(Action::ToggleHelp);
    }

    match key.code {
        AppKeyCode::Up => Some(Action::ScrollForm(-1)),
        AppKeyCode::Down => Some(Action::ScrollForm(1)),
        AppKeyCode::PageUp => Some(Action::ScrollForm(-10)),
        AppKeyCode::PageDown => Some(Action::ScrollForm(10)),
        AppKeyCode::Tab => Some(Action::ToggleFocus),
        _ => None,
    }
}

fn handle_focused_key_event(
    key: AppKeyEvent,
    state: &AppState,
    config: &TuiConfig,
) -> Option<Action> {
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

fn is_form_text_input(key: AppKeyEvent, state: &AppState) -> bool {
    let command = state.domain.current_command().clone();
    let args = form::visible_args(&command, state.ui.active_tab);
    let Some(item) = args
        .iter()
        .find(|item| item.order_index == state.ui.selected_arg_index)
    else {
        return false;
    };
    if !item.widget.accepts_text_input() {
        return false;
    }

    if key.code == AppKeyCode::Enter {
        return false;
    }

    if matches!(item.widget, FieldWidget::OptionalValue) && key.code == AppKeyCode::Char(' ') {
        return state
            .domain
            .current_form()
            .filter(|_| state.domain.is_touched(&item.arg.id))
            .and_then(|form| form.compatibility_value(item.arg))
            .is_some_and(|value| matches!(value, ArgValue::Text(_)));
    }

    !matches!(
        key.code,
        AppKeyCode::Tab | AppKeyCode::Up | AppKeyCode::Down | AppKeyCode::Esc
    )
}

#[cfg(test)]
mod tests {
    use clap::{Arg, ArgAction};

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
            ..CommandSpec::default()
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

    #[test]
    fn counter_fields_route_left_and_right_to_widget_input() {
        let mut state = AppState::from_command(
            &clap::Command::new("tool")
                .arg(Arg::new("verbose").short('v').action(ArgAction::Count)),
        );
        state.ui.focus = Focus::Form;

        let right = handle_key_event(
            key(AppKeyCode::Right),
            &state,
            &FrameSnapshot::default(),
            &TuiConfig::default(),
        );
        let left = handle_key_event(
            key(AppKeyCode::Left),
            &state,
            &FrameSnapshot::default(),
            &TuiConfig::default(),
        );

        assert_eq!(right, Some(Action::FormWidgetInput(key(AppKeyCode::Right))));
        assert_eq!(left, Some(Action::FormWidgetInput(key(AppKeyCode::Left))));
    }

    #[test]
    fn repeated_text_fields_route_enter_to_widget_input() {
        let mut state = AppState::from_command(
            &clap::Command::new("tool").arg(
                Arg::new("include")
                    .long("include")
                    .action(ArgAction::Append)
                    .num_args(1),
            ),
        );
        state.ui.focus = Focus::Form;

        let action = handle_key_event(
            key(AppKeyCode::Enter),
            &state,
            &FrameSnapshot::default(),
            &TuiConfig::default(),
        );

        assert_eq!(
            action,
            Some(Action::FormWidgetInput(key(AppKeyCode::Enter)))
        );
    }

    #[test]
    fn repeated_text_fields_route_row_shortcuts_to_widget_input() {
        let mut state = AppState::from_command(
            &clap::Command::new("tool").arg(
                Arg::new("include")
                    .long("include")
                    .action(ArgAction::Append)
                    .num_args(1),
            ),
        );
        state.ui.focus = Focus::Form;
        let alt_up = AppKeyEvent::new(
            AppKeyCode::Up,
            AppKeyModifiers {
                alt: true,
                ..AppKeyModifiers::default()
            },
        );
        let ctrl_delete = AppKeyEvent::new(
            AppKeyCode::Delete,
            AppKeyModifiers {
                control: true,
                ..AppKeyModifiers::default()
            },
        );

        let move_action = handle_key_event(
            alt_up,
            &state,
            &FrameSnapshot::default(),
            &TuiConfig::default(),
        );
        let remove_action = handle_key_event(
            ctrl_delete,
            &state,
            &FrameSnapshot::default(),
            &TuiConfig::default(),
        );

        assert_eq!(move_action, Some(Action::FormWidgetInput(alt_up)));
        assert_eq!(remove_action, Some(Action::FormWidgetInput(ctrl_delete)));
    }

    #[test]
    fn optional_value_space_activates_when_no_explicit_text() {
        let mut state = AppState::from_command(
            &clap::Command::new("tool").arg(
                Arg::new("color")
                    .long("color")
                    .action(ArgAction::Set)
                    .num_args(0..=1),
            ),
        );
        state.ui.focus = Focus::Form;

        let action = handle_key_event(
            key(AppKeyCode::Char(' ')),
            &state,
            &FrameSnapshot::default(),
            &TuiConfig::default(),
        );

        assert_eq!(action, Some(Action::ActivateFormField));
    }

    #[test]
    fn optional_value_space_activates_when_value_is_only_defaulted() {
        let mut state = AppState::from_command(
            &clap::Command::new("tool").arg(
                Arg::new("color")
                    .long("color")
                    .action(ArgAction::Set)
                    .default_value("auto")
                    .num_args(0..=1),
            ),
        );
        state.ui.focus = Focus::Form;

        let action = handle_key_event(
            key(AppKeyCode::Char(' ')),
            &state,
            &FrameSnapshot::default(),
            &TuiConfig::default(),
        );

        assert_eq!(action, Some(Action::ActivateFormField));
    }

    #[test]
    fn search_key_is_treated_as_text_input_in_form_fields() {
        let mut state =
            AppState::from_command(&clap::Command::new("tool").arg(Arg::new("path").long("path")));
        state.ui.focus = Focus::Form;

        let action = handle_key_event(
            key(AppKeyCode::Char('/')),
            &state,
            &FrameSnapshot::default(),
            &TuiConfig::default(),
        );

        assert_eq!(
            action,
            Some(Action::FormTextInput(key(AppKeyCode::Char('/'))))
        );
    }

    #[test]
    fn search_key_still_focuses_search_outside_text_input() {
        let state = AppState::new(command());

        let action = handle_key_event(
            key(AppKeyCode::Char('/')),
            &state,
            &FrameSnapshot::default(),
            &TuiConfig::default(),
        );

        assert_eq!(action, Some(Action::FocusSearch));
    }
}
