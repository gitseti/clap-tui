use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};

use crate::config::TuiConfig;
use crate::input::{AppState, Focus, SidebarItemLayout};

use super::screen::ScreenView;
use super::styles;

pub(crate) fn render_sidebar(
    frame: &mut Frame<'_>,
    state: &mut AppState,
    config: &TuiConfig,
    area: Rect,
    vm: &ScreenView,
) {
    let search_focused = matches!(state.interaction.focus, Focus::Search);
    let sidebar_focused = matches!(state.interaction.focus, Focus::Sidebar);
    let panel_focused = search_focused || sidebar_focused;
    let panel = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(styles::panel_border(config, panel_focused))
        .title(Line::from(Span::styled(
            "Commands",
            styles::panel_title(config, false),
        )))
        .style(styles::panel(config));
    frame.render_widget(panel, area);
    state.layout.sidebar = Some(area);

    let inner = area.inner(ratatui::layout::Margin {
        horizontal: 1,
        vertical: 1,
    });
    let sidebar = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(inner);

    let mut search_style = if state.command.search.is_empty() {
        styles::placeholder(config)
    } else {
        Style::default().fg(config.theme.text)
    };
    if search_focused {
        search_style = search_style.add_modifier(Modifier::BOLD);
        search_style = search_style.fg(config.theme.text);
    }

    let search = Paragraph::new(if state.command.search.is_empty() {
        "/ search commands".to_string()
    } else {
        state.command.search.clone()
    })
    .style(search_style.bg(if search_focused {
        config.theme.surface_raised
    } else {
        config.theme.input_bg
    }));
    frame.render_widget(search, sidebar[0]);
    state.layout.search = Some(sidebar[0]);

    frame.render_widget(
        Paragraph::new("─".repeat(sidebar[1].width as usize))
            .style(Style::default().fg(config.theme.divider)),
        sidebar[1],
    );

    state.layout.sidebar_items.clear();
    let list_area = sidebar[2];
    let content_y = list_area.y;
    let content_x = list_area.x;
    let content_height = usize::from(list_area.height);
    for (index, item) in vm.tree_items.iter().take(content_height).enumerate() {
        let row_y = content_y.saturating_add(u16::try_from(index).unwrap_or(list_area.height));
        let row_rect = Rect::new(content_x, row_y, list_area.width, 1);
        let selected = item.path == state.command.selected_path;
        let row_style = if selected {
            if sidebar_focused {
                styles::list_highlight(config)
            } else {
                styles::list_highlight_unfocused(config)
            }
        } else if item.indent > 0 {
            Style::default()
                .fg(config.theme.dim)
                .bg(config.theme.panel_bg)
        } else {
            Style::default()
                .fg(config.theme.text)
                .bg(config.theme.panel_bg)
        };
        let rail = if selected { "|" } else { " " };
        let line = Line::from(vec![
            Span::styled(
                rail,
                Style::default().fg(if selected {
                    config.theme.accent
                } else {
                    config.theme.panel_bg
                }),
            ),
            Span::raw(" "),
            Span::styled(item.label.clone(), row_style),
        ]);
        frame.render_widget(Paragraph::new(line).style(row_style), row_rect);

        let caret = if item.has_children {
            let caret_x = content_x
                .saturating_add(u16::try_from(item.indent).unwrap_or(0))
                .saturating_add(2);
            Some(Rect::new(caret_x, row_y, 1, 1))
        } else {
            None
        };
        state.layout.sidebar_items.push(SidebarItemLayout {
            path: item.path.clone(),
            row: row_rect,
            caret,
            has_children: item.has_children,
        });
    }
}
