use super::{Action, Effect, apply_action};
use crate::{
    TuiConfig, controller,
    frame_snapshot::FrameSnapshot,
    input::{AppState, Focus, HoverTarget},
    runtime::{AppEvent, AppKeyCode, AppKeyEvent, AppKeyModifiers},
};
use clap::{Arg, ArgAction, ArgGroup, Command};
use ratatui::{Terminal, backend::TestBackend};

fn draw(state: &mut AppState, width: u16, height: u16) -> (TestBackend, FrameSnapshot) {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    let mut snapshot = FrameSnapshot::default();
    terminal
        .draw(|frame| {
            snapshot = crate::ui::render(frame, state, &TuiConfig::default());
        })
        .unwrap();
    (terminal.backend().clone(), snapshot)
}

fn jump_action(state: &AppState, snapshot: &FrameSnapshot) -> Action {
    let event = AppEvent::Key(AppKeyEvent::new(
        AppKeyCode::F(8),
        AppKeyModifiers::default(),
    ));
    match controller::handle_event(&event, state, snapshot, &TuiConfig::default()) {
        controller::EventDecision::Action(action) => action,
        other => panic!("expected jump, got {other:?}"),
    }
}

#[test]
fn keyboard_footer_and_invalid_run_reveal_the_same_offscreen_issue() {
    let command = (0..12)
        .fold(Command::new("tool"), |command, index| {
            command.arg(
                Arg::new(format!("optional{index}"))
                    .long(format!("optional{index}"))
                    .display_order(index),
            )
        })
        .arg(
            Arg::new("last")
                .long("last")
                .required(true)
                .display_order(99),
        );
    for size in [(80, 24), (60, 20)] {
        for trigger in 0..3 {
            let mut state = AppState::from_command(&command);
            let (backend, snapshot) = draw(&mut state, size.0, size.1);
            assert!(
                !snapshot
                    .layout
                    .form_fields
                    .iter()
                    .any(|field| field.arg_id == "last")
            );
            let button = snapshot
                .layout
                .footer_buttons
                .iter()
                .find(|button| button.target == HoverTarget::FooterStatus)
                .expect("validation button");
            for x in button.rect.x..button.rect.right() {
                assert_eq!(
                    snapshot.footer_target_at(x, button.rect.y),
                    Some(HoverTarget::FooterStatus)
                );
                let cell = &backend.buffer()[(x, button.rect.y)];
                assert_eq!(cell.bg, TuiConfig::default().theme.shell_bg);
                assert_eq!(cell.fg, TuiConfig::default().theme.error);
            }
            state.ui.help_open = true;
            state.ui.dropdown_open = Some("optional0".into());
            let action = match trigger {
                0 => jump_action(&state, &snapshot),
                1 => Action::ClickFooter(HoverTarget::FooterStatus),
                _ => Action::Run,
            };
            assert_eq!(apply_action(&action, &mut state, &snapshot), Effect::None);
            assert_eq!(state.ui.focus, Focus::Form);
            assert!(!state.ui.help_open && state.ui.dropdown_open.is_none());
            let (backend, after) = draw(&mut state, size.0, size.1);
            assert!(
                after
                    .layout
                    .form_fields
                    .iter()
                    .any(|field| field.arg_id == "last" && field.input.height > 0)
            );
            let text = backend
                .buffer()
                .content
                .iter()
                .map(ratatui::buffer::Cell::symbol)
                .collect::<String>();
            assert!(text.contains("Provide --last"));
            assert!(text.contains("F8 Provide --last"));
            let focus = state.ui.focus.clone();
            let selection = state.ui.selected_arg_index;
            state.domain.set_text_value("last", "fixed");
            assert!(state.derived_validation().is_valid());
            assert_eq!(state.ui.focus, focus);
            assert_eq!(state.ui.selected_arg_index, selection);
            assert!(matches!(
                apply_action(&Action::Run, &mut state, &after),
                Effect::Run(_)
            ));
        }
    }
}

#[test]
fn all_jump_paths_reveal_errors_after_tall_repeated_fields() {
    let command = Command::new("tool")
        .arg(
            Arg::new("items")
                .long("items")
                .action(ArgAction::Append)
                .display_order(0),
        )
        .arg(
            Arg::new("last")
                .long("last")
                .required(true)
                .display_order(1),
        );
    for (width, height) in [(80, 24), (60, 20)] {
        for action in [
            Action::JumpToError,
            Action::ClickFooter(HoverTarget::FooterStatus),
            Action::Run,
        ] {
            let mut state = AppState::from_command(&command);
            state.domain.replace_occurrences(
                "items",
                (0..30)
                    .map(|index| crate::input::InputValueOccurrence {
                        values: vec![format!("item{index}")],
                        source: crate::input::InputSource::User,
                    })
                    .collect(),
            );
            let (_, before) = draw(&mut state, width, height);
            assert!(
                !before
                    .layout
                    .form_fields
                    .iter()
                    .any(|field| field.arg_id == "last")
            );
            assert_eq!(apply_action(&action, &mut state, &before), Effect::None);
            let (backend, after) = draw(&mut state, width, height);
            let field = after
                .layout
                .form_fields
                .iter()
                .find(|field| field.arg_id == "last")
                .expect("error target must be visible");
            assert_eq!(state.ui.focus, Focus::Form);
            assert_eq!(state.ui.selected_arg_index, 1);
            assert_eq!(field.input.height, 3);
            let rect = field
                .description
                .expect("error instruction must be visible");
            let text = (rect.x..rect.right())
                .map(|x| backend.buffer()[(x, rect.y)].symbol())
                .collect::<String>();
            assert_eq!(text.trim(), "Provide --last");
        }
    }
}

#[test]
fn missing_subcommand_targets_sidebar_and_unknown_target_is_a_noop() {
    let mut state = AppState::from_command(
        &Command::new("tool")
            .subcommand_required(true)
            .subcommand(Command::new("child")),
    );
    let (_, snapshot) = draw(&mut state, 80, 24);
    state.ui.focus_form();
    state.ui.help_open = true;
    state.ui.search_query = "no match".into();
    apply_action(&Action::JumpToError, &mut state, &snapshot);
    assert_eq!(state.ui.focus, Focus::Sidebar);
    assert!(state.ui.search_query.is_empty() && !state.ui.help_open);
    let mut state = AppState::from_command(&Command::new("tool").arg_required_else_help(true));
    state.ui.focus_form();
    apply_action(&Action::JumpToError, &mut state, &snapshot);
    assert_eq!(state.ui.focus, Focus::Form);
}

#[test]
fn long_group_instruction_wraps_once_and_highlights_every_alternative() {
    let command = Command::new("tool")
        .arg_required_else_help(true)
        .group(ArgGroup::new("mode").args(["fast", "safe"]).required(true))
        .arg(
            Arg::new("fast")
                .long("required-mode-fast")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("safe")
                .long("required-mode-safe")
                .action(ArgAction::SetTrue),
        );
    for size in [(80, 24), (60, 20)] {
        let mut state = AppState::from_command(&command);
        let (backend, snapshot) = draw(&mut state, size.0, size.1);
        let descriptions = snapshot
            .layout
            .form_fields
            .iter()
            .filter_map(|field| field.description)
            .collect::<Vec<_>>();
        assert_eq!(descriptions.len(), 1);
        assert!(descriptions[0].height > 1);
        for field in &snapshot.layout.form_fields {
            let label = field.label.unwrap();
            assert_eq!(
                backend.buffer()[(label.x, label.y)].fg,
                TuiConfig::default().theme.error
            );
        }
        assert!(snapshot.form_scroll_max < 10);
    }
}

#[test]
fn partially_scrolled_error_renders_the_correct_continuation() {
    let command = Command::new("tool").arg(Arg::new("name").long("name").value_parser(
        |_: &str| -> Result<String, String> { Err("alpha beta gamma delta epsilon ".repeat(30)) },
    ));
    let mut state = AppState::from_command(&command);
    state.domain.set_text_value("name", "bad");
    state.ui.form_scroll = 5;
    let (backend, snapshot) = draw(&mut state, 60, 20);
    let field = snapshot
        .layout
        .form_fields
        .iter()
        .find(|field| field.arg_id == "name")
        .unwrap();
    let rect = field.description.unwrap();
    assert!(field.description_clip_top > 0);
    let message = state
        .derived_validation()
        .field_error("name")
        .unwrap()
        .to_owned();
    let wrapped = crate::layout::form::wrap_validation_message(&message, rect.width);
    let expected = wrapped
        .lines()
        .nth(usize::from(field.description_clip_top))
        .unwrap();
    let actual = (rect.x..rect.right())
        .map(|x| backend.buffer()[(x, rect.y)].symbol())
        .collect::<String>();
    assert_eq!(actual.trim_end(), expected);
}

#[test]
fn overlapping_instructions_share_one_description_with_the_full_wrapped_height() {
    let command = Command::new("tool")
        .arg(
            Arg::new("shared")
                .long("shared")
                .action(ArgAction::SetTrue)
                .display_order(0),
        )
        .arg(
            Arg::new("a")
                .long("a")
                .action(ArgAction::SetTrue)
                .display_order(1),
        )
        .arg(
            Arg::new("b")
                .long("b")
                .action(ArgAction::SetTrue)
                .display_order(2),
        )
        .group(ArgGroup::new("first").args(["shared", "a"]).required(true))
        .group(ArgGroup::new("second").args(["shared", "b"]).required(true));
    for size in [(80, 24), (60, 20)] {
        let mut state = AppState::from_command(&command);
        let (backend, snapshot) = draw(&mut state, size.0, size.1);
        let fields = &snapshot.layout.form_fields;
        assert_eq!(
            fields
                .iter()
                .filter(|field| field.description.is_some())
                .count(),
            1
        );
        let field = fields
            .iter()
            .find(|field| field.arg_id == "shared")
            .unwrap();
        let rect = field.description.unwrap();
        let message = state.derived_validation().inline_message("shared").unwrap();
        let wrapped = crate::layout::form::wrap_validation_message(&message, rect.width);
        assert_eq!(usize::from(rect.height), wrapped.lines().count());
        let actual = (rect.y..rect.bottom())
            .flat_map(|y| (rect.x..rect.right()).map(move |x| (x, y)))
            .map(|point| backend.buffer()[point].symbol())
            .collect::<String>();
        assert_eq!(
            actual.split_whitespace().collect::<String>(),
            message.split_whitespace().collect::<String>()
        );
    }
}
