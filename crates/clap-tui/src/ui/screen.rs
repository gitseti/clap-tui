use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders};
use std::collections::HashSet;

use crate::config::TuiConfig;
use crate::frame_snapshot::FrameSnapshot;
use crate::input::{ActiveTab, AppState, CommandFormState, Focus, UiState};
use crate::pipeline::{self, ValidationState};
use crate::query::{
    form,
    tree::{self, TreeItem},
};
use crate::spec::{CommandPath, CommandSpec};

use super::{dropdown, footer, form as form_ui, header, layout, preview, sidebar, styles, toast};

#[derive(Debug, Clone)]
pub(crate) struct ScreenView<'a> {
    pub(crate) command: &'a CommandSpec,
    pub(crate) root: &'a CommandSpec,
    pub(crate) tree_items: Vec<TreeItem>,
    pub(crate) active_args: Vec<form::OrderedArg<'a>>,
    pub(crate) preview_argv: Vec<String>,
    pub(crate) validation: ValidationState,
    pub(crate) inputs: Option<&'a CommandFormState>,
}

impl<'a> ScreenView<'a> {
    pub(crate) fn build(
        command: &'a CommandSpec,
        root: &'a CommandSpec,
        expanded: &HashSet<String>,
        search_query: &str,
        active_tab: ActiveTab,
        inputs: Option<&'a CommandFormState>,
        derived: pipeline::DerivedState,
    ) -> Self {
        Self {
            command,
            root,
            tree_items: tree::tree_items(root, expanded, search_query),
            active_args: form::visible_args(command, active_tab),
            preview_argv: derived.argv,
            validation: derived.validation,
            inputs,
        }
    }
}

pub(crate) fn render(
    frame: &mut Frame<'_>,
    state: &mut AppState,
    config: &TuiConfig,
) -> FrameSnapshot {
    let size = frame.area();

    let background = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(styles::panel_border(config, false))
        .style(styles::panel(config));
    frame.render_widget(background, size);
    let derived = pipeline::derive(state);
    let selected_path = state.domain.selected_path().clone();
    let vm = ScreenView::build(
        state.domain.current_command(),
        &state.domain.root,
        &state.domain.expanded,
        &state.ui.search_query,
        state.ui.active_tab,
        state.domain.current_form(),
        derived,
    );
    let screen_layout = layout::build_screen_layout(&state.ui, config, size, &vm);
    let frame_snapshot = screen_layout.snapshot.clone();
    render_main(
        frame,
        &state.ui,
        &selected_path,
        config,
        screen_layout.areas.main,
        screen_layout.areas.header,
        &vm,
        &frame_snapshot,
    );
    sidebar::render_sidebar(
        frame,
        &state.ui,
        &selected_path,
        config,
        screen_layout.areas.sidebar,
        &vm,
        &frame_snapshot.layout,
    );
    dropdown::render_dropdown(
        frame,
        &state.ui,
        &frame_snapshot,
        &state.domain,
        config,
        Rect::default(),
        &vm,
    );
    preview::render_preview(frame, &state.ui, config, screen_layout.areas.preview, &vm);
    footer::render_footer(
        frame,
        &state.ui,
        config,
        screen_layout.areas.footer,
        &vm,
        &frame_snapshot.layout,
    );
    toast::render_toast(frame, state, config, size);
    frame_snapshot
}

#[allow(clippy::too_many_arguments)]
fn render_main(
    frame: &mut Frame<'_>,
    ui: &UiState,
    selected_path: &CommandPath,
    config: &TuiConfig,
    area: Rect,
    header_area: Rect,
    vm: &ScreenView<'_>,
    frame_snapshot: &FrameSnapshot,
) {
    let workspace_focused = matches!(ui.focus, Focus::Form);
    let workspace = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(styles::panel_border(config, workspace_focused))
        .title(workspace_title(config, vm.command))
        .style(styles::panel(config));
    frame.render_widget(workspace, area);
    if header_area.height == 0 || header_area.width == 0 {
        return;
    }

    header::render_header(frame, config, header_area, vm);
    form_ui::render_form(frame, ui, selected_path, config, vm, frame_snapshot);
}

fn workspace_title(config: &TuiConfig, command: &CommandSpec) -> Line<'static> {
    Line::from(vec![
        Span::raw(" "),
        Span::styled(command.name.clone(), Style::default().fg(config.theme.text)),
        Span::raw(" "),
    ])
}
