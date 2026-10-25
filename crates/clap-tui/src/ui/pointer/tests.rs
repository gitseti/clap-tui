use clap::{Arg, ArgAction, Command};
use ratatui::{
    Terminal,
    backend::TestBackend,
    layout::{Position, Rect},
};

use crate::{
    TuiConfig, controller,
    frame_snapshot::FrameSnapshot,
    input::{AppState, HoverTarget},
    layout::form::FormFieldLayout,
    runtime::{
        AppEvent, AppKeyCode, AppKeyEvent, AppKeyModifiers,
        AppMouseButton::Left,
        AppMouseEvent,
        AppMouseEventKind::{self, Down, Drag, Moved, Up},
        MousePointer::{
            self, Default as Normal, PointingHand as Hand, ResizeHorizontal as Resize, Text,
        },
    },
    update::{Effect, apply_action},
};

struct Harness {
    state: AppState,
    terminal: Terminal<TestBackend>,
    snapshot: FrameSnapshot,
}

impl Harness {
    fn new(command: &Command, width: u16, height: u16) -> Self {
        let mut ui = Self {
            state: AppState::from_command(command),
            terminal: Terminal::new(TestBackend::new(width, height)).unwrap(),
            snapshot: FrameSnapshot::default(),
        };
        ui.draw();
        ui
    }

    fn draw(&mut self) {
        self.terminal
            .draw(|frame| {
                self.snapshot = crate::ui::render(frame, &mut self.state, &TuiConfig::default());
            })
            .unwrap();
    }

    fn send(&mut self, event: &AppEvent) {
        self.state.ui.observe_pointer(event);
        if let controller::EventDecision::Action(action) =
            controller::handle_event(event, &self.state, &self.snapshot, &TuiConfig::default())
        {
            assert_eq!(
                apply_action(&action, &mut self.state, &self.snapshot),
                Effect::None
            );
        }
        self.draw();
    }

    fn mouse(&mut self, kind: AppMouseEventKind, point: Position) {
        self.send(&AppEvent::Mouse(AppMouseEvent {
            kind,
            column: point.x,
            row: point.y,
            modifiers: AppKeyModifiers::default(),
        }));
    }

    fn hover(&mut self, point: Position, pointer: MousePointer) {
        let focus = self.state.ui.focus.clone();
        let path = self.state.domain.selected_path().clone();
        let argv = self.state.authoritative_argv();
        let validation = self.state.derived_validation();
        self.mouse(Moved, point);
        assert_eq!(self.snapshot.mouse_pointer, pointer, "at {point:?}");
        assert_eq!(self.state.ui.focus, focus);
        assert_eq!(*self.state.domain.selected_path(), path);
        assert_eq!(self.state.authoritative_argv(), argv);
        assert_eq!(self.state.derived_validation(), validation);
    }

    fn field(&self, id: &str) -> FormFieldLayout {
        self.snapshot.form_field_layout(id).unwrap().clone()
    }
}

fn content(rect: Rect) -> Position {
    Position::new(rect.x + 1, rect.y + 1)
}

fn command() -> Command {
    Command::new("tool")
        .arg(Arg::new("name").long("name").help("A name"))
        .arg(Arg::new("flag").long("flag").action(ArgAction::SetTrue))
        .arg(Arg::new("count").long("count").action(ArgAction::Count))
        .arg(Arg::new("choice").long("choice").value_parser(["a", "b"]))
        .arg(
            Arg::new("multi")
                .long("multi")
                .value_parser(["a", "b"])
                .action(ArgAction::Append),
        )
        .arg(Arg::new("items").long("items").action(ArgAction::Append))
        .arg(
            Arg::new("maybe")
                .long("maybe")
                .num_args(0..=1)
                .default_missing_value("auto"),
        )
        .subcommand(Command::new("child"))
}

#[test]
fn every_widget_uses_its_actionable_hit_regions_without_hover_side_effects() {
    let mut ui = Harness::new(&command(), 100, 80);
    for (id, expected) in [
        ("name", Text),
        ("flag", Hand),
        ("choice", Hand),
        ("multi", Hand),
        ("items", Text),
        ("maybe", Hand),
    ] {
        ui.hover(content(ui.field(id).input), expected);
    }
    let field = ui.field("name");
    ui.hover(Position::new(field.input.x, field.input.y), Normal);
    ui.hover(field.label.unwrap().as_position(), Normal);
    ui.hover(field.description.unwrap().as_position(), Normal);
    let counter = ui.field("count").input;
    ui.hover(content(counter), Normal);
    ui.hover(Position::new(counter.right() - 6, counter.y + 1), Hand);
    ui.hover(Position::new(counter.right() - 2, counter.y + 1), Hand);
    ui.hover(Position::new(counter.right() - 2, counter.y), Normal);
}

#[test]
fn navigation_shortcuts_preview_and_actionable_errors_are_hands() {
    for (width, height) in [(80, 24), (60, 20)] {
        let mut ui = Harness::new(&command(), width, height);
        let header = ui.snapshot.layout.sidebar_toggle.unwrap();
        ui.hover(header.as_position(), Hand);
        let sidebar = ui.snapshot.layout.sidebar.unwrap();
        ui.hover(sidebar.as_position(), Hand);
        let item = ui.snapshot.layout.sidebar_items[0].row;
        ui.hover(item.as_position(), Hand);
        let search = ui.snapshot.layout.search.unwrap();
        ui.hover(content(search), Text);
        ui.hover(search.as_position(), Normal);
        let buttons = ui.snapshot.layout.footer_buttons.clone();
        for button in buttons {
            ui.hover(button.rect.as_position(), Hand);
        }
        let preview = ui.snapshot.layout.preview.unwrap();
        ui.hover(preview.as_position(), Hand);
        ui.hover(Position::new(sidebar.x + 1, sidebar.bottom() - 1), Normal);
    }
    let cmd = Command::new("tool").arg(Arg::new("required").long("required").required(true));
    let mut ui = Harness::new(&cmd, 100, 24);
    let error = ui
        .snapshot
        .layout
        .footer_buttons
        .iter()
        .find(|button| button.target == HoverTarget::FooterStatus)
        .unwrap()
        .rect;
    ui.hover(error.as_position(), Hand);
    ui.hover(content(ui.field("required").input), Text);
}

#[test]
fn repeated_row_buttons_and_optional_value_modes_follow_current_state() {
    let mut ui = Harness::new(&command(), 100, 80);
    let repeated = ui.field("items").input;
    ui.hover(Position::new(repeated.right() - 7, repeated.y + 1), Normal);
    let add = Position::new(repeated.right() - 3, repeated.y + 1);
    ui.hover(add, Hand);
    ui.mouse(Down(Left), add);
    ui.mouse(Up(Left), add);
    assert_eq!(ui.field("items").full_input_height, 6);
    let repeated = ui.field("items").input;
    ui.hover(Position::new(repeated.right() - 5, repeated.y + 1), Hand);
    ui.hover(Position::new(repeated.x + 1, repeated.y + 4), Text);
    let maybe = content(ui.field("maybe").input);
    ui.mouse(Down(Left), maybe);
    ui.mouse(Up(Left), maybe);
    ui.hover(maybe, Hand);
    ui.send(&AppEvent::Paste("explicit".into()));
    assert_eq!(ui.snapshot.mouse_pointer, Text);
    ui.hover(maybe, Text);
}

#[test]
fn inherited_disabled_fields_are_normal_and_authored_values_stay_editable() {
    let cmd = Command::new("tool")
        .args_conflicts_with_subcommands(true)
        .arg(Arg::new("name").long("name"))
        .subcommand(Command::new("child"));
    let mut ui = Harness::new(&cmd, 100, 24);
    ui.state.select_command_path(&["child".into()]).unwrap();
    ui.draw();
    ui.hover(content(ui.field("name").input), Normal);
    ui.state.select_command_path(&[]).unwrap();
    ui.state.domain.set_text_value("name", "keep");
    ui.state.select_command_path(&["child".into()]).unwrap();
    ui.draw();
    ui.hover(content(ui.field("name").input), Text);
}

#[test]
fn popup_rows_override_underlying_fields_and_closing_recomputes_stationary_pointer() {
    let cmd = Command::new("tool")
        .arg(Arg::new("choice").long("choice").value_parser(["a", "b"]))
        .arg(Arg::new("name").long("name"));
    let mut ui = Harness::new(&cmd, 100, 24);
    let choice = content(ui.field("choice").input);
    ui.mouse(Down(Left), choice);
    ui.mouse(Up(Left), choice);
    let popup = ui.snapshot.layout.dropdown.unwrap();
    ui.hover(content(popup), Hand);
    ui.hover(popup.as_position(), Normal);
    ui.hover(Position::new(popup.x + 1, popup.bottom() - 1), Normal);
    let point = content(popup);
    ui.hover(point, Hand);
    ui.send(&AppEvent::Key(AppKeyEvent::new(
        AppKeyCode::Esc,
        AppKeyModifiers::default(),
    )));
    assert!(ui.snapshot.layout.dropdown.is_none());
    // The same coordinates are now over the following text field's border.
    assert_ne!(ui.snapshot.mouse_pointer, Hand);
}

#[test]
fn help_and_expanded_preview_respect_visible_interaction_and_track_motion() {
    let mut ui = Harness::new(&command(), 80, 24);
    let text = content(ui.field("name").input);
    ui.hover(text, Text);
    ui.state.ui.toggle_help();
    ui.draw();
    assert_eq!(ui.snapshot.mouse_pointer, Normal);
    let divider = ui.snapshot.layout.sidebar_divider.unwrap();
    ui.hover(Position::new(divider.x, divider.y + 3), Resize);
    ui.state.ui.preview_open = true;
    ui.state.ui.dismiss_transient_interaction();
    ui.draw();
    assert_eq!(ui.snapshot.mouse_pointer, Hand);
    ui.hover(Position::new(2, 12), Hand);
    ui.send(&AppEvent::Key(AppKeyEvent::new(
        AppKeyCode::Esc,
        AppKeyModifiers::default(),
    )));
    assert!(!ui.state.ui.preview_open);
    assert_eq!(ui.state.ui.mouse_position, Some(Position::new(2, 12)));
    assert_eq!(ui.snapshot.mouse_pointer, Normal);
}

#[test]
fn text_selection_keeps_ibeam_during_drag_and_recovers_after_missing_release() {
    let mut ui = Harness::new(&command(), 80, 24);
    ui.state.domain.set_text_value("name", "select me");
    ui.draw();
    let text = content(ui.field("name").input);
    let copy = Position::new(text.x + 4, ui.snapshot.layout.preview.unwrap().y);
    let owner = ui
        .state
        .domain
        .arg_for_input("name")
        .unwrap()
        .owner_path()
        .clone();
    for release in [Moved, Up(Left)] {
        ui.mouse(Down(Left), text);
        ui.mouse(Drag(Left), copy);
        assert_eq!(ui.snapshot.mouse_pointer, Text);
        assert!(ui.state.ui.mouse_select.is_some());
        let selection = ui.state.ui.editors.editor(&owner, "name").unwrap().clone();
        assert!(selection.selection_anchor().is_some());
        ui.mouse(release, copy);
        assert!(ui.state.ui.mouse_select.is_none());
        assert_eq!(ui.snapshot.mouse_pointer, Hand);
        assert_eq!(ui.state.ui.editors.editor(&owner, "name"), Some(&selection));
    }
    ui.hover(text, Text);
    ui.mouse(Down(Left), text);
    ui.send(&AppEvent::FocusLost);
    assert!(ui.state.ui.mouse_position.is_none());
    assert!(ui.state.ui.mouse_select.is_none());
    assert_eq!(ui.snapshot.mouse_pointer, Normal);
}

#[test]
fn stationary_pointer_is_recomputed_after_scroll_resize_and_command_navigation() {
    let mut ui = Harness::new(&command(), 80, 24);
    let text = content(ui.field("name").input);
    ui.hover(text, Text);
    ui.state.ui.form_scroll = 3;
    ui.draw();
    assert_ne!(ui.snapshot.mouse_pointer, Text);
    ui.state.ui.form_scroll = 0;
    ui.draw();
    assert_eq!(ui.snapshot.mouse_pointer, Text);
    ui.hover(
        Position::new(ui.field("name").input.right() - 2, text.y),
        Text,
    );
    ui.terminal.backend_mut().resize(60, 20);
    ui.send(&AppEvent::Resize {
        width: 60,
        height: 20,
    });
    assert_eq!(ui.snapshot.mouse_pointer, Normal);
    ui.hover(content(ui.field("name").input), Text);
    let cmd =
        Command::new("tool").subcommand(Command::new("child").arg(Arg::new("name").long("name")));
    let mut ui = Harness::new(&cmd, 80, 24);
    ui.state.select_command_path(&["child".into()]).unwrap();
    ui.draw();
    let point = content(ui.field("name").input);
    ui.state.select_command_path(&[]).unwrap();
    ui.draw();
    ui.hover(point, Normal);
    ui.state.select_command_path(&["child".into()]).unwrap();
    ui.draw();
    assert_eq!(ui.snapshot.mouse_pointer, Text);
}

#[test]
fn clipped_input_border_is_normal_and_visible_content_remains_editable() {
    let mut ui = Harness::new(&command(), 60, 20);
    ui.state.ui.form_scroll = 1;
    ui.draw();
    let text = ui.field("name");
    assert_eq!(text.input_clip_top, 1);
    ui.hover(Position::new(text.input.x + 1, text.input.y), Text);
    ui.state.ui.form_scroll = 2;
    ui.draw();
    let text = ui.field("name");
    ui.hover(Position::new(text.input.x + 1, text.input.y), Normal);
}

#[test]
fn diagnostic_without_a_jump_target_does_not_advertise_a_button() {
    use super::super::{layout::build_screen_layout, screen::ScreenView};
    use crate::pipeline::{ValidationIssue, ValidationState};

    let mut state = AppState::from_command(&Command::new("tool"));
    state.ui.mouse_position = Some(Position::new(30, 23));
    let derived = state.derived().clone();
    let mut view = ScreenView::from_state(&state, derived);
    view.validation = ValidationState {
        issues: vec![ValidationIssue::arguments("Opaque failure", Vec::new())],
    };
    let layout = build_screen_layout(
        &state.ui,
        &TuiConfig::default(),
        Rect::new(0, 0, 100, 24),
        &view,
    );
    assert_eq!(
        layout.snapshot.footer_target_at(30, 23),
        Some(HoverTarget::FooterStatus)
    );
    assert_eq!(super::resolve(&state.ui, &layout.snapshot, &view), Normal);
}
