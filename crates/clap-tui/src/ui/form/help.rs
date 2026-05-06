use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, BorderType, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
};

use crate::config::TuiConfig;
use crate::frame_snapshot::{help_overlay_content_rect, help_overlay_popup_rect};
#[cfg(test)]
use crate::pipeline::EffectiveArgValue;
use crate::pipeline::EffectiveValueSource;
use crate::query::form::FieldWidget;
use crate::spec::{ArgSpec, CommandPath, CommandSpec, format_command_path};

use super::fields::{self, FieldRenderModel};
use super::styles;

pub(super) fn render_help_overlay(
    frame: &mut Frame<'_>,
    config: &TuiConfig,
    area: Rect,
    scroll: u16,
    help: &str,
) {
    let popup = help_overlay_popup_rect(area);
    let block = Block::default()
        .title(Line::from(vec![
            Span::raw(" "),
            Span::styled("Help", styles::preview_title(config)),
            Span::raw(" "),
        ]))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(styles::panel_border(config, true))
        .style(styles::overlay_panel(config, true));
    let inner = help_overlay_content_rect(area);
    frame.render_widget(block, popup);
    frame.render_widget(
        Paragraph::new(help.to_string())
            .style(Style::default().fg(config.theme.text))
            .scroll((scroll, 0)),
        inner,
    );

    let visible_height = inner.height;
    if usize::from(visible_height) < help.lines().count() {
        let steps = usize::from(
            u16::try_from(help.lines().count())
                .unwrap_or(u16::MAX)
                .saturating_sub(visible_height),
        )
        .saturating_add(1);
        let mut scrollbar_state = ScrollbarState::new(steps)
            .position(usize::from(scroll))
            .viewport_content_length(usize::from(visible_height));
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .track_symbol(Some("┃"))
            .thumb_symbol("█")
            .begin_style(styles::scrollbar_cap(config, true))
            .end_style(styles::scrollbar_cap(config, true))
            .thumb_style(styles::scrollbar_thumb(config, true))
            .track_style(styles::scrollbar_track(config));
        frame.render_stateful_widget(scrollbar, inner, &mut scrollbar_state);
    }
}

pub(super) fn render_field_help(
    frame: &mut Frame<'_>,
    config: &TuiConfig,
    description: Option<Rect>,
    root: &CommandSpec,
    selected_path: &CommandPath,
    model: &FieldRenderModel<'_>,
) {
    let Some(help_rect) = description else {
        return;
    };
    let Some(help) = field_help_text(root, selected_path, model) else {
        return;
    };
    frame.render_widget(
        Paragraph::new(Line::from(Span::raw(help))).style(if model.field_error.is_some() {
            Style::default().fg(config.theme.error)
        } else {
            styles::help(config)
        }),
        help_rect,
    );
}

fn field_help_text(
    root: &CommandSpec,
    selected_path: &CommandPath,
    model: &FieldRenderModel<'_>,
) -> Option<String> {
    if let Some(field_error) = model.field_error {
        return Some(field_error.to_string());
    }

    let mut parts = Vec::new();
    if let Some(reason) = model.semantic_reason {
        parts.push(reason.to_string());
    }
    let primary_help = if model.selected {
        model
            .arg
            .long_help()
            .filter(|long_help| Some(*long_help) != model.arg.help.as_deref())
            .map(str::to_string)
            .or_else(|| model.arg.help.clone())
            .or_else(|| model.arg.value_hint.clone())
    } else {
        model
            .arg
            .help
            .clone()
            .or_else(|| model.arg.value_hint.clone())
    };
    if let Some(help) = primary_help {
        parts.push(help);
    }
    if let Some(effective_value) = model.effective_value {
        match effective_value.source {
            EffectiveValueSource::DefaultMissing if !effective_value.values.is_empty() => parts
                .push(format!(
                    "Implicit value: {}",
                    fields::render_effective_value(model.arg, &effective_value.values)
                )),
            EffectiveValueSource::ConditionalDefault => {
                parts.push("Value is default-derived under the current conditions.".to_string());
            }
            _ => {}
        }
    }
    if model.selected && model.arg.is_inherited_for(selected_path) && !selected_path.is_empty() {
        parts.push(format!(
            "Defined on {}. Editing here updates that shared option for commands in this lineage.",
            format_command_path(&root.name, model.arg.owner_path())
        ));
    }
    if model.selected
        && let Some(hint) = widget_help_hint(model.widget)
    {
        parts.push(hint.to_string());
    }

    (!parts.is_empty()).then(|| parts.join("  "))
}

pub(super) fn section_heading_line(config: &TuiConfig, heading: &str, width: u16) -> Line<'static> {
    let title = format!(" {heading} ");
    let title_width = u16::try_from(title.chars().count()).unwrap_or(width);
    if width <= title_width.saturating_add(2) {
        return Line::from(Span::styled(
            heading.to_string(),
            Style::default()
                .fg(config.theme.result_accent)
                .add_modifier(Modifier::BOLD),
        ));
    }

    let line_width = width.saturating_sub(title_width);
    Line::from(vec![
        Span::styled(
            title,
            Style::default()
                .fg(config.theme.result_accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "─".repeat(usize::from(line_width)),
            Style::default().fg(config.theme.divider),
        ),
    ])
}

fn widget_help_hint(widget: FieldWidget) -> Option<&'static str> {
    match widget {
        FieldWidget::RepeatedText => Some(
            "Enter moves or adds a line. Up/Down switches lines. Backspace removes an empty line. Alt+Up/Down reorders.",
        ),
        FieldWidget::MultiChoice => Some("Space toggles choices. Enter finishes selection."),
        FieldWidget::Counter => Some("Right/+ increments. Left/- decrements."),
        FieldWidget::OptionalValue => Some("Right enables. Left/Delete disables."),
        _ => None,
    }
}

pub(super) fn required_empty_prompt(
    arg: &ArgSpec,
    widget: FieldWidget,
    required: bool,
) -> Option<String> {
    let _ = arg;
    if !required {
        return None;
    }

    Some(match widget {
        FieldWidget::RepeatedText | FieldWidget::SingleText => {
            "Enter a value to continue".to_string()
        }
        FieldWidget::SingleChoice => "Press Enter to choose a value".to_string(),
        FieldWidget::MultiChoice => "Press Space to choose at least one value".to_string(),
        _ => return None,
    })
}

#[derive(Debug, Clone, Copy)]
#[cfg(test)]
pub(super) struct FieldHelpContext<'a> {
    pub(super) selected: bool,
    pub(super) field_error: Option<&'a str>,
    pub(super) effective_value: Option<&'a EffectiveArgValue>,
    pub(super) semantic_reason: Option<&'a str>,
}

#[cfg(test)]
pub(super) fn field_help_text_for_test(
    root: &CommandSpec,
    arg: &ArgSpec,
    widget: FieldWidget,
    selected_path: &CommandPath,
    help: FieldHelpContext<'_>,
) -> Option<String> {
    if let Some(field_error) = help.field_error {
        return Some(field_error.to_string());
    }

    let mut parts = Vec::new();
    if let Some(reason) = help.semantic_reason {
        parts.push(reason.to_string());
    }
    let primary_help = if help.selected {
        arg.long_help()
            .filter(|long_help| Some(*long_help) != arg.help.as_deref())
            .map(str::to_string)
            .or_else(|| arg.help.clone())
            .or_else(|| arg.value_hint.clone())
    } else {
        arg.help.clone().or_else(|| arg.value_hint.clone())
    };
    if let Some(help) = primary_help {
        parts.push(help);
    }
    if let Some(effective_value) = help.effective_value {
        match effective_value.source {
            EffectiveValueSource::DefaultMissing if !effective_value.values.is_empty() => parts
                .push(format!(
                    "Implicit value: {}",
                    fields::render_effective_value(arg, &effective_value.values)
                )),
            EffectiveValueSource::ConditionalDefault => {
                parts.push("Value is default-derived under the current conditions.".to_string());
            }
            _ => {}
        }
    }
    if help.selected && arg.is_inherited_for(selected_path) && !selected_path.is_empty() {
        parts.push(format!(
            "Defined on {}. Editing here updates that shared option for commands in this lineage.",
            format_command_path(&root.name, arg.owner_path())
        ));
    }
    if help.selected
        && let Some(hint) = widget_help_hint(widget)
    {
        parts.push(hint.to_string());
    }

    (!parts.is_empty()).then(|| parts.join("  "))
}
