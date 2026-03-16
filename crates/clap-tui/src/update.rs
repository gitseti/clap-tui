mod command;
mod form;
mod global;
mod pointer;
mod sidebar;

use crate::frame_snapshot::FrameSnapshot;
use crate::input::{ActiveTab, AppState, HoverTarget};
use crate::query::form as form_query;
use crate::runtime::{AppKeyEvent, AppMouseEvent};

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
        command::apply(action, state)
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
        global::apply(action, state)
    } else if matches!(
        action,
        Action::MoveSidebarSelection(_)
            | Action::CollapseSelected
            | Action::ExpandSelected
            | Action::SelectSidebar
            | Action::ClickSidebar { .. }
    ) {
        sidebar::apply(action, state, frame_snapshot)
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
        form::apply(action, state, frame_snapshot)
    } else {
        pointer::apply(action, state, frame_snapshot)
    };
    normalize_state(state);
    effect
}

pub(crate) fn normalize_state(state: &mut AppState) {
    state.domain.ensure_defaults();
    let current_command = state.domain.current_command().clone();
    let active_args = form_query::visible_args(&current_command, state.ui.active_tab);
    let visible = form_query::visible_arg_pairs(&active_args);
    state.ui.ensure_active_tab_visible(&visible);
    state.ui.ensure_selected_arg_visible(&visible);
}
