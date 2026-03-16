use crate::controller::navigation;
use crate::form_editor::{self, EditResult};
use crate::frame_snapshot::FrameSnapshot;
use crate::input::{ActiveTab, AppState, Focus, HoverTarget, MouseSelection};
use crate::runtime::{AppKeyCode, AppKeyEvent, AppMouseEvent};
use crate::view::{argv, command_tree, form};

#[derive(Debug, Clone)]
pub(crate) enum Action {
    Exit,
    Run,
    CopyPreview,
    SearchInput(AppKeyEvent),
    ChoiceInput { arg_id: String, key: AppKeyEvent },
    FormTextInput(AppKeyEvent),
    ToggleFocus,
    ToggleHelp,
    CycleTabs,
    FocusSearch,
    MoveSidebarSelection(isize),
    MoveFormSelection(isize),
    CollapseSelected,
    ExpandSelected,
    SelectSidebar,
    ActivateFormField,
    UpdateHover { x: u16, y: u16 },
    UpdateMouseSelection(AppMouseEvent),
    ClearMouseSelection,
    CloseDropdown,
    ClickDropdownChoice { arg_id: String, row: u16 },
    ClickFooter(HoverTarget),
    ClickSidebar { x: u16, y: u16 },
    SwitchTab(ActiveTab),
    ClickForm(AppMouseEvent),
    ScrollDropdown(i16),
    ScrollForm(i16),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Effect {
    None,
    Exit,
    Run(Vec<String>),
    CopyToClipboard(String),
}

pub(crate) fn apply_action(
    action: &Action,
    state: &mut AppState,
    frame_snapshot: &FrameSnapshot,
) -> Effect {
    let effect = if matches!(action, Action::Exit | Action::Run | Action::CopyPreview) {
        apply_command_action(action, state)
    } else if matches!(
        action,
        Action::SearchInput(_)
            | Action::ToggleFocus
            | Action::ToggleHelp
            | Action::CycleTabs
            | Action::FocusSearch
            | Action::CloseDropdown
            | Action::ClickFooter(_)
            | Action::SwitchTab(_)
    ) {
        apply_global_action(action, state)
    } else if matches!(
        action,
        Action::MoveSidebarSelection(_)
            | Action::CollapseSelected
            | Action::ExpandSelected
            | Action::SelectSidebar
            | Action::ClickSidebar { .. }
    ) {
        apply_sidebar_action(action, state, frame_snapshot)
    } else if matches!(
        action,
        Action::ChoiceInput { .. }
            | Action::FormTextInput(_)
            | Action::MoveFormSelection(_)
            | Action::ActivateFormField
            | Action::ClickDropdownChoice { .. }
            | Action::ClickForm(_)
            | Action::ScrollDropdown(_)
            | Action::ScrollForm(_)
    ) {
        apply_form_action(action, state, frame_snapshot)
    } else {
        apply_mouse_action(action, state, frame_snapshot)
    };
    normalize_state(state);
    effect
}

pub(crate) fn normalize_state(state: &mut AppState) {
    state.domain.ensure_defaults();
    let current_command = state.domain.current_command().clone();
    let active_args = form::visible_args(&current_command, state.ui.active_tab);
    let visible = active_args
        .iter()
        .map(|item| (item.order_index, item.arg))
        .collect::<Vec<_>>();
    state.ui.ensure_active_tab_visible(&visible);
    state.ui.ensure_selected_arg_visible(&visible);
}

fn apply_command_action(action: &Action, state: &mut AppState) -> Effect {
    match action {
        Action::Exit => Effect::Exit,
        Action::Run => Effect::Run(argv::build_argv(state)),
        Action::CopyPreview => Effect::CopyToClipboard(argv::build_argv(state).join(" ")),
        _ => Effect::None,
    }
}

fn apply_global_action(action: &Action, state: &mut AppState) -> Effect {
    match action {
        Action::SearchInput(key) => {
            apply_search_input(*key, state);
            Effect::None
        }
        Action::ToggleFocus => {
            state.ui.focus = match state.ui.focus {
                Focus::Sidebar => Focus::Form,
                _ => Focus::Sidebar,
            };
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
            state.ui.focus = Focus::Search;
            Effect::None
        }
        Action::CloseDropdown => {
            state.ui.dropdown_open = None;
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

fn apply_sidebar_action(
    action: &Action,
    state: &mut AppState,
    frame_snapshot: &FrameSnapshot,
) -> Effect {
    match action {
        Action::MoveSidebarSelection(delta) => navigation::move_sidebar_selection(state, *delta),
        Action::CollapseSelected => navigation::collapse_selected(state),
        Action::ExpandSelected => navigation::expand_selected(state),
        Action::SelectSidebar => navigation::select_sidebar(state),
        Action::ClickSidebar { x, y } => apply_sidebar_click(*x, *y, state, frame_snapshot),
        _ => {}
    }
    Effect::None
}

fn apply_form_action(
    action: &Action,
    state: &mut AppState,
    frame_snapshot: &FrameSnapshot,
) -> Effect {
    match action {
        Action::ChoiceInput { arg_id, key } => {
            apply_choice_input(*key, state, frame_snapshot, arg_id);
        }
        Action::FormTextInput(key) => apply_form_text_input(*key, state),
        Action::MoveFormSelection(delta) => {
            navigation::move_form_selection(state, frame_snapshot, *delta);
        }
        Action::ActivateFormField => navigation::activate_form_field(state, frame_snapshot),
        Action::ClickDropdownChoice { arg_id, row } => {
            apply_dropdown_click(*row, state, frame_snapshot, arg_id);
        }
        Action::ClickForm(event) => apply_form_click(*event, state, frame_snapshot),
        Action::ScrollDropdown(delta) => navigation::scroll_enum(state, *delta),
        Action::ScrollForm(delta) => navigation::scroll_form(state, frame_snapshot, *delta),
        _ => {}
    }
    Effect::None
}

fn apply_mouse_action(
    action: &Action,
    state: &mut AppState,
    frame_snapshot: &FrameSnapshot,
) -> Effect {
    match action {
        Action::UpdateHover { x, y } => apply_hover_update(state, frame_snapshot, *x, *y),
        Action::UpdateMouseSelection(event) => {
            apply_mouse_selection(state, frame_snapshot, *event);
        }
        Action::ClearMouseSelection => state.ui.mouse_select = None,
        _ => {}
    }
    Effect::None
}

fn apply_search_input(key: AppKeyEvent, state: &mut AppState) {
    match key.code {
        AppKeyCode::Esc | AppKeyCode::Enter => state.ui.focus = Focus::Sidebar,
        AppKeyCode::Backspace => {
            state.ui.search_query.pop();
        }
        AppKeyCode::Char(c) => state.ui.search_query.push(c),
        _ => {}
    }
}

fn apply_form_text_input(key: AppKeyEvent, state: &mut AppState) {
    if state.ui.help_open {
        return;
    }
    let command = state.domain.current_command().clone();
    let args = form::visible_args(&command, state.ui.active_tab);
    let Some(item) = args
        .iter()
        .find(|item| item.order_index == state.ui.selected_arg_index)
    else {
        return;
    };
    if !item.arg.accepts_text_input() {
        return;
    }

    let _ = matches!(
        form_editor::apply_key_to_text_field(state, item.arg, key),
        EditResult::Handled
    );
}

fn apply_choice_input(
    key: AppKeyEvent,
    state: &mut AppState,
    frame_snapshot: &FrameSnapshot,
    arg_id: &str,
) {
    let command = state.domain.current_command().clone();
    let Some(arg) = command.args.iter().find(|arg| arg.id == arg_id) else {
        state.ui.dropdown_open = None;
        return;
    };
    let len = arg.choices.len();

    match key.code {
        AppKeyCode::Up => {
            if len == 0 {
                return;
            }
            let current = state
                .domain
                .current_form()
                .and_then(|inputs| inputs.values.get(&arg.id))
                .and_then(|value| match value {
                    crate::input::ArgValue::Choice(selected) => {
                        arg.choices.iter().position(|choice| choice == selected)
                    }
                    _ => None,
                })
                .unwrap_or(0);
            let next = if current == 0 { len - 1 } else { current - 1 };
            state
                .domain
                .set_choice_value(&arg.id, arg.choices[next].clone());
            state.domain.mark_touched(&arg.id);
            navigation::ensure_enum_visible(state, frame_snapshot, next, len);
        }
        AppKeyCode::Down => {
            state.domain.cycle_choice(&arg.id, &arg.choices);
            let current = state
                .domain
                .current_form()
                .and_then(|inputs| inputs.values.get(&arg.id))
                .and_then(|value| match value {
                    crate::input::ArgValue::Choice(selected) => {
                        arg.choices.iter().position(|choice| choice == selected)
                    }
                    _ => None,
                })
                .unwrap_or(0);
            state.domain.mark_touched(&arg.id);
            navigation::ensure_enum_visible(state, frame_snapshot, current, len);
        }
        AppKeyCode::Esc | AppKeyCode::Enter | AppKeyCode::Char(' ') => {
            state.ui.dropdown_open = None;
        }
        _ => {}
    }
}

fn apply_mouse_selection(
    state: &mut AppState,
    frame_snapshot: &FrameSnapshot,
    event: AppMouseEvent,
) {
    let Some(mut selection) = state.ui.mouse_select.take() else {
        return;
    };
    if let Some((row, col)) =
        frame_snapshot.input_position_from_point(&selection.arg_id, event.column, event.row, true)
    {
        let arg = state
            .domain
            .current_command()
            .args
            .iter()
            .find(|arg| arg.id == selection.arg_id)
            .cloned();
        if let Some(arg) = arg {
            if !selection.active {
                form_editor::start_selection(
                    state,
                    &arg,
                    selection.anchor_row,
                    selection.anchor_col,
                );
                selection.active = true;
            }
            form_editor::set_cursor_from_click(state, &arg, row, col);
        }
    }
    state.ui.mouse_select = Some(selection);
}

fn apply_sidebar_click(x: u16, y: u16, state: &mut AppState, frame_snapshot: &FrameSnapshot) {
    if let Some(sidebar_area) = frame_snapshot.layout.sidebar
        && y == sidebar_area.y
    {
        navigation::select_root(state);
        state.ui.focus = Focus::Sidebar;
        return;
    }

    let Some((path, caret_hit, has_children)) = frame_snapshot.sidebar_item_at(x, y).map(|item| {
        (
            item.path.clone(),
            FrameSnapshot::sidebar_caret_contains(item, x, y),
            item.has_children,
        )
    }) else {
        return;
    };

    if *state.domain.selected_path() != path
        && state.domain.select_command_path(path.as_slice()).is_ok()
    {
        let command = state.domain.current_command().clone();
        let args = form::visible_args(&command, state.ui.active_tab);
        let visible = args
            .iter()
            .map(|item| (item.order_index, item.arg))
            .collect::<Vec<_>>();
        state.ui.focus_first_tab(&visible);
    }
    if caret_hit && has_children {
        let items = command_tree::tree_items(
            &state.domain.root,
            &state.domain.expanded,
            &state.ui.search_query,
        );
        if let Some(item) = items.iter().find(|item| item.path == path) {
            if item.expanded {
                navigation::collapse_selected(state);
            } else {
                navigation::expand_selected(state);
            }
        }
    }
    state.ui.focus = Focus::Sidebar;
}

fn apply_form_click(event: AppMouseEvent, state: &mut AppState, frame_snapshot: &FrameSnapshot) {
    if state.ui.help_open {
        return;
    }
    let Some(content_y) =
        frame_snapshot.form_content_y(event.row, state.ui.form_scroll(frame_snapshot))
    else {
        return;
    };
    let command = state.domain.current_command().clone();
    let args = form::visible_args(&command, state.ui.active_tab);
    if let Some(hit) = form::hit_test_form_content(&args, content_y) {
        state.ui.selected_arg_index = hit.order_index;
        state.ui.focus = Focus::Form;
        if hit.is_flag && hit.in_input {
            state.domain.toggle_flag(&hit.arg_id);
            state.domain.mark_touched(&hit.arg_id);
        }
        if hit.uses_choice_input && hit.in_input {
            let total = command
                .args
                .iter()
                .find(|arg| arg.id == hit.arg_id)
                .map_or(0, |arg| arg.choices.len());
            navigation::open_enum_dropdown(state, frame_snapshot, &hit.arg_id, total);
        }
        if hit.accepts_text_input
            && let Some(arg) = command
                .args
                .iter()
                .find(|arg| arg.id == hit.arg_id)
                .cloned()
        {
            form_editor::clear_selection(state, &arg);
            state.ui.mouse_select = None;
            if let Some((row, col)) = frame_snapshot.input_position_from_point(
                &hit.arg_id,
                event.column,
                event.row,
                false,
            ) {
                state.ui.mouse_select = Some(MouseSelection {
                    arg_id: hit.arg_id.clone(),
                    anchor_row: row,
                    anchor_col: col,
                    active: false,
                });
            }
            if let Some((row, col)) = frame_snapshot.input_position_from_point(
                &hit.arg_id,
                event.column,
                event.row,
                false,
            ) {
                form_editor::set_cursor_from_click(state, &arg, row, col);
            } else if hit.in_label {
                form_editor::set_cursor_from_click(state, &arg, 0, 0);
            }
        }
        navigation::ensure_form_visible(state, frame_snapshot);
    }
}

fn apply_dropdown_click(
    row: u16,
    state: &mut AppState,
    frame_snapshot: &FrameSnapshot,
    arg_id: &str,
) {
    let command = state.domain.current_command().clone();
    let Some(arg) = command.args.iter().find(|arg| arg.id == arg_id) else {
        return;
    };
    if let Some(index) = frame_snapshot.dropdown_choice_index(row, state.ui.dropdown_scroll)
        && let Some(choice) = arg.choices.get(index)
    {
        state.domain.set_choice_value(&arg.id, choice.clone());
        state.ui.dropdown_open = None;
        state.domain.mark_touched(&arg.id);
    }
}

fn apply_footer_click(target: HoverTarget, state: &mut AppState) -> Effect {
    match target {
        HoverTarget::Run => Effect::Run(argv::build_argv(state)),
        HoverTarget::Exit => Effect::Exit,
        HoverTarget::Search => {
            state.ui.focus = Focus::Search;
            Effect::None
        }
        HoverTarget::Focus => {
            state.ui.focus = Focus::Sidebar;
            Effect::None
        }
        HoverTarget::Help => {
            navigation::toggle_help_tab(state);
            Effect::None
        }
        HoverTarget::Preview => Effect::None,
    }
}

fn apply_hover_update(state: &mut AppState, frame_snapshot: &FrameSnapshot, x: u16, y: u16) {
    state.ui.hover = frame_snapshot.footer_target_at(x, y);
    if state.ui.hover.is_none() && frame_snapshot.preview_contains(x, y) {
        state.ui.hover = Some(HoverTarget::Preview);
    }
    state.ui.hover_tab = frame_snapshot.tab_at(x, y);
}

#[cfg(test)]
mod tests {
    use crate::frame_snapshot::FrameSnapshot;
    use crate::runtime::{AppKeyModifiers, AppMouseButton, AppMouseEventKind};
    use crate::spec::{ArgKind, ArgSpec, CommandSpec, ValueCardinality};

    use super::{Action, Effect, apply_action};
    use crate::input::MouseSelection;
    use crate::runtime::{AppKeyCode, AppKeyEvent, AppMouseEvent};

    fn command(args: Vec<ArgSpec>) -> CommandSpec {
        CommandSpec {
            name: "tool".to_string(),
            version: None,
            about: None,
            help: String::new(),
            args,
            subcommands: Vec::new(),
        }
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
        }
    }

    #[test]
    fn search_reducer_appends_and_exits_search_mode() {
        let mut state = crate::input::AppState::new(command(Vec::new()));
        state.ui.focus = crate::input::Focus::Search;
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
    fn mouse_selection_reducer_starts_editor_selection_on_drag() {
        let mut path = arg("path", "path", ArgKind::Positional);
        path.position = Some(1);
        let mut state = crate::input::AppState::new(command(vec![path]));
        state.ui.mouse_select = Some(MouseSelection {
            arg_id: "path".to_string(),
            anchor_row: 0,
            anchor_col: 0,
            active: false,
        });
        let mut snapshot = FrameSnapshot::default();
        snapshot
            .layout
            .form_inputs
            .insert("path".to_string(), ratatui::layout::Rect::new(0, 0, 12, 3));

        let action = Action::UpdateMouseSelection(AppMouseEvent {
            kind: AppMouseEventKind::Drag(AppMouseButton::Left),
            column: 2,
            row: 1,
            modifiers: AppKeyModifiers::default(),
        });
        let effect = apply_action(&action, &mut state, &snapshot);

        assert_eq!(effect, Effect::None);
        assert!(
            state
                .ui
                .mouse_select
                .as_ref()
                .is_some_and(|selection| selection.active)
        );
        let editor = state
            .ui
            .editors
            .editor(state.domain.selected_path(), "path")
            .expect("editor");
        assert_eq!(
            editor.selection_anchor(),
            Some(crate::editor_state::TextPosition::default())
        );
    }

    #[test]
    fn sidebar_title_click_reselects_root() {
        let mut state = crate::input::AppState::new(CommandSpec {
            name: "tool".to_string(),
            version: Some("1.0.0".to_string()),
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
        });
        state
            .domain
            .select_command_path(&["build".to_string()])
            .expect("valid path");
        let mut snapshot = FrameSnapshot::default();
        snapshot.layout.sidebar = Some(ratatui::layout::Rect::new(0, 0, 20, 8));

        let action = Action::ClickSidebar { x: 2, y: 0 };
        let effect = apply_action(&action, &mut state, &snapshot);

        assert_eq!(effect, Effect::None);
        assert!(state.domain.selected_path().is_empty());
        assert_eq!(state.domain.current_command().name, "tool");
        assert!(matches!(state.ui.focus, crate::input::Focus::Sidebar));
    }
}
