use ratatui::style::{Modifier, Style};

use crate::config::TuiConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Surface {
    Shell,
    Workspace,
    Control,
    ControlActive,
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

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MetadataKind {
    Global,
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
        Surface::Shell => config.theme.shell_bg,
        Surface::Workspace => config.theme.workspace_bg,
        Surface::Control => config.theme.input_bg,
        Surface::ControlActive => config.theme.surface_raised,
        Surface::Result => config.theme.preview_bg,
        Surface::Overlay => config.theme.overlay_bg,
    };
    Style::default().bg(bg)
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn panel_surface(config: &TuiConfig, focused: bool) -> Style {
    surface(
        config,
        if focused {
            Surface::Workspace
        } else {
            Surface::Shell
        },
    )
}

pub(crate) fn panel_border(config: &TuiConfig, focused: bool) -> Style {
    Style::default().fg(if focused {
        config.theme.panel_focus_border
    } else {
        config.theme.shell_border
    })
}

pub(crate) fn input(config: &TuiConfig, selected: bool) -> Style {
    surface(
        config,
        if selected {
            Surface::ControlActive
        } else {
            Surface::Control
        },
    )
}

pub(crate) fn field_border(config: &TuiConfig, focused: bool, invalid: bool) -> Style {
    let color = if invalid {
        config.theme.error
    } else if focused {
        config.theme.focus
    } else {
        config.theme.border
    };
    Style::default().fg(color)
}

pub(crate) fn flag_toggle(config: &TuiConfig, selected: bool) -> Style {
    input(config, selected).fg(if selected {
        config.theme.text
    } else {
        config.theme.metadata
    })
}

pub(crate) fn compact_control(config: &TuiConfig, selected: bool) -> Style {
    input(config, selected)
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
            .fg(config.theme.shell_bg)
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
            .bg(config.theme.badge_bg)
            .add_modifier(Modifier::BOLD)
    }
}

pub(crate) fn checkbox_chip(config: &TuiConfig, selected: bool, enabled: bool) -> Style {
    let (fg, bg) = if enabled {
        (config.theme.shell_bg, config.theme.accent)
    } else if selected {
        (config.theme.text, config.theme.focus_bg)
    } else {
        (config.theme.metadata, config.theme.badge_bg)
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
        Style::default()
            .fg(config.theme.dim)
            .add_modifier(Modifier::BOLD)
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
            .bg(config.theme.sidebar_bg),
        SidebarRowState::IdleChild => Style::default()
            .fg(config.theme.metadata)
            .bg(config.theme.sidebar_bg),
        SidebarRowState::ActiveFocused => Style::default()
            .fg(config.theme.selection_fg)
            .bg(config.theme.selection_bg)
            .add_modifier(Modifier::BOLD),
        SidebarRowState::ActiveUnfocused => Style::default()
            .fg(config.theme.selected_idle_fg)
            .bg(config.theme.selected_idle_bg)
            .add_modifier(Modifier::BOLD),
    }
}

pub(crate) fn subtle_chip(config: &TuiConfig, hovered: bool) -> Style {
    let style = Style::default()
        .fg(config.theme.dim)
        .bg(config.theme.shell_bg);
    if hovered {
        style.fg(config.theme.metadata)
    } else {
        style
    }
}

pub(crate) fn status_chip(config: &TuiConfig, hovered: bool) -> Style {
    let _ = hovered;
    Style::default()
        .fg(config.theme.error)
        .bg(config.theme.shell_bg)
        .add_modifier(Modifier::BOLD)
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn success_chip(config: &TuiConfig, hovered: bool) -> Style {
    let style = Style::default()
        .fg(config.theme.success)
        .bg(config.theme.shell_bg);
    if hovered {
        style.bg(config.theme.badge_bg)
    } else {
        style
    }
}

pub(crate) fn secondary_chip(config: &TuiConfig, hovered: bool) -> Style {
    if hovered {
        Style::default()
            .fg(config.theme.text)
            .bg(config.theme.badge_bg)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(config.theme.secondary_action_fg)
            .bg(config.theme.shell_bg)
            .add_modifier(Modifier::BOLD)
    }
}

pub(crate) fn primary_chip(config: &TuiConfig, hovered: bool) -> Style {
    let style = Style::default()
        .fg(config.theme.primary_action_fg)
        .bg(config.theme.primary_action_bg)
        .add_modifier(Modifier::BOLD);
    if hovered {
        style
            .fg(config.theme.text)
            .bg(lift_color(config.theme.primary_action_bg, 12))
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

#[allow(dead_code)]
pub(crate) fn metadata_badge(config: &TuiConfig, kind: MetadataKind) -> Style {
    match kind {
        MetadataKind::Global => Style::default()
            .fg(config.theme.metadata)
            .bg(config.theme.badge_bg),
        MetadataKind::Default => Style::default()
            .fg(config.theme.info)
            .bg(config.theme.badge_bg),
        MetadataKind::Env => Style::default()
            .fg(config.theme.info)
            .bg(config.theme.badge_bg)
            .add_modifier(Modifier::BOLD),
        MetadataKind::Implicit => Style::default()
            .fg(config.theme.warning)
            .bg(config.theme.badge_bg)
            .add_modifier(Modifier::BOLD),
        MetadataKind::Conditional => Style::default()
            .fg(config.theme.warning)
            .bg(config.theme.badge_bg),
    }
}

pub(crate) fn preview_title(config: &TuiConfig) -> Style {
    Style::default()
        .fg(config.theme.text)
        .bg(config.theme.header_bg)
        .add_modifier(Modifier::BOLD)
}

fn lift_color(color: ratatui::style::Color, amount: u8) -> ratatui::style::Color {
    match color {
        ratatui::style::Color::Rgb(r, g, b) => ratatui::style::Color::Rgb(
            r.saturating_add(amount),
            g.saturating_add(amount),
            b.saturating_add(amount),
        ),
        other => other,
    }
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
            config.theme.dim
        })
        .add_modifier(Modifier::BOLD)
}

pub(crate) fn scrollbar_cap(config: &TuiConfig, focused: bool) -> Style {
    scrollbar_thumb(config, focused)
}

pub(crate) fn scrollbar_track(config: &TuiConfig) -> Style {
    Style::default().fg(config.theme.divider)
}

#[cfg(test)]
mod tests {
    use ratatui::style::Modifier;

    use crate::config::TuiConfig;

    use super::{
        MetadataKind, SidebarRowState, field_border, metadata_badge, panel_border, panel_surface,
        primary_chip, secondary_chip, sidebar_row, status_chip, subtle_chip, success_chip,
    };

    #[test]
    fn panel_border_uses_focus_color_for_focused_panels() {
        let config = TuiConfig::default();

        assert_eq!(
            panel_border(&config, true).fg,
            Some(config.theme.panel_focus_border)
        );
    }

    #[test]
    fn panel_surface_dims_unfocused_panels() {
        let config = TuiConfig::default();

        assert_eq!(
            panel_surface(&config, true).bg,
            Some(config.theme.workspace_bg)
        );
        assert_eq!(
            panel_surface(&config, false).bg,
            Some(config.theme.shell_bg)
        );
    }

    #[test]
    fn field_border_uses_error_for_unfocused_invalid_fields() {
        let config = TuiConfig::default();

        assert_eq!(
            field_border(&config, false, true).fg,
            Some(config.theme.error)
        );
    }

    #[test]
    fn field_border_uses_focus_color_for_focused_fields() {
        let config = TuiConfig::default();

        assert_eq!(
            field_border(&config, true, false).fg,
            Some(config.theme.focus)
        );
    }

    #[test]
    fn metadata_badges_distinguish_semantic_kinds() {
        let config = TuiConfig::default();

        let global = metadata_badge(&config, MetadataKind::Global);
        let default = metadata_badge(&config, MetadataKind::Default);
        let env = metadata_badge(&config, MetadataKind::Env);
        let implicit = metadata_badge(&config, MetadataKind::Implicit);
        let conditional = metadata_badge(&config, MetadataKind::Conditional);

        assert_eq!(global.fg, Some(config.theme.metadata));
        assert_eq!(default.fg, Some(config.theme.info));
        assert_eq!(env.fg, Some(config.theme.info));
        assert_eq!(implicit.fg, Some(config.theme.warning));
        assert_eq!(conditional.fg, Some(config.theme.warning));
        assert!(env.add_modifier.contains(Modifier::BOLD));
        assert!(implicit.add_modifier.contains(Modifier::BOLD));
        assert!(!default.add_modifier.contains(Modifier::BOLD));
        assert!(!conditional.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn footer_success_chip_uses_success_color() {
        let config = TuiConfig::default();

        assert_eq!(success_chip(&config, false).fg, Some(config.theme.success));
    }

    #[test]
    fn footer_action_chips_highlight_without_underlines() {
        let config = TuiConfig::default();

        let idle = primary_chip(&config, false);
        let secondary = secondary_chip(&config, false);
        let hovered = primary_chip(&config, true);
        let secondary_hovered = secondary_chip(&config, true);

        assert_eq!(idle.bg, Some(config.theme.primary_action_bg));
        assert_eq!(idle.fg, Some(config.theme.primary_action_fg));
        assert_ne!(idle.bg, secondary.bg);
        assert_ne!(idle.fg, secondary.fg);
        assert_eq!(secondary.bg, Some(config.theme.shell_bg));
        assert_eq!(secondary.fg, Some(config.theme.secondary_action_fg));
        assert_eq!(secondary_hovered.bg, Some(config.theme.badge_bg));
        assert_eq!(secondary_hovered.fg, Some(config.theme.text));
        assert_ne!(hovered.bg, Some(config.theme.primary_action_bg));
        assert_eq!(hovered.fg, Some(config.theme.text));
        assert!(!hovered.add_modifier.contains(Modifier::UNDERLINED));
        assert!(
            !secondary_hovered
                .add_modifier
                .contains(Modifier::UNDERLINED)
        );
        assert!(secondary_hovered.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn footer_hover_states_keep_run_stronger_than_exit() {
        let config = TuiConfig::default();

        let primary_idle = primary_chip(&config, false);
        let primary_hover = primary_chip(&config, true);
        let secondary_idle = secondary_chip(&config, false);
        let secondary_hover = secondary_chip(&config, true);

        assert_ne!(primary_idle.fg, primary_hover.fg);
        assert_eq!(secondary_idle.bg, Some(config.theme.shell_bg));
        assert_eq!(secondary_hover.bg, Some(config.theme.badge_bg));
        assert_ne!(primary_hover.bg, secondary_hover.bg);
    }

    #[test]
    fn footer_status_and_subtle_chips_stay_discoverable_without_filled_hover_states() {
        let config = TuiConfig::default();

        let subtle_idle = subtle_chip(&config, false);
        let subtle_hover = subtle_chip(&config, true);
        let status_idle = status_chip(&config, false);
        let status_hover = status_chip(&config, true);

        assert_eq!(subtle_idle.bg, Some(config.theme.shell_bg));
        assert_eq!(subtle_hover.bg, Some(config.theme.shell_bg));
        assert_eq!(subtle_idle.fg, Some(config.theme.dim));
        assert_eq!(subtle_hover.fg, Some(config.theme.metadata));
        assert_eq!(status_idle.bg, Some(config.theme.shell_bg));
        assert_eq!(status_hover.bg, Some(config.theme.shell_bg));
        assert_eq!(status_idle.fg, Some(config.theme.error));
        assert_eq!(status_hover.fg, Some(config.theme.error));
        assert!(status_idle.add_modifier.contains(Modifier::BOLD));
        assert_eq!(status_idle, status_hover);
    }

    #[test]
    fn footer_hierarchy_remains_ordered_across_theme_presets() {
        for config in [
            TuiConfig::default(),
            TuiConfig {
                theme: crate::config::Theme::from_preset(
                    crate::config::ThemePreset::HighContrastDark,
                ),
                ..TuiConfig::default()
            },
            TuiConfig {
                theme: crate::config::Theme::from_preset(crate::config::ThemePreset::Light),
                ..TuiConfig::default()
            },
        ] {
            let primary = primary_chip(&config, false);
            let secondary = secondary_chip(&config, false);
            let subtle = subtle_chip(&config, false);

            assert_ne!(primary.bg, secondary.bg);
            assert_eq!(secondary.bg, Some(config.theme.shell_bg));
            assert_eq!(subtle.bg, Some(config.theme.shell_bg));
            assert_eq!(secondary.fg, Some(config.theme.secondary_action_fg));
            assert_eq!(subtle.fg, Some(config.theme.dim));
        }
    }

    #[test]
    fn unfocused_active_sidebar_row_uses_idle_selection_colors() {
        let config = TuiConfig::default();

        let style = sidebar_row(&config, SidebarRowState::ActiveUnfocused);

        assert_eq!(style.bg, Some(config.theme.selected_idle_bg));
        assert_eq!(style.fg, Some(config.theme.selected_idle_fg));
    }
}
