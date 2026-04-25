use crate::controller::navigation;
use crate::frame_snapshot::FrameSnapshot;
use crate::input::{AppState, Focus, HoverTarget};
use crate::runtime::{AppKeyCode, AppKeyEvent};

use super::{Action, Effect};

pub(crate) fn apply(
    action: &Action,
    state: &mut AppState,
    frame_snapshot: &FrameSnapshot,
) -> Effect {
    match action {
        Action::Escape => {
            navigation::handle_escape(state);
            Effect::None
        }
        Action::SearchInput(key) => {
            apply_search_input(*key, state, frame_snapshot);
            Effect::None
        }
        Action::Paste(text) => {
            apply_paste(text, state);
            Effect::None
        }
        Action::ToggleFocus => {
            state.ui.focus_next();
            Effect::None
        }
        Action::ReverseFocus => {
            state.ui.focus_previous();
            Effect::None
        }
        Action::ToggleHelp => {
            navigation::toggle_help_tab(state);
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
        Action::ClickFooter(target) => apply_footer_click(*target, state, frame_snapshot),
        Action::SwitchTab(tab) => {
            navigation::switch_tab(state, *tab);
            Effect::None
        }
        _ => Effect::None,
    }
}

fn apply_search_input(key: AppKeyEvent, state: &mut AppState, frame_snapshot: &FrameSnapshot) {
    match key.code {
        AppKeyCode::Tab => state.ui.focus_form(),
        AppKeyCode::Esc | AppKeyCode::Enter | AppKeyCode::BackTab => {
            state.ui.focus_sidebar();
        }
        AppKeyCode::Up => {
            navigation::move_sidebar_selection(state, frame_snapshot, -1);
        }
        AppKeyCode::Down => {
            navigation::move_sidebar_selection(state, frame_snapshot, 1);
        }
        AppKeyCode::Backspace => {
            state.ui.search_query.pop();
            navigation::clamp_sidebar_selection_to_search(state, frame_snapshot);
        }
        AppKeyCode::Char(c) => {
            state.ui.search_query.push(c);
            navigation::clamp_sidebar_selection_to_search(state, frame_snapshot);
        }
        _ => {}
    }
}

fn apply_paste(text: &str, state: &mut AppState) {
    match state.ui.focus {
        Focus::Search => state.ui.search_query.push_str(text),
        Focus::Form => super::form::apply_paste_text(state, text),
        Focus::Sidebar => {}
    }
}

fn apply_footer_click(
    target: HoverTarget,
    state: &mut AppState,
    frame_snapshot: &FrameSnapshot,
) -> Effect {
    match target {
        HoverTarget::Run => {
            state.ui.dismiss_transient_interaction();
            Effect::Run(state.authoritative_argv())
        }
        HoverTarget::Exit => {
            state.ui.dismiss_transient_interaction();
            Effect::Exit
        }
        HoverTarget::Search => {
            state.ui.focus_search();
            Effect::None
        }
        HoverTarget::Focus => {
            state.ui.focus_next();
            Effect::None
        }
        HoverTarget::Help => {
            navigation::toggle_help_tab(state);
            Effect::None
        }
        HoverTarget::FooterStatus => {
            let validation = state.derived_validation();
            if validation.summary.is_some() {
                navigation::focus_first_invalid_field(state, frame_snapshot, &validation);
            }
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
                ..CommandSpec::default()
            }],
            ..CommandSpec::default()
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
            ..crate::spec::CommandSpec::default()
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
    fn search_reducer_clamps_sidebar_selection_and_scroll_to_visible_matches() {
        let mut state = crate::input::AppState::new(crate::spec::CommandSpec {
            name: "tool".to_string(),
            version: None,
            about: None,
            help: String::new(),
            args: Vec::new(),
            subcommands: vec![
                crate::spec::CommandSpec {
                    name: "build".to_string(),
                    version: None,
                    about: None,
                    help: String::new(),
                    args: Vec::new(),
                    subcommands: Vec::new(),
                    ..crate::spec::CommandSpec::default()
                },
                crate::spec::CommandSpec {
                    name: "deploy".to_string(),
                    version: None,
                    about: None,
                    help: String::new(),
                    args: Vec::new(),
                    subcommands: Vec::new(),
                    ..crate::spec::CommandSpec::default()
                },
            ],
            ..crate::spec::CommandSpec::default()
        });
        state
            .select_command_path(&["build".to_string()])
            .expect("valid path");
        state.ui.focus_search();
        state.ui.sidebar_scroll = 3;
        let mut snapshot = FrameSnapshot::default();
        snapshot.layout.sidebar_list = Some(ratatui::layout::Rect::new(0, 0, 20, 4));

        let effect = apply_action(
            &Action::SearchInput(AppKeyEvent::new(
                AppKeyCode::Char('p'),
                AppKeyModifiers::default(),
            )),
            &mut state,
            &snapshot,
        );

        assert_eq!(effect, Effect::None);
        assert_eq!(state.ui.search_query, "p");
        assert_eq!(
            state.domain.selected_path().as_slice(),
            &["deploy".to_string()]
        );
        assert_eq!(state.ui.sidebar_scroll, 0);
    }

    #[test]
    fn search_reducer_moves_sidebar_selection_with_arrow_keys_while_search_stays_focused() {
        let mut state = crate::input::AppState::new(crate::spec::CommandSpec {
            name: "tool".to_string(),
            version: None,
            about: None,
            help: String::new(),
            args: Vec::new(),
            subcommands: vec![
                crate::spec::CommandSpec {
                    name: "build".to_string(),
                    version: None,
                    about: None,
                    help: String::new(),
                    args: Vec::new(),
                    subcommands: Vec::new(),
                    ..crate::spec::CommandSpec::default()
                },
                crate::spec::CommandSpec {
                    name: "deploy".to_string(),
                    version: None,
                    about: None,
                    help: String::new(),
                    args: Vec::new(),
                    subcommands: Vec::new(),
                    ..crate::spec::CommandSpec::default()
                },
            ],
            ..crate::spec::CommandSpec::default()
        });
        state.ui.focus_search();
        state.ui.search_query = "dep".to_string();
        let mut snapshot = FrameSnapshot::default();
        snapshot.layout.sidebar_list = Some(ratatui::layout::Rect::new(0, 0, 20, 4));

        let effect = apply_action(
            &Action::SearchInput(AppKeyEvent::new(
                AppKeyCode::Down,
                AppKeyModifiers::default(),
            )),
            &mut state,
            &snapshot,
        );

        assert_eq!(effect, Effect::None);
        assert!(matches!(state.ui.focus, Focus::Search));
        assert_eq!(
            state.domain.selected_path().as_slice(),
            &["deploy".to_string()]
        );

        let effect = apply_action(
            &Action::SearchInput(AppKeyEvent::new(AppKeyCode::Up, AppKeyModifiers::default())),
            &mut state,
            &snapshot,
        );

        assert_eq!(effect, Effect::None);
        assert!(matches!(state.ui.focus, Focus::Search));
        assert!(state.domain.selected_path().is_empty());
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

    #[test]
    fn toggle_focus_closes_dropdown() {
        let mut state = crate::input::AppState::new(command_with_build());
        state.ui.focus = Focus::Form;
        state.ui.dropdown_open = Some("color".to_string());
        let snapshot = FrameSnapshot::default();

        let effect = apply_action(&Action::ToggleFocus, &mut state, &snapshot);

        assert_eq!(effect, Effect::None);
        assert!(state.ui.dropdown_open.is_none());
        assert!(matches!(state.ui.focus, Focus::Sidebar));
    }

    #[test]
    fn focus_search_closes_dropdown() {
        let mut state = crate::input::AppState::new(command_with_build());
        state.ui.focus = Focus::Form;
        state.ui.dropdown_open = Some("color".to_string());
        let snapshot = FrameSnapshot::default();

        let effect = apply_action(&Action::FocusSearch, &mut state, &snapshot);

        assert_eq!(effect, Effect::None);
        assert!(state.ui.dropdown_open.is_none());
        assert!(matches!(state.ui.focus, Focus::Search));
    }

    #[test]
    fn focus_traversal_cycles_through_sidebar_search_and_form() {
        let mut state = crate::input::AppState::new(command_with_build());
        let snapshot = FrameSnapshot::default();

        assert!(matches!(state.ui.focus, Focus::Sidebar));

        let effect = apply_action(&Action::ToggleFocus, &mut state, &snapshot);
        assert_eq!(effect, Effect::None);
        assert!(matches!(state.ui.focus, Focus::Search));

        let effect = apply_action(&Action::ToggleFocus, &mut state, &snapshot);
        assert_eq!(effect, Effect::None);
        assert!(matches!(state.ui.focus, Focus::Form));

        let effect = apply_action(&Action::ToggleFocus, &mut state, &snapshot);
        assert_eq!(effect, Effect::None);
        assert!(matches!(state.ui.focus, Focus::Sidebar));

        state.ui.focus = Focus::Form;
        let effect = apply_action(&Action::ReverseFocus, &mut state, &snapshot);
        assert_eq!(effect, Effect::None);
        assert!(matches!(state.ui.focus, Focus::Search));

        let effect = apply_action(&Action::ReverseFocus, &mut state, &snapshot);
        assert_eq!(effect, Effect::None);
        assert!(matches!(state.ui.focus, Focus::Sidebar));
    }

    #[test]
    fn footer_status_click_focuses_first_invalid_field_when_available() {
        let mut state = crate::input::AppState::from_command(
            &clap::Command::new("tool").arg(clap::Arg::new("name").long("name").required(true)),
        );
        state.ui.focus = Focus::Sidebar;
        let mut snapshot = FrameSnapshot::default();
        snapshot.layout.form_view = Some(ratatui::layout::Rect::new(0, 0, 40, 6));

        let effect = apply_action(
            &Action::ClickFooter(crate::input::HoverTarget::FooterStatus),
            &mut state,
            &snapshot,
        );

        assert_eq!(effect, Effect::None);
        assert_eq!(state.ui.focus, Focus::Form);
        assert_eq!(state.ui.selected_arg_index, 0);
    }

    #[test]
    fn footer_status_click_focuses_first_required_group_member_when_available() {
        let mut state = crate::input::AppState::from_command(
            &clap::Command::new("tool")
                .group(
                    clap::ArgGroup::new("mode")
                        .args(["fast", "safe"])
                        .required(true),
                )
                .arg(
                    clap::Arg::new("fast")
                        .long("fast")
                        .action(clap::ArgAction::SetTrue)
                        .group("mode"),
                )
                .arg(
                    clap::Arg::new("safe")
                        .long("safe")
                        .action(clap::ArgAction::SetTrue)
                        .group("mode"),
                ),
        );
        state.ui.focus = Focus::Sidebar;
        let mut snapshot = FrameSnapshot::default();
        snapshot.layout.form_view = Some(ratatui::layout::Rect::new(0, 0, 40, 6));

        let effect = apply_action(
            &Action::ClickFooter(crate::input::HoverTarget::FooterStatus),
            &mut state,
            &snapshot,
        );

        assert_eq!(effect, Effect::None);
        assert_eq!(state.ui.focus, Focus::Form);
        assert_eq!(state.ui.selected_arg_index, 0);
    }
}
