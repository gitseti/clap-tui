use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};

use crate::config::TuiConfig;
use crate::query::form::FieldWidget;

use super::fields::FieldRenderModel;
use super::styles;

pub(super) fn render_flag_toggle(
    buffer: &mut Buffer,
    config: &TuiConfig,
    area: Rect,
    model: &FieldRenderModel<'_>,
) {
    let enabled = model.value == "[x]";
    let spans = vec![
        Span::styled(
            if enabled { " [✓] " } else { " [ ] " },
            styles::checkbox_chip(config, model.selected, enabled),
        ),
        Span::raw(" "),
        Span::styled(
            if enabled { "Enabled" } else { "Disabled" },
            styles::compact_control_value(config, model.selected, !enabled),
        ),
    ];
    let line = Line::from(spans);
    Paragraph::new(line)
        .block(
            model
                .block
                .clone()
                .style(styles::input(config, model.selected)),
        )
        .style(styles::flag_toggle(config, model.selected))
        .render(area, buffer);
}

pub(super) fn render_compact_control(
    buffer: &mut Buffer,
    config: &TuiConfig,
    area: Rect,
    model: &FieldRenderModel<'_>,
) {
    let display = if model.value.is_empty() {
        compact_placeholder(model.widget, model.required)
    } else {
        &model.value
    };
    Paragraph::new(compact_control_line(
        config,
        model.widget,
        display,
        area.width.saturating_sub(2),
        model.selected,
        model.uses_muted_non_user_value || model.is_default || model.shows_choice_placeholder,
        model.dropdown_open,
    ))
    .block(
        model
            .block
            .clone()
            .style(styles::input(config, model.selected)),
    )
    .style(styles::compact_control(config, model.selected))
    .render(area, buffer);
}

fn compact_placeholder(widget: FieldWidget, required: bool) -> &'static str {
    match widget {
        FieldWidget::Counter => "0",
        FieldWidget::MultiChoice if required => "Press Space to choose",
        FieldWidget::SingleChoice if required => "Press Enter to choose",
        _ => "Select...",
    }
}

pub(super) fn compact_control_line(
    config: &TuiConfig,
    widget: FieldWidget,
    value: &str,
    inner_width: u16,
    selected: bool,
    is_default: bool,
    open: bool,
) -> Line<'static> {
    let value_style = styles::compact_control_value(config, selected, is_default);
    let affordance = match widget {
        FieldWidget::Counter => " -  + ",
        _ if open => " ^ ",
        _ => " v ",
    };
    let affordance_width = u16::try_from(affordance.chars().count()).unwrap_or(6);
    let available_value = inner_width.saturating_sub(affordance_width + 1);
    let value_width = u16::try_from(value.chars().count()).unwrap_or(available_value);
    let padding = available_value.saturating_sub(value_width.saturating_add(1));
    if matches!(widget, FieldWidget::Counter) {
        return Line::from(vec![
            Span::raw(" "),
            Span::styled(value.to_string(), value_style),
            Span::raw(" ".repeat(usize::from(padding))),
            Span::styled(
                " - ",
                styles::compact_control_affordance(config, selected, true),
            ),
            Span::raw(" "),
            Span::styled(
                " + ",
                styles::compact_control_affordance(config, selected, true),
            ),
        ]);
    }
    Line::from(vec![
        Span::raw(" "),
        Span::styled(value.to_string(), value_style),
        Span::raw(" ".repeat(usize::from(padding))),
        Span::styled(
            affordance,
            styles::compact_control_affordance(config, selected, open),
        ),
    ])
}

#[cfg(test)]
mod tests {
    use ratatui::style::Modifier;

    use super::compact_control_line;
    use crate::TuiConfig;
    use crate::query::form::FieldWidget;

    #[test]
    fn open_choice_control_uses_open_affordance() {
        let line = compact_control_line(
            &TuiConfig::default(),
            FieldWidget::SingleChoice,
            "dev",
            20,
            false,
            false,
            true,
        );
        let rendered = line
            .spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect::<String>();

        assert!(rendered.contains('^'));
        assert!(!rendered.contains(" v "));
    }

    #[test]
    fn filled_default_backed_choice_renders_with_placeholder_tone() {
        let config = TuiConfig::default();
        let line = compact_control_line(
            &config,
            FieldWidget::SingleChoice,
            "fast",
            20,
            false,
            true,
            false,
        );

        assert_eq!(line.spans[1].style.fg, Some(config.theme.metadata));
        assert!(!line.spans[1].style.add_modifier.contains(Modifier::BOLD));
    }
}
