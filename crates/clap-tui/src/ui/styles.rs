use ratatui::style::{Modifier, Style};

use crate::config::TuiConfig;

pub(crate) fn panel(config: &TuiConfig) -> Style {
    Style::default().bg(config.theme.panel_bg)
}

pub(crate) fn header(config: &TuiConfig) -> Style {
    Style::default().bg(config.theme.header_bg)
}

pub(crate) fn input(config: &TuiConfig, selected: bool) -> Style {
    if selected {
        Style::default().bg(config.theme.focus_bg)
    } else {
        Style::default().bg(config.theme.input_bg)
    }
}

pub(crate) fn label(config: &TuiConfig, selected: bool) -> Style {
    if selected {
        Style::default()
            .fg(config.theme.accent)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(config.theme.dim)
    }
}

pub(crate) fn help(config: &TuiConfig) -> Style {
    Style::default().fg(config.theme.dim)
}

pub(crate) fn placeholder(config: &TuiConfig) -> Style {
    Style::default().fg(config.theme.dim)
}

pub(crate) fn list_highlight(config: &TuiConfig) -> Style {
    Style::default()
        .fg(config.theme.accent)
        .bg(config.theme.focus_bg)
        .add_modifier(Modifier::BOLD)
}
