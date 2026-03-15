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
        Style::default().bg(config.theme.surface_raised)
    } else {
        Style::default().bg(config.theme.input_bg)
    }
}

pub(crate) fn flag_toggle(config: &TuiConfig, selected: bool) -> Style {
    if selected {
        Style::default()
            .bg(config.theme.surface_raised)
            .fg(config.theme.selection_fg)
    } else {
        Style::default().bg(config.theme.panel_bg)
    }
}

pub(crate) fn compact_control(config: &TuiConfig, selected: bool) -> Style {
    let bg = if selected {
        config.theme.selection_bg
    } else {
        config.theme.surface_raised
    };
    Style::default().bg(bg)
}

pub(crate) fn compact_control_value(config: &TuiConfig, selected: bool, is_default: bool) -> Style {
    let mut style = Style::default().fg(if is_default {
        config.theme.dim
    } else {
        config.theme.text
    });
    if selected {
        style = style.add_modifier(Modifier::BOLD);
    }
    style
}

pub(crate) fn compact_control_affordance(
    config: &TuiConfig,
    selected: bool,
    emphasized: bool,
) -> Style {
    let fg = if emphasized {
        config.theme.panel_bg
    } else {
        config.theme.accent
    };
    let bg = if emphasized || selected {
        config.theme.accent
    } else {
        config.theme.focus_bg
    };
    Style::default().fg(fg).bg(bg).add_modifier(Modifier::BOLD)
}

pub(crate) fn checkbox_chip(config: &TuiConfig, selected: bool, enabled: bool) -> Style {
    let bg = if enabled {
        config.theme.accent
    } else if selected {
        config.theme.focus_bg
    } else {
        config.theme.surface_raised
    };
    let fg = if enabled {
        config.theme.panel_bg
    } else if selected {
        config.theme.selection_fg
    } else {
        config.theme.text
    };
    Style::default().fg(fg).bg(bg).add_modifier(Modifier::BOLD)
}

pub(crate) fn selection(config: &TuiConfig) -> Style {
    Style::default()
        .fg(config.theme.selection_fg)
        .bg(config.theme.selection_bg)
        .add_modifier(Modifier::BOLD)
}

pub(crate) fn label(config: &TuiConfig, selected: bool) -> Style {
    if selected {
        Style::default()
            .fg(config.theme.text)
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
    selection(config)
}

pub(crate) fn list_highlight_unfocused(config: &TuiConfig) -> Style {
    Style::default()
        .fg(config.theme.selection_fg)
        .bg(config.theme.surface_raised)
}

pub(crate) fn subtle_chip(config: &TuiConfig, hovered: bool) -> Style {
    let style = Style::default()
        .fg(config.theme.dim)
        .bg(config.theme.pill_bg);
    if hovered {
        style.fg(config.theme.text).add_modifier(Modifier::BOLD)
    } else {
        style
    }
}

pub(crate) fn secondary_chip(config: &TuiConfig, hovered: bool) -> Style {
    let style = Style::default()
        .fg(config.theme.text)
        .bg(config.theme.pill_bg);
    if hovered {
        style
            .fg(config.theme.panel_bg)
            .bg(config.theme.accent)
            .add_modifier(Modifier::BOLD)
    } else {
        style
    }
}

pub(crate) fn primary_chip(config: &TuiConfig, hovered: bool) -> Style {
    let style = Style::default()
        .fg(config.theme.text)
        .bg(config.theme.pill_bg);
    if hovered {
        style
            .fg(config.theme.panel_bg)
            .bg(config.theme.accent)
            .add_modifier(Modifier::BOLD)
    } else {
        style
    }
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
