use crossterm::event::{self, MouseButton, MouseEventKind};

use crate::config::TuiConfig;
use crate::frame_snapshot::FrameSnapshot;
use crate::input::AppState;
use crate::update::Action;

pub(crate) fn handle_mouse_event(
    event: event::MouseEvent,
    state: &AppState,
    frame_snapshot: &FrameSnapshot,
    _config: &TuiConfig,
) -> Option<Action> {
    if let MouseEventKind::Moved = event.kind {
        return state.ui.mouse_select.as_ref().map_or(
            Some(Action::UpdateHover {
                x: event.column,
                y: event.row,
            }),
            |_| Some(Action::UpdateMouseSelection(event)),
        );
    }
    if let MouseEventKind::Drag(MouseButton::Left) = event.kind
        && state.ui.mouse_select.is_some()
    {
        return Some(Action::UpdateMouseSelection(event));
    }
    if let MouseEventKind::Up(MouseButton::Left) = event.kind {
        return Some(Action::ClearMouseSelection);
    }
    if let MouseEventKind::Down(MouseButton::Left) = event.kind {
        if state.ui.dropdown_open.is_some() && frame_snapshot.dropdown_visible_rows().is_some() {
            if frame_snapshot.dropdown_contains(event.column, event.row) {
                return state
                    .ui
                    .dropdown_open
                    .clone()
                    .map(|arg_id| Action::ClickDropdownChoice {
                        arg_id,
                        row: event.row,
                    });
            } else {
                return Some(Action::CloseDropdown);
            }
        }
        if let Some(target) = frame_snapshot.footer_target_at(event.column, event.row) {
            return Some(Action::ClickFooter(target));
        }
        if frame_snapshot.preview_contains(event.column, event.row) {
            return Some(Action::CopyPreview);
        }
        if frame_snapshot.search_contains(event.column, event.row) {
            return Some(Action::FocusSearch);
        }
        if frame_snapshot.sidebar_contains(event.column, event.row) {
            return Some(Action::ClickSidebar {
                x: event.column,
                y: event.row,
            });
        }
        if let Some(tab) = frame_snapshot.tab_at(event.column, event.row) {
            return Some(Action::SwitchTab(tab));
        }
        if frame_snapshot.form_contains(event.column, event.row) {
            return Some(Action::ClickForm(event));
        }
    }
    if frame_snapshot.dropdown_contains(event.column, event.row) && state.ui.dropdown_open.is_some() {
        match event.kind {
            MouseEventKind::ScrollDown => return Some(Action::ScrollDropdown(1)),
            MouseEventKind::ScrollUp => return Some(Action::ScrollDropdown(-1)),
            _ => {}
        }
    }
    if let MouseEventKind::ScrollDown = event.kind {
        return Some(Action::ScrollForm(2));
    }
    if let MouseEventKind::ScrollUp = event.kind {
        return Some(Action::ScrollForm(-2));
    }
    None
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
    use ratatui::layout::Rect;

    use super::handle_mouse_event;
    use crate::config::TuiConfig;
    use crate::frame_snapshot::{
        FooterButtonLayout, FrameSnapshot, SidebarItemLayout, TabButtonLayout,
    };
    use crate::input::{ActiveTab, AppState, Focus, HoverTarget};
    use crate::spec::{ArgKind, ArgSpec, CommandSpec, ValueCardinality};
    use crate::update::{Effect, apply_action};

    fn arg(id: &str, name: &str, kind: ArgKind) -> ArgSpec {
        ArgSpec {
            id: id.to_string(),
            display_name: name.to_string(),
            help: None,
            required: false,
            kind,
            default_values: Vec::new(),
            choices: Vec::new(),
            position: None,
            value_cardinality: ValueCardinality::One,
            value_hint: None,
        }
    }

    fn command(args: Vec<ArgSpec>) -> CommandSpec {
        CommandSpec {
            name: "tool".to_string(),
            about: None,
            help: String::new(),
            args,
            subcommands: Vec::new(),
        }
    }

    fn click(column: u16, row: u16) -> MouseEvent {
        MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column,
            row,
            modifiers: KeyModifiers::empty(),
        }
    }

    fn scroll(kind: MouseEventKind, column: u16, row: u16) -> MouseEvent {
        MouseEvent {
            kind,
            column,
            row,
            modifiers: KeyModifiers::empty(),
        }
    }

    #[test]
    fn flag_description_click_selects_without_toggling() {
        let mut verbose = arg("verbose", "--verbose", ArgKind::Flag);
        verbose.help = Some("Enable verbose output".to_string());
        let mut state = AppState::new(command(vec![verbose]));
        let mut frame_snapshot = FrameSnapshot::default();
        state.ui.active_tab = ActiveTab::Options;
        frame_snapshot.layout.form = Some(Rect::new(0, 0, 30, 10));
        frame_snapshot.layout.form_view = Some(Rect::new(0, 0, 30, 10));

        let action = handle_mouse_event(click(1, 1), &state, &frame_snapshot, &TuiConfig::default())
            .expect("click action");
        let effect = apply_action(action, &mut state, &frame_snapshot);
        assert_eq!(effect, Effect::None);

        assert_eq!(state.ui.selected_arg_index, 0);
        assert!(matches!(state.ui.focus, Focus::Form));
        assert!(!state.domain.is_touched("verbose"));
        assert_eq!(
            state
                .domain
                .current_form()
                .and_then(|inputs| inputs.values.get("verbose")),
            Some(&crate::input::ArgValue::Bool(false))
        );

        let action = handle_mouse_event(click(1, 0), &state, &frame_snapshot, &TuiConfig::default())
            .expect("toggle action");
        let effect = apply_action(action, &mut state, &frame_snapshot);
        assert_eq!(effect, Effect::None);

        assert!(state.domain.is_touched("verbose"));
    }

    #[test]
    fn footer_click_runs_or_focuses_search_based_on_button_hit() {
        let mut state = AppState::new(command(Vec::new()));
        let mut frame_snapshot = FrameSnapshot::default();
        frame_snapshot.layout.footer = Some(Rect::new(0, 20, 40, 1));
        frame_snapshot.layout.footer_buttons = vec![
            FooterButtonLayout {
                target: HoverTarget::Run,
                rect: Rect::new(0, 20, 8, 1),
            },
            FooterButtonLayout {
                target: HoverTarget::Search,
                rect: Rect::new(10, 20, 8, 1),
            },
        ];

        let run_action =
            handle_mouse_event(click(1, 20), &state, &frame_snapshot, &TuiConfig::default())
                .expect("run action");
        let run_effect = apply_action(run_action, &mut state, &frame_snapshot);
        assert_eq!(run_effect, Effect::Run(vec!["tool".to_string()]));

        let search_action = handle_mouse_event(
            click(11, 20),
            &state,
            &frame_snapshot,
            &TuiConfig::default(),
        )
        .expect("search action");
        let search_effect = apply_action(search_action, &mut state, &frame_snapshot);
        assert_eq!(search_effect, Effect::None);
        assert!(matches!(state.ui.focus, Focus::Search));
    }

    #[test]
    fn tab_click_switches_active_tab() {
        let mut state = AppState::new(command(Vec::new()));
        let mut frame_snapshot = FrameSnapshot::default();
        state.ui.active_tab = ActiveTab::Options;
        frame_snapshot.layout.form_tabs = vec![TabButtonLayout {
            tab: ActiveTab::Help,
            rect: Rect::new(0, 0, 8, 1),
        }];

        let action =
            handle_mouse_event(click(1, 0), &state, &frame_snapshot, &TuiConfig::default())
                .expect("tab action");
        let effect = apply_action(action, &mut state, &frame_snapshot);

        assert_eq!(effect, Effect::None);
        assert_eq!(state.ui.active_tab, ActiveTab::Help);
    }

    #[test]
    fn dropdown_click_selects_choice_and_scroll_hits_dropdown_only() {
        let mut color = arg("color", "--color", ArgKind::Enum);
        color.choices = vec!["red".to_string(), "green".to_string(), "blue".to_string()];
        let mut state = AppState::new(command(vec![color]));
        let mut frame_snapshot = FrameSnapshot::default();
        state.ui.dropdown_open = Some("color".to_string());
        state.ui.dropdown_scroll = 1;
        frame_snapshot.layout.dropdown = Some(Rect::new(0, 5, 20, 5));

        let action =
            handle_mouse_event(click(1, 7), &state, &frame_snapshot, &TuiConfig::default())
                .expect("dropdown action");
        let effect = apply_action(action, &mut state, &frame_snapshot);

        assert_eq!(effect, Effect::None);
        assert_eq!(
            state
                .domain
                .current_form()
                .and_then(|inputs| inputs.values.get("color")),
            Some(&crate::input::ArgValue::Choice("blue".to_string()))
        );
        assert!(state.domain.is_touched("color"));
        assert!(state.ui.dropdown_open.is_none());
    }

    #[test]
    fn dropdown_scroll_events_adjust_dropdown_scroll_without_touching_form_scroll() {
        let mut color = arg("color", "--color", ArgKind::Enum);
        color.choices = (0..8).map(|index| format!("choice-{index}")).collect();
        let mut state = AppState::new(command(vec![color]));
        let mut frame_snapshot = FrameSnapshot::default();
        state.ui.dropdown_open = Some("color".to_string());
        frame_snapshot.layout.dropdown = Some(Rect::new(0, 5, 20, 5));
        state.ui.form_scroll = 4;

        let action = handle_mouse_event(
            scroll(MouseEventKind::ScrollDown, 1, 6),
            &state,
            &frame_snapshot,
            &TuiConfig::default(),
        )
        .expect("scroll action");
        let effect = apply_action(action, &mut state, &frame_snapshot);

        assert_eq!(effect, Effect::None);
        assert_eq!(state.ui.dropdown_scroll, 1);
        assert_eq!(state.ui.form_scroll, 4);
    }

    #[test]
    fn sidebar_caret_click_selects_command_and_expands_item() {
        let mut state = AppState::new(CommandSpec {
            name: "tool".to_string(),
            about: None,
            help: String::new(),
            args: Vec::new(),
            subcommands: vec![CommandSpec {
                name: "build".to_string(),
                about: None,
                help: String::new(),
                args: Vec::new(),
                subcommands: vec![CommandSpec {
                    name: "release".to_string(),
                    about: None,
                    help: String::new(),
                    args: Vec::new(),
                    subcommands: Vec::new(),
                }],
            }],
        });
        state
            .domain
            .select_command_path(&["build".to_string()])
            .expect("valid path");
        state.domain.expanded.remove("tool::build");
        let mut frame_snapshot = FrameSnapshot::default();
        frame_snapshot.layout.sidebar = Some(Rect::new(0, 0, 20, 10));
        frame_snapshot.layout.sidebar_items = vec![SidebarItemLayout {
            path: vec!["build".to_string()].into(),
            row: Rect::new(0, 1, 20, 1),
            caret: Some(Rect::new(2, 1, 1, 1)),
            has_children: true,
        }];

        let action =
            handle_mouse_event(click(2, 1), &state, &frame_snapshot, &TuiConfig::default())
                .expect("sidebar action");
        let effect = apply_action(action, &mut state, &frame_snapshot);

        assert_eq!(effect, Effect::None);
        assert_eq!(state.domain.selected_path().as_slice(), &["build".to_string()]);
        assert!(state.domain.expanded.contains("tool::build"));
        assert!(matches!(state.ui.focus, Focus::Sidebar));
    }
}
