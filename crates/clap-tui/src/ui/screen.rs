use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Margin, Rect};
use ratatui::widgets::{Block, BorderType, Borders};
use std::collections::HashSet;

use crate::config::TuiConfig;
use crate::frame_snapshot::FrameSnapshot;
use crate::input::{ActiveTab, AppState, CommandFormState, Focus, UiState};
use crate::spec::{CommandPath, CommandSpec};
use crate::view::argv::build_argv;
use crate::view::command_tree::{self, TreeItem};
use crate::view::form;

use super::{dropdown, footer, form as form_ui, header, preview, sidebar, styles, toast};

#[derive(Debug, Clone)]
pub(crate) struct ScreenView<'a> {
    pub(crate) command: &'a CommandSpec,
    pub(crate) tree_items: Vec<TreeItem>,
    pub(crate) visible_tabs: [ActiveTab; 3],
    pub(crate) active_args: Vec<form::OrderedArg<'a>>,
    pub(crate) preview_argv: Vec<String>,
    pub(crate) inputs: Option<&'a CommandFormState>,
}

impl<'a> ScreenView<'a> {
    pub(crate) fn build(
        command: &'a CommandSpec,
        root: &CommandSpec,
        expanded: &HashSet<String>,
        search_query: &str,
        active_tab: ActiveTab,
        inputs: Option<&'a CommandFormState>,
        preview_argv: Vec<String>,
    ) -> Self {
        Self {
            command,
            tree_items: command_tree::tree_items(root, expanded, search_query),
            visible_tabs: UiState::visible_tabs(),
            active_args: form::visible_args(command, active_tab),
            preview_argv,
            inputs,
        }
    }
}

pub(crate) fn render(frame: &mut Frame<'_>, state: &mut AppState, config: &TuiConfig) -> FrameSnapshot {
    let size = frame.area();
    let mut sidebar_width =
        u16::try_from(u32::from(size.width) * u32::from(config.layout.sidebar_ratio) / 100)
            .unwrap_or(size.width);
    sidebar_width = sidebar_width.clamp(22, 30);

    let mut frame_snapshot = FrameSnapshot::default();

    let background = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(styles::panel_border(config, false))
        .style(styles::panel(config));
    frame.render_widget(background, size);
    let inner_size = size.inner(Margin {
        horizontal: 1,
        vertical: 1,
    });

    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(inner_size);

    let body_area = vertical[0];
    let preview_area = vertical[1];
    let footer_area = vertical[2];

    let root = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(
                sidebar_width
                    .max(20)
                    .min(body_area.width.saturating_sub(20)),
            ),
            Constraint::Min(20),
        ])
        .split(body_area);

    let sidebar_area = root[0];
    let main_area = root[1];

    frame_snapshot.layout.sidebar = Some(sidebar_area);
    frame_snapshot.layout.preview = Some(preview_area);
    frame_snapshot.layout.footer = Some(footer_area);

    let preview_argv = build_argv(state);
    let (domain, ui) = (&state.domain, &mut state.ui);
    let selected_path = domain.selected_path();
    let vm = ScreenView::build(
        domain.current_command(),
        &domain.root,
        &domain.expanded,
        &ui.search_query,
        ui.active_tab,
        domain.current_form(),
        preview_argv,
    );
    render_main(
        frame,
        ui,
        selected_path,
        config,
        main_area,
        &vm,
        &mut frame_snapshot,
    );
    sidebar::render_sidebar(
        frame,
        ui,
        selected_path,
        config,
        sidebar_area,
        &vm,
        &mut frame_snapshot.layout,
    );
    dropdown::render_dropdown(
        frame,
        ui,
        &frame_snapshot,
        domain,
        config,
        Rect::default(),
        &vm,
    );
    preview::render_preview(frame, ui, config, preview_area, &vm);
    footer::render_footer(
        frame,
        ui,
        config,
        footer_area,
        &vm,
        &mut frame_snapshot.layout,
    );
    toast::render_toast(frame, state, config, size);
    frame_snapshot
}

fn render_main(
    frame: &mut Frame<'_>,
    ui: &mut UiState,
    selected_path: &CommandPath,
    config: &TuiConfig,
    area: Rect,
    vm: &ScreenView<'_>,
    frame_snapshot: &mut FrameSnapshot,
) {
    let workspace_focused = matches!(ui.focus, Focus::Form);
    let workspace = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(styles::panel_border(config, workspace_focused))
        .style(styles::panel(config));
    frame.render_widget(workspace, area);

    let inner = area.inner(Margin {
        horizontal: 1,
        vertical: 1,
    });
    if inner.height == 0 || inner.width == 0 {
        return;
    }

    let header_height = inner.height.min(2);
    let body_area = Rect::new(
        inner.x,
        inner.y.saturating_add(header_height),
        inner.width,
        inner.height.saturating_sub(header_height),
    );

    let main = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(header_height), Constraint::Min(0)])
        .split(Rect::new(inner.x, inner.y, inner.width, inner.height));

    header::render_header(frame, selected_path, config, main[0], vm);
    form_ui::render_form(frame, ui, selected_path, config, body_area, vm, frame_snapshot);
}
