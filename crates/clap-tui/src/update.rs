mod command;
mod form;
mod global;
mod pointer;
mod sidebar;

use crate::frame_snapshot::FrameSnapshot;
use crate::input::{ActiveTab, AppState, HoverTarget};
use crate::runtime::{AppKeyEvent, AppMouseEvent};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Action {
    Exit,
    Run,
    CopyPreview,
    Escape,
    SearchInput(AppKeyEvent),
    ChoiceInput { arg_id: String, key: AppKeyEvent },
    FormTextInput(AppKeyEvent),
    ToggleFocus,
    ToggleHelp,
    CycleTabs,
    FocusSearch,
    MoveSidebarSelection(isize),
    MoveFormSelection(isize),
    SidebarRight,
    CollapseSelected,
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
    match action {
        Action::Exit | Action::Run | Action::CopyPreview => command::apply(action, state),
        Action::Escape
        | Action::SearchInput(_)
        | Action::ToggleFocus
        | Action::ToggleHelp
        | Action::CycleTabs
        | Action::FocusSearch
        | Action::CloseDropdown
        | Action::ClickFooter(_)
        | Action::SwitchTab(_) => global::apply(action, state),
        Action::MoveSidebarSelection(_)
        | Action::SidebarRight
        | Action::CollapseSelected
        | Action::SelectSidebar
        | Action::ClickSidebar { .. } => sidebar::apply(action, state, frame_snapshot),
        Action::ChoiceInput { .. }
        | Action::FormTextInput(_)
        | Action::MoveFormSelection(_)
        | Action::ActivateFormField
        | Action::ClickDropdownChoice { .. }
        | Action::ClickForm(_)
        | Action::ScrollDropdown(_)
        | Action::ScrollForm(_) => form::apply(action, state, frame_snapshot),
        Action::UpdateHover { .. }
        | Action::UpdateMouseSelection(_)
        | Action::ClearMouseSelection => pointer::apply(action, state, frame_snapshot),
    }
}
