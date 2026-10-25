use clap::{Arg, Command};
use ratatui::{Terminal, backend::TestBackend};

use crate::{
    TuiConfig, controller,
    frame_snapshot::FrameSnapshot,
    input::{AppState, Focus, HoverTarget, MouseSelection},
    runtime::{
        AppEvent, AppKeyCode, AppKeyEvent, AppKeyModifiers, AppMouseButton, AppMouseEvent,
        AppMouseEventKind,
    },
    update::{Effect, apply_action},
};

fn draw(state: &mut AppState, width: u16, height: u16) -> FrameSnapshot {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    let mut snapshot = FrameSnapshot::default();
    terminal
        .draw(|frame| snapshot = crate::ui::render(frame, state, &TuiConfig::default()))
        .unwrap();
    snapshot
}

fn event(state: &mut AppState, snapshot: &FrameSnapshot, event: &AppEvent) {
    let controller::EventDecision::Action(action) =
        controller::handle_event(event, state, snapshot, &TuiConfig::default())
    else {
        panic!("expected action for {event:?}");
    };
    assert_eq!(apply_action(&action, state, snapshot), Effect::None);
}

fn mouse(kind: AppMouseEventKind, column: u16, row: u16) -> AppEvent {
    AppEvent::Mouse(AppMouseEvent {
        kind,
        column,
        row,
        modifiers: AppKeyModifiers::default(),
    })
}

fn toggle() -> AppEvent {
    AppEvent::Key(AppKeyEvent::new(
        AppKeyCode::Char('b'),
        AppKeyModifiers {
            control: true,
            ..AppKeyModifiers::default()
        },
    ))
}

#[test]
fn drag_collapse_and_reopen_keep_width_and_form_values() {
    for (width, height) in [(80, 24), (60, 20)] {
        let mut state =
            AppState::from_command(&Command::new("tool").arg(Arg::new("name").long("name")));
        state.domain.set_text_value("name", "keep this");
        let argv = state.authoritative_argv();
        state.ui.focus_form();
        state.ui.dropdown_open = Some("name".into());
        state.ui.mouse_select = Some(MouseSelection {
            arg_id: "name".into(),
            anchor_row: 0,
            anchor_col: 0,
            active: false,
        });
        let before = draw(&mut state, width, height);
        let divider = before.layout.sidebar_divider.unwrap();
        let original = before.layout.sidebar.unwrap().width;
        event(
            &mut state,
            &before,
            &mouse(
                AppMouseEventKind::Down(AppMouseButton::Left),
                divider.x,
                divider.y + 2,
            ),
        );
        assert!(state.ui.sidebar.is_resizing());
        assert!(state.ui.mouse_select.is_none() && state.ui.dropdown_open.is_none());
        event(
            &mut state,
            &before,
            &mouse(
                AppMouseEventKind::Drag(AppMouseButton::Left),
                divider.x + 7,
                divider.y + 2,
            ),
        );
        let resized = draw(&mut state, width, height);
        assert_eq!(resized.layout.sidebar.unwrap().width, original + 7);
        // Release over the footer, outside the divider; the gesture still ends.
        event(
            &mut state,
            &resized,
            &mouse(
                AppMouseEventKind::Up(AppMouseButton::Left),
                divider.x + 9,
                height - 1,
            ),
        );
        assert!(!state.ui.sidebar.is_resizing());
        assert_eq!(state.ui.focus, Focus::Form);
        let resized = draw(&mut state, width, height);
        assert_eq!(resized.layout.sidebar.unwrap().width, original + 9);
        event(&mut state, &resized, &toggle());
        let hidden = draw(&mut state, width, height);
        assert!(hidden.layout.sidebar.is_none() && hidden.layout.search.is_none());
        assert!(hidden.layout.sidebar_items.is_empty());
        assert_eq!(hidden.layout.form.unwrap().width, width - 2);
        for key in [AppKeyCode::Tab, AppKeyCode::BackTab] {
            event(
                &mut state,
                &hidden,
                &AppEvent::Key(AppKeyEvent::new(key, AppKeyModifiers::default())),
            );
            assert_eq!(state.ui.focus, Focus::Form);
            assert!(state.ui.sidebar.collapsed);
        }
        let control = hidden.layout.sidebar_toggle.unwrap();
        assert_eq!(control.width, 3);
        event(
            &mut state,
            &hidden,
            &mouse(
                AppMouseEventKind::Down(AppMouseButton::Left),
                control.right() - 1,
                control.y,
            ),
        );
        let reopened = draw(&mut state, width, height);
        assert_eq!(reopened.layout.sidebar.unwrap().width, original + 9);
        assert_eq!(state.ui.focus, Focus::Sidebar);
        assert_eq!(state.authoritative_argv(), argv);
    }
}

#[test]
fn widths_are_clamped_and_terminal_resize_retains_the_preference() {
    let mut state = AppState::from_command(&Command::new("tool"));
    let before = draw(&mut state, 100, 24);
    let divider = before.layout.sidebar_divider.unwrap();
    event(
        &mut state,
        &before,
        &mouse(AppMouseEventKind::Down(AppMouseButton::Left), divider.x, 2),
    );
    event(
        &mut state,
        &before,
        &mouse(AppMouseEventKind::Drag(AppMouseButton::Left), 0, 2),
    );
    assert_eq!(draw(&mut state, 100, 24).layout.sidebar.unwrap().width, 16);
    event(
        &mut state,
        &before,
        &mouse(AppMouseEventKind::Up(AppMouseButton::Left), u16::MAX, 2),
    );
    assert_eq!(draw(&mut state, 100, 24).layout.sidebar.unwrap().width, 80);
    let narrow = draw(&mut state, 60, 20);
    assert_eq!(narrow.layout.sidebar.unwrap().width, 40);
    assert_eq!(draw(&mut state, 100, 24).layout.sidebar.unwrap().width, 80);
    for width in 0..22 {
        let snapshot = draw(&mut state, width, 10);
        assert!(snapshot.layout.form.unwrap().right() <= width);
    }
}

#[test]
fn search_and_command_errors_reopen_the_sidebar_and_reveal_selection() {
    let command = (0..30).fold(
        Command::new("tool").subcommand(
            Command::new("parent")
                .subcommand_required(true)
                .subcommand(Command::new("child")),
        ),
        |command, index| command.subcommand(Command::new(format!("peer{index:02}"))),
    );
    let mut state = AppState::from_command(&command);
    state.select_command_path(&["parent".into()]).unwrap();
    state.domain.expanded.insert("tool::parent".into());
    state.ui.sidebar_scroll = 20;
    state.ui.toggle_sidebar();
    let hidden = draw(&mut state, 80, 24);
    event(
        &mut state,
        &hidden,
        &AppEvent::Key(AppKeyEvent::new(
            AppKeyCode::F(8),
            AppKeyModifiers::default(),
        )),
    );
    assert!(!state.ui.sidebar.collapsed);
    let visible = draw(&mut state, 80, 24);
    assert!(
        visible
            .layout
            .sidebar_items
            .iter()
            .any(|item| item.path.as_slice() == ["parent"])
    );
    event(&mut state, &visible, &toggle());
    let hidden = draw(&mut state, 80, 24);
    let search = hidden
        .layout
        .footer_buttons
        .iter()
        .find(|button| button.target == HoverTarget::Search)
        .unwrap()
        .rect;
    event(
        &mut state,
        &hidden,
        &mouse(
            AppMouseEventKind::Down(AppMouseButton::Left),
            search.x,
            search.y,
        ),
    );
    assert!(!state.ui.sidebar.collapsed);
    assert_eq!(state.ui.focus, Focus::Search);
}

#[test]
fn preview_owns_events_and_argument_errors_leave_sidebar_hidden() {
    let mut state = AppState::from_command(
        &Command::new("tool").arg(Arg::new("name").long("name").required(true)),
    );
    state.ui.toggle_sidebar();
    let hidden = draw(&mut state, 80, 24);
    event(
        &mut state,
        &hidden,
        &AppEvent::Key(AppKeyEvent::new(
            AppKeyCode::F(8),
            AppKeyModifiers::default(),
        )),
    );
    assert!(state.ui.sidebar.collapsed);
    assert_eq!(state.ui.focus, Focus::Form);
    state.ui.preview_open = true;
    let preview = draw(&mut state, 80, 24);
    assert!(preview.layout.sidebar_divider.is_none());
    for input in [
        toggle(),
        mouse(AppMouseEventKind::Drag(AppMouseButton::Left), 20, 2),
    ] {
        assert_eq!(
            controller::handle_event(&input, &state, &preview, &TuiConfig::default()),
            controller::EventDecision::Ignore
        );
        assert!(state.ui.sidebar.collapsed);
    }
}
