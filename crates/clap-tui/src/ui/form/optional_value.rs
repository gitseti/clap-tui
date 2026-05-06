use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::config::TuiConfig;
use crate::input::{ArgInput, InputSource, UiState};
use crate::pipeline::{EffectiveArgValue, EffectiveValueSource};

use super::fields::FieldRenderModel;
use super::{styles, text};

pub(super) fn render_optional_value(
    frame: &mut Frame<'_>,
    ui: &UiState,
    area: Rect,
    config: &TuiConfig,
    model: &FieldRenderModel<'_>,
) {
    match optional_value_visual_state(
        model.current_input,
        &model.value,
        model.source_badge,
        model.effective_value,
    ) {
        OptionalValueVisualState::Explicit if model.selected => {
            text::render_textarea_field(frame, ui, model, None, area, config);
        }
        OptionalValueVisualState::Explicit => {
            frame.render_widget(
                Paragraph::new(model.value.to_string())
                    .block(
                        model
                            .block
                            .clone()
                            .style(styles::input(config, model.selected)),
                    )
                    .style(model.text_style),
                area,
            );
        }
        OptionalValueVisualState::Present { detail } if model.selected => {
            text::render_textarea_value(
                frame,
                ui,
                model,
                "",
                Some(format!("Present · {detail}")),
                area,
                config,
            );
        }
        OptionalValueVisualState::Present { detail } => {
            frame.render_widget(
                Paragraph::new(Line::from(vec![
                    Span::styled(
                        " Present ",
                        styles::checkbox_chip(config, model.selected, true),
                    ),
                    Span::raw(" "),
                    Span::styled(detail, styles::placeholder(config)),
                ]))
                .block(
                    model
                        .block
                        .clone()
                        .style(styles::input(config, model.selected)),
                )
                .style(Style::default()),
                area,
            );
        }
        OptionalValueVisualState::Off { detail } if model.selected => {
            text::render_textarea_value(
                frame,
                ui,
                model,
                "",
                Some(format!("Off · {detail}")),
                area,
                config,
            );
        }
        OptionalValueVisualState::Off { detail } => {
            frame.render_widget(
                Paragraph::new(Line::from(vec![
                    Span::styled(
                        " Off ",
                        styles::checkbox_chip(config, model.selected, false),
                    ),
                    Span::raw(" "),
                    Span::styled(detail, styles::placeholder(config)),
                ]))
                .block(
                    model
                        .block
                        .clone()
                        .style(styles::input(config, model.selected)),
                )
                .style(Style::default()),
                area,
            );
        }
    }
}

enum OptionalValueVisualState {
    Explicit,
    Present { detail: String },
    Off { detail: String },
}

fn optional_value_visual_state(
    current_input: Option<&crate::input::ArgInputState>,
    value: &str,
    source: Option<EffectiveValueSource>,
    effective_value: Option<&EffectiveArgValue>,
) -> OptionalValueVisualState {
    if let Some(input) = current_input {
        match &input.value {
            ArgInput::Flag { present: true, .. } => {
                return OptionalValueVisualState::Present {
                    detail: present_detail(effective_value),
                };
            }
            ArgInput::Values { occurrences }
                if occurrences
                    .iter()
                    .any(|occurrence| occurrence.values.iter().any(|entry| !entry.is_empty())) =>
            {
                let input_source = input.input_source().map(optional_input_source_label);
                if input.touched || matches!(input.input_source(), Some(InputSource::User)) {
                    return OptionalValueVisualState::Explicit;
                }
                return OptionalValueVisualState::Off {
                    detail: off_detail(input_source, value),
                };
            }
            _ => {}
        }
    }

    if value.is_empty() {
        OptionalValueVisualState::Off {
            detail: "Right/Space enables".to_string(),
        }
    } else {
        OptionalValueVisualState::Off {
            detail: off_detail(source.map(optional_effective_source_label), value),
        }
    }
}

fn present_detail(effective_value: Option<&EffectiveArgValue>) -> String {
    effective_value
        .filter(|effective| effective.source == EffectiveValueSource::DefaultMissing)
        .filter(|effective| !effective.values.is_empty())
        .map_or_else(
            || "bare flag, type to add a value".to_string(),
            |effective| format!("bare flag, implicit: {}", effective.values.join(" ")),
        )
}

fn off_detail(source: Option<&'static str>, value: &str) -> String {
    match (source, value.is_empty()) {
        (Some(source), false) => format!("{source}: {value}"),
        (None, false) => format!("effective: {value}"),
        _ => "Right/Space enables".to_string(),
    }
}

fn optional_input_source_label(source: InputSource) -> &'static str {
    match source {
        InputSource::User => "value",
        InputSource::Default => "default",
        InputSource::Env => "env",
    }
}

fn optional_effective_source_label(source: EffectiveValueSource) -> &'static str {
    match source {
        EffectiveValueSource::User => "value",
        EffectiveValueSource::Default => "default",
        EffectiveValueSource::Env => "env",
        EffectiveValueSource::DefaultMissing => "default-missing",
        EffectiveValueSource::ConditionalDefault => "conditional",
    }
}
