use clap::{Arg, Command};
use ratatui::{Terminal, backend::TestBackend};

use crate::{
    TuiConfig, controller,
    frame_snapshot::FrameSnapshot,
    input::{AppState, Focus, HoverTarget, MouseSelection},
    runtime::{
        AppEvent, AppKeyCode, AppKeyEvent, AppKeyModifiers,
        AppMouseButton::Left,
        AppMouseEvent,
        AppMouseEventKind::{self, Down, Drag, Moved, Up},
        MousePointer,
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
    state.ui.observe_pointer(event);
    let controller::EventDecision::Action(action) =
        controller::handle_event(event, state, snapshot, &TuiConfig::default())
    else {
        panic!("expected action for {event:?}");
    };
    assert_eq!(apply_action(&action, state, snapshot), Effect::None);
}

fn send_mouse(
    state: &mut AppState,
    snapshot: &FrameSnapshot,
    kind: AppMouseEventKind,
    column: u16,
    row: u16,
) {
    event(
        state,
        snapshot,
        &AppEvent::Mouse(AppMouseEvent {
            kind,
            column,
            row,
            modifiers: AppKeyModifiers::default(),
        }),
    );
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
fn header_and_footer_toggles_highlight_independently_but_share_the_action() {
    let mut state = AppState::from_command(&Command::new("tool"));
    let config = TuiConfig::default();
    let mut terminal = Terminal::new(TestBackend::new(100, 24)).unwrap();
    let mut snapshot = FrameSnapshot::default();
    terminal
        .draw(|frame| snapshot = crate::ui::render(frame, &mut state, &config))
        .unwrap();
    let header = snapshot.layout.sidebar_toggle.unwrap();
    let footer = snapshot
        .layout
        .footer_buttons
        .iter()
        .find(|button| button.target == HoverTarget::FooterSidebarToggle)
        .unwrap()
        .rect;
    for (target, area) in [
        (HoverTarget::SidebarToggle, header),
        (HoverTarget::FooterSidebarToggle, footer),
    ] {
        send_mouse(&mut state, &snapshot, Moved, area.x, area.y);
        assert_eq!(state.ui.hover, Some(target));
        terminal
            .draw(|frame| snapshot = crate::ui::render(frame, &mut state, &config))
            .unwrap();
        let buffer = terminal.backend().buffer();
        assert_eq!(
            buffer[(header.x + 1, header.y)].fg,
            if target == HoverTarget::SidebarToggle {
                config.theme.accent
            } else {
                config.theme.metadata
            }
        );
        assert_eq!(
            buffer[(header.x + 1, header.y)].bg,
            config.theme.workspace_bg
        );
        assert_eq!(
            buffer[(footer.x, footer.y)].fg,
            if target == HoverTarget::FooterSidebarToggle {
                config.theme.metadata
            } else {
                config.theme.dim
            }
        );
    }
    send_mouse(&mut state, &snapshot, Down(Left), footer.x, footer.y);
    assert!(state.ui.sidebar.collapsed);
}

#[test]
fn button_free_motion_recovers_from_a_missing_release_without_resizing() {
    for target in [
        HoverTarget::Run,
        HoverTarget::SidebarToggle,
        HoverTarget::SidebarDivider,
    ] {
        let mut state = AppState::from_command(&Command::new("tool"));
        let before = draw(&mut state, 80, 24);
        let divider = before.layout.sidebar_divider.unwrap();
        send_mouse(&mut state, &before, Down(Left), divider.x, 4);
        send_mouse(&mut state, &before, Drag(Left), divider.x + 4, 4);
        let dragged = draw(&mut state, 80, 24);
        let width = dragged.layout.sidebar.unwrap().width;
        assert_eq!(dragged.mouse_pointer, MousePointer::ResizeHorizontal);
        let area = match target {
            HoverTarget::Run => {
                dragged
                    .layout
                    .footer_buttons
                    .iter()
                    .find(|button| button.target == target)
                    .unwrap()
                    .rect
            }
            HoverTarget::SidebarToggle => dragged.layout.sidebar_toggle.unwrap(),
            HoverTarget::SidebarDivider => dragged.layout.sidebar_divider.unwrap(),
            _ => unreachable!(),
        };
        send_mouse(&mut state, &dragged, Moved, area.x, area.y);
        assert!(!state.ui.sidebar.is_resizing());
        assert_eq!(state.ui.hover, Some(target));
        let recovered = draw(&mut state, 80, 24);
        assert_eq!(recovered.layout.sidebar.unwrap().width, width);
        assert_eq!(
            recovered.mouse_pointer,
            if target == HoverTarget::SidebarDivider {
                MousePointer::ResizeHorizontal
            } else {
                MousePointer::PointingHand
            }
        );
        let button = recovered.layout.sidebar_toggle.unwrap();
        send_mouse(&mut state, &recovered, Down(Left), button.x, button.y);
        assert!(
            state.ui.sidebar.collapsed,
            "normal clicks resume after capture ends"
        );
    }
}

#[test]
fn rendered_pointer_matches_divider_interaction_during_help_and_dragging() {
    let mut state = AppState::from_command(&Command::new("tool"));
    let before = draw(&mut state, 80, 24);
    let divider = before.layout.sidebar_divider.unwrap();
    send_mouse(&mut state, &before, Moved, divider.x, 4);
    event(
        &mut state,
        &before,
        &AppEvent::Key(AppKeyEvent::new(
            AppKeyCode::Char('?'),
            AppKeyModifiers::default(),
        )),
    );
    assert!(state.ui.help_open);
    let help = draw(&mut state, 80, 24);
    assert_eq!(help.mouse_pointer, MousePointer::ResizeHorizontal);
    send_mouse(&mut state, &help, Down(Left), divider.x, 4);
    state.ui.hover = None;
    send_mouse(&mut state, &help, Drag(Left), divider.x + 4, 4);
    let dragged = draw(&mut state, 80, 24);
    assert_eq!(
        dragged.layout.sidebar.unwrap().width,
        before.layout.sidebar.unwrap().width + 4
    );
    assert_eq!(dragged.mouse_pointer, MousePointer::ResizeHorizontal);
    send_mouse(&mut state, &dragged, Up(Left), divider.x + 4, 4);
    assert_eq!(
        draw(&mut state, 80, 24).mouse_pointer,
        MousePointer::ResizeHorizontal
    );
    state.ui.hover = Some(HoverTarget::SidebarDivider);
    state.ui.toggle_sidebar();
    assert_eq!(
        draw(&mut state, 80, 24).mouse_pointer,
        MousePointer::Default
    );
    state.ui.preview_open = true;
    state.ui.hover = Some(HoverTarget::SidebarDivider);
    assert_eq!(
        draw(&mut state, 80, 24).mouse_pointer,
        MousePointer::PointingHand
    );
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
        let button = before.layout.sidebar_toggle.unwrap();
        assert_eq!(button.width, 3);
        assert_eq!(button.x, before.layout.form.unwrap().x);
        let original = before.layout.sidebar.unwrap().width;
        send_mouse(&mut state, &before, Down(Left), divider.x, divider.y + 2);
        assert!(state.ui.sidebar.is_resizing());
        assert!(state.ui.mouse_select.is_none() && state.ui.dropdown_open.is_none());
        send_mouse(
            &mut state,
            &before,
            Drag(Left),
            divider.x + 7,
            divider.y + 2,
        );
        let resized = draw(&mut state, width, height);
        assert_eq!(resized.layout.sidebar.unwrap().width, original + 7);
        // Release over the footer, outside the divider; the gesture still ends.
        send_mouse(&mut state, &resized, Up(Left), divider.x + 9, height - 1);
        assert!(!state.ui.sidebar.is_resizing());
        assert_eq!(state.ui.focus, Focus::Form);
        let resized = draw(&mut state, width, height);
        assert_eq!(resized.layout.sidebar.unwrap().width, original + 9);
        event(&mut state, &resized, &toggle());
        let hidden = draw(&mut state, width, height);
        assert!(hidden.layout.sidebar.is_none() && hidden.layout.search.is_none());
        assert!(hidden.layout.sidebar_items.is_empty());
        assert_eq!(hidden.layout.form.unwrap().width, width - 2);
        assert!(hidden.layout.sidebar_divider.is_none());
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
        assert_eq!(control.x, hidden.layout.form.unwrap().x);
        send_mouse(
            &mut state,
            &hidden,
            Down(Left),
            control.right() - 1,
            control.y,
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
    send_mouse(&mut state, &before, Down(Left), divider.x, 2);
    send_mouse(&mut state, &before, Drag(Left), 0, 2);
    assert_eq!(draw(&mut state, 100, 24).layout.sidebar.unwrap().width, 16);
    send_mouse(&mut state, &before, Up(Left), u16::MAX, 2);
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
    send_mouse(&mut state, &hidden, Down(Left), search.x, search.y);
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
        AppEvent::Mouse(AppMouseEvent {
            kind: Drag(Left),
            column: 20,
            row: 2,
            modifiers: AppKeyModifiers::default(),
        }),
    ] {
        assert_eq!(
            controller::handle_event(&input, &state, &preview, &TuiConfig::default()),
            controller::EventDecision::Ignore
        );
        assert!(state.ui.sidebar.collapsed);
    }
}
