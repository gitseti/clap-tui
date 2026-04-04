use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, BorderType, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
    StatefulWidget,
};

use crate::config::TuiConfig;
use crate::frame_snapshot::{FrameLayout, SidebarItemLayout};
use crate::input::{Focus, UiState};
use crate::query::tree::{TreeItem, TreeRow};
use crate::spec::CommandPath;

use super::screen::ScreenView;
use super::styles::{self, SidebarRowState};

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
    frame_layout.sidebar_list = Some(list_area);
    let content_y = list_area.y;
    let content_x = list_area.x;
    let content_height = usize::from(list_area.height);
    let scroll = vm
        .sidebar_scroll
        .min(vm.tree_rows.len().saturating_sub(content_height));
    for (index, row) in vm
        .tree_rows
        .iter()
        .skip(scroll)
        .take(content_height)
        .enumerate()
    {
        let TreeRow::Item(item) = row else {
            continue;
        };
        let row_y = content_y.saturating_add(u16::try_from(index).unwrap_or(list_area.height));
        let row_rect = Rect::new(content_x, row_y, list_area.width, 1);
        let caret = if item.has_children {
            let caret_x = content_x
                .saturating_add(u16::try_from(item.indent).unwrap_or(0))
                .saturating_add(0);
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
    _frame_layout: &FrameLayout,
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
    let (search_area, divider_area, list_area) = sidebar_sections(area);
    render_search(frame, ui, config, search_area, divider_area, search_focused);

    let list_view = SidebarListView::new(ui, vm, list_area);
    render_rows(frame, selected_path, config, vm, list_view, sidebar_focused);
    render_scrollbar(frame, config, list_view, vm.tree_rows.len(), panel_focused);
}

#[derive(Debug, Clone, Copy)]
struct SidebarListView {
    area: Rect,
    visible_rows: usize,
    row_width: u16,
    scroll: usize,
    needs_scrollbar: bool,
}

impl SidebarListView {
    fn new(ui: &UiState, vm: &ScreenView<'_>, area: Rect) -> Self {
        let visible_rows = usize::from(area.height);
        let needs_scrollbar = vm.tree_rows.len() > visible_rows && visible_rows > 0;
        let scroll = ui
            .sidebar_scroll(vm.tree_rows.len(), visible_rows)
            .min(vm.tree_rows.len().saturating_sub(visible_rows));
        Self {
            area,
            visible_rows,
            row_width: area.width.saturating_sub(u16::from(needs_scrollbar)),
            scroll,
            needs_scrollbar,
        }
    }
}

fn sidebar_sections(area: Rect) -> (Rect, Rect, Rect) {
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
    (sidebar[0], sidebar[1], sidebar[2])
}

fn render_search(
    frame: &mut Frame<'_>,
    ui: &UiState,
    config: &TuiConfig,
    search_area: Rect,
    divider_area: Rect,
    search_focused: bool,
) {
    let mut search_style = if ui.search_query.is_empty() {
        styles::placeholder(config)
    } else {
        Style::default().fg(config.theme.text)
    };
    if search_focused {
        search_style = search_style
            .add_modifier(Modifier::BOLD)
            .fg(config.theme.text);
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
    frame.render_widget(search, search_area);
    frame.render_widget(
        Paragraph::new("─".repeat(divider_area.width as usize))
            .style(Style::default().fg(config.theme.divider)),
        divider_area,
    );
}

fn render_rows(
    frame: &mut Frame<'_>,
    selected_path: &CommandPath,
    config: &TuiConfig,
    vm: &ScreenView<'_>,
    list_view: SidebarListView,
    sidebar_focused: bool,
) {
    for (index, row) in vm
        .tree_rows
        .iter()
        .skip(list_view.scroll)
        .take(list_view.visible_rows)
        .enumerate()
    {
        let row_y = list_view
            .area
            .y
            .saturating_add(u16::try_from(index).unwrap_or(list_view.area.height));
        let row_rect = Rect::new(list_view.area.x, row_y, list_view.row_width, 1);
        render_row(frame, selected_path, config, row, row_rect, sidebar_focused);
    }
}

fn render_row(
    frame: &mut Frame<'_>,
    selected_path: &CommandPath,
    config: &TuiConfig,
    row: &TreeRow,
    row_rect: Rect,
    sidebar_focused: bool,
) {
    match row {
        TreeRow::Item(item) => {
            let selected = item.path == *selected_path;
            let row_style = sidebar_row_style(config, item, selected, sidebar_focused);
            let line = sidebar_line(config, item, row_style, selected);
            frame.render_widget(Paragraph::new(line).style(row_style), row_rect);
        }
        TreeRow::Heading { title, indent } => {
            frame.render_widget(
                Paragraph::new(sidebar_heading_line(config, title, *indent))
                    .style(Style::default().bg(config.theme.panel_bg)),
                row_rect,
            );
        }
    }
}

fn sidebar_row_style(
    config: &TuiConfig,
    item: &TreeItem,
    selected: bool,
    sidebar_focused: bool,
) -> Style {
    if selected {
        return if sidebar_focused {
            styles::list_highlight(config)
        } else {
            styles::list_highlight_unfocused(config)
        };
    }

    let mut style = styles::sidebar_row(
        config,
        if item.indent > 0 {
            SidebarRowState::IdleChild
        } else {
            SidebarRowState::IdleRoot
        },
    );
    if item.has_children || item.indent == 0 {
        style = style.add_modifier(Modifier::BOLD);
    }

    style
}

fn render_scrollbar(
    frame: &mut Frame<'_>,
    config: &TuiConfig,
    list_view: SidebarListView,
    total_rows: usize,
    panel_focused: bool,
) {
    if !list_view.needs_scrollbar {
        return;
    }

    let scroll_steps = total_rows
        .saturating_sub(list_view.visible_rows)
        .saturating_add(1);
    let mut scrollbar_state = ScrollbarState::new(scroll_steps)
        .position(list_view.scroll)
        .viewport_content_length(list_view.visible_rows);
    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .track_symbol(Some("┃"))
        .thumb_symbol("█")
        .begin_style(styles::scrollbar_cap(config, panel_focused))
        .end_style(styles::scrollbar_cap(config, panel_focused))
        .thumb_style(styles::scrollbar_thumb(config, panel_focused))
        .track_style(styles::scrollbar_track(config));
    StatefulWidget::render(
        scrollbar,
        list_view.area,
        frame.buffer_mut(),
        &mut scrollbar_state,
    );
}

fn sidebar_heading_line(config: &TuiConfig, title: &str, indent: usize) -> Line<'static> {
    Line::from(vec![
        Span::raw(" ".repeat(indent)),
        Span::styled("· ", Style::default().fg(config.theme.divider)),
        Span::styled(
            title.to_string(),
            Style::default()
                .fg(config.theme.dim)
                .add_modifier(Modifier::BOLD),
        ),
    ])
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
            Style::default()
                .fg(if root_selected {
                    config.theme.text
                } else {
                    config.theme.dim
                })
                .add_modifier(Modifier::BOLD),
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
    row_style: Style,
    selected: bool,
) -> Line<'static> {
    let prefix_style = if selected {
        row_style
    } else if item.has_children {
        Style::default()
            .fg(config.theme.accent)
            .add_modifier(Modifier::BOLD)
    } else if item.indent > 0 {
        Style::default().fg(config.theme.dim)
    } else {
        Style::default()
            .fg(config.theme.text)
            .add_modifier(Modifier::BOLD)
    };
    let mut spans = vec![Span::styled(item.prefix(), prefix_style)];
    if item.path.is_empty() {
        spans.push(Span::styled(
            item.display_label.clone(),
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
        spans.push(Span::styled(item.display_label.clone(), row_style));
    }
    Line::from(spans)
}

#[cfg(test)]
mod tests {
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui::layout::Rect;
    use ratatui::style::Modifier;

    use super::{
        populate_layout, render_sidebar, sidebar_heading_line, sidebar_line, sidebar_row_style,
        sidebar_title,
    };
    use crate::config::TuiConfig;
    use crate::frame_snapshot::FrameLayout;
    use crate::input::{Focus, UiState};
    use crate::query::tree::{TreeItem, TreeRow};
    use crate::spec::{CommandPath, CommandSpec};
    use crate::ui::screen::ScreenView;

    fn ui_state() -> UiState {
        UiState {
            focus: Focus::Sidebar,
            active_tab: crate::input::ActiveTab::Inputs,
            last_non_help_tab: crate::input::ActiveTab::Inputs,
            help_open: false,
            help_scroll: 0,
            selected_arg_index: 0,
            search_query: String::new(),
            editors: crate::editor_state::EditorState::default(),
            dropdown_open: None,
            dropdown_scroll: 0,
            dropdown_cursor: 0,
            sidebar_scroll: 1,
            form_scroll: 0,
            hover: None,
            hover_tab: None,
            mouse_select: None,
        }
    }

    fn buffer_text(backend: &TestBackend) -> String {
        backend
            .buffer()
            .content
            .iter()
            .map(ratatui::buffer::Cell::symbol)
            .collect::<String>()
    }

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
            ..CommandSpec::default()
        };

        let line = sidebar_title(&config, &root, false);

        assert_eq!(line.spans[0].content.as_ref(), " ");
        assert_eq!(line.spans[1].content.as_ref(), "ls");
        assert_eq!(line.spans[1].style.fg, Some(config.theme.dim));
        assert!(line.spans[1].style.add_modifier.contains(Modifier::BOLD));
        assert_eq!(line.spans[2].content.as_ref(), " · ");
        assert_eq!(line.spans[3].content.as_ref(), "1.2.3");
        assert_eq!(line.spans[3].style.fg, Some(config.theme.dim));
        assert_eq!(line.spans[4].content.as_ref(), " ");
    }

    #[test]
    fn sidebar_heading_line_marks_sections_with_a_subtle_marker() {
        let config = TuiConfig::default();

        let line = sidebar_heading_line(&config, "Applets", 2);

        assert_eq!(line.spans[1].content.as_ref(), "· ");
        assert_eq!(line.spans[1].style.fg, Some(config.theme.divider));
        assert_eq!(line.spans[2].content.as_ref(), "Applets");
    }

    #[test]
    fn root_leaf_rows_use_stronger_hierarchy_typography() {
        let config = TuiConfig::default();
        let item = TreeItem {
            name: "serve".to_string(),
            display_label: "serve (srv)".to_string(),
            version: None,
            path: CommandPath::from(vec!["serve".to_string()]),
            has_children: false,
            indent: 0,
            expanded: false,
        };

        let row_style = sidebar_row_style(&config, &item, false, true);
        let line = sidebar_line(&config, &item, row_style, false);

        assert_eq!(line.spans[1].content.as_ref(), "serve (srv)");
        assert_eq!(line.spans[1].style.fg, Some(config.theme.text));
        assert!(line.spans[1].style.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn branch_rows_use_prominent_prefixes_for_tree_hierarchy() {
        let config = TuiConfig::default();
        let item = TreeItem {
            name: "workflow".to_string(),
            display_label: "workflow".to_string(),
            version: None,
            path: CommandPath::from(vec!["workflow".to_string()]),
            has_children: true,
            indent: 0,
            expanded: false,
        };

        let row_style = sidebar_row_style(&config, &item, false, true);
        let line = sidebar_line(&config, &item, row_style, false);

        assert_eq!(line.spans[0].style.fg, Some(config.theme.accent));
        assert!(line.spans[1].style.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn sidebar_heading_line_renders_heading_text() {
        let config = TuiConfig::default();

        let line = sidebar_heading_line(&config, "Applets", 2);

        assert_eq!(line.spans[1].content.as_ref(), "· ");
        assert_eq!(line.spans[2].content.as_ref(), "Applets");
    }

    #[test]
    fn nested_rows_render_visible_depth_guides() {
        let config = TuiConfig::default();
        let item = TreeItem {
            name: "release".to_string(),
            display_label: "release".to_string(),
            version: None,
            path: CommandPath::from(vec!["build".to_string(), "release".to_string()]),
            has_children: false,
            indent: 2,
            expanded: false,
        };

        let line = sidebar_line(&config, &item, ratatui::style::Style::default(), false);

        assert_eq!(line.spans[0].content.as_ref(), "| ");
        assert_eq!(line.spans[0].style.fg, Some(config.theme.dim));
    }

    #[test]
    fn populate_layout_uses_sidebar_scroll_for_visible_hit_targets() {
        let root = CommandSpec {
            name: "tool".to_string(),
            version: None,
            about: None,
            help: String::new(),
            args: Vec::new(),
            subcommands: Vec::new(),
            ..CommandSpec::default()
        };
        let vm = ScreenView {
            command: &root,
            root: &root,
            selected_path: CommandPath::default(),
            tree_rows: vec![
                TreeRow::Item(TreeItem {
                    name: "alpha".to_string(),
                    display_label: "alpha".to_string(),
                    version: None,
                    path: CommandPath::from(vec!["alpha".to_string()]),
                    has_children: false,
                    indent: 0,
                    expanded: false,
                }),
                TreeRow::Item(TreeItem {
                    name: "beta".to_string(),
                    display_label: "beta".to_string(),
                    version: None,
                    path: CommandPath::from(vec!["beta".to_string()]),
                    has_children: false,
                    indent: 0,
                    expanded: false,
                }),
                TreeRow::Item(TreeItem {
                    name: "gamma".to_string(),
                    display_label: "gamma".to_string(),
                    version: None,
                    path: CommandPath::from(vec!["gamma".to_string()]),
                    has_children: false,
                    indent: 0,
                    expanded: false,
                }),
                TreeRow::Item(TreeItem {
                    name: "delta".to_string(),
                    display_label: "delta".to_string(),
                    version: None,
                    path: CommandPath::from(vec!["delta".to_string()]),
                    has_children: false,
                    indent: 0,
                    expanded: false,
                }),
                TreeRow::Item(TreeItem {
                    name: "epsilon".to_string(),
                    display_label: "epsilon".to_string(),
                    version: None,
                    path: CommandPath::from(vec!["epsilon".to_string()]),
                    has_children: false,
                    indent: 0,
                    expanded: false,
                }),
            ],
            sidebar_scroll: 1,
            active_args: Vec::new(),
            preview_argv: Vec::new(),
            validation: crate::pipeline::ValidationState::default(),
            effective_values: std::collections::BTreeMap::new(),
            inputs: None,
        };
        let mut layout = FrameLayout::default();

        populate_layout(Rect::new(0, 0, 20, 7), &vm, &mut layout);

        assert_eq!(layout.sidebar_items.len(), 3);
        assert_eq!(
            layout.sidebar_items[0].path.as_slice(),
            &["beta".to_string()]
        );
        assert_eq!(
            layout.sidebar_items[1].path.as_slice(),
            &["gamma".to_string()]
        );
        assert_eq!(
            layout.sidebar_items[2].path.as_slice(),
            &["delta".to_string()]
        );
    }

    #[test]
    fn sidebar_renders_scrollbar_when_tree_overflows_visible_rows() {
        let root = CommandSpec {
            name: "tool".to_string(),
            version: None,
            about: None,
            help: String::new(),
            args: Vec::new(),
            subcommands: Vec::new(),
            ..CommandSpec::default()
        };
        let vm = ScreenView {
            command: &root,
            root: &root,
            selected_path: CommandPath::default(),
            tree_rows: (0..8)
                .map(|index| {
                    TreeRow::Item(TreeItem {
                        name: format!("cmd-{index}"),
                        display_label: format!("cmd-{index}"),
                        version: None,
                        path: CommandPath::from(vec![format!("cmd-{index}")]),
                        has_children: false,
                        indent: 0,
                        expanded: false,
                    })
                })
                .collect(),
            sidebar_scroll: 2,
            active_args: Vec::new(),
            preview_argv: Vec::new(),
            validation: crate::pipeline::ValidationState::default(),
            effective_values: std::collections::BTreeMap::new(),
            inputs: None,
        };
        let mut terminal = Terminal::new(TestBackend::new(24, 8)).expect("terminal");

        terminal
            .draw(|frame| {
                render_sidebar(
                    frame,
                    &ui_state(),
                    &CommandPath::default(),
                    &TuiConfig::default(),
                    Rect::new(0, 0, 24, 8),
                    &vm,
                    &FrameLayout::default(),
                );
            })
            .expect("draw");

        let rendered = buffer_text(terminal.backend());
        assert!(rendered.contains('┃') || rendered.contains('█'));
    }

    #[test]
    fn sidebar_scrollbar_thumb_dims_when_sidebar_is_unfocused() {
        let root = CommandSpec {
            name: "tool".to_string(),
            version: None,
            about: None,
            help: String::new(),
            args: Vec::new(),
            subcommands: Vec::new(),
            ..CommandSpec::default()
        };
        let vm = ScreenView {
            command: &root,
            root: &root,
            selected_path: CommandPath::default(),
            tree_rows: (0..8)
                .map(|index| {
                    TreeRow::Item(TreeItem {
                        name: format!("cmd-{index}"),
                        display_label: format!("cmd-{index}"),
                        version: None,
                        path: CommandPath::from(vec![format!("cmd-{index}")]),
                        has_children: false,
                        indent: 0,
                        expanded: false,
                    })
                })
                .collect(),
            sidebar_scroll: 2,
            active_args: Vec::new(),
            preview_argv: Vec::new(),
            validation: crate::pipeline::ValidationState::default(),
            effective_values: std::collections::BTreeMap::new(),
            inputs: None,
        };
        let mut ui = ui_state();
        ui.focus = Focus::Form;
        let mut terminal = Terminal::new(TestBackend::new(24, 8)).expect("terminal");

        terminal
            .draw(|frame| {
                render_sidebar(
                    frame,
                    &ui,
                    &CommandPath::default(),
                    &TuiConfig::default(),
                    Rect::new(0, 0, 24, 8),
                    &vm,
                    &FrameLayout::default(),
                );
            })
            .expect("draw");

        let config = TuiConfig::default();
        let thumb_cell = terminal
            .backend()
            .buffer()
            .content
            .iter()
            .find(|cell| cell.symbol() == "█")
            .expect("scrollbar thumb");
        let arrow_cell = terminal
            .backend()
            .buffer()
            .content
            .iter()
            .find(|cell| cell.symbol() == "▲" || cell.symbol() == "▼")
            .expect("scrollbar arrow");
        assert_eq!(thumb_cell.fg, config.theme.dim);
        assert_eq!(arrow_cell.fg, config.theme.dim);
    }
}
