use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, BorderType, Borders, List, ListItem, ListState, Scrollbar, ScrollbarOrientation,
    ScrollbarState,
};

use crate::config::TuiConfig;
use crate::input::AppState;

use super::screen::ScreenView;

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
    let total = arg.possible_values.len();
    let visible = rect.height.saturating_sub(2) as usize;
    let start = state.enum_scroll.min(total.saturating_sub(visible));
    let end = (start + visible).min(total);
    let items = arg
        .possible_values
        .iter()
        .skip(start)
        .take(visible)
        .map(|value| {
            let line = Line::from(Span::styled(
                value.clone(),
                Style::default().fg(config.theme.text),
            ));
            ListItem::new(line)
        })
        .collect::<Vec<_>>();
    let mut list_state = ListState::default();
    let current_idx = state
        .current_inputs()
        .and_then(|inputs| inputs.values.get(&arg.id))
        .and_then(|value| match value {
            crate::input::ArgValue::Enum(idx) => Some(*idx),
            _ => None,
        })
        .unwrap_or(0);
    if current_idx >= start && current_idx < end {
        list_state.select(Some(current_idx - start));
    }
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title("Select"),
        )
        .highlight_style(
            Style::default()
                .fg(config.theme.text)
                .bg(config.theme.focus_bg)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ")
        .style(Style::default().bg(config.theme.input_bg));
    frame.render_stateful_widget(list, rect, &mut list_state);

    if total > visible && visible > 0 {
        let scroll_steps = total.saturating_sub(visible).saturating_add(1);
        let mut scrollbar_state = ScrollbarState::new(scroll_steps)
            .position(state.enum_scroll)
            .viewport_content_length(visible);
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .track_symbol(Some("┃"))
            .thumb_symbol("█")
            .thumb_style(
                Style::default()
                    .fg(config.theme.accent)
                    .add_modifier(Modifier::BOLD),
            )
            .track_style(Style::default().fg(config.theme.text));
        frame.render_stateful_widget(scrollbar, rect, &mut scrollbar_state);
    }
}
