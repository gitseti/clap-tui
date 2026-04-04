use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, BorderType, Borders, Clear, List, ListItem, ListState, Scrollbar, ScrollbarOrientation,
    ScrollbarState, StatefulWidget,
};

use crate::config::TuiConfig;
use crate::frame_snapshot::FrameSnapshot;
use crate::input::{DomainState, UiState};
use crate::query::form::{self, FieldWidget};
use crate::spec::choice_value_matches_default;

use super::screen::ScreenView;

#[derive(Debug, Clone, PartialEq, Eq)]
struct DropdownItem {
    label: String,
    text_style: Style,
}

#[derive(Debug, Clone)]
struct DropdownView {
    rect: Rect,
    items: Vec<DropdownItem>,
    selected_index: Option<usize>,
    scroll_position: usize,
    visible_rows: usize,
    total_rows: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DropdownWidgetState {
    selected_index: Option<usize>,
    scroll_position: usize,
    visible_rows: usize,
    total_rows: usize,
}

struct DropdownWidget<'a> {
    config: &'a TuiConfig,
    items: &'a [DropdownItem],
}

impl StatefulWidget for DropdownWidget<'_> {
    type State = DropdownWidgetState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let mut list_state = ListState::default();
        list_state.select(state.selected_index);

        let list_items = self
            .items
            .iter()
            .map(|item| {
                let line = Line::from(Span::styled(item.label.clone(), item.text_style));
                ListItem::new(line)
            })
            .collect::<Vec<_>>();

        let list = List::new(list_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(super::styles::panel_border(self.config, false))
                    .style(Style::default().bg(self.config.theme.surface_raised)),
            )
            .highlight_style(super::styles::selection(self.config))
            .highlight_symbol("› ")
            .style(Style::default().bg(self.config.theme.surface_raised));

        StatefulWidget::render(list, area, buf, &mut list_state);

        if state.total_rows > state.visible_rows && state.visible_rows > 0 {
            let scroll_steps = state
                .total_rows
                .saturating_sub(state.visible_rows)
                .saturating_add(1);
            let mut scrollbar_state = ScrollbarState::new(scroll_steps)
                .position(state.scroll_position)
                .viewport_content_length(state.visible_rows);
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .track_symbol(Some("┃"))
                .thumb_symbol("█")
                .begin_style(super::styles::scrollbar_cap(self.config, true))
                .end_style(super::styles::scrollbar_cap(self.config, true))
                .thumb_style(super::styles::scrollbar_thumb(self.config, true))
                .track_style(super::styles::scrollbar_track(self.config));
            StatefulWidget::render(scrollbar, area, buf, &mut scrollbar_state);
        }
    }
}

pub(crate) fn render_dropdown(
    frame: &mut Frame<'_>,
    ui: &UiState,
    frame_snapshot: &FrameSnapshot,
    domain: &DomainState,
    config: &TuiConfig,
    _area: Rect,
    _vm: &ScreenView<'_>,
) {
    let Some(view) = build_dropdown_view(ui, frame_snapshot, domain, config) else {
        return;
    };
    let mut widget_state = DropdownWidgetState {
        selected_index: view.selected_index,
        scroll_position: view.scroll_position,
        visible_rows: view.visible_rows,
        total_rows: view.total_rows,
    };
    let widget = DropdownWidget {
        config,
        items: &view.items,
    };

    frame.render_widget(Clear, view.rect);
    frame.render_stateful_widget(widget, view.rect, &mut widget_state);
}

fn build_dropdown_view(
    ui: &UiState,
    frame_snapshot: &FrameSnapshot,
    domain: &DomainState,
    config: &TuiConfig,
) -> Option<DropdownView> {
    let arg_id = ui.dropdown_open.as_ref()?;
    let rect = frame_snapshot.layout.dropdown?;
    let arg = domain.arg_for_input(arg_id)?;

    let current_form = domain.current_form();
    let widget = form::widget_for(arg);
    let is_touched = current_form
        .as_ref()
        .is_some_and(|form| form.is_touched(&arg.id));
    let selected_values = current_form
        .as_ref()
        .map_or_else(Vec::new, |form| form.selected_values(arg));
    let total_rows = arg.choices.len();
    let visible_rows = rect.height.saturating_sub(2) as usize;
    let scroll_position = ui.dropdown_scroll(total_rows, visible_rows);
    let selected_row = ui.dropdown_cursor(total_rows);
    let selected_index = (selected_row >= scroll_position
        && selected_row < scroll_position.saturating_add(visible_rows))
    .then_some(selected_row - scroll_position);

    let items = arg
        .choices
        .iter()
        .enumerate()
        .skip(scroll_position)
        .take(visible_rows)
        .map(|(index, value)| {
            let is_default = !is_touched && choice_value_matches_default(arg, value);
            let is_selected = index == selected_row;
            let is_checked = selected_values.iter().any(|selected| selected == value);
            let text_style = match (is_selected, is_default) {
                (true, _) => Style::default()
                    .fg(config.theme.selection_fg)
                    .add_modifier(ratatui::style::Modifier::BOLD),
                (false, true) => Style::default().fg(config.theme.dim),
                (false, false) => Style::default().fg(config.theme.text),
            };
            let description = arg
                .choice_metadata(value)
                .and_then(|choice| choice.help.as_deref())
                .map(|help| format!("  {help}"))
                .unwrap_or_default();
            DropdownItem {
                label: match widget {
                    FieldWidget::MultiChoice => {
                        format!(
                            "{} {}{}",
                            if is_checked { "[x]" } else { "[ ]" },
                            value,
                            description
                        )
                    }
                    _ => format!("{value}{description}"),
                },
                text_style,
            }
        })
        .collect();

    Some(DropdownView {
        rect,
        items,
        selected_index,
        scroll_position,
        visible_rows,
        total_rows,
    })
}

#[cfg(test)]
mod tests {
    use crate::TuiConfig;
    use crate::frame_snapshot::{MAX_DROPDOWN_ROWS, dropdown_geometry};
    use crate::input::AppState;
    use crate::ui::dropdown::build_dropdown_view;
    use clap::{Arg, Command, builder::PossibleValue};
    use ratatui::layout::Rect;

    #[test]
    fn dropdown_uses_input_width() {
        let form_view = Rect::new(10, 5, 60, 20);
        let input_rect = Rect::new(14, 8, 24, 3);
        let layout = dropdown_geometry(form_view, input_rect, 4).expect("layout");

        assert_eq!(layout.rect.x, input_rect.x);
        assert_eq!(layout.rect.width, input_rect.width);
    }

    #[test]
    fn dropdown_prefers_below_when_space_exists() {
        let form_view = Rect::new(10, 5, 60, 20);
        let input_rect = Rect::new(14, 8, 24, 3);
        let layout = dropdown_geometry(form_view, input_rect, 4).expect("layout");

        assert_eq!(layout.rect.y, input_rect.y + input_rect.height);
        assert_eq!(layout.visible_rows, 4);
    }

    #[test]
    fn dropdown_flips_above_when_below_is_too_tight() {
        let form_view = Rect::new(10, 5, 60, 12);
        let input_rect = Rect::new(14, 12, 24, 3);
        let layout = dropdown_geometry(form_view, input_rect, 4).expect("layout");

        assert_eq!(layout.rect.y + layout.rect.height, input_rect.y);
        assert_eq!(layout.visible_rows, 4);
    }

    #[test]
    fn dropdown_height_respects_space_and_max_rows() {
        let form_view = Rect::new(10, 5, 60, 11);
        let input_rect = Rect::new(14, 8, 24, 3);
        let layout = dropdown_geometry(form_view, input_rect, 20).expect("layout");

        assert_eq!(layout.visible_rows, 3);
        assert_eq!(layout.rect.height, 5);

        let roomy_form = Rect::new(10, 5, 60, 30);
        let roomy_layout = dropdown_geometry(roomy_form, input_rect, 20).expect("layout");
        assert_eq!(roomy_layout.visible_rows, MAX_DROPDOWN_ROWS as usize);
        assert_eq!(roomy_layout.rect.height, MAX_DROPDOWN_ROWS + 2);
    }

    #[test]
    fn dropdown_returns_none_when_neither_side_has_room_for_rows() {
        let form_view = Rect::new(0, 0, 40, 3);
        let input_rect = Rect::new(2, 1, 20, 1);

        assert_eq!(dropdown_geometry(form_view, input_rect, 2), None);
    }

    #[test]
    fn dropdown_geometry_clamps_to_form_width_when_input_would_overflow() {
        let form_view = Rect::new(10, 5, 20, 10);
        let input_rect = Rect::new(24, 8, 12, 3);
        let layout = dropdown_geometry(form_view, input_rect, 4).expect("layout");

        assert_eq!(
            layout.rect.x + layout.rect.width,
            form_view.x + form_view.width
        );
    }

    #[test]
    fn dropdown_hides_hidden_choices_and_shows_choice_help() {
        let mut state = AppState::from_command(&Command::new("tool").arg(
            Arg::new("mode").long("mode").value_parser([
                PossibleValue::new("fast").help("Fast path"),
                PossibleValue::new("secret").hide(true),
            ]),
        ));
        state.ui.dropdown_open = Some("mode".to_string());
        let mut snapshot = crate::frame_snapshot::FrameSnapshot::default();
        snapshot.layout.dropdown = Some(Rect::new(0, 0, 30, 4));

        let view = build_dropdown_view(&state.ui, &snapshot, &state.domain, &TuiConfig::default())
            .expect("dropdown view");

        assert_eq!(view.total_rows, 1);
        assert_eq!(view.items[0].label, "fast  Fast path");
    }

    #[test]
    fn dropdown_builds_for_inherited_global_choice_fields() {
        let mut state = AppState::from_command(
            &Command::new("tool")
                .arg(
                    Arg::new("color")
                        .long("color")
                        .global(true)
                        .value_parser(["red", "green"]),
                )
                .subcommand(Command::new("admin")),
        );
        state
            .select_command_path(&["admin".to_string()])
            .expect("valid path");
        state.ui.dropdown_open = Some("color".to_string());
        let mut snapshot = crate::frame_snapshot::FrameSnapshot::default();
        snapshot.layout.dropdown = Some(Rect::new(0, 0, 30, 4));

        let view = build_dropdown_view(&state.ui, &snapshot, &state.domain, &TuiConfig::default())
            .expect("dropdown view");

        assert_eq!(view.total_rows, 2);
        assert_eq!(view.items[0].label, "red");
        assert_eq!(view.items[1].label, "green");
    }

    #[test]
    fn selected_default_row_uses_selection_contrast_instead_of_default_dim() {
        let mut state = AppState::from_command(
            &Command::new("tool").arg(
                Arg::new("mode")
                    .long("mode")
                    .default_value("fast")
                    .value_parser(["fast", "safe"]),
            ),
        );
        state.ui.dropdown_open = Some("mode".to_string());
        state.ui.dropdown_cursor = 0;
        let mut snapshot = crate::frame_snapshot::FrameSnapshot::default();
        snapshot.layout.dropdown = Some(Rect::new(0, 0, 30, 4));
        let config = TuiConfig::default();

        let view = build_dropdown_view(&state.ui, &snapshot, &state.domain, &config)
            .expect("dropdown view");

        assert_eq!(view.items[0].text_style.fg, Some(config.theme.selection_fg));
    }
}
