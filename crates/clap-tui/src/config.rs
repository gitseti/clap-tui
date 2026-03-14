use ratatui::style::Color;

/// UI theming options.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Theme {
    /// Primary text color.
    pub text: Color,
    /// Accent color for highlights.
    pub accent: Color,
    /// Border color.
    pub border: Color,
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
    /// Header band background color.
    pub header_bg: Color,
    /// Pill background color.
    pub pill_bg: Color,
    /// Divider color.
    pub divider: Color,
}

/// Built-in theme presets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ThemePreset {
    CalmDark,
    HighContrastDark,
    Light,
}

impl Theme {
    /// Build a theme from a built-in preset.
    pub fn from_preset(preset: ThemePreset) -> Self {
        match preset {
            ThemePreset::CalmDark => Self {
                text: Color::Rgb(236, 241, 246),
                accent: Color::Rgb(74, 201, 178),
                border: Color::Rgb(64, 78, 92),
                error: Color::Rgb(255, 99, 110),
                dim: Color::Rgb(140, 156, 171),
                input_bg: Color::Rgb(28, 38, 47),
                focus_bg: Color::Rgb(36, 54, 66),
                panel_bg: Color::Rgb(24, 32, 40),
                header_bg: Color::Rgb(18, 24, 31),
                pill_bg: Color::Rgb(22, 30, 38),
                divider: Color::Rgb(52, 66, 80),
            },
            ThemePreset::HighContrastDark => Self {
                text: Color::Rgb(245, 247, 250),
                accent: Color::Rgb(92, 214, 190),
                border: Color::Rgb(90, 106, 122),
                error: Color::Rgb(255, 99, 110),
                dim: Color::Rgb(175, 188, 202),
                input_bg: Color::Rgb(26, 34, 42),
                focus_bg: Color::Rgb(44, 64, 78),
                panel_bg: Color::Rgb(20, 26, 34),
                header_bg: Color::Rgb(14, 20, 26),
                pill_bg: Color::Rgb(18, 26, 34),
                divider: Color::Rgb(88, 102, 116),
            },
            ThemePreset::Light => Self {
                text: Color::Rgb(24, 32, 40),
                accent: Color::Rgb(34, 149, 132),
                border: Color::Rgb(180, 188, 196),
                error: Color::Rgb(199, 58, 71),
                dim: Color::Rgb(96, 108, 120),
                input_bg: Color::Rgb(243, 246, 250),
                focus_bg: Color::Rgb(223, 233, 243),
                panel_bg: Color::Rgb(248, 250, 252),
                header_bg: Color::Rgb(238, 242, 246),
                pill_bg: Color::Rgb(235, 240, 245),
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
    /// Sidebar width ratio (percent of total width).
    pub sidebar_ratio: u16,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self { sidebar_ratio: 30 }
    }
}

/// Top-level configuration.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct TuiConfig {
    /// Theme configuration.
    pub theme: Theme,
    /// Key bindings.
    pub keymap: Keymap,
    /// Initial command path to select.
    pub start_command: Option<String>,
    /// Layout configuration.
    pub layout: LayoutConfig,
}

impl Default for TuiConfig {
    fn default() -> Self {
        Self {
            theme: Theme::default(),
            keymap: Keymap::default(),
            start_command: None,
            layout: LayoutConfig::default(),
        }
    }
}
