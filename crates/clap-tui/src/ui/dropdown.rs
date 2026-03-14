use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, BorderType, Borders, Clear, List, ListItem, ListState, Scrollbar, ScrollbarOrientation,
    ScrollbarState,
};

use crate::config::TuiConfig;
use crate::input::AppState;
use crate::spec::enum_value_matches_default;

use super::screen::ScreenView;

pub(crate) const MAX_DROPDOWN_ROWS: u16 = 6;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DropdownLayout {
    pub(crate) rect: Rect,
    pub(crate) visible_rows: usize,
}

pub(crate) fn dropdown_layout(
    form_view: Rect,
    input_rect: Rect,
    total_options: usize,
) -> Option<DropdownLayout> {
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

    let popup_height = visible_rows as u16 + 2;
    let y = if place_below {
        input_rect.y.saturating_add(input_rect.height)
    } else {
        input_rect.y.saturating_sub(popup_height)
    };

    Some(DropdownLayout {
        rect: Rect::new(input_rect.x, y, input_rect.width, popup_height),
        visible_rows,
    })
}

pub(crate) fn render_dropdown(
    frame: &mut Frame<'_>,
    state: &mut AppState,
    config: &TuiConfig,
    _area: Rect,
    _vm: &ScreenView,
) {
    let Some(arg_id) = state.enum_open.clone() else {
        return;
    };
    let Some(rect) = state.layout.dropdown else {
        return;
    };
    let Some(arg) = state.current_command().args.iter().find(|a| a.id == arg_id) else {
        return;
    };
    let is_touched = state.is_touched(&arg.id);
    let total = arg.possible_values.len();
    let visible = rect.height.saturating_sub(2) as usize;
    let start = state.enum_scroll.min(total.saturating_sub(visible));
    let end = (start + visible).min(total);
    let current_idx = state
        .current_inputs()
        .and_then(|inputs| inputs.values.get(&arg.id))
        .and_then(|value| match value {
            crate::input::ArgValue::Enum(idx) => Some(*idx),
            _ => None,
        })
        .unwrap_or(0);
    let items = arg
        .possible_values
        .iter()
        .enumerate()
        .skip(start)
        .take(visible)
        .map(|(index, value)| {
            let is_default = !is_touched && enum_value_matches_default(arg, index);
            let is_selected = index == current_idx;
            let text_style = match (is_selected, is_default) {
                (true, true) => Style::default().fg(config.theme.dim),
                (true, false) => Style::default()
                    .fg(config.theme.text)
                    .add_modifier(Modifier::BOLD),
                (false, true) => Style::default().fg(config.theme.dim),
                (false, false) => Style::default().fg(config.theme.text),
            };
            let line = Line::from(Span::styled(value.clone(), text_style));
            ListItem::new(line)
        })
        .collect::<Vec<_>>();
    let mut list_state = ListState::default();
    if current_idx >= start && current_idx < end {
        list_state.select(Some(current_idx - start));
    }
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(super::styles::panel_border(config, false))
                .style(Style::default().bg(config.theme.input_bg)),
        )
        .highlight_style(Style::default().bg(config.theme.focus_bg))
        .highlight_symbol("› ")
        .style(Style::default().bg(config.theme.input_bg));
    frame.render_widget(Clear, rect);
    frame.render_stateful_widget(list, rect, &mut list_state);

    if total > visible && visible > 0 {
        let scroll_steps = total.saturating_sub(visible).saturating_add(1);
        let mut scrollbar_state = ScrollbarState::new(scroll_steps)
            .position(state.enum_scroll)
            .viewport_content_length(visible);
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .track_symbol(Some("┃"))
            .thumb_symbol("█")
            .thumb_style(Style::default().fg(config.theme.border))
            .track_style(Style::default().fg(config.theme.dim));
        frame.render_stateful_widget(scrollbar, rect, &mut scrollbar_state);
    }
}

#[cfg(test)]
mod tests {
    use super::{MAX_DROPDOWN_ROWS, dropdown_layout};
    use ratatui::layout::Rect;

    #[test]
    fn dropdown_uses_input_width() {
        let form_view = Rect::new(10, 5, 60, 20);
        let input_rect = Rect::new(14, 8, 24, 3);
        let layout = dropdown_layout(form_view, input_rect, 4).expect("layout");

        assert_eq!(layout.rect.x, input_rect.x);
        assert_eq!(layout.rect.width, input_rect.width);
    }

    #[test]
    fn dropdown_prefers_below_when_space_exists() {
        let form_view = Rect::new(10, 5, 60, 20);
        let input_rect = Rect::new(14, 8, 24, 3);
        let layout = dropdown_layout(form_view, input_rect, 4).expect("layout");

        assert_eq!(layout.rect.y, input_rect.y + input_rect.height);
        assert_eq!(layout.visible_rows, 4);
    }

    #[test]
    fn dropdown_flips_above_when_below_is_too_tight() {
        let form_view = Rect::new(10, 5, 60, 12);
        let input_rect = Rect::new(14, 12, 24, 3);
        let layout = dropdown_layout(form_view, input_rect, 4).expect("layout");

        assert_eq!(layout.rect.y + layout.rect.height, input_rect.y);
        assert_eq!(layout.visible_rows, 4);
    }

    #[test]
    fn dropdown_height_respects_space_and_max_rows() {
        let form_view = Rect::new(10, 5, 60, 11);
        let input_rect = Rect::new(14, 8, 24, 3);
        let layout = dropdown_layout(form_view, input_rect, 20).expect("layout");

        assert_eq!(layout.visible_rows, 3);
        assert_eq!(layout.rect.height, 5);

        let roomy_form = Rect::new(10, 5, 60, 30);
        let roomy_layout = dropdown_layout(roomy_form, input_rect, 20).expect("layout");
        assert_eq!(roomy_layout.visible_rows, MAX_DROPDOWN_ROWS as usize);
        assert_eq!(roomy_layout.rect.height, MAX_DROPDOWN_ROWS + 2);
    }
}
