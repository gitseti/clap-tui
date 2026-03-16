use crate::controller::navigation;
use crate::input::{AppState, Focus, HoverTarget};
use crate::runtime::{AppKeyCode, AppKeyEvent};

use super::{Action, Effect};

pub(crate) fn apply(action: &Action, state: &mut AppState) -> Effect {
    match action {
        Action::Escape => {
            navigation::handle_escape(state);
            Effect::None
        }
        Action::SearchInput(key) => {
            apply_search_input(*key, state);
            Effect::None
        }
        Action::ToggleFocus => {
            state.ui.toggle_focus();
            Effect::None
        }
        Action::ToggleHelp => {
            navigation::toggle_help_tab(state);
            Effect::None
        }
        Action::CycleTabs => {
            navigation::cycle_tabs(state);
            Effect::None
        }
        Action::FocusSearch => {
            state.ui.focus_search();
            Effect::None
        }
        Action::CloseDropdown => {
            state.ui.close_dropdown();
            Effect::None
        }
        Action::ClickFooter(target) => apply_footer_click(*target, state),
        Action::SwitchTab(tab) => {
            navigation::switch_tab(state, *tab);
            Effect::None
        }
        _ => Effect::None,
    }
}

fn apply_search_input(key: AppKeyEvent, state: &mut AppState) {
    match key.code {
        AppKeyCode::Esc | AppKeyCode::Enter => state.ui.focus_sidebar(),
        AppKeyCode::Backspace => {
            state.ui.search_query.pop();
        }
        AppKeyCode::Char(c) => state.ui.search_query.push(c),
        _ => {}
    }
}

fn apply_footer_click(target: HoverTarget, state: &mut AppState) -> Effect {
    match target {
        HoverTarget::Run => Effect::Run(crate::pipeline::build_command_line(state)),
        HoverTarget::Exit => Effect::Exit,
        HoverTarget::Search => {
            state.ui.focus_search();
            Effect::None
        }
        HoverTarget::Focus => {
            state.ui.focus = match state.ui.focus {
                Focus::Sidebar => Focus::Form,
                _ => Focus::Sidebar,
            };
            Effect::None
        }
        HoverTarget::Help => {
            navigation::toggle_help_tab(state);
            Effect::None
        }
        HoverTarget::Preview => Effect::None,
    }
}

#[cfg(test)]
mod tests {
    use crate::frame_snapshot::FrameSnapshot;
    use crate::input::Focus;
    use crate::runtime::{AppKeyCode, AppKeyEvent, AppKeyModifiers};
    use crate::spec::CommandSpec;

    use super::super::{Action, Effect, apply_action};

    fn command_with_build() -> CommandSpec {
        CommandSpec {
            name: "tool".to_string(),
            version: None,
            about: None,
            help: String::new(),
            args: Vec::new(),
            subcommands: vec![CommandSpec {
                name: "build".to_string(),
                version: None,
                about: None,
                help: String::new(),
                args: Vec::new(),
                subcommands: Vec::new(),
            }],
        }
    }

    #[test]
    fn search_reducer_appends_and_exits_search_mode() {
        let mut state = crate::input::AppState::new(crate::spec::CommandSpec {
            name: "tool".to_string(),
            version: None,
            about: None,
            help: String::new(),
            args: Vec::new(),
            subcommands: Vec::new(),
        });
        state.ui.focus_search();
        let snapshot = FrameSnapshot::default();

        let action = Action::SearchInput(AppKeyEvent::new(
            AppKeyCode::Char('b'),
            AppKeyModifiers::default(),
        ));
        let effect = apply_action(&action, &mut state, &snapshot);
        assert_eq!(effect, Effect::None);
        assert_eq!(state.ui.search_query, "b");

        let action = Action::SearchInput(AppKeyEvent::new(
            AppKeyCode::Esc,
            AppKeyModifiers::default(),
        ));
        let effect = apply_action(&action, &mut state, &snapshot);
        assert_eq!(effect, Effect::None);
        assert!(matches!(state.ui.focus, crate::input::Focus::Sidebar));
    }

    #[test]
    fn escape_closes_help_before_anything_else() {
        let mut state = crate::input::AppState::new(command_with_build());
        state.ui.help_open = true;
        state.ui.dropdown_open = Some("color".to_string());
        state.ui.focus = Focus::Form;
        let snapshot = FrameSnapshot::default();

        let effect = apply_action(&Action::Escape, &mut state, &snapshot);

        assert_eq!(effect, Effect::None);
        assert!(!state.ui.help_open);
        assert!(state.ui.dropdown_open.is_none());
        assert!(matches!(state.ui.focus, Focus::Form));
    }

    #[test]
    fn escape_closes_dropdown_without_changing_focus() {
        let mut state = crate::input::AppState::new(command_with_build());
        state.ui.dropdown_open = Some("color".to_string());
        state.ui.focus = Focus::Form;
        let snapshot = FrameSnapshot::default();

        let effect = apply_action(&Action::Escape, &mut state, &snapshot);

        assert_eq!(effect, Effect::None);
        assert!(state.ui.dropdown_open.is_none());
        assert!(matches!(state.ui.focus, Focus::Form));
    }

    #[test]
    fn escape_returns_search_focus_to_sidebar_without_clearing_query() {
        let mut state = crate::input::AppState::new(command_with_build());
        state.ui.focus = Focus::Search;
        state.ui.search_query = "build".to_string();
        let snapshot = FrameSnapshot::default();

        let effect = apply_action(&Action::Escape, &mut state, &snapshot);

        assert_eq!(effect, Effect::None);
        assert!(matches!(state.ui.focus, Focus::Sidebar));
        assert_eq!(state.ui.search_query, "build");
    }

    #[test]
    fn escape_returns_form_focus_to_sidebar() {
        let mut state = crate::input::AppState::new(command_with_build());
        state.ui.focus = Focus::Form;
        let snapshot = FrameSnapshot::default();

        let effect = apply_action(&Action::Escape, &mut state, &snapshot);

        assert_eq!(effect, Effect::None);
        assert!(matches!(state.ui.focus, Focus::Sidebar));
    }

    #[test]
    fn escape_in_sidebar_reselects_root_before_becoming_noop() {
        let mut state = crate::input::AppState::new(command_with_build());
        state
            .select_command_path(&["build".to_string()])
            .expect("valid path");
        state.ui.focus = Focus::Sidebar;
        let snapshot = FrameSnapshot::default();

        let effect = apply_action(&Action::Escape, &mut state, &snapshot);

        assert_eq!(effect, Effect::None);
        assert!(state.domain.selected_path().is_empty());
        assert_eq!(state.domain.current_command().name, "tool");
        assert!(matches!(state.ui.focus, Focus::Sidebar));

        let effect = apply_action(&Action::Escape, &mut state, &snapshot);

        assert_eq!(effect, Effect::None);
        assert!(state.domain.selected_path().is_empty());
        assert_eq!(state.domain.current_command().name, "tool");
    }
}
