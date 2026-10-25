use crate::{
    frame_snapshot::FrameSnapshot,
    input::{HoverTarget, UiState},
    runtime::MousePointer,
};

pub(super) fn resolve(ui: &UiState, snapshot: &FrameSnapshot) -> MousePointer {
    if snapshot.layout.sidebar_divider.is_some()
        && (ui.sidebar.is_resizing() || ui.hover == Some(HoverTarget::SidebarDivider))
    {
        MousePointer::ResizeHorizontal
    } else {
        MousePointer::Default
    }
}
