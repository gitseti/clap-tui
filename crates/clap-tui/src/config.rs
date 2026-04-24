use ratatui::style::Color;

/// UI colors used across the TUI.
///
/// Most applications can start from [`Theme::from_preset`] and override only the fields they
/// want to change.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Theme {
    /// Primary text color.
    pub text: Color,
    /// Accent color for interactive UI chrome.
    pub accent: Color,
    /// Accent color for breadcrumb and command/result emphasis.
    pub result_accent: Color,
    /// Focus treatment color for active controls and panels.
    pub focus: Color,
    /// Success-oriented feedback color.
    pub success: Color,
    /// Informative accent for source and state metadata.
    pub info: Color,
    /// Caution-oriented color reserved for non-error warning states.
    pub warning: Color,
    /// Passive metadata and descriptive text color.
    pub metadata: Color,
    /// Border color.
    pub border: Color,
    /// Border color for passive shell chrome.
    pub shell_border: Color,
    /// Focused interactive panel border color.
    pub panel_focus_border: Color,
    /// Error color.
    pub error: Color,
    /// Dimmed text color.
    pub dim: Color,
    /// Input background color.
    pub input_bg: Color,
    /// Background for the outer shell frame.
    pub shell_bg: Color,
    /// Background for the navigation surface.
    pub sidebar_bg: Color,
    /// Background for the active editing surface.
    pub workspace_bg: Color,
    /// Focused row background color.
    pub focus_bg: Color,
    /// Panel background color.
    pub panel_bg: Color,
    /// Raised surface for selected rows and compact controls.
    pub surface_raised: Color,
    /// Header band background color.
    pub header_bg: Color,
    /// Filled background for selected items.
    pub selection_bg: Color,
    /// Foreground for selected items.
    pub selection_fg: Color,
    /// Background for selected but unfocused items.
    pub selected_idle_bg: Color,
    /// Foreground for selected but unfocused items.
    pub selected_idle_fg: Color,
    /// Pill background color.
    pub pill_bg: Color,
    /// Badge background color for compact metadata chips.
    pub badge_bg: Color,
    /// Primary action background color.
    pub primary_action_bg: Color,
    /// Primary action foreground color.
    pub primary_action_fg: Color,
    /// Secondary action background color.
    pub secondary_action_bg: Color,
    /// Secondary action foreground color.
    pub secondary_action_fg: Color,
    /// Background for the read-only preview band.
    pub preview_bg: Color,
    /// Background for overlays such as dropdowns and help.
    pub overlay_bg: Color,
    /// Divider color.
    pub divider: Color,
}

/// Built-in theme presets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ThemePreset {
    /// Balanced dark preset with muted surfaces and teal accents.
    CalmDark,
    /// Higher-contrast dark preset for brighter terminals.
    HighContrastDark,
    /// Light preset with subdued neutral surfaces.
    Light,
}

impl Theme {
    /// Build a theme from a built-in preset.
    #[must_use]
    #[allow(clippy::too_many_lines)]
    pub fn from_preset(preset: ThemePreset) -> Self {
        match preset {
            ThemePreset::CalmDark => Self {
                text: Color::Rgb(214, 225, 228),
                accent: Color::Rgb(67, 225, 232),
                result_accent: Color::Rgb(67, 225, 232),
                focus: Color::Rgb(242, 252, 255),
                success: Color::Rgb(110, 214, 154),
                info: Color::Rgb(109, 198, 210),
                warning: Color::Rgb(232, 186, 92),
                metadata: Color::Rgb(122, 150, 156),
                border: Color::Rgb(40, 78, 86),
                shell_border: Color::Rgb(25, 49, 56),
                panel_focus_border: Color::Rgb(242, 252, 255),
                error: Color::Rgb(255, 99, 110),
                dim: Color::Rgb(122, 150, 156),
                input_bg: Color::Rgb(15, 34, 38),
                shell_bg: Color::Rgb(5, 14, 17),
                sidebar_bg: Color::Rgb(8, 20, 24),
                workspace_bg: Color::Rgb(10, 24, 29),
                focus_bg: Color::Rgb(18, 58, 66),
                panel_bg: Color::Rgb(10, 24, 29),
                surface_raised: Color::Rgb(17, 47, 53),
                header_bg: Color::Rgb(14, 47, 54),
                selection_bg: Color::Rgb(12, 88, 97),
                selection_fg: Color::Rgb(245, 251, 252),
                selected_idle_bg: Color::Rgb(12, 61, 68),
                selected_idle_fg: Color::Rgb(230, 243, 245),
                pill_bg: Color::Rgb(12, 28, 32),
                badge_bg: Color::Rgb(16, 39, 44),
                primary_action_bg: Color::Rgb(110, 214, 154),
                primary_action_fg: Color::Rgb(5, 14, 17),
                secondary_action_bg: Color::Rgb(16, 39, 44),
                secondary_action_fg: Color::Rgb(230, 243, 245),
                preview_bg: Color::Rgb(8, 20, 24),
                overlay_bg: Color::Rgb(12, 26, 31),
                divider: Color::Rgb(32, 78, 86),
            },
            ThemePreset::HighContrastDark => Self {
                text: Color::Rgb(245, 247, 250),
                accent: Color::Rgb(102, 218, 194),
                result_accent: Color::Rgb(245, 185, 96),
                focus: Color::Rgb(228, 235, 242),
                success: Color::Rgb(125, 229, 171),
                info: Color::Rgb(136, 201, 222),
                warning: Color::Rgb(242, 195, 102),
                metadata: Color::Rgb(175, 188, 202),
                border: Color::Rgb(90, 106, 122),
                shell_border: Color::Rgb(62, 75, 90),
                panel_focus_border: Color::Rgb(228, 235, 242),
                error: Color::Rgb(255, 99, 110),
                dim: Color::Rgb(175, 188, 202),
                input_bg: Color::Rgb(26, 34, 42),
                shell_bg: Color::Rgb(10, 15, 22),
                sidebar_bg: Color::Rgb(16, 22, 30),
                workspace_bg: Color::Rgb(20, 28, 36),
                focus_bg: Color::Rgb(44, 64, 78),
                panel_bg: Color::Rgb(20, 26, 34),
                surface_raised: Color::Rgb(30, 40, 50),
                header_bg: Color::Rgb(14, 20, 26),
                selection_bg: Color::Rgb(44, 64, 78),
                selection_fg: Color::Rgb(245, 247, 250),
                selected_idle_bg: Color::Rgb(30, 40, 50),
                selected_idle_fg: Color::Rgb(245, 247, 250),
                pill_bg: Color::Rgb(18, 26, 34),
                badge_bg: Color::Rgb(23, 31, 40),
                primary_action_bg: Color::Rgb(92, 214, 190),
                primary_action_fg: Color::Rgb(20, 26, 34),
                secondary_action_bg: Color::Rgb(34, 45, 56),
                secondary_action_fg: Color::Rgb(238, 242, 246),
                preview_bg: Color::Rgb(14, 20, 26),
                overlay_bg: Color::Rgb(18, 25, 33),
                divider: Color::Rgb(88, 102, 116),
            },
            ThemePreset::Light => Self {
                text: Color::Rgb(24, 32, 40),
                accent: Color::Rgb(34, 149, 132),
                result_accent: Color::Rgb(181, 116, 32),
                focus: Color::Rgb(114, 126, 138),
                success: Color::Rgb(49, 148, 96),
                info: Color::Rgb(70, 120, 150),
                warning: Color::Rgb(160, 113, 25),
                metadata: Color::Rgb(96, 108, 120),
                border: Color::Rgb(180, 188, 196),
                shell_border: Color::Rgb(204, 210, 218),
                panel_focus_border: Color::Rgb(114, 126, 138),
                error: Color::Rgb(199, 58, 71),
                dim: Color::Rgb(96, 108, 120),
                input_bg: Color::Rgb(243, 246, 250),
                shell_bg: Color::Rgb(236, 240, 244),
                sidebar_bg: Color::Rgb(242, 246, 250),
                workspace_bg: Color::Rgb(250, 252, 255),
                focus_bg: Color::Rgb(223, 233, 243),
                panel_bg: Color::Rgb(248, 250, 252),
                surface_raised: Color::Rgb(238, 243, 248),
                header_bg: Color::Rgb(238, 242, 246),
                selection_bg: Color::Rgb(223, 233, 243),
                selection_fg: Color::Rgb(24, 32, 40),
                selected_idle_bg: Color::Rgb(238, 243, 248),
                selected_idle_fg: Color::Rgb(24, 32, 40),
                pill_bg: Color::Rgb(235, 240, 245),
                badge_bg: Color::Rgb(231, 237, 243),
                primary_action_bg: Color::Rgb(34, 149, 132),
                primary_action_fg: Color::Rgb(248, 250, 252),
                secondary_action_bg: Color::Rgb(226, 233, 239),
                secondary_action_fg: Color::Rgb(24, 32, 40),
                preview_bg: Color::Rgb(238, 242, 246),
                overlay_bg: Color::Rgb(244, 247, 251),
                divider: Color::Rgb(200, 208, 216),
            },
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Theme::from_preset(ThemePreset::CalmDark)
    }
}

/// Key bindings for main actions.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Keymap {
    /// Toggle help tab.
    pub help: char,
    /// Activate search.
    pub search: char,
}

impl Default for Keymap {
    fn default() -> Self {
        Self {
            help: '?',
            search: '/',
        }
    }
}

/// Layout configuration.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct LayoutConfig {
    /// Preferred sidebar width as a percentage of the terminal width.
    ///
    /// The rendered sidebar is clamped to fit the active layout so the main pane keeps a
    /// usable width. Compact layouts clamp more aggressively than roomy layouts.
    pub sidebar_ratio: u16,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self { sidebar_ratio: 24 }
    }
}

/// Top-level configuration for [`crate::Tui`] and [`crate::TuiApp`].
///
/// Most applications only need to customize the theme or `start_command`.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct TuiConfig {
    /// Theme configuration.
    pub theme: Theme,
    /// Key bindings.
    pub keymap: Keymap,
    /// Initial command path to select, using `::`-separated command names such as
    /// `build::release`.
    ///
    /// Unknown paths leave the root command selected and show a non-error toast at startup.
    pub start_command: Option<String>,
    /// Layout configuration.
    pub layout: LayoutConfig,
}
