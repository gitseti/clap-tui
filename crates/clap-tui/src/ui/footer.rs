use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Widget;

use crate::config::TuiConfig;
use crate::frame_snapshot::{FooterButtonLayout, FrameLayout};
#[cfg(test)]
use crate::input::AppState;
use crate::input::{HoverTarget, UiState};

use super::screen::ScreenView;
use super::styles;

#[derive(Debug, Clone, PartialEq, Eq)]
struct FooterChip {
    target: HoverTarget,
    label: String,
    hovered: bool,
    variant: FooterChipVariant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FooterChipVariant {
    Primary,
    Secondary,
    Subtle,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FooterView {
    actions: Vec<FooterChip>,
    hints: Vec<FooterChip>,
    gap_width: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FooterButtonSpec {
    target: HoverTarget,
    label: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FooterButtonGroup {
    chips: Vec<FooterButtonSpec>,
}

struct FooterWidget<'a> {
    config: &'a TuiConfig,
    view: &'a FooterView,
}

impl Widget for FooterWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Widget::render(Line::from(footer_spans(self.config, self.view)), area, buf);
    }
}

pub(crate) fn render_footer(
    frame: &mut Frame<'_>,
    ui: &UiState,
    config: &TuiConfig,
    area: Rect,
    _vm: &ScreenView<'_>,
    _frame_layout: &FrameLayout,
) {
    let view = build_footer_view(ui, area);
    frame.render_widget(
        FooterWidget {
            config,
            view: &view,
        },
        area,
    );
}

pub(crate) fn populate_layout(ui: &UiState, area: Rect, frame_layout: &mut FrameLayout) {
    let view = build_footer_view(ui, area);
    frame_layout.footer = Some(area);
    frame_layout.footer_buttons = layout_footer_buttons(area, &view);
}

fn build_footer_view(ui: &UiState, area: Rect) -> FooterView {
    let actions = vec![
        build_chip(
            ui,
            HoverTarget::Run,
            "Ctrl+Enter Run",
            FooterChipVariant::Primary,
        ),
        build_chip(
            ui,
            HoverTarget::Exit,
            "Ctrl+C Exit",
            FooterChipVariant::Secondary,
        ),
    ];
    let hints = vec![
        build_chip(
            ui,
            HoverTarget::Search,
            "/ Search",
            FooterChipVariant::Subtle,
        ),
        build_chip(
            ui,
            HoverTarget::Focus,
            "Tab Focus",
            FooterChipVariant::Subtle,
        ),
        build_chip(ui, HoverTarget::Help, "? Help", FooterChipVariant::Subtle),
    ];
    FooterView {
        gap_width: footer_gap_width(area, &actions, &hints),
        actions,
        hints,
    }
}

fn build_chip(
    ui: &UiState,
    target: HoverTarget,
    chip: &str,
    variant: FooterChipVariant,
) -> FooterChip {
    FooterChip {
        target,
        label: format!(" {chip} "),
        hovered: ui.hover == Some(target),
        variant,
    }
}

fn footer_gap_width(area: Rect, actions: &[FooterChip], hints: &[FooterChip]) -> u16 {
    let action_width = chips_width(actions);
    let hint_width = chips_width(hints);
    let min_right_x = area
        .x
        .saturating_add(action_width)
        .saturating_add(u16::from(action_width > 0));
    let preferred_right_x = area.x.saturating_add(area.width.saturating_sub(hint_width));
    preferred_right_x
        .max(min_right_x)
        .saturating_sub(area.x.saturating_add(action_width))
}

fn footer_spans(config: &TuiConfig, view: &FooterView) -> Vec<Span<'static>> {
    let mut spans = spans_for_chips(config, &view.actions);
    if view.gap_width > 0 {
        spans.push(Span::raw(" ".repeat(usize::from(view.gap_width))));
    }
    spans.extend(spans_for_chips(config, &view.hints));
    spans
}

fn spans_for_chips(config: &TuiConfig, chips: &[FooterChip]) -> Vec<Span<'static>> {
    chips
        .iter()
        .enumerate()
        .flat_map(|(index, chip)| {
            let mut spans = Vec::with_capacity(2);
            if index > 0 {
                spans.push(Span::raw(" "));
            }
            spans.push(Span::styled(
                chip.label.clone(),
                chip_style(config, chip.variant, chip.hovered),
            ));
            spans
        })
        .collect()
}

fn layout_footer_buttons(area: Rect, view: &FooterView) -> Vec<FooterButtonLayout> {
    let action_group = FooterButtonGroup {
        chips: view
            .actions
            .iter()
            .map(|chip| FooterButtonSpec {
                target: chip.target,
                label: chip.label.clone(),
            })
            .collect(),
    };
    let hint_group = FooterButtonGroup {
        chips: view
            .hints
            .iter()
            .map(|chip| FooterButtonSpec {
                target: chip.target,
                label: chip.label.clone(),
            })
            .collect(),
    };

    let action_width = group_width(&action_group);
    let hint_start_x = area
        .x
        .saturating_add(action_width)
        .saturating_add(view.gap_width);
    let mut layouts = layout_group(area.x, area, &action_group);
    layouts.extend(layout_group(hint_start_x, area, &hint_group));
    layouts
}

fn layout_group(start_x: u16, area: Rect, group: &FooterButtonGroup) -> Vec<FooterButtonLayout> {
    let mut layouts = Vec::with_capacity(group.chips.len());
    let mut cursor_x = start_x;
    for (index, chip) in group.chips.iter().enumerate() {
        if index > 0 {
            cursor_x = cursor_x.saturating_add(1);
        }
        let width = chip_width(&chip.label);
        layouts.push(FooterButtonLayout {
            target: chip.target,
            rect: Rect::new(cursor_x, area.y, width, 1),
        });
        cursor_x = cursor_x.saturating_add(width);
    }
    layouts
}

fn group_width(group: &FooterButtonGroup) -> u16 {
    let labels = group
        .chips
        .iter()
        .map(|chip| chip_width(&chip.label))
        .sum::<u16>();
    let gaps = u16::try_from(group.chips.len().saturating_sub(1)).unwrap_or(0);
    labels.saturating_add(gaps)
}

fn chips_width(chips: &[FooterChip]) -> u16 {
    let labels = chips
        .iter()
        .map(|chip| chip_width(&chip.label))
        .sum::<u16>();
    let gaps = u16::try_from(chips.len().saturating_sub(1)).unwrap_or(0);
    labels.saturating_add(gaps)
}

fn chip_width(label: &str) -> u16 {
    u16::try_from(label.chars().count()).unwrap_or(u16::MAX)
}

fn chip_style(config: &TuiConfig, variant: FooterChipVariant, hovered: bool) -> Style {
    match variant {
        FooterChipVariant::Primary => styles::primary_chip(config, hovered),
        FooterChipVariant::Secondary => styles::secondary_chip(config, hovered),
        FooterChipVariant::Subtle => styles::subtle_chip(config, hovered),
    }
}

#[cfg(test)]
fn build_test_state() -> AppState {
    AppState::new(crate::spec::CommandSpec {
        name: "tool".to_string(),
        about: None,
        help: String::new(),
        args: Vec::new(),
        subcommands: Vec::new(),
    })
}

#[cfg(test)]
mod tests {
    use ratatui::layout::Rect;

    use super::{build_footer_view, build_test_state, layout_footer_buttons};
    use crate::input::HoverTarget;

    #[test]
    fn footer_layout_preserves_button_order_and_spacing() {
        let state = build_test_state();
        let view = build_footer_view(&state.ui, Rect::new(0, 0, 60, 1));
        let layouts = layout_footer_buttons(Rect::new(0, 0, 60, 1), &view);

        let targets = layouts.iter().map(|item| item.target).collect::<Vec<_>>();
        assert_eq!(
            targets,
            vec![
                HoverTarget::Run,
                HoverTarget::Exit,
                HoverTarget::Search,
                HoverTarget::Focus,
                HoverTarget::Help,
            ]
        );
        assert_eq!(
            layouts[0].rect.x + layouts[0].rect.width + 1,
            layouts[1].rect.x
        );
        assert_eq!(
            layouts[2].rect.x + layouts[2].rect.width + 1,
            layouts[3].rect.x
        );
        assert_eq!(
            layouts[3].rect.x + layouts[3].rect.width + 1,
            layouts[4].rect.x
        );
    }

    #[test]
    fn footer_view_marks_hovered_targets() {
        let mut state = build_test_state();
        state.ui.hover = Some(HoverTarget::Run);

        let view = build_footer_view(&state.ui, Rect::new(0, 0, 60, 1));

        assert!(view.actions[0].hovered);
        assert!(!view.actions[1].hovered);
        assert_eq!(view.hints.len(), 3);
    }
}
