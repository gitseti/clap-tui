use ratatui::layout::{Margin, Position};

use crate::{
    frame_snapshot::FrameSnapshot,
    input::{HoverTarget, UiState},
    layout::{form::FormFieldLayout, hit_test},
    query::form::{self, FieldWidget, OptionalValueAction},
    repeated_field::REPEATED_ROW_HEIGHT,
    runtime::MousePointer,
};

use super::screen::ScreenView;

pub(super) fn resolve(
    ui: &UiState,
    snapshot: &FrameSnapshot,
    view: &ScreenView<'_>,
) -> MousePointer {
    if ui.sidebar.is_resizing() && snapshot.layout.sidebar_divider.is_some() {
        return MousePointer::ResizeHorizontal;
    }
    if ui.mouse_select.is_some() && !ui.preview_open && !ui.help_open {
        return MousePointer::Text;
    }
    let Some(point) = ui.mouse_position else {
        return MousePointer::Default;
    };
    if snapshot.preview_contains(point.x, point.y) {
        return MousePointer::PointingHand;
    }
    if snapshot.dropdown_contains(point.x, point.y) {
        return dropdown_pointer(ui, snapshot, view, point);
    }
    if snapshot
        .layout
        .sidebar_divider
        .is_some_and(|area| area.contains(point))
    {
        return MousePointer::ResizeHorizontal;
    }
    if snapshot
        .layout
        .sidebar_toggle
        .is_some_and(|area| area.contains(point))
        || snapshot
            .footer_target_at(point.x, point.y)
            .is_some_and(|target| {
                target != HoverTarget::FooterStatus || view.validation.has_target()
            })
        || snapshot.tab_at(point.x, point.y).is_some()
        || snapshot.sidebar_item_at(point.x, point.y).is_some()
        || snapshot
            .layout
            .sidebar
            .is_some_and(|area| area.contains(point) && point.y == area.y)
    {
        return MousePointer::PointingHand;
    }
    if snapshot.layout.search.is_some_and(|area| {
        area.inner(Margin {
            horizontal: 1,
            vertical: 1,
        })
        .contains(point)
    }) {
        return MousePointer::Text;
    }
    if ui.help_open {
        return MousePointer::Default;
    }
    field_pointer(snapshot, view, point)
}

fn dropdown_pointer(
    ui: &UiState,
    snapshot: &FrameSnapshot,
    view: &ScreenView<'_>,
    point: Position,
) -> MousePointer {
    let Some(item) = view
        .active_args
        .iter()
        .find(|item| ui.dropdown_open.as_deref() == Some(item.arg.id.as_str()))
    else {
        return MousePointer::Default;
    };
    let total = item.arg.choices.len()
        + usize::from(form::single_choice_has_clear_row(
            item.arg,
            item.widget,
            view.field_required(item.arg),
        ));
    let scroll = ui.dropdown_scroll(total, snapshot.dropdown_visible_rows().unwrap_or(0));
    if view.field_can_edit(item.arg)
        && snapshot
            .dropdown_choice_index(point.y, scroll)
            .is_some_and(|index| index < total)
    {
        MousePointer::PointingHand
    } else {
        MousePointer::Default
    }
}

fn field_pointer(snapshot: &FrameSnapshot, view: &ScreenView<'_>, point: Position) -> MousePointer {
    let Some(field) = snapshot
        .layout
        .form_fields
        .iter()
        .find(|field| field.input.contains(point))
    else {
        return MousePointer::Default;
    };
    let Some(item) = view
        .active_args
        .iter()
        .find(|item| item.arg.id == field.arg_id)
    else {
        return MousePointer::Default;
    };
    if !view.field_can_edit(item.arg) {
        return MousePointer::Default;
    }
    match item.widget {
        FieldWidget::SingleText => text_pointer(field, point),
        FieldWidget::RepeatedText => {
            let rows = usize::from(field.full_input_height / REPEATED_ROW_HEIGHT).max(1);
            match hit_test::repeated_input_target_from_point(field, rows, point.x, point.y) {
                Some(hit_test::RepeatedInputTarget::Text { .. }) => MousePointer::Text,
                Some(hit_test::RepeatedInputTarget::Add { .. }) => MousePointer::PointingHand,
                Some(hit_test::RepeatedInputTarget::Remove { .. }) if rows > 1 => {
                    MousePointer::PointingHand
                }
                _ => MousePointer::Default,
            }
        }
        FieldWidget::Counter => {
            if hit_test::compact_control_content_row_contains(field, point.y)
                && hit_test::counter_click_target(field.input, point.x).is_some()
            {
                MousePointer::PointingHand
            } else {
                MousePointer::Default
            }
        }
        FieldWidget::OptionalValue => {
            let input = view
                .inputs
                .as_ref()
                .and_then(|inputs| inputs.input(&item.arg.id));
            if form::optional_value_action(input) == OptionalValueAction::Edit {
                text_pointer(field, point)
            } else {
                MousePointer::PointingHand
            }
        }
        FieldWidget::Toggle | FieldWidget::SingleChoice | FieldWidget::MultiChoice => {
            MousePointer::PointingHand
        }
    }
}

fn text_pointer(field: &FormFieldLayout, point: Position) -> MousePointer {
    if hit_test::text_input_position_from_point(field, point.x, point.y, false).is_some() {
        MousePointer::Text
    } else {
        MousePointer::Default
    }
}

#[cfg(test)]
mod tests;
