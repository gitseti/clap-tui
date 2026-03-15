use std::collections::HashMap;

use ratatui::layout::Rect;

use crate::input::{ActiveTab, HoverTarget};
use crate::spec::CommandPath;

#[derive(Debug, Default, Clone)]
pub struct FrameLayout {
    pub sidebar: Option<Rect>,
    pub form: Option<Rect>,
    pub search: Option<Rect>,
    pub preview: Option<Rect>,
    pub footer: Option<Rect>,
    pub dropdown: Option<Rect>,
    pub sidebar_items: Vec<SidebarItemLayout>,
    pub form_inputs: HashMap<String, Rect>,
    pub form_view: Option<Rect>,
    pub form_tabs: Vec<TabButtonLayout>,
    pub footer_buttons: Vec<FooterButtonLayout>,
}

#[derive(Debug, Default, Clone)]
pub struct FrameSnapshot {
    pub layout: FrameLayout,
    pub form_scroll_max: u16,
}

impl FrameSnapshot {
    pub fn form_scroll(&self, requested_scroll: u16) -> u16 {
        requested_scroll.min(self.form_scroll_max)
    }

    pub fn footer_target_at(&self, x: u16, y: u16) -> Option<HoverTarget> {
        self.layout
            .footer_buttons
            .iter()
            .find(|button| contains(button.rect, x, y))
            .map(|button| button.target)
    }

    pub fn tab_at(&self, x: u16, y: u16) -> Option<ActiveTab> {
        self.layout
            .form_tabs
            .iter()
            .find(|tab| contains(tab.rect, x, y))
            .map(|tab| tab.tab)
    }

    pub fn sidebar_item_at(&self, x: u16, y: u16) -> Option<&SidebarItemLayout> {
        self.layout
            .sidebar_items
            .iter()
            .find(|item| contains(item.row, x, y))
    }

    pub fn sidebar_caret_contains(&self, item: &SidebarItemLayout, x: u16, y: u16) -> bool {
        item.caret.is_some_and(|caret| contains(caret, x, y))
    }

    pub fn form_input_rect(&self, arg_id: &str) -> Option<Rect> {
        self.layout.form_inputs.get(arg_id).copied()
    }

    pub fn form_view_rect(&self) -> Option<Rect> {
        self.layout.form_view
    }

    pub fn dropdown_contains(&self, x: u16, y: u16) -> bool {
        self.layout.dropdown.is_some_and(|area| contains(area, x, y))
    }

    pub fn preview_contains(&self, x: u16, y: u16) -> bool {
        self.layout.preview.is_some_and(|area| contains(area, x, y))
    }

    pub fn search_contains(&self, x: u16, y: u16) -> bool {
        self.layout.search.is_some_and(|area| contains(area, x, y))
    }

    pub fn sidebar_contains(&self, x: u16, y: u16) -> bool {
        self.layout.sidebar.is_some_and(|area| contains(area, x, y))
    }

    pub fn form_contains(&self, x: u16, y: u16) -> bool {
        self.layout.form.is_some_and(|area| contains(area, x, y))
    }

    pub fn form_content_y(&self, row: u16, scroll: u16) -> Option<u16> {
        let form_view = self.layout.form_view?;
        if row < form_view.y || row >= form_view.y + form_view.height {
            return None;
        }
        Some(row.saturating_sub(form_view.y).saturating_add(scroll))
    }

    pub fn input_position_from_point(
        &self,
        arg_id: &str,
        x: u16,
        y: u16,
        clamp: bool,
    ) -> Option<(u16, u16)> {
        let input_rect = self.form_input_rect(arg_id)?;
        let inner_x = input_rect.x.saturating_add(1);
        let inner_y = input_rect.y.saturating_add(1);
        let inner_w = input_rect.width.saturating_sub(2);
        let inner_h = input_rect.height.saturating_sub(2);
        if inner_w == 0 || inner_h == 0 {
            return None;
        }
        if !clamp
            && (x < inner_x || y < inner_y || x >= inner_x + inner_w || y >= inner_y + inner_h)
        {
            return None;
        }
        let x = if clamp {
            x.clamp(inner_x, inner_x + inner_w - 1)
        } else {
            x
        };
        let y = if clamp {
            y.clamp(inner_y, inner_y + inner_h - 1)
        } else {
            y
        };
        Some((
            y.saturating_sub(inner_y).min(inner_h.saturating_sub(1)),
            x.saturating_sub(inner_x).min(inner_w.saturating_sub(1)),
        ))
    }

    pub fn dropdown_choice_index(&self, row: u16, scroll: usize) -> Option<usize> {
        let area = self.layout.dropdown?;
        if row <= area.y || row >= area.y + area.height - 1 {
            return None;
        }
        Some(usize::from(row.saturating_sub(area.y + 1)) + scroll)
    }

    pub fn dropdown_visible_rows(&self) -> Option<usize> {
        self.layout
            .dropdown
            .map(|dropdown| usize::from(dropdown.height.saturating_sub(2)))
    }
}

fn contains(area: Rect, x: u16, y: u16) -> bool {
    x >= area.x && x < area.x + area.width && y >= area.y && y < area.y + area.height
}

#[cfg(test)]
mod tests {
    use ratatui::layout::Rect;

    use super::{FooterButtonLayout, FrameSnapshot, SidebarItemLayout, TabButtonLayout};
    use crate::input::{ActiveTab, HoverTarget};

    #[test]
    fn query_helpers_hit_expected_targets() {
        let mut snapshot = FrameSnapshot::default();
        snapshot.layout.footer_buttons = vec![FooterButtonLayout {
            target: HoverTarget::Run,
            rect: Rect::new(0, 10, 8, 1),
        }];
        snapshot.layout.form_tabs = vec![TabButtonLayout {
            tab: ActiveTab::Help,
            rect: Rect::new(0, 0, 8, 1),
        }];
        snapshot.layout.sidebar_items = vec![SidebarItemLayout {
            path: vec!["build".to_string()].into(),
            row: Rect::new(0, 2, 20, 1),
            caret: None,
            has_children: true,
        }];

        assert_eq!(snapshot.footer_target_at(1, 10), Some(HoverTarget::Run));
        assert_eq!(snapshot.tab_at(1, 0), Some(ActiveTab::Help));
        assert_eq!(
            snapshot.sidebar_item_at(1, 2).map(|item| item.path.as_slice()),
            Some(&["build".to_string()][..])
        );
    }

    #[test]
    fn input_and_dropdown_queries_respect_inner_geometry() {
        let mut snapshot = FrameSnapshot::default();
        snapshot
            .layout
            .form_inputs
            .insert("name".to_string(), Rect::new(10, 5, 12, 3));
        snapshot.layout.form_view = Some(Rect::new(0, 5, 40, 10));
        snapshot.layout.dropdown = Some(Rect::new(10, 8, 12, 5));

        assert_eq!(
            snapshot.input_position_from_point("name", 11, 6, false),
            Some((0, 0))
        );
        assert_eq!(snapshot.form_content_y(7, 3), Some(5));
        assert!(snapshot.dropdown_contains(11, 9));
        assert_eq!(snapshot.dropdown_choice_index(10, 2), Some(3));
        assert_eq!(snapshot.dropdown_visible_rows(), Some(3));
    }
}

#[derive(Debug, Clone)]
pub struct FooterButtonLayout {
    pub target: HoverTarget,
    pub rect: Rect,
}

#[derive(Debug, Clone)]
pub struct SidebarItemLayout {
    pub path: CommandPath,
    pub row: Rect,
    pub caret: Option<Rect>,
    pub has_children: bool,
}

#[derive(Debug, Clone)]
pub struct TabButtonLayout {
    pub tab: ActiveTab,
    pub rect: Rect,
}
