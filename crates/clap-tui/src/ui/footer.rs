use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};

use crate::config::TuiConfig;
use crate::frame_snapshot::{FooterButtonLayout, FrameLayout};
#[cfg(test)]
use crate::input::AppState;
use crate::input::{HoverTarget, UiState};
use crate::pipeline::ValidationState;

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
    Status,
    Subtle,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FooterView {
    actions: Vec<FooterChip>,
    status: Option<FooterChip>,
    hints: Vec<FooterChip>,
    status_offset: Option<u16>,
    hints_offset: Option<u16>,
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

const FOOTER_ZONE_GUTTER: u16 = 2;

impl Widget for FooterWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Widget::render(
            Paragraph::new(Line::from(footer_spans(self.config, self.view)))
                .style(Style::default().bg(self.config.theme.shell_bg)),
            area,
            buf,
        );
    }
}

pub(crate) fn render_footer(
    frame: &mut Frame<'_>,
    ui: &UiState,
    config: &TuiConfig,
    area: Rect,
    vm: &ScreenView<'_>,
    _frame_layout: &FrameLayout,
) {
    let view = build_footer_view(ui, area, &vm.validation);
    frame.render_widget(
        FooterWidget {
            config,
            view: &view,
        },
        area,
    );
}

pub(crate) fn populate_layout(
    ui: &UiState,
    area: Rect,
    validation: &ValidationState,
    frame_layout: &mut FrameLayout,
) {
    let view = build_footer_view(ui, area, validation);
    frame_layout.footer = Some(area);
    frame_layout.footer_buttons = layout_footer_buttons(area, &view);
}

fn build_footer_view(ui: &UiState, area: Rect, validation: &ValidationState) -> FooterView {
    let actions = vec![
        build_chip(
            ui,
            HoverTarget::Run,
            "Ctrl+R Run",
            FooterChipVariant::Primary,
        ),
        build_chip(
            ui,
            HoverTarget::Exit,
            "Ctrl+C Exit",
            FooterChipVariant::Secondary,
        ),
    ];
    let candidates = vec![
        build_chip(
            ui,
            HoverTarget::Search,
            "/ Search",
            FooterChipVariant::Subtle,
        ),
        build_chip(ui, HoverTarget::Focus, "Focus", FooterChipVariant::Subtle),
        build_chip(ui, HoverTarget::Help, "? Help", FooterChipVariant::Subtle),
    ];
    let status = validation
        .summary
        .as_ref()
        .map(|summary| build_status_chip(ui, summary));
    let (status, hints) = fit_footer_zones(area, &actions, status, candidates);

    FooterView {
        status_offset: status
            .as_ref()
            .map(|_| chips_width(&actions).saturating_add(zone_gutter(!actions.is_empty(), true))),
        hints_offset: hint_zone_offset(area, &hints),
        actions,
        status,
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

fn build_status_chip(ui: &UiState, chip: &str) -> FooterChip {
    FooterChip {
        target: HoverTarget::FooterStatus,
        label: format!(" {chip} "),
        hovered: ui.hover == Some(HoverTarget::FooterStatus),
        variant: FooterChipVariant::Status,
    }
}

fn footer_spans(config: &TuiConfig, view: &FooterView) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    let mut cursor = 0;

    append_zone(&mut spans, &mut cursor, 0, config, view.actions.as_slice());

    if let Some(status_offset) = view.status_offset
        && let Some(status) = view.status.as_ref()
    {
        append_zone(
            &mut spans,
            &mut cursor,
            status_offset,
            config,
            std::slice::from_ref(status),
        );
    }

    if let Some(hints_offset) = view.hints_offset {
        append_zone(
            &mut spans,
            &mut cursor,
            hints_offset,
            config,
            view.hints.as_slice(),
        );
    }

    spans
}

fn append_zone(
    spans: &mut Vec<Span<'static>>,
    cursor: &mut u16,
    zone_offset: u16,
    config: &TuiConfig,
    chips: &[FooterChip],
) {
    if chips.is_empty() {
        return;
    }

    let gap_width = zone_offset.saturating_sub(*cursor);
    if gap_width > 0 {
        spans.push(Span::raw(" ".repeat(usize::from(gap_width))));
    }
    spans.extend(spans_for_chips(config, chips));
    *cursor = zone_offset.saturating_add(chips_width(chips));
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
    let status_group = FooterButtonGroup {
        chips: view
            .status
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

    let mut layouts = layout_group(area.x, area, &action_group);
    if let Some(status_offset) = view.status_offset {
        layouts.extend(layout_group(
            area.x.saturating_add(status_offset),
            area,
            &status_group,
        ));
    }
    if let Some(hints_offset) = view.hints_offset {
        layouts.extend(layout_group(
            area.x.saturating_add(hints_offset),
            area,
            &hint_group,
        ));
    }
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

fn chips_width(chips: &[FooterChip]) -> u16 {
    let labels = chips
        .iter()
        .map(|chip| chip_width(&chip.label))
        .sum::<u16>();
    let gaps = u16::try_from(chips.len().saturating_sub(1)).unwrap_or(0);
    labels.saturating_add(gaps)
}

fn fit_footer_zones(
    area: Rect,
    actions: &[FooterChip],
    status: Option<FooterChip>,
    mut hints: Vec<FooterChip>,
) -> (Option<FooterChip>, Vec<FooterChip>) {
    let Some(status_chip) = status else {
        while !zone_order_fits(area, actions, None, &hints) && !hints.is_empty() {
            hints.pop();
        }
        return (None, hints);
    };

    while !zone_order_fits(area, actions, Some(&status_chip), &hints) && !hints.is_empty() {
        hints.pop();
    }

    if zone_order_fits(area, actions, Some(&status_chip), &hints) {
        return (Some(status_chip), hints);
    }

    let available_status_width = available_status_width(area, actions, &hints);
    let truncated_status = truncate_chip(&status_chip, available_status_width);

    (truncated_status, hints)
}

fn zone_order_fits(
    area: Rect,
    actions: &[FooterChip],
    status: Option<&FooterChip>,
    hints: &[FooterChip],
) -> bool {
    let action_width = chips_width(actions);
    let status_width = status.map_or(0, |chip| chip_width(&chip.label));
    let hint_width = chips_width(hints);
    let total_width = action_width
        .saturating_add(zone_gutter(!actions.is_empty(), status.is_some()))
        .saturating_add(status_width)
        .saturating_add(zone_gutter(status.is_some(), !hints.is_empty()))
        .saturating_add(hint_width);

    total_width <= area.width
}

fn available_status_width(area: Rect, actions: &[FooterChip], hints: &[FooterChip]) -> u16 {
    let reserved_width = chips_width(actions)
        .saturating_add(zone_gutter(!actions.is_empty(), true))
        .saturating_add(zone_gutter(true, !hints.is_empty()))
        .saturating_add(chips_width(hints));

    area.width.saturating_sub(reserved_width)
}

fn hint_zone_offset(area: Rect, hints: &[FooterChip]) -> Option<u16> {
    if hints.is_empty() {
        None
    } else {
        Some(area.width.saturating_sub(chips_width(hints)))
    }
}

fn zone_gutter(left_zone_present: bool, right_zone_present: bool) -> u16 {
    if left_zone_present && right_zone_present {
        FOOTER_ZONE_GUTTER
    } else {
        0
    }
}

fn chip_width(label: &str) -> u16 {
    u16::try_from(label.chars().count()).unwrap_or(u16::MAX)
}

fn chip_style(config: &TuiConfig, variant: FooterChipVariant, hovered: bool) -> Style {
    match variant {
        FooterChipVariant::Primary => {
            styles::footer_chip(config, styles::FooterChipKind::Primary, hovered)
        }
        FooterChipVariant::Secondary => {
            styles::footer_chip(config, styles::FooterChipKind::Secondary, hovered)
        }
        FooterChipVariant::Status => {
            styles::footer_chip(config, styles::FooterChipKind::Status, hovered)
        }
        FooterChipVariant::Subtle => {
            styles::footer_chip(config, styles::FooterChipKind::Subtle, hovered)
        }
    }
}

fn truncate_chip(chip: &FooterChip, available_width: u16) -> Option<FooterChip> {
    if available_width < 8 {
        return None;
    }
    let inner = usize::from(available_width.saturating_sub(2));
    let label = chip.label.trim();
    let mut truncated = label
        .chars()
        .take(inner.saturating_sub(1))
        .collect::<String>();
    truncated.push('…');
    Some(FooterChip {
        label: format!(" {truncated} "),
        ..chip.clone()
    })
}

#[cfg(test)]
fn build_test_state() -> AppState {
    AppState::new(crate::spec::CommandSpec {
        name: "tool".to_string(),
        version: None,
        about: None,
        help: String::new(),
        args: Vec::new(),
        subcommands: Vec::new(),
        ..crate::spec::CommandSpec::default()
    })
}

#[cfg(test)]
mod tests {
    use ratatui::layout::Rect;

    use super::{build_footer_view, build_test_state, layout_footer_buttons};
    use crate::input::HoverTarget;
    use crate::pipeline::ValidationState;

    #[test]
    fn footer_layout_preserves_button_order_and_spacing() {
        let state = build_test_state();
        let view = build_footer_view(
            &state.ui,
            Rect::new(0, 0, 80, 1),
            &ValidationState::default(),
        );
        let layouts = layout_footer_buttons(Rect::new(0, 0, 80, 1), &view);

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
        assert!(layouts[2].rect.x > layouts[1].rect.x + layouts[1].rect.width);
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

        let view = build_footer_view(
            &state.ui,
            Rect::new(0, 0, 60, 1),
            &ValidationState::default(),
        );

        assert!(view.actions[0].hovered);
        assert!(!view.actions[1].hovered);
        assert!(view.status.is_none());
        assert_eq!(view.hints.len(), 3);
    }

    #[test]
    fn footer_view_uses_compact_focus_hint() {
        let state = build_test_state();
        let view = build_footer_view(
            &state.ui,
            Rect::new(0, 0, 80, 1),
            &ValidationState::default(),
        );

        let focus_hint = view
            .hints
            .iter()
            .find(|hint| hint.target == HoverTarget::Focus)
            .expect("focus hint");

        assert_eq!(focus_hint.label, " Focus ");
    }

    #[test]
    fn footer_view_includes_validation_summary_chip() {
        let state = build_test_state();
        let view = build_footer_view(
            &state.ui,
            Rect::new(0, 0, 80, 1),
            &ValidationState {
                is_valid: false,
                summary: Some("Missing required argument: --name".to_string()),
                field_errors: std::collections::BTreeMap::new(),
            },
        );

        assert_eq!(
            view.status.as_ref().map(|chip| chip.label.as_str()),
            Some(" Missing required argument: --name ")
        );
        assert_eq!(
            view.status.as_ref().map(|chip| chip.target),
            Some(HoverTarget::FooterStatus)
        );
    }

    #[test]
    fn footer_status_does_not_highlight_when_preview_is_hovered() {
        let mut state = build_test_state();
        state.ui.hover = Some(HoverTarget::Preview);

        let view = build_footer_view(
            &state.ui,
            Rect::new(0, 0, 80, 1),
            &ValidationState {
                is_valid: false,
                summary: Some("Missing required argument: --name".to_string()),
                field_errors: std::collections::BTreeMap::new(),
            },
        );

        assert_eq!(view.status.as_ref().map(|chip| chip.hovered), Some(false));
        assert_eq!(
            view.status.as_ref().map(|chip| chip.target),
            Some(HoverTarget::FooterStatus)
        );
    }

    #[test]
    fn footer_view_drops_low_priority_hints_before_actions_or_status() {
        let state = build_test_state();
        let view = build_footer_view(
            &state.ui,
            Rect::new(0, 0, 50, 1),
            &ValidationState {
                is_valid: false,
                summary: Some("Missing required argument: --name".to_string()),
                field_errors: std::collections::BTreeMap::new(),
            },
        );

        assert_eq!(view.actions.len(), 2);
        assert_eq!(
            view.status.as_ref().map(|chip| chip.target),
            Some(HoverTarget::FooterStatus)
        );
        assert!(
            view.hints
                .iter()
                .all(|hint| hint.target != HoverTarget::Help)
        );
    }

    #[test]
    fn footer_view_anchors_status_in_middle_zone_not_screen_center() {
        let state = build_test_state();
        let view = build_footer_view(
            &state.ui,
            Rect::new(0, 0, 100, 1),
            &ValidationState {
                is_valid: false,
                summary: Some("Invalid value".to_string()),
                field_errors: std::collections::BTreeMap::new(),
            },
        );

        assert_eq!(view.status_offset, Some(28));
        assert_eq!(view.hints_offset, Some(73));
        assert!(view.status_offset.unwrap() < 50);
    }

    #[test]
    fn footer_view_truncates_status_with_right_ellipsis_after_hints_yield() {
        let state = build_test_state();
        let view = build_footer_view(
            &state.ui,
            Rect::new(0, 0, 40, 1),
            &ValidationState {
                is_valid: false,
                summary: Some("Missing required argument: --name".to_string()),
                field_errors: std::collections::BTreeMap::new(),
            },
        );

        assert!(view.hints.is_empty());
        assert_eq!(
            view.status.as_ref().map(|chip| chip.label.as_str()),
            Some(" Missing r… ")
        );
    }

    #[test]
    fn footer_view_preserves_search_before_other_passive_hints() {
        let state = build_test_state();
        let view = build_footer_view(
            &state.ui,
            Rect::new(0, 0, 56, 1),
            &ValidationState {
                is_valid: false,
                summary: Some("Invalid value".to_string()),
                field_errors: std::collections::BTreeMap::new(),
            },
        );

        let targets = view
            .hints
            .iter()
            .map(|hint| hint.target)
            .collect::<Vec<_>>();
        assert_eq!(targets, vec![HoverTarget::Search]);
    }
}
