use ratatui::layout::{Constraint, Direction, Layout, Rect};

const MIN_SIDEBAR_WIDTH: u16 = 16;
const MIN_MAIN_WIDTH: u16 = 20;
pub(crate) const TOGGLE_WIDTH: u16 = 3;

#[derive(Debug, Default)]
pub struct SidebarState {
    pub(crate) collapsed: bool,
    preferred_width: Option<u16>,
    drag: Option<ResizeDrag>,
}

#[derive(Debug)]
struct ResizeDrag {
    column: u16,
    width: u16,
}

impl SidebarState {
    pub(crate) fn width(&self, available: u16, default: u16) -> u16 {
        if self.collapsed {
            0
        } else {
            clamp_width(self.preferred_width.unwrap_or(default), available)
        }
    }

    pub(crate) fn start_resize(&mut self, column: u16, width: u16) {
        self.drag = Some(ResizeDrag { column, width });
    }

    pub(crate) fn resize_to(&mut self, column: u16, available: u16) {
        if let Some(drag) = &self.drag {
            let width = i32::from(drag.width) + i32::from(column) - i32::from(drag.column);
            let width = u16::try_from(width.max(0)).unwrap_or(u16::MAX);
            self.preferred_width = Some(clamp_width(width, available));
        }
    }

    pub(crate) fn stop_resize(&mut self) {
        self.drag = None;
    }

    pub(crate) fn is_resizing(&self) -> bool {
        self.drag.is_some()
    }
}

fn clamp_width(width: u16, available: u16) -> u16 {
    let max = available.saturating_sub(MIN_MAIN_WIDTH);
    width.clamp(MIN_SIDEBAR_WIDTH.min(max), max)
}

pub(crate) fn sections(area: Rect) -> (Rect, Rect, Rect) {
    let areas = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(area);
    (areas[0], areas[1], areas[2])
}

#[cfg(test)]
mod tests;
