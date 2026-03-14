use ratatui::style::{Modifier, Style};

use crate::config::TuiConfig;

pub(crate) fn panel(config: &TuiConfig) -> Style {
    Style::default().bg(config.theme.panel_bg)
}

pub(crate) fn panel_border(config: &TuiConfig, focused: bool) -> Style {
    let color = if focused {
        config.theme.panel_focus_border
    } else {
        config.theme.border
    };
    Style::default().fg(color)
}

pub(crate) fn panel_title(config: &TuiConfig, focused: bool) -> Style {
    let color = if focused {
        config.theme.panel_focus_border
    } else {
        config.theme.dim
    };
    let style = Style::default().fg(color);
    if focused {
        style.add_modifier(Modifier::BOLD)
    } else {
        style
    }
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

pub(crate) fn flag_toggle(config: &TuiConfig, selected: bool) -> Style {
    input(config, selected)
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

pub(crate) fn list_highlight_unfocused(config: &TuiConfig) -> Style {
    Style::default()
        .fg(config.theme.text)
        .bg(config.theme.input_bg)
}

#[cfg(test)]
mod tests {
    use ratatui::style::Color;

    use crate::config::TuiConfig;

    use super::panel_border;

    #[test]
    fn panel_border_uses_focus_border_for_focused_panels() {
        let config = TuiConfig::default();

        assert_eq!(
            panel_border(&config, true).fg,
            Some(config.theme.panel_focus_border)
        );
    }

    #[test]
    fn panel_border_uses_inactive_border_for_unfocused_panels() {
        let config = TuiConfig::default();

        assert_eq!(panel_border(&config, false).fg, Some(config.theme.border));
        assert_ne!(panel_border(&config, false).fg, Some(Color::Reset));
    }
}
