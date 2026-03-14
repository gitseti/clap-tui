use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Margin, Rect};
use ratatui::widgets::{Block, BorderType, Borders};

use crate::config::TuiConfig;
use crate::input::{ActiveTab, AppState, CommandInputs};
use crate::spec::{ArgSpec, CommandSpec};
use crate::view::argv;
use crate::view::command_tree::{self, TreeItem};
use crate::view::form;

use super::{dropdown, footer, form as form_ui, header, preview, sidebar, styles};

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
        let active_args = form::visible_args(&command, state.active_tab)
            .into_iter()
            .map(|item| ScreenArg {
                order_index: item.order_index,
                arg: item.arg.clone(),
            })
            .collect();
        Self {
            command,
            tree_items: command_tree::tree_items(&state.root, &state.expanded, &state.search),
            visible_tabs: state.visible_tabs(),
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
    let sidebar_width = (size.width as u32 * config.layout.sidebar_ratio as u32 / 100) as u16;

    state.ensure_defaults();
    state.ensure_active_tab_visible();
    if state.active_tab != ActiveTab::Help {
        state.ensure_selected_arg_visible();
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
    state.layout.footer = Some(footer_area);

    let vm = ScreenView::build(state);
    render_main(frame, state, config, main_area, &vm);
    sidebar::render_sidebar(frame, state, config, sidebar_area, &vm);
    dropdown::render_dropdown(frame, state, config, Rect::default(), &vm);
    preview::render_preview(frame, state, config, preview_area, &vm);
    footer::render_footer(frame, state, config, footer_area, &vm);
}

fn render_main(
    frame: &mut Frame<'_>,
    state: &mut AppState,
    config: &TuiConfig,
    area: Rect,
    vm: &ScreenView,
) {
    let main = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(8)])
        .split(area);

    header::render_header(frame, state, config, main[0], vm);
    form_ui::render_form(frame, state, config, main[1], vm);
}
