use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph};

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
    let sidebar = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);
    let search_focused = matches!(state.focus, Focus::Search);
    let sidebar_focused = matches!(state.focus, Focus::Sidebar);

    let mut search_style = if state.search.is_empty() {
        styles::placeholder(config)
    } else {
        Style::default().fg(config.theme.text)
    };
    if search_focused {
        search_style = search_style.add_modifier(Modifier::BOLD);
        if !state.search.is_empty() {
            search_style = search_style.fg(config.theme.accent);
        }
    }

    let search = Paragraph::new(format!(
        "🔍 {}",
        if state.search.is_empty() {
            "/ to search".to_string()
        } else {
            state.search.clone()
        }
    ))
    .style(search_style)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(styles::panel_border(config, search_focused))
            .title(Line::from(Span::styled(
                "Search",
                styles::panel_title(config, search_focused),
            )))
            .style(styles::panel(config)),
    );
    frame.render_widget(search, sidebar[0]);
    state.layout.search = Some(sidebar[0]);

    state.layout.sidebar_items.clear();
    let mut list_state = ListState::default();
    let selected_index = vm
        .tree_items
        .iter()
        .position(|item| item.path == state.selected_path)
        .unwrap_or(0);
    list_state.select(Some(selected_index));

    let list_items = vm
        .tree_items
        .iter()
        .map(|item| ListItem::new(Line::from(item.label.clone())))
        .collect::<Vec<_>>();

    let list = List::new(list_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(styles::panel_border(config, sidebar_focused))
                .title(Line::from(Span::styled(
                    "Commands",
                    styles::panel_title(config, sidebar_focused),
                )))
                .style(styles::panel(config)),
        )
        .style(
            Style::default()
                .fg(config.theme.dim)
                .bg(config.theme.panel_bg),
        )
        .highlight_style(if sidebar_focused {
            styles::list_highlight(config)
        } else {
            styles::list_highlight_unfocused(config)
        })
        .highlight_symbol("> ");

    frame.render_stateful_widget(list, sidebar[1], &mut list_state);
    state.layout.sidebar = Some(sidebar[1]);

    let list_area = sidebar[1];
    let content_y = list_area.y.saturating_add(1);
    let content_x = list_area.x.saturating_add(1);
    let content_height = list_area.height.saturating_sub(2) as usize;
    for (index, item) in vm.tree_items.iter().take(content_height).enumerate() {
        let row_y = content_y.saturating_add(index as u16);
        let row_rect = Rect::new(content_x, row_y, list_area.width.saturating_sub(2), 1);
        let caret = if item.has_children {
            let caret_x = content_x.saturating_add(item.indent as u16);
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
