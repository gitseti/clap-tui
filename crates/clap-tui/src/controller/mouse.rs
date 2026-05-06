use crate::config::TuiConfig;
use crate::frame_snapshot::FrameSnapshot;
use crate::input::AppState;
use crate::runtime::{AppMouseButton, AppMouseEvent, AppMouseEventKind};
use crate::update::Action;

pub(crate) fn handle_mouse_event(
    event: AppMouseEvent,
    state: &AppState,
    frame_snapshot: &FrameSnapshot,
    _config: &TuiConfig,
) -> Option<Action> {
    if let AppMouseEventKind::Moved = event.kind {
        return state.ui.mouse_select.as_ref().map_or(
            Some(Action::UpdateHover {
                x: event.column,
                y: event.row,
            }),
            |_| Some(Action::UpdateMouseSelection(event)),
        );
    }
    if let AppMouseEventKind::Drag(AppMouseButton::Left) = event.kind
        && state.ui.mouse_select.is_some()
    {
        return Some(Action::UpdateMouseSelection(event));
    }
    if let AppMouseEventKind::Up(AppMouseButton::Left) = event.kind {
        return Some(Action::ClearMouseSelection);
    }
    if let AppMouseEventKind::Down(AppMouseButton::Left) = event.kind {
        let dropdown_open =
            state.ui.dropdown_open.is_some() && frame_snapshot.dropdown_visible_rows().is_some();
        if dropdown_open && frame_snapshot.dropdown_contains(event.column, event.row) {
            return state
                .ui
                .dropdown_open
                .clone()
                .map(|arg_id| Action::ClickDropdownChoice {
                    arg_id,
                    row: event.row,
                });
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
        if state.ui.help_open && frame_snapshot.form_contains(event.column, event.row) {
            return Some(Action::ToggleHelp);
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
        if dropdown_open {
            return Some(Action::CloseDropdown);
        }
    }
    if frame_snapshot.dropdown_contains(event.column, event.row) && state.ui.dropdown_open.is_some()
    {
        match event.kind {
            AppMouseEventKind::ScrollDown => return Some(Action::ScrollDropdown(1)),
            AppMouseEventKind::ScrollUp => return Some(Action::ScrollDropdown(-1)),
            _ => {}
        }
    }
    if state.ui.help_open && frame_snapshot.form_contains(event.column, event.row) {
        match event.kind {
            AppMouseEventKind::ScrollDown => return Some(Action::ScrollForm(2)),
            AppMouseEventKind::ScrollUp => return Some(Action::ScrollForm(-2)),
            _ => {}
        }
    }
    if state.ui.help_open {
        return None;
    }
    if frame_snapshot.sidebar_contains(event.column, event.row) {
        match event.kind {
            AppMouseEventKind::ScrollDown => return Some(Action::ScrollSidebar(2)),
            AppMouseEventKind::ScrollUp => return Some(Action::ScrollSidebar(-2)),
            _ => {}
        }
    }
    if let AppMouseEventKind::ScrollDown = event.kind {
        return Some(Action::ScrollForm(2));
    }
    if let AppMouseEventKind::ScrollUp = event.kind {
        return Some(Action::ScrollForm(-2));
    }
    None
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;

    use ratatui::layout::Rect;

    use super::handle_mouse_event;
    use crate::config::TuiConfig;
    use crate::frame_snapshot::{
        FooterButtonLayout, FrameSnapshot, SidebarItemLayout, TabButtonLayout,
    };
    use crate::input::{ActiveTab, AppState, Focus, HoverTarget};
    use crate::runtime::{AppKeyModifiers, AppMouseButton, AppMouseEvent, AppMouseEventKind};
    use crate::spec::{ArgKind, ArgSpec, CommandSpec, ValueCardinality};
    use crate::update::{Action, Effect, apply_action};

    fn os_vec(values: &[&str]) -> Vec<OsString> {
        values.iter().map(OsString::from).collect()
    }

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
            ..ArgSpec::default()
        }
    }

    fn command(args: Vec<ArgSpec>) -> CommandSpec {
        CommandSpec {
            name: "tool".to_string(),
            version: None,
            about: None,
            help: String::new(),
            args,
            subcommands: Vec::new(),
            ..CommandSpec::default()
        }
    }

    fn click(column: u16, row: u16) -> AppMouseEvent {
        AppMouseEvent {
            kind: AppMouseEventKind::Down(AppMouseButton::Left),
            column,
            row,
            modifiers: AppKeyModifiers::default(),
        }
    }

    fn scroll(kind: AppMouseEventKind, column: u16, row: u16) -> AppMouseEvent {
        AppMouseEvent {
            kind,
            column,
            row,
            modifiers: AppKeyModifiers::default(),
        }
    }

    #[test]
    fn flag_description_click_selects_without_toggling() {
        let mut verbose = arg("verbose", "--verbose", ArgKind::Flag);
        verbose.help = Some("Enable verbose output".to_string());
        let mut state = AppState::new(command(vec![verbose]));
        let mut frame_snapshot = FrameSnapshot::default();
        state.ui.active_tab = ActiveTab::Inputs;
        frame_snapshot.layout.form = Some(Rect::new(0, 0, 30, 10));
        frame_snapshot.layout.form_view = Some(Rect::new(0, 0, 30, 10));
        frame_snapshot
            .layout
            .form_fields
            .push(crate::layout::form::FormFieldLayout {
                arg_id: "verbose".to_string(),
                heading: None,
                label: Some(Rect::new(0, 0, 12, 1)),
                input: Rect::new(13, 0, 12, 3),
                input_clip_top: 0,
                description: Some(Rect::new(13, 3, 12, 1)),
            });

        let action =
            handle_mouse_event(click(1, 0), &state, &frame_snapshot, &TuiConfig::default())
                .expect("click action");
        let effect = apply_action(&action, &mut state, &frame_snapshot);
        assert_eq!(effect, Effect::None);

        assert_eq!(state.ui.selected_arg_index, 0);
        assert!(matches!(state.ui.focus, Focus::Form));
        assert!(!state.domain.is_touched("verbose"));
        assert_eq!(
            state.domain.current_form().and_then(|inputs| {
                state
                    .domain
                    .current_command()
                    .args
                    .iter()
                    .find(|arg| arg.id == "verbose")
                    .and_then(|arg| inputs.compatibility_value(arg))
            }),
            Some(crate::input::ArgValue::Bool(false))
        );

        let action =
            handle_mouse_event(click(14, 1), &state, &frame_snapshot, &TuiConfig::default())
                .expect("toggle action");
        let effect = apply_action(&action, &mut state, &frame_snapshot);
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
        let run_effect = apply_action(&run_action, &mut state, &frame_snapshot);
        assert_eq!(run_effect, Effect::Run(os_vec(&["tool"])));

        let search_action = handle_mouse_event(
            click(11, 20),
            &state,
            &frame_snapshot,
            &TuiConfig::default(),
        )
        .expect("search action");
        let search_effect = apply_action(&search_action, &mut state, &frame_snapshot);
        assert_eq!(search_effect, Effect::None);
        assert!(matches!(state.ui.focus, Focus::Search));
    }

    #[test]
    fn footer_run_click_closes_dropdown_before_running() {
        let mut state = AppState::new(command(Vec::new()));
        let mut frame_snapshot = FrameSnapshot::default();
        state.ui.dropdown_open = Some("color".to_string());
        frame_snapshot.layout.footer = Some(Rect::new(0, 20, 20, 1));
        frame_snapshot.layout.footer_buttons = vec![FooterButtonLayout {
            target: HoverTarget::Run,
            rect: Rect::new(0, 20, 8, 1),
        }];

        let action =
            handle_mouse_event(click(1, 20), &state, &frame_snapshot, &TuiConfig::default())
                .expect("run action");
        let effect = apply_action(&action, &mut state, &frame_snapshot);

        assert_eq!(effect, Effect::Run(os_vec(&["tool"])));
        assert!(state.ui.dropdown_open.is_none());
    }

    #[test]
    fn preview_click_copies_full_width_dock() {
        let state = AppState::new(command(Vec::new()));
        let mut frame_snapshot = FrameSnapshot::default();
        frame_snapshot.layout.preview = Some(Rect::new(0, 18, 80, 4));

        let action = handle_mouse_event(
            click(72, 20),
            &state,
            &frame_snapshot,
            &TuiConfig::default(),
        );

        assert_eq!(action, Some(Action::CopyPreview));
    }

    #[test]
    fn outside_click_retargets_search_while_closing_dropdown() {
        let mut color = arg("color", "--color", ArgKind::Enum);
        color.choices = vec!["red".to_string(), "green".to_string()];
        let mut state = AppState::new(command(vec![color]));
        let mut frame_snapshot = FrameSnapshot::default();
        state.ui.dropdown_open = Some("color".to_string());
        frame_snapshot.layout.dropdown = Some(Rect::new(20, 5, 20, 5));
        frame_snapshot.layout.search = Some(Rect::new(0, 0, 20, 1));

        let action =
            handle_mouse_event(click(1, 0), &state, &frame_snapshot, &TuiConfig::default())
                .expect("search action");
        let effect = apply_action(&action, &mut state, &frame_snapshot);

        assert_eq!(effect, Effect::None);
        assert!(state.ui.dropdown_open.is_none());
        assert!(matches!(state.ui.focus, Focus::Search));
    }

    #[test]
    fn outside_click_retargets_form_while_closing_dropdown() {
        let mut color = arg("color", "--color", ArgKind::Enum);
        color.choices = vec!["red".to_string(), "green".to_string()];
        let path = arg("path", "--path", ArgKind::Option);
        let mut state = AppState::new(command(vec![color, path]));
        let mut frame_snapshot = FrameSnapshot::default();
        state.ui.dropdown_open = Some("color".to_string());
        frame_snapshot.layout.dropdown = Some(Rect::new(20, 5, 20, 5));
        frame_snapshot.layout.form = Some(Rect::new(0, 2, 30, 8));
        frame_snapshot.layout.form_view = Some(Rect::new(0, 2, 30, 8));

        let action =
            handle_mouse_event(click(1, 6), &state, &frame_snapshot, &TuiConfig::default())
                .expect("form action");
        let effect = apply_action(&action, &mut state, &frame_snapshot);

        assert_eq!(effect, Effect::None);
        assert!(state.ui.dropdown_open.is_none());
        assert!(matches!(state.ui.focus, Focus::Form));
    }

    #[test]
    fn tab_click_switches_active_tab() {
        let mut state = AppState::new(command(Vec::new()));
        let mut frame_snapshot = FrameSnapshot::default();
        state.ui.active_tab = ActiveTab::Inputs;
        frame_snapshot.layout.form_tabs = vec![TabButtonLayout {
            tab: ActiveTab::Inputs,
            rect: Rect::new(0, 0, 8, 1),
        }];

        let action =
            handle_mouse_event(click(1, 0), &state, &frame_snapshot, &TuiConfig::default())
                .expect("tab action");
        let effect = apply_action(&action, &mut state, &frame_snapshot);

        assert_eq!(effect, Effect::None);
        assert_eq!(state.ui.active_tab, ActiveTab::Inputs);
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
        let effect = apply_action(&action, &mut state, &frame_snapshot);

        assert_eq!(effect, Effect::None);
        assert_eq!(
            state.domain.current_form().and_then(|inputs| {
                state
                    .domain
                    .current_command()
                    .args
                    .iter()
                    .find(|arg| arg.id == "color")
                    .and_then(|arg| inputs.compatibility_value(arg))
            }),
            Some(crate::input::ArgValue::Choice("green".to_string()))
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
            scroll(AppMouseEventKind::ScrollDown, 1, 6),
            &state,
            &frame_snapshot,
            &TuiConfig::default(),
        )
        .expect("scroll action");
        let effect = apply_action(&action, &mut state, &frame_snapshot);

        assert_eq!(effect, Effect::None);
        assert_eq!(state.ui.dropdown_scroll, 1);
        assert_eq!(state.ui.form_scroll, 4);
    }

    #[test]
    fn sidebar_scroll_events_adjust_sidebar_scroll_without_touching_form_scroll() {
        let mut state = AppState::new(CommandSpec {
            name: "tool".to_string(),
            version: None,
            about: None,
            help: String::new(),
            args: Vec::new(),
            subcommands: (0..6)
                .map(|index| CommandSpec {
                    name: format!("cmd-{index}"),
                    version: None,
                    about: None,
                    help: String::new(),
                    args: Vec::new(),
                    subcommands: Vec::new(),
                    ..CommandSpec::default()
                })
                .collect(),
            ..CommandSpec::default()
        });
        state
            .select_command_path(&["cmd-2".to_string()])
            .expect("valid path");
        let mut frame_snapshot = FrameSnapshot::default();
        frame_snapshot.layout.sidebar = Some(Rect::new(0, 0, 20, 10));
        frame_snapshot.layout.sidebar_list = Some(Rect::new(1, 2, 18, 3));
        state.ui.form_scroll = 4;

        let action = handle_mouse_event(
            scroll(AppMouseEventKind::ScrollDown, 2, 3),
            &state,
            &frame_snapshot,
            &TuiConfig::default(),
        )
        .expect("scroll action");
        let effect = apply_action(&action, &mut state, &frame_snapshot);

        assert_eq!(effect, Effect::None);
        assert_eq!(state.ui.sidebar_scroll, 2);
        assert_eq!(state.ui.form_scroll, 4);
    }

    #[test]
    fn sidebar_scroll_events_over_search_or_chrome_still_scroll_sidebar() {
        let mut state = AppState::new(CommandSpec {
            name: "tool".to_string(),
            version: None,
            about: None,
            help: String::new(),
            args: Vec::new(),
            subcommands: (0..6)
                .map(|index| CommandSpec {
                    name: format!("cmd-{index}"),
                    version: None,
                    about: None,
                    help: String::new(),
                    args: Vec::new(),
                    subcommands: Vec::new(),
                    ..CommandSpec::default()
                })
                .collect(),
            ..CommandSpec::default()
        });
        state
            .select_command_path(&["cmd-2".to_string()])
            .expect("valid path");
        let mut frame_snapshot = FrameSnapshot::default();
        frame_snapshot.layout.sidebar = Some(Rect::new(0, 0, 20, 10));
        frame_snapshot.layout.sidebar_list = Some(Rect::new(1, 2, 18, 3));
        frame_snapshot.layout.search = Some(Rect::new(1, 1, 18, 1));
        state.ui.form_scroll = 4;

        let action = handle_mouse_event(
            scroll(AppMouseEventKind::ScrollDown, 2, 1),
            &state,
            &frame_snapshot,
            &TuiConfig::default(),
        )
        .expect("scroll action");
        let effect = apply_action(&action, &mut state, &frame_snapshot);

        assert_eq!(effect, Effect::None);
        assert_eq!(state.ui.sidebar_scroll, 2);
        assert_eq!(state.ui.form_scroll, 4);
    }

    #[test]
    fn dropdown_click_uses_clamped_scroll_position() {
        let mut color = arg("color", "--color", ArgKind::Enum);
        color.choices = (0..8).map(|index| format!("choice-{index}")).collect();
        let mut state = AppState::new(command(vec![color]));
        let mut frame_snapshot = FrameSnapshot::default();
        state.ui.dropdown_open = Some("color".to_string());
        state.ui.dropdown_scroll = 20;
        frame_snapshot.layout.dropdown = Some(Rect::new(0, 5, 20, 5));

        let action =
            handle_mouse_event(click(1, 7), &state, &frame_snapshot, &TuiConfig::default())
                .expect("dropdown action");
        let effect = apply_action(&action, &mut state, &frame_snapshot);

        assert_eq!(effect, Effect::None);
        assert_eq!(
            state.domain.current_form().and_then(|inputs| {
                state
                    .domain
                    .current_command()
                    .args
                    .iter()
                    .find(|arg| arg.id == "color")
                    .and_then(|arg| inputs.compatibility_value(arg))
            }),
            Some(crate::input::ArgValue::Choice("choice-6".to_string()))
        );
    }

    #[test]
    fn outside_click_retargets_sidebar_while_closing_dropdown() {
        let mut color = arg("color", "--color", ArgKind::Enum);
        color.choices = vec!["red".to_string(), "green".to_string()];
        let mut state = AppState::new(CommandSpec {
            name: "tool".to_string(),
            version: None,
            about: None,
            help: String::new(),
            args: vec![color],
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
        });
        let mut frame_snapshot = FrameSnapshot::default();
        state.ui.dropdown_open = Some("color".to_string());
        frame_snapshot.layout.dropdown = Some(Rect::new(20, 5, 20, 5));
        frame_snapshot.layout.sidebar = Some(Rect::new(0, 0, 20, 10));
        frame_snapshot.layout.sidebar_items = vec![SidebarItemLayout {
            path: vec!["build".to_string()].into(),
            row: Rect::new(0, 1, 20, 1),
            caret: None,
            has_children: false,
        }];

        let action =
            handle_mouse_event(click(1, 1), &state, &frame_snapshot, &TuiConfig::default())
                .expect("sidebar action");
        let effect = apply_action(&action, &mut state, &frame_snapshot);

        assert_eq!(effect, Effect::None);
        assert!(state.ui.dropdown_open.is_none());
        assert_eq!(
            state.domain.selected_path().as_slice(),
            &["build".to_string()]
        );
        assert!(matches!(state.ui.focus, Focus::Sidebar));
    }

    #[test]
    fn sidebar_caret_click_selects_command_and_expands_item() {
        let mut state = AppState::new(CommandSpec {
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
                subcommands: vec![CommandSpec {
                    name: "release".to_string(),
                    version: None,
                    about: None,
                    help: String::new(),
                    args: Vec::new(),
                    subcommands: Vec::new(),
                    ..CommandSpec::default()
                }],
                ..CommandSpec::default()
            }],
            ..CommandSpec::default()
        });
        state
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
        let effect = apply_action(&action, &mut state, &frame_snapshot);

        assert_eq!(effect, Effect::None);
        assert_eq!(
            state.domain.selected_path().as_slice(),
            &["build".to_string()]
        );
        assert!(state.domain.expanded.contains("tool::build"));
        assert!(matches!(state.ui.focus, Focus::Sidebar));
    }

    #[test]
    fn help_overlay_ignores_scroll_events_outside_form_area() {
        let mut state = AppState::new(command(Vec::new()));
        state.ui.help_open = true;
        let mut frame_snapshot = FrameSnapshot::default();
        frame_snapshot.layout.form = Some(Rect::new(20, 0, 40, 10));
        frame_snapshot.layout.form_view = Some(Rect::new(20, 0, 40, 10));
        frame_snapshot.layout.sidebar = Some(Rect::new(0, 0, 20, 10));

        let action = handle_mouse_event(
            scroll(AppMouseEventKind::ScrollDown, 5, 5),
            &state,
            &frame_snapshot,
            &TuiConfig::default(),
        );

        assert!(action.is_none());
    }
}
