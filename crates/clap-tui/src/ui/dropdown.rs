use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, BorderType, Borders, Clear, List, ListItem, ListState, Scrollbar, ScrollbarOrientation,
    ScrollbarState, StatefulWidget,
};

use crate::config::TuiConfig;
use crate::frame_snapshot::FrameSnapshot;
use crate::input::{DomainState, UiState};
use crate::spec::choice_value_matches_default;

use super::screen::ScreenView;

pub(crate) const MAX_DROPDOWN_ROWS: u16 = 6;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DropdownLayout {
    pub(crate) rect: Rect,
    pub(crate) visible_rows: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DropdownItem {
    label: String,
    text_style: Style,
}

#[derive(Debug, Clone)]
struct DropdownView {
    rect: Rect,
    items: Vec<DropdownItem>,
    selected_index: Option<usize>,
    scroll_position: usize,
    visible_rows: usize,
    total_rows: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DropdownWidgetState {
    selected_index: Option<usize>,
    scroll_position: usize,
    visible_rows: usize,
    total_rows: usize,
}

struct DropdownWidget<'a> {
    config: &'a TuiConfig,
    items: &'a [DropdownItem],
}

impl StatefulWidget for DropdownWidget<'_> {
    type State = DropdownWidgetState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let mut list_state = ListState::default();
        list_state.select(state.selected_index);

        let list_items = self
            .items
            .iter()
            .map(|item| {
                let line = Line::from(Span::styled(item.label.clone(), item.text_style));
                ListItem::new(line)
            })
            .collect::<Vec<_>>();

        let list = List::new(list_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(super::styles::panel_border(self.config, false))
                    .style(Style::default().bg(self.config.theme.surface_raised)),
            )
            .highlight_style(super::styles::selection(self.config))
            .highlight_symbol("› ")
            .style(Style::default().bg(self.config.theme.surface_raised));

        StatefulWidget::render(list, area, buf, &mut list_state);

        if state.total_rows > state.visible_rows && state.visible_rows > 0 {
            let scroll_steps = state
                .total_rows
                .saturating_sub(state.visible_rows)
                .saturating_add(1);
            let mut scrollbar_state = ScrollbarState::new(scroll_steps)
                .position(state.scroll_position)
                .viewport_content_length(state.visible_rows);
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .track_symbol(Some("┃"))
                .thumb_symbol("█")
                .thumb_style(Style::default().fg(self.config.theme.panel_focus_border))
                .track_style(Style::default().fg(self.config.theme.dim));
            StatefulWidget::render(scrollbar, area, buf, &mut scrollbar_state);
        }
    }
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

    let popup_height = u16::try_from(visible_rows).unwrap_or(MAX_DROPDOWN_ROWS) + 2;
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
    ui: &UiState,
    frame_snapshot: &FrameSnapshot,
    domain: &DomainState,
    config: &TuiConfig,
    _area: Rect,
    _vm: &ScreenView<'_>,
) {
    let Some(view) = build_dropdown_view(ui, frame_snapshot, domain, config) else {
        return;
    };
    let mut widget_state = DropdownWidgetState {
        selected_index: view.selected_index,
        scroll_position: view.scroll_position,
        visible_rows: view.visible_rows,
        total_rows: view.total_rows,
    };
    let widget = DropdownWidget {
        config,
        items: &view.items,
    };

    frame.render_widget(Clear, view.rect);
    frame.render_stateful_widget(widget, view.rect, &mut widget_state);
}

fn build_dropdown_view(
    ui: &UiState,
    frame_snapshot: &FrameSnapshot,
    domain: &DomainState,
    config: &TuiConfig,
) -> Option<DropdownView> {
    let arg_id = ui.dropdown_open.as_ref()?;
    let rect = frame_snapshot.layout.dropdown?;
    let arg = domain.current_command().args.iter().find(|arg| &arg.id == arg_id)?;

    let is_touched = domain
        .current_form()
        .is_some_and(|form| form.touched.contains(&arg.id));
    let total_rows = arg.choices.len();
    let visible_rows = rect.height.saturating_sub(2) as usize;
    let scroll_position = ui.dropdown_scroll.min(total_rows.saturating_sub(visible_rows));
    let selected_row = domain
        .current_form()
        .and_then(|inputs| inputs.values.get(&arg.id))
        .and_then(|value| match value {
            crate::input::ArgValue::Choice(selected) => {
                arg.choices.iter().position(|choice| choice == selected)
            }
            _ => None,
        })
        .unwrap_or(0);
    let selected_index = (selected_row >= scroll_position
        && selected_row < scroll_position.saturating_add(visible_rows))
    .then_some(selected_row - scroll_position);

    let items = arg
        .choices
        .iter()
        .enumerate()
        .skip(scroll_position)
        .take(visible_rows)
        .map(|(index, value)| {
            let is_default = !is_touched && choice_value_matches_default(arg, value);
            let is_selected = index == selected_row;
            let text_style = match (is_selected, is_default) {
                (true, false) => Style::default().fg(config.theme.selection_fg),
                (_, true) => Style::default().fg(config.theme.dim),
                (false, false) => Style::default().fg(config.theme.text),
            };
            DropdownItem {
                label: value.clone(),
                text_style,
            }
        })
        .collect();

    Some(DropdownView {
        rect,
        items,
        selected_index,
        scroll_position,
        visible_rows,
        total_rows,
    })
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

    #[test]
    fn dropdown_returns_none_when_neither_side_has_room_for_rows() {
        let form_view = Rect::new(0, 0, 40, 3);
        let input_rect = Rect::new(2, 1, 20, 1);

        assert_eq!(dropdown_layout(form_view, input_rect, 2), None);
    }
}
