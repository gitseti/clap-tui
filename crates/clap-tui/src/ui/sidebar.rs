use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};

use crate::config::TuiConfig;
use crate::frame_snapshot::{FrameLayout, SidebarItemLayout};
use crate::input::{Focus, UiState};
use crate::query::tree::TreeItem;
use crate::spec::CommandPath;

use super::screen::ScreenView;
use super::styles;

pub(crate) fn populate_layout(area: Rect, vm: &ScreenView<'_>, frame_layout: &mut FrameLayout) {
    frame_layout.sidebar = Some(area);
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

    frame_layout.search = Some(sidebar[0]);
    frame_layout.sidebar_items.clear();
    let list_area = sidebar[2];
    let content_y = list_area.y;
    let content_x = list_area.x;
    let content_height = usize::from(list_area.height);
    for (index, item) in vm.tree_items.iter().take(content_height).enumerate() {
        let row_y = content_y.saturating_add(u16::try_from(index).unwrap_or(list_area.height));
        let row_rect = Rect::new(content_x, row_y, list_area.width, 1);
        let caret = if item.has_children {
            let caret_x = content_x
                .saturating_add(u16::try_from(item.indent).unwrap_or(0))
                .saturating_add(2);
            Some(Rect::new(caret_x, row_y, 1, 1))
        } else {
            None
        };
        frame_layout.sidebar_items.push(SidebarItemLayout {
            path: item.path.clone(),
            row: row_rect,
            caret,
            has_children: item.has_children,
        });
    }
}

pub(crate) fn render_sidebar(
    frame: &mut Frame<'_>,
    ui: &UiState,
    selected_path: &CommandPath,
    config: &TuiConfig,
    area: Rect,
    vm: &ScreenView<'_>,
    frame_layout: &FrameLayout,
) {
    let search_focused = matches!(ui.focus, Focus::Search);
    let sidebar_focused = matches!(ui.focus, Focus::Sidebar);
    let panel_focused = search_focused || sidebar_focused;
    let root_selected = selected_path.is_empty();
    let panel = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(styles::panel_border(config, panel_focused))
        .title(sidebar_title(config, vm.root, root_selected))
        .style(styles::panel(config));
    frame.render_widget(panel, area);
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

    let mut search_style = if ui.search_query.is_empty() {
        styles::placeholder(config)
    } else {
        Style::default().fg(config.theme.text)
    };
    if search_focused {
        search_style = search_style.add_modifier(Modifier::BOLD);
        search_style = search_style.fg(config.theme.text);
    }

    let search = Paragraph::new(if ui.search_query.is_empty() {
        "/ search commands".to_string()
    } else {
        ui.search_query.clone()
    })
    .style(search_style.bg(if search_focused {
        config.theme.surface_raised
    } else {
        config.theme.input_bg
    }));
    frame.render_widget(search, sidebar[0]);
    frame.render_widget(
        Paragraph::new("─".repeat(sidebar[1].width as usize))
            .style(Style::default().fg(config.theme.divider)),
        sidebar[1],
    );

    for layout in &frame_layout.sidebar_items {
        let Some(item) = vm.tree_items.iter().find(|item| item.path == layout.path) else {
            continue;
        };
        let selected = item.path == *selected_path;
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
        let line = sidebar_line(config, item, rail, row_style, selected);
        frame.render_widget(Paragraph::new(line).style(row_style), layout.row);
    }
}

fn sidebar_title(
    config: &TuiConfig,
    root: &crate::spec::CommandSpec,
    root_selected: bool,
) -> Line<'static> {
    let mut spans = vec![
        Span::raw(" "),
        Span::styled(
            root.name.clone(),
            Style::default().fg(if root_selected {
                config.theme.text
            } else {
                config.theme.dim
            }),
        ),
    ];
    if let Some(version) = root.version.as_ref() {
        spans.push(Span::raw(" · "));
        spans.push(Span::styled(
            version.clone(),
            Style::default().fg(if root_selected {
                config.theme.text
            } else {
                config.theme.dim
            }),
        ));
    }
    spans.push(Span::raw(" "));
    Line::from(spans)
}

fn sidebar_line(
    config: &TuiConfig,
    item: &TreeItem,
    rail: &'static str,
    row_style: Style,
    selected: bool,
) -> Line<'static> {
    let mut spans = vec![
        Span::styled(
            rail,
            Style::default().fg(if selected {
                config.theme.accent
            } else {
                config.theme.panel_bg
            }),
        ),
        Span::raw(" "),
        Span::styled(item.prefix(), row_style),
    ];
    if item.path.is_empty() {
        spans.push(Span::styled(
            item.name.clone(),
            Style::default()
                .fg(config.theme.accent)
                .add_modifier(Modifier::BOLD | Modifier::ITALIC),
        ));
        if let Some(version) = &item.version {
            spans.push(Span::raw(" "));
            spans.push(Span::styled(
                version.clone(),
                Style::default().fg(if selected {
                    config.theme.selection_fg
                } else {
                    config.theme.dim
                }),
            ));
        }
    } else {
        spans.push(Span::styled(item.name.clone(), row_style));
    }
    Line::from(spans)
}

#[cfg(test)]
mod tests {
    use ratatui::style::Modifier;

    use super::{sidebar_line, sidebar_title};
    use crate::config::TuiConfig;
    use crate::query::tree::TreeItem;
    use crate::spec::{CommandPath, CommandSpec};

    #[test]
    fn sidebar_title_uses_binary_styling_and_shows_version() {
        let config = TuiConfig::default();
        let root = CommandSpec {
            name: "ls".to_string(),
            version: Some("1.2.3".to_string()),
            about: None,
            help: String::new(),
            args: Vec::new(),
            subcommands: Vec::new(),
        };

        let line = sidebar_title(&config, &root, false);

        assert_eq!(line.spans[0].content.as_ref(), " ");
        assert_eq!(line.spans[1].content.as_ref(), "ls");
        assert_eq!(line.spans[1].style.fg, Some(config.theme.dim));
        assert!(!line.spans[1].style.add_modifier.contains(Modifier::BOLD));
        assert!(!line.spans[1].style.add_modifier.contains(Modifier::ITALIC));
        assert_eq!(line.spans[2].content.as_ref(), " · ");
        assert_eq!(line.spans[3].content.as_ref(), "1.2.3");
        assert_eq!(line.spans[3].style.fg, Some(config.theme.dim));
        assert_eq!(line.spans[4].content.as_ref(), " ");
    }

    #[test]
    fn non_root_rows_keep_standard_row_styling() {
        let config = TuiConfig::default();
        let item = TreeItem {
            name: "serve".to_string(),
            version: None,
            path: CommandPath::from(vec!["serve".to_string()]),
            has_children: false,
            indent: 0,
            expanded: false,
        };

        let line = sidebar_line(&config, &item, " ", ratatui::style::Style::default(), false);

        assert_eq!(line.spans[3].content.as_ref(), "serve");
        assert_ne!(line.spans[3].style.fg, Some(config.theme.accent));
    }
}
