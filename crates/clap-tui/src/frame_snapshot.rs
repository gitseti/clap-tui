use std::collections::HashMap;

use ratatui::layout::Rect;

use crate::input::{ActiveTab, HoverTarget};
use crate::spec::CommandPath;

pub(crate) const MAX_DROPDOWN_ROWS: u16 = 6;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DropdownGeometry {
    pub(crate) rect: Rect,
    pub(crate) visible_rows: usize,
}

#[derive(Debug, Default, Clone)]
pub struct FrameLayout {
    pub sidebar: Option<Rect>,
    pub form: Option<Rect>,
    pub search: Option<Rect>,
    pub preview: Option<Rect>,
    pub footer: Option<Rect>,
    pub dropdown: Option<Rect>,
    pub sidebar_items: Vec<SidebarItemLayout>,
    pub form_fields: Vec<FormFieldLayout>,
    pub form_inputs: HashMap<String, Rect>,
    pub form_view: Option<Rect>,
    pub form_tabs: Vec<TabButtonLayout>,
    pub footer_buttons: Vec<FooterButtonLayout>,
}

#[derive(Debug, Default, Clone)]
pub struct FrameSnapshot {
    pub layout: FrameLayout,
    pub form_scroll_max: u16,
    pub help_scroll_max: u16,
}

impl FrameSnapshot {
    pub fn form_scroll(&self, requested_scroll: u16) -> u16 {
        requested_scroll.min(self.form_scroll_max)
    }

    pub fn help_scroll(&self, requested_scroll: u16) -> u16 {
        requested_scroll.min(self.help_scroll_max)
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

    pub fn sidebar_caret_contains(item: &SidebarItemLayout, x: u16, y: u16) -> bool {
        item.caret.is_some_and(|caret| contains(caret, x, y))
    }

    pub fn form_input_rect(&self, arg_id: &str) -> Option<Rect> {
        self.layout.form_inputs.get(arg_id).copied()
    }

    pub fn form_view_rect(&self) -> Option<Rect> {
        self.layout.form_view
    }

    pub fn dropdown_contains(&self, x: u16, y: u16) -> bool {
        self.layout
            .dropdown
            .is_some_and(|area| contains(area, x, y))
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

    pub fn dropdown_geometry_for_input(
        &self,
        arg_id: &str,
        total_options: usize,
    ) -> Option<DropdownGeometry> {
        self.layout
            .form_view
            .zip(self.form_input_rect(arg_id))
            .and_then(|(form_view, input_rect)| {
                dropdown_geometry(form_view, input_rect, total_options)
            })
    }
}

pub(crate) fn dropdown_geometry(
    form_view: Rect,
    input_rect: Rect,
    total_options: usize,
) -> Option<DropdownGeometry> {
    if total_options == 0 || form_view.height == 0 || input_rect.width < 3 {
        return None;
    }

    let desired_rows = total_options.min(MAX_DROPDOWN_ROWS as usize);
    let available_below = form_view
        .y
        .saturating_add(form_view.height)
        .saturating_sub(input_rect.y.saturating_add(input_rect.height));
    let available_above = input_rect.y.saturating_sub(form_view.y);

    let rows_below = available_below.saturating_sub(2) as usize;
    let rows_above = available_above.saturating_sub(2) as usize;

    let place_below = if rows_below >= desired_rows || rows_above == 0 {
        rows_below > 0
    } else if rows_above >= desired_rows || rows_below == 0 {
        false
    } else {
        rows_below >= rows_above
    };

    let visible_rows = if place_below { rows_below } else { rows_above }.min(desired_rows);
    if visible_rows == 0 {
        return None;
    }

    let popup_height = u16::try_from(visible_rows).unwrap_or(MAX_DROPDOWN_ROWS) + 2;
    let y = if place_below {
        input_rect.y.saturating_add(input_rect.height)
    } else {
        input_rect.y.saturating_sub(popup_height)
    };

    Some(DropdownGeometry {
        rect: Rect::new(input_rect.x, y, input_rect.width, popup_height),
        visible_rows,
    })
}

fn contains(area: Rect, x: u16, y: u16) -> bool {
    x >= area.x && x < area.x + area.width && y >= area.y && y < area.y + area.height
}

#[cfg(test)]
mod tests {
    use ratatui::layout::Rect;

    use super::{
        FooterButtonLayout, FrameSnapshot, SidebarItemLayout, TabButtonLayout, dropdown_geometry,
    };
    use crate::input::{ActiveTab, HoverTarget};

    #[test]
    fn query_helpers_hit_expected_targets() {
        let mut snapshot = FrameSnapshot::default();
        snapshot.layout.footer_buttons = vec![FooterButtonLayout {
            target: HoverTarget::Run,
            rect: Rect::new(0, 10, 8, 1),
        }];
        snapshot.layout.form_tabs = vec![TabButtonLayout {
            tab: ActiveTab::Inputs,
            rect: Rect::new(0, 0, 8, 1),
        }];
        snapshot.layout.sidebar_items = vec![SidebarItemLayout {
            path: vec!["build".to_string()].into(),
            row: Rect::new(0, 2, 20, 1),
            caret: None,
            has_children: true,
        }];

        assert_eq!(snapshot.footer_target_at(1, 10), Some(HoverTarget::Run));
        assert_eq!(snapshot.tab_at(1, 0), Some(ActiveTab::Inputs));
        assert_eq!(
            snapshot
                .sidebar_item_at(1, 2)
                .map(|item| item.path.as_slice()),
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

    #[test]
    fn dropdown_geometry_matches_expected_popup_layout() {
        let form_view = Rect::new(10, 5, 60, 20);
        let input_rect = Rect::new(14, 8, 24, 3);
        let geometry = dropdown_geometry(form_view, input_rect, 4).expect("geometry");

        assert_eq!(geometry.rect.x, input_rect.x);
        assert_eq!(geometry.rect.width, input_rect.width);
        assert_eq!(geometry.rect.y, input_rect.y + input_rect.height);
        assert_eq!(geometry.visible_rows, 4);
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

#[derive(Debug, Clone)]
pub struct FormFieldLayout {
    pub arg_id: String,
    pub label: Option<Rect>,
    pub input: Rect,
    pub description: Option<Rect>,
}
