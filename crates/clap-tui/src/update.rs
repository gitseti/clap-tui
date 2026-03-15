use crossterm::event::{KeyCode, KeyEvent, MouseEvent};
use tui_textarea::CursorMove;

use crate::form_editor::{self, EditResult};
use crate::frame_snapshot::FrameSnapshot;
use crate::input::{ActiveTab, AppState, Focus, HoverTarget, MouseSelection};
use crate::view::{argv, command_tree, form};

use crate::controller::navigation;

#[derive(Debug, Clone)]
pub(crate) enum Action {
    Exit,
    Run,
    CopyPreview,
    SearchInput(KeyEvent),
    ChoiceInput { arg_id: String, key: KeyEvent },
    FormTextInput(KeyEvent),
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
    UpdateMouseSelection(MouseEvent),
    ClearMouseSelection,
    CloseDropdown,
    ClickDropdownChoice { arg_id: String, row: u16 },
    ClickFooter(HoverTarget),
    ClickSidebar { x: u16, y: u16 },
    SwitchTab(ActiveTab),
    ClickForm(MouseEvent),
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
    action: Action,
    state: &mut AppState,
    frame_snapshot: &FrameSnapshot,
) -> Effect {
    let effect = match action {
        Action::Exit => Effect::Exit,
        Action::Run => Effect::Run(argv::build_argv(state)),
        Action::CopyPreview => Effect::CopyToClipboard(argv::build_argv(state).join(" ")),
        Action::SearchInput(key) => {
            handle_search_input(key, state);
            Effect::None
        }
        Action::ChoiceInput { arg_id, key } => {
            handle_choice_input(key, state, frame_snapshot, &arg_id);
            Effect::None
        }
        Action::FormTextInput(key) => {
            handle_form_text_input(key, state);
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
        Action::MoveSidebarSelection(delta) => {
            navigation::move_sidebar_selection(state, delta);
            Effect::None
        }
        Action::MoveFormSelection(delta) => {
            navigation::move_form_selection(state, frame_snapshot, delta);
            Effect::None
        }
        Action::CollapseSelected => {
            navigation::collapse_selected(state);
            Effect::None
        }
        Action::ExpandSelected => {
            navigation::expand_selected(state);
            Effect::None
        }
        Action::SelectSidebar => {
            navigation::select_sidebar(state);
            Effect::None
        }
        Action::ActivateFormField => {
            navigation::activate_form_field(state, frame_snapshot);
            Effect::None
        }
        Action::UpdateHover { x, y } => {
            update_hover(state, frame_snapshot, x, y);
            Effect::None
        }
        Action::UpdateMouseSelection(event) => {
            update_mouse_selection(state, frame_snapshot, event);
            Effect::None
        }
        Action::ClearMouseSelection => {
            state.ui.mouse_select = None;
            Effect::None
        }
        Action::CloseDropdown => {
            state.ui.dropdown_open = None;
            Effect::None
        }
        Action::ClickDropdownChoice { arg_id, row } => {
            handle_dropdown_click(row, state, frame_snapshot, &arg_id);
            Effect::None
        }
        Action::ClickFooter(target) => handle_footer_click(target, state),
        Action::ClickSidebar { x, y } => {
            handle_sidebar_click(x, y, state, frame_snapshot);
            Effect::None
        }
        Action::SwitchTab(tab) => {
            navigation::switch_tab(state, tab);
            Effect::None
        }
        Action::ClickForm(event) => {
            handle_form_click(event, state, frame_snapshot);
            Effect::None
        }
        Action::ScrollDropdown(delta) => {
            navigation::scroll_enum(state, delta);
            Effect::None
        }
        Action::ScrollForm(delta) => {
            navigation::scroll_form(state, frame_snapshot, delta);
            Effect::None
        }
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
    if state.ui.active_tab != ActiveTab::Help {
        state.ui.ensure_selected_arg_visible(&visible);
    }
}

fn handle_search_input(key: KeyEvent, state: &mut AppState) {
    match key.code {
        KeyCode::Esc | KeyCode::Enter => state.ui.focus = Focus::Sidebar,
        KeyCode::Backspace => {
            state.ui.search_query.pop();
        }
        KeyCode::Char(c) => state.ui.search_query.push(c),
        _ => {}
    }
}

fn handle_form_text_input(key: KeyEvent, state: &mut AppState) {
    if matches!(state.ui.active_tab, ActiveTab::Help) {
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

fn handle_choice_input(
    key: KeyEvent,
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
        KeyCode::Up => {
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
            state.domain.set_choice_value(&arg.id, arg.choices[next].clone());
            state.domain.mark_touched(&arg.id);
            navigation::ensure_enum_visible(state, frame_snapshot, next, len);
        }
        KeyCode::Down => {
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
        KeyCode::Esc | KeyCode::Enter | KeyCode::Char(' ') => {
            state.ui.dropdown_open = None;
        }
        _ => {}
    }
}

fn update_mouse_selection(
    state: &mut AppState,
    frame_snapshot: &FrameSnapshot,
    event: MouseEvent,
) {
    let Some(mut selection) = state.ui.mouse_select.take() else {
        return;
    };
    if let Some((row, col)) = frame_snapshot.input_position_from_point(
        &selection.arg_id,
        event.column,
        event.row,
        true,
    ) {
        let arg = state
            .domain
            .current_command()
            .args
            .iter()
            .find(|arg| arg.id == selection.arg_id)
            .cloned();
        if let Some(arg) = arg {
            let displayed = form_editor::displayed_text(state, &arg);
            let selected_path = state.domain.selected_path().clone();
            let textarea =
                form_editor::ensure_editor(&mut state.ui, &selected_path, &arg, &displayed);
            if !selection.active {
                textarea.move_cursor(CursorMove::Jump(selection.anchor_row, selection.anchor_col));
                textarea.start_selection();
                selection.active = true;
            }
            textarea.move_cursor(CursorMove::Jump(row, col));
        }
    }
    state.ui.mouse_select = Some(selection);
}

fn handle_sidebar_click(x: u16, y: u16, state: &mut AppState, frame_snapshot: &FrameSnapshot) {
    let Some((path, caret_hit, has_children)) = frame_snapshot
        .sidebar_item_at(x, y)
        .map(|item| {
            (
                item.path.clone(),
                frame_snapshot.sidebar_caret_contains(item, x, y),
                item.has_children,
            )
        })
    else {
        return;
    };

    if *state.domain.selected_path() != path {
        if state.domain.select_command_path(path.as_slice()).is_ok() {
            let command = state.domain.current_command().clone();
            let args = form::visible_args(&command, state.ui.active_tab);
            let visible = args
                .iter()
                .map(|item| (item.order_index, item.arg))
                .collect::<Vec<_>>();
            state.ui.focus_first_tab(&visible);
        }
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

fn handle_form_click(event: MouseEvent, state: &mut AppState, frame_snapshot: &FrameSnapshot) {
    if matches!(state.ui.active_tab, ActiveTab::Help) {
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
        if hit.accepts_text_input {
            if let Some(arg) = command.args.iter().find(|arg| arg.id == hit.arg_id).cloned() {
                let displayed = form_editor::displayed_text(state, &arg);
                let selected_path = state.domain.selected_path().clone();
                let textarea =
                    form_editor::ensure_editor(&mut state.ui, &selected_path, &arg, &displayed);
                textarea.cancel_selection();
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
        }
        navigation::ensure_form_visible(state, frame_snapshot);
    }
}

fn handle_dropdown_click(
    row: u16,
    state: &mut AppState,
    frame_snapshot: &FrameSnapshot,
    arg_id: &str,
) {
    let command = state.domain.current_command().clone();
    let Some(arg) = command.args.iter().find(|arg| arg.id == arg_id) else {
        return;
    };
    if let Some(index) = frame_snapshot.dropdown_choice_index(row, state.ui.dropdown_scroll) {
        if let Some(choice) = arg.choices.get(index) {
            state.domain.set_choice_value(&arg.id, choice.clone());
            state.ui.dropdown_open = None;
            state.domain.mark_touched(&arg.id);
        }
    }
}

fn handle_footer_click(target: HoverTarget, state: &mut AppState) -> Effect {
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

fn update_hover(state: &mut AppState, frame_snapshot: &FrameSnapshot, x: u16, y: u16) {
    state.ui.hover = frame_snapshot.footer_target_at(x, y);
    if state.ui.hover.is_none() && frame_snapshot.preview_contains(x, y) {
        state.ui.hover = Some(HoverTarget::Preview);
    }
    state.ui.hover_tab = frame_snapshot.tab_at(x, y);
}
