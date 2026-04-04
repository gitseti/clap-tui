use ratatui::style::{Modifier, Style};

use crate::config::TuiConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Surface {
    Panel,
    Raised,
    Result,
    Overlay,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SidebarRowState {
    IdleRoot,
    IdleChild,
    ActiveFocused,
    ActiveUnfocused,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MetadataKind {
    Global,
    Inherited,
    Default,
    Env,
    Implicit,
    Conditional,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FooterChipKind {
    Primary,
    Secondary,
    Status,
    Subtle,
}

pub(crate) fn surface(config: &TuiConfig, surface: Surface) -> Style {
    let bg = match surface {
        Surface::Panel => config.theme.panel_bg,
        Surface::Raised => config.theme.surface_raised,
        Surface::Result => config.theme.preview_bg,
        Surface::Overlay => config.theme.overlay_bg,
    };
    Style::default().bg(bg)
}

pub(crate) fn panel(config: &TuiConfig) -> Style {
    surface(config, Surface::Panel)
}

pub(crate) fn panel_border(config: &TuiConfig, focused: bool) -> Style {
    Style::default().fg(if focused {
        config.theme.focus
    } else {
        config.theme.border
    })
}

pub(crate) fn input(config: &TuiConfig, selected: bool) -> Style {
    surface(
        config,
        if selected {
            Surface::Raised
        } else {
            Surface::Panel
        },
    )
    .bg(if selected {
        config.theme.surface_raised
    } else {
        config.theme.input_bg
    })
}

pub(crate) fn field_border(config: &TuiConfig, focused: bool, invalid: bool) -> Style {
    let color = if invalid && focused {
        config.theme.error
    } else if focused {
        config.theme.focus
    } else if invalid {
        config.theme.warning
    } else {
        config.theme.border
    };
    Style::default().fg(color)
}

pub(crate) fn flag_toggle(config: &TuiConfig, selected: bool) -> Style {
    input(config, selected).fg(if selected {
        config.theme.selection_fg
    } else {
        config.theme.text
    })
}

pub(crate) fn compact_control(config: &TuiConfig, selected: bool) -> Style {
    surface(
        config,
        if selected {
            Surface::Raised
        } else {
            Surface::Overlay
        },
    )
}

pub(crate) fn compact_control_value(config: &TuiConfig, selected: bool, is_default: bool) -> Style {
    let mut style = Style::default().fg(if is_default {
        config.theme.metadata
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
    if emphasized {
        Style::default()
            .fg(config.theme.panel_bg)
            .bg(config.theme.accent)
            .add_modifier(Modifier::BOLD)
    } else if selected {
        Style::default()
            .fg(config.theme.text)
            .bg(config.theme.focus_bg)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(config.theme.metadata)
            .bg(config.theme.pill_bg)
            .add_modifier(Modifier::BOLD)
    }
}

pub(crate) fn checkbox_chip(config: &TuiConfig, selected: bool, enabled: bool) -> Style {
    let (fg, bg) = if enabled {
        (config.theme.panel_bg, config.theme.accent)
    } else if selected {
        (config.theme.text, config.theme.focus_bg)
    } else {
        (config.theme.metadata, config.theme.pill_bg)
    };
    Style::default().fg(fg).bg(bg).add_modifier(Modifier::BOLD)
}

pub(crate) fn selection(config: &TuiConfig) -> Style {
    Style::default()
        .fg(config.theme.selection_fg)
        .bg(config.theme.selection_bg)
        .add_modifier(Modifier::BOLD)
}

pub(crate) fn list_highlight(config: &TuiConfig) -> Style {
    sidebar_row(config, SidebarRowState::ActiveFocused)
}

pub(crate) fn list_highlight_unfocused(config: &TuiConfig) -> Style {
    sidebar_row(config, SidebarRowState::ActiveUnfocused)
}

pub(crate) fn label(config: &TuiConfig, focused: bool) -> Style {
    if focused {
        Style::default()
            .fg(config.theme.text)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(config.theme.metadata)
    }
}

pub(crate) fn help(config: &TuiConfig) -> Style {
    Style::default().fg(config.theme.metadata)
}

pub(crate) fn placeholder(config: &TuiConfig) -> Style {
    Style::default().fg(config.theme.metadata)
}

pub(crate) fn required_prompt(config: &TuiConfig) -> Style {
    Style::default()
        .fg(config.theme.info)
        .add_modifier(Modifier::BOLD)
}

pub(crate) fn sidebar_row(config: &TuiConfig, state: SidebarRowState) -> Style {
    match state {
        SidebarRowState::IdleRoot => Style::default()
            .fg(config.theme.text)
            .bg(config.theme.panel_bg),
        SidebarRowState::IdleChild => Style::default()
            .fg(config.theme.metadata)
            .bg(config.theme.panel_bg),
        SidebarRowState::ActiveFocused => selection(config),
        SidebarRowState::ActiveUnfocused => Style::default()
            .fg(config.theme.selected_idle_fg)
            .bg(config.theme.selected_idle_bg)
            .add_modifier(Modifier::BOLD),
    }
}

pub(crate) fn subtle_chip(config: &TuiConfig, hovered: bool) -> Style {
    let style = Style::default()
        .fg(config.theme.metadata)
        .bg(config.theme.pill_bg);
    if hovered {
        style.fg(config.theme.text).add_modifier(Modifier::BOLD)
    } else {
        style
    }
}

pub(crate) fn status_chip(config: &TuiConfig, hovered: bool) -> Style {
    let style = Style::default()
        .fg(config.theme.error)
        .bg(config.theme.pill_bg)
        .add_modifier(Modifier::BOLD);
    if hovered {
        style.fg(config.theme.panel_bg).bg(config.theme.error)
    } else {
        style
    }
}

pub(crate) fn success_chip(config: &TuiConfig, hovered: bool) -> Style {
    let style = Style::default()
        .fg(config.theme.success)
        .bg(config.theme.pill_bg)
        .add_modifier(Modifier::BOLD);
    if hovered {
        style.fg(config.theme.panel_bg).bg(config.theme.success)
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
            .bg(config.theme.focus)
            .add_modifier(Modifier::BOLD)
    } else {
        style
    }
}

pub(crate) fn primary_chip(config: &TuiConfig, hovered: bool) -> Style {
    let style = Style::default()
        .fg(config.theme.primary_action_fg)
        .bg(config.theme.primary_action_bg)
        .add_modifier(Modifier::BOLD);
    if hovered {
        style
            .fg(config.theme.primary_action_bg)
            .bg(config.theme.primary_action_fg)
            .add_modifier(Modifier::BOLD)
    } else {
        style
    }
}

pub(crate) fn footer_chip(config: &TuiConfig, kind: FooterChipKind, hovered: bool) -> Style {
    match kind {
        FooterChipKind::Primary => primary_chip(config, hovered),
        FooterChipKind::Secondary => secondary_chip(config, hovered),
        FooterChipKind::Status => status_chip(config, hovered),
        FooterChipKind::Subtle => subtle_chip(config, hovered),
    }
}

pub(crate) fn metadata_badge(config: &TuiConfig, _kind: MetadataKind) -> Style {
    Style::default()
        .fg(config.theme.metadata)
        .bg(config.theme.pill_bg)
}

pub(crate) fn preview_border(config: &TuiConfig, emphasized: bool) -> Style {
    Style::default().fg(if emphasized {
        config.theme.focus
    } else {
        config.theme.divider
    })
}

pub(crate) fn preview_title(config: &TuiConfig) -> Style {
    Style::default()
        .fg(config.theme.text)
        .add_modifier(Modifier::BOLD)
}

pub(crate) fn overlay_panel(config: &TuiConfig, focused: bool) -> Style {
    surface(config, Surface::Overlay).fg(if focused {
        config.theme.focus
    } else {
        config.theme.border
    })
}

pub(crate) fn scrollbar_thumb(config: &TuiConfig, focused: bool) -> Style {
    Style::default()
        .fg(if focused {
            config.theme.focus
        } else {
            config.theme.metadata
        })
        .add_modifier(Modifier::BOLD)
}

pub(crate) fn scrollbar_cap(config: &TuiConfig, focused: bool) -> Style {
    scrollbar_thumb(config, focused)
}

pub(crate) fn scrollbar_track(config: &TuiConfig) -> Style {
    Style::default().fg(config.theme.metadata)
}

#[cfg(test)]
mod tests {
    use crate::config::TuiConfig;

    use super::{
        MetadataKind, SidebarRowState, field_border, metadata_badge, panel_border, sidebar_row,
        success_chip,
    };

    #[test]
    fn panel_border_uses_focus_color_for_focused_panels() {
        let config = TuiConfig::default();

        assert_eq!(panel_border(&config, true).fg, Some(config.theme.focus));
    }

    #[test]
    fn field_border_uses_warning_for_unfocused_invalid_fields() {
        let config = TuiConfig::default();

        assert_eq!(
            field_border(&config, false, true).fg,
            Some(config.theme.warning)
        );
    }

    #[test]
    fn metadata_badges_share_one_quiet_metadata_style() {
        let config = TuiConfig::default();

        assert_eq!(
            metadata_badge(&config, MetadataKind::Inherited).fg,
            Some(config.theme.metadata)
        );
        assert_eq!(
            metadata_badge(&config, MetadataKind::Default).fg,
            Some(config.theme.metadata)
        );
        assert_eq!(
            metadata_badge(&config, MetadataKind::Global).fg,
            Some(config.theme.metadata)
        );
    }

    #[test]
    fn footer_success_chip_uses_success_color() {
        let config = TuiConfig::default();

        assert_eq!(success_chip(&config, false).fg, Some(config.theme.success));
    }

    #[test]
    fn unfocused_active_sidebar_row_uses_idle_selection_colors() {
        let config = TuiConfig::default();

        let style = sidebar_row(&config, SidebarRowState::ActiveUnfocused);

        assert_eq!(style.bg, Some(config.theme.selected_idle_bg));
        assert_eq!(style.fg, Some(config.theme.selected_idle_fg));
    }
}
