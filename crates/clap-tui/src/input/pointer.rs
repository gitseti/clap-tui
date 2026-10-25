use ratatui::layout::Position;

use super::UiState;
use crate::runtime::{AppEvent, AppMouseButton, AppMouseEventKind};

impl UiState {
    /// Record input coordinates before routing, including motion owned by an overlay.
    pub(crate) fn observe_pointer(&mut self, event: &AppEvent) -> bool {
        match event {
            AppEvent::Mouse(mouse) => {
                let position = Some(Position::new(mouse.column, mouse.row));
                let mut changed = self.mouse_position != position;
                self.mouse_position = position;
                if matches!(
                    mouse.kind,
                    AppMouseEventKind::Moved | AppMouseEventKind::Up(AppMouseButton::Left)
                ) {
                    changed |= self.mouse_select.is_some();
                    self.clear_mouse_selection();
                }
                changed
            }
            AppEvent::FocusLost => {
                let changed = self.mouse_position.take().is_some()
                    || self.sidebar.is_resizing()
                    || self.mouse_select.is_some();
                self.sidebar.stop_resize();
                self.clear_mouse_selection();
                self.hover = None;
                self.hover_tab = None;
                changed
            }
            _ => false,
        }
    }
}
