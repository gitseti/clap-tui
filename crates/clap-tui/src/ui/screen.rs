use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Margin, Rect};
use ratatui::widgets::{Block, BorderType, Borders};

use crate::config::TuiConfig;
use crate::input::{ActiveTab, AppState, CommandInputs, Focus};
use crate::spec::{ArgSpec, CommandSpec};
use crate::view::argv;
use crate::view::command_tree::{self, TreeItem};
use crate::view::form;

use super::{dropdown, footer, form as form_ui, header, preview, sidebar, styles, toast};

#[derive(Debug, Clone)]
pub(crate) struct ScreenArg {
    pub(crate) order_index: usize,
    pub(crate) arg: ArgSpec,
}

#[derive(Debug, Clone)]
pub(crate) struct ScreenView {
    pub(crate) command: CommandSpec,
    pub(crate) tree_items: Vec<TreeItem>,
    pub(crate) visible_tabs: Vec<ActiveTab>,
    pub(crate) active_args: Vec<ScreenArg>,
    pub(crate) preview_argv: Vec<String>,
    pub(crate) inputs: Option<CommandInputs>,
}

impl ScreenView {
    pub(crate) fn build(state: &AppState) -> Self {
        let command = state.current_command().clone();
        let active_args = form::visible_args(&command, state.command.active_tab)
            .into_iter()
            .map(|item| ScreenArg {
                order_index: item.order_index,
                arg: item.arg.clone(),
            })
            .collect();
        Self {
            command,
            tree_items: command_tree::tree_items(
                &state.command.root,
                &state.command.expanded,
                &state.command.search,
            ),
            visible_tabs: AppState::visible_tabs().to_vec(),
            active_args,
            preview_argv: argv::build_argv(state),
            inputs: state.current_inputs().cloned(),
        }
    }

    pub(crate) fn ordered_active_args(&self) -> Vec<form::OrderedArg<'_>> {
        self.active_args
            .iter()
            .map(|item| form::OrderedArg {
                order_index: item.order_index,
                arg: &item.arg,
            })
            .collect()
    }
}

pub(crate) fn render(frame: &mut Frame<'_>, state: &mut AppState, config: &TuiConfig) {
    let size = frame.area();
    let mut sidebar_width =
        u16::try_from(u32::from(size.width) * u32::from(config.layout.sidebar_ratio) / 100)
            .unwrap_or(size.width);
    sidebar_width = sidebar_width.clamp(22, 30);

    state.ensure_defaults();
    let current_command = state.current_command().clone();
    let active_args = form::visible_args(&current_command, state.command.active_tab);
    let visible = active_args
        .iter()
        .map(|item| (item.order_index, item.arg))
        .collect::<Vec<_>>();
    state.ensure_active_tab_visible(&visible);
    if state.command.active_tab != ActiveTab::Help {
        state.ensure_selected_arg_visible(&visible);
    }

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

    state.layout.sidebar = Some(sidebar_area);
    state.layout.preview = Some(preview_area);
    state.layout.footer = Some(footer_area);

    let vm = ScreenView::build(state);
    render_main(frame, state, config, main_area, &vm);
    sidebar::render_sidebar(frame, state, config, sidebar_area, &vm);
    dropdown::render_dropdown(frame, state, config, Rect::default(), &vm);
    preview::render_preview(frame, state, config, preview_area, &vm);
    footer::render_footer(frame, state, config, footer_area, &vm);
    toast::render_toast(frame, state, config, size);
}

fn render_main(
    frame: &mut Frame<'_>,
    state: &mut AppState,
    config: &TuiConfig,
    area: Rect,
    vm: &ScreenView,
) {
    let workspace_focused = matches!(state.interaction.focus, Focus::Form);
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

    header::render_header(frame, state, config, main[0], vm);
    form_ui::render_form(frame, state, config, body_area, vm);
}
