use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::config::TuiConfig;
use crate::input::{AppState, FooterButtonLayout, HoverTarget};

use super::screen::ScreenView;
use super::styles;

pub(crate) fn render_footer(
    frame: &mut Frame<'_>,
    state: &mut AppState,
    config: &TuiConfig,
    area: Rect,
    _vm: &ScreenView,
) {
    let actions = vec![
        (HoverTarget::Run, "Ctrl+Enter Run"),
        (HoverTarget::Exit, "Ctrl+C Exit"),
    ];
    let hints = vec![
        (HoverTarget::Search, "/ Search"),
        (HoverTarget::Focus, "Tab Focus"),
        (HoverTarget::Help, "? Help"),
    ];

    state.layout.footer_buttons.clear();
    let action_width = group_width(&actions);
    let hint_width = group_width(&hints);

    let left_x = area.x;
    let min_right_x = left_x
        .saturating_add(action_width)
        .saturating_add(u16::from(action_width > 0));
    let preferred_right_x = area.x.saturating_add(area.width.saturating_sub(hint_width));
    let right_x = preferred_right_x.max(min_right_x);

    let mut spans = Vec::new();
    let mut cursor_x = left_x;
    for (index, (target, chip)) in actions.iter().enumerate() {
        if index > 0 {
            spans.push(Span::raw(" "));
            cursor_x = cursor_x.saturating_add(1);
        }
        let label = format!(" {chip} ");
        let rect = Rect::new(
            cursor_x,
            area.y,
            u16::try_from(label.chars().count()).unwrap_or(area.width),
            1,
        );
        state.layout.footer_buttons.push(FooterButtonLayout {
            target: *target,
            rect,
        });
        let style = match target {
            HoverTarget::Run => {
                styles::primary_chip(config, state.interaction.hover == Some(*target))
            }
            HoverTarget::Exit => {
                styles::secondary_chip(config, state.interaction.hover == Some(*target))
            }
            _ => styles::subtle_chip(config, state.interaction.hover == Some(*target)),
        };
        spans.push(Span::styled(label.clone(), style));
        cursor_x = cursor_x.saturating_add(u16::try_from(label.chars().count()).unwrap_or(0));
    }

    let gap = right_x.saturating_sub(cursor_x);
    if gap > 0 {
        spans.push(Span::raw(" ".repeat(gap as usize)));
    }
    cursor_x = right_x;

    for (index, (target, chip)) in hints.iter().enumerate() {
        if index > 0 {
            spans.push(Span::raw(" "));
            cursor_x = cursor_x.saturating_add(1);
        }
        let label = format!(" {chip} ");
        let rect = Rect::new(
            cursor_x,
            area.y,
            u16::try_from(label.chars().count()).unwrap_or(area.width),
            1,
        );
        state.layout.footer_buttons.push(FooterButtonLayout {
            target: *target,
            rect,
        });
        spans.push(Span::styled(
            label.clone(),
            styles::subtle_chip(config, state.interaction.hover == Some(*target)),
        ));
        cursor_x = cursor_x.saturating_add(u16::try_from(label.chars().count()).unwrap_or(0));
    }

    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

fn group_width(items: &[(HoverTarget, &str)]) -> u16 {
    let labels = items
        .iter()
        .map(|(_, chip)| u16::try_from(chip.chars().count()).unwrap_or(0) + 2)
        .sum::<u16>();
    let gaps = u16::try_from(items.len().saturating_sub(1)).unwrap_or(0);
    labels.saturating_add(gaps)
}
