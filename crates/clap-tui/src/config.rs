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
    /// Accent color for highlights.
    pub accent: Color,
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
    /// Focused interactive panel border color.
    pub panel_focus_border: Color,
    /// Error color.
    pub error: Color,
    /// Dimmed text color.
    pub dim: Color,
    /// Input background color.
    pub input_bg: Color,
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
    /// Primary action background color.
    pub primary_action_bg: Color,
    /// Primary action foreground color.
    pub primary_action_fg: Color,
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
    pub fn from_preset(preset: ThemePreset) -> Self {
        match preset {
            ThemePreset::CalmDark => Self {
                text: Color::Rgb(236, 241, 246),
                accent: Color::Rgb(74, 201, 178),
                focus: Color::Rgb(214, 222, 230),
                success: Color::Rgb(110, 214, 154),
                info: Color::Rgb(116, 182, 204),
                warning: Color::Rgb(232, 186, 92),
                metadata: Color::Rgb(140, 156, 171),
                border: Color::Rgb(64, 78, 92),
                panel_focus_border: Color::Rgb(214, 222, 230),
                error: Color::Rgb(255, 99, 110),
                dim: Color::Rgb(140, 156, 171),
                input_bg: Color::Rgb(28, 38, 47),
                focus_bg: Color::Rgb(36, 54, 66),
                panel_bg: Color::Rgb(24, 32, 40),
                surface_raised: Color::Rgb(31, 43, 53),
                header_bg: Color::Rgb(18, 24, 31),
                selection_bg: Color::Rgb(36, 54, 66),
                selection_fg: Color::Rgb(236, 241, 246),
                selected_idle_bg: Color::Rgb(31, 43, 53),
                selected_idle_fg: Color::Rgb(236, 241, 246),
                pill_bg: Color::Rgb(22, 30, 38),
                primary_action_bg: Color::Rgb(74, 201, 178),
                primary_action_fg: Color::Rgb(24, 32, 40),
                preview_bg: Color::Rgb(18, 24, 31),
                overlay_bg: Color::Rgb(21, 29, 36),
                divider: Color::Rgb(52, 66, 80),
            },
            ThemePreset::HighContrastDark => Self {
                text: Color::Rgb(245, 247, 250),
                accent: Color::Rgb(92, 214, 190),
                focus: Color::Rgb(228, 235, 242),
                success: Color::Rgb(125, 229, 171),
                info: Color::Rgb(136, 201, 222),
                warning: Color::Rgb(242, 195, 102),
                metadata: Color::Rgb(175, 188, 202),
                border: Color::Rgb(90, 106, 122),
                panel_focus_border: Color::Rgb(228, 235, 242),
                error: Color::Rgb(255, 99, 110),
                dim: Color::Rgb(175, 188, 202),
                input_bg: Color::Rgb(26, 34, 42),
                focus_bg: Color::Rgb(44, 64, 78),
                panel_bg: Color::Rgb(20, 26, 34),
                surface_raised: Color::Rgb(30, 40, 50),
                header_bg: Color::Rgb(14, 20, 26),
                selection_bg: Color::Rgb(44, 64, 78),
                selection_fg: Color::Rgb(245, 247, 250),
                selected_idle_bg: Color::Rgb(30, 40, 50),
                selected_idle_fg: Color::Rgb(245, 247, 250),
                pill_bg: Color::Rgb(18, 26, 34),
                primary_action_bg: Color::Rgb(92, 214, 190),
                primary_action_fg: Color::Rgb(20, 26, 34),
                preview_bg: Color::Rgb(14, 20, 26),
                overlay_bg: Color::Rgb(18, 25, 33),
                divider: Color::Rgb(88, 102, 116),
            },
            ThemePreset::Light => Self {
                text: Color::Rgb(24, 32, 40),
                accent: Color::Rgb(34, 149, 132),
                focus: Color::Rgb(114, 126, 138),
                success: Color::Rgb(49, 148, 96),
                info: Color::Rgb(70, 120, 150),
                warning: Color::Rgb(160, 113, 25),
                metadata: Color::Rgb(96, 108, 120),
                border: Color::Rgb(180, 188, 196),
                panel_focus_border: Color::Rgb(114, 126, 138),
                error: Color::Rgb(199, 58, 71),
                dim: Color::Rgb(96, 108, 120),
                input_bg: Color::Rgb(243, 246, 250),
                focus_bg: Color::Rgb(223, 233, 243),
                panel_bg: Color::Rgb(248, 250, 252),
                surface_raised: Color::Rgb(238, 243, 248),
                header_bg: Color::Rgb(238, 242, 246),
                selection_bg: Color::Rgb(223, 233, 243),
                selection_fg: Color::Rgb(24, 32, 40),
                selected_idle_bg: Color::Rgb(238, 243, 248),
                selected_idle_fg: Color::Rgb(24, 32, 40),
                pill_bg: Color::Rgb(235, 240, 245),
                primary_action_bg: Color::Rgb(34, 149, 132),
                primary_action_fg: Color::Rgb(248, 250, 252),
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

/// Top-level configuration for [`crate::TuiLauncher`], [`crate::TypedTuiApp`], and [`crate::TuiApp`].
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
