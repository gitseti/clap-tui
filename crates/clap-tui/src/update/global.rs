use crate::controller::navigation;
use crate::input::{AppState, Focus, HoverTarget};
use crate::runtime::{AppKeyCode, AppKeyEvent};

use super::{Action, Effect};

pub(crate) fn apply(action: &Action, state: &mut AppState) -> Effect {
    match action {
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
    use crate::runtime::{AppKeyCode, AppKeyEvent, AppKeyModifiers};

    use super::super::{Action, Effect, apply_action};

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
}
