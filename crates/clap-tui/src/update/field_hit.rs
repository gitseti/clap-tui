use ratatui::layout::Rect;

use crate::layout::form::FormFieldLayout;
use crate::repeated_field::{
    REPEATED_ROW_HEIGHT, repeated_add_rect, repeated_remove_rect, repeated_row_textarea_rect,
};

const COMPACT_INPUT_HEIGHT: u16 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RepeatedInputTarget {
    Remove { row: u16 },
    Add { row: u16 },
    Text { row: u16, col: u16 },
}

#[derive(Debug, Clone, Copy)]
struct SignedRect {
    x: i32,
    y: i32,
    width: u16,
    height: u16,
}

impl SignedRect {
    fn from_rect(rect: Rect) -> Self {
        Self {
            x: i32::from(rect.x),
            y: i32::from(rect.y),
            width: rect.width,
            height: rect.height,
        }
    }

    fn contains(self, x: u16, y: u16) -> bool {
        let x = i32::from(x);
        let y = i32::from(y);
        x >= self.x
            && x < self.x + i32::from(self.width)
            && y >= self.y
            && y < self.y + i32::from(self.height)
    }
}

pub(crate) fn text_input_position_from_point(
    field: &FormFieldLayout,
    x: u16,
    y: u16,
    clamp: bool,
) -> Option<(u16, u16)> {
    input_position_from_clipped_control(field, COMPACT_INPUT_HEIGHT, x, y, clamp)
}

pub(crate) fn compact_control_content_row_contains(field: &FormFieldLayout, row: u16) -> bool {
    if row < field.input.y || row >= field.input.y.saturating_add(field.input.height) {
        return false;
    }
    let visible_offset = row.saturating_sub(field.input.y);
    field.input_clip_top.saturating_add(visible_offset) == 1
}

pub(crate) fn repeated_input_target_from_point(
    field: &FormFieldLayout,
    total_rows: usize,
    x: u16,
    y: u16,
) -> Option<RepeatedInputTarget> {
    let row_index = repeated_row_index_at_point(field, total_rows, y)?;
    let row = u16::try_from(row_index).unwrap_or(u16::MAX);
    let row_rect = repeated_row_rect(field, row_index);
    let show_add = row_index + 1 == total_rows;

    if let Some(remove) = repeated_child_rect(
        row_rect,
        repeated_remove_rect(row_rect.rect(), true, show_add),
    ) && remove.contains(x, y)
    {
        return Some(RepeatedInputTarget::Remove { row });
    }
    if let Some(add) = repeated_child_rect(row_rect, repeated_add_rect(row_rect.rect()))
        && add.contains(x, y)
    {
        return Some(RepeatedInputTarget::Add { row });
    }

    repeated_text_position_from_point(field, total_rows, x, y, false)
        .map(|(row, col)| RepeatedInputTarget::Text { row, col })
}

pub(crate) fn repeated_text_position_from_point(
    field: &FormFieldLayout,
    total_rows: usize,
    x: u16,
    y: u16,
    clamp: bool,
) -> Option<(u16, u16)> {
    if total_rows == 0 {
        return None;
    }
    if !clamp {
        let row_index = repeated_row_index_at_point(field, total_rows, y)?;
        let show_add = row_index + 1 == total_rows;
        let row_rect = repeated_row_rect(field, row_index);
        let textarea = repeated_child_rect(
            row_rect,
            Some(repeated_row_textarea_rect(row_rect.rect(), true, show_add)),
        )?;
        let inner = inner_rect(textarea)?;
        let visible_inner = visible_inner_rect(inner, field.input)?;
        return point_position_in_inner(inner, visible_inner, x, y, false)
            .map(|(_, col)| (u16::try_from(row_index).unwrap_or(u16::MAX), col));
    }

    let mut nearest: Option<(usize, SignedRect, SignedRect, i32)> = None;
    for row_index in 0..total_rows {
        let show_add = row_index + 1 == total_rows;
        let row_rect = repeated_row_rect(field, row_index);
        let Some(textarea) = repeated_child_rect(
            row_rect,
            Some(repeated_row_textarea_rect(row_rect.rect(), true, show_add)),
        ) else {
            continue;
        };
        let Some(inner) = inner_rect(textarea) else {
            continue;
        };
        let Some(visible_inner) = visible_inner_rect(inner, field.input) else {
            continue;
        };
        let distance = vertical_distance_to_rect(visible_inner, y);
        let replace = nearest.is_none_or(|(_, _, _, current)| distance < current);
        if replace {
            nearest = Some((row_index, inner, visible_inner, distance));
        }
    }

    let (row_index, textarea, visible_inner, _) = nearest?;
    point_position_in_inner(textarea, visible_inner, x, y, true)
        .map(|(_, col)| (u16::try_from(row_index).unwrap_or(u16::MAX), col))
}

fn input_position_from_clipped_control(
    field: &FormFieldLayout,
    full_height: u16,
    x: u16,
    y: u16,
    clamp: bool,
) -> Option<(u16, u16)> {
    let full = SignedRect {
        x: i32::from(field.input.x),
        y: i32::from(field.input.y) - i32::from(field.input_clip_top),
        width: field.input.width,
        height: full_height,
    };
    let inner = inner_rect(full)?;
    let visible_inner = visible_inner_rect(inner, field.input)?;
    point_position_in_inner(inner, visible_inner, x, y, clamp)
}

fn repeated_row_index_at_point(
    field: &FormFieldLayout,
    total_rows: usize,
    y: u16,
) -> Option<usize> {
    if y < field.input.y || y >= field.input.y.saturating_add(field.input.height) {
        return None;
    }
    let relative_y = y.saturating_sub(field.input.y);
    let content_y = field.input_clip_top.saturating_add(relative_y);
    let row_index = usize::from(content_y / REPEATED_ROW_HEIGHT);
    (row_index < total_rows).then_some(row_index)
}

fn repeated_row_rect(field: &FormFieldLayout, row_index: usize) -> SignedRect {
    let row_offset = u16::try_from(row_index)
        .unwrap_or(u16::MAX)
        .saturating_mul(REPEATED_ROW_HEIGHT);
    SignedRect {
        x: i32::from(field.input.x),
        y: i32::from(field.input.y) + i32::from(row_offset) - i32::from(field.input_clip_top),
        width: field.input.width,
        height: REPEATED_ROW_HEIGHT,
    }
}

fn repeated_child_rect(row: SignedRect, child: Option<Rect>) -> Option<SignedRect> {
    let child = child?;
    Some(SignedRect {
        x: i32::from(child.x),
        y: row.y + i32::from(child.y),
        width: child.width,
        height: child.height,
    })
}

fn inner_rect(rect: SignedRect) -> Option<SignedRect> {
    let width = rect.width.checked_sub(2)?;
    let height = rect.height.checked_sub(2)?;
    (width > 0 && height > 0).then_some(SignedRect {
        x: rect.x + 1,
        y: rect.y + 1,
        width,
        height,
    })
}

fn visible_inner_rect(inner: SignedRect, visible_input: Rect) -> Option<SignedRect> {
    intersect_signed_rects(inner, SignedRect::from_rect(visible_input))
}

fn intersect_signed_rects(a: SignedRect, b: SignedRect) -> Option<SignedRect> {
    let left = a.x.max(b.x);
    let top = a.y.max(b.y);
    let right = (a.x + i32::from(a.width)).min(b.x + i32::from(b.width));
    let bottom = (a.y + i32::from(a.height)).min(b.y + i32::from(b.height));
    if left >= right || top >= bottom {
        return None;
    }
    Some(SignedRect {
        x: left,
        y: top,
        width: u16::try_from(right - left).ok()?,
        height: u16::try_from(bottom - top).ok()?,
    })
}

fn point_position_in_inner(
    full_inner: SignedRect,
    visible_inner: SignedRect,
    x: u16,
    y: u16,
    clamp: bool,
) -> Option<(u16, u16)> {
    if !clamp && !visible_inner.contains(x, y) {
        return None;
    }

    let x = i32::from(x);
    let y = i32::from(y);
    let clamped_x = if clamp {
        x.clamp(
            visible_inner.x,
            visible_inner.x + i32::from(visible_inner.width) - 1,
        )
    } else {
        x
    };
    let clamped_y = if clamp {
        y.clamp(
            visible_inner.y,
            visible_inner.y + i32::from(visible_inner.height) - 1,
        )
    } else {
        y
    };

    Some((
        u16::try_from(clamped_y.saturating_sub(full_inner.y)).ok()?,
        u16::try_from(clamped_x.saturating_sub(full_inner.x)).ok()?,
    ))
}

fn vertical_distance_to_rect(rect: SignedRect, y: u16) -> i32 {
    let y = i32::from(y);
    if y < rect.y {
        rect.y - y
    } else if y >= rect.y + i32::from(rect.height) {
        y - (rect.y + i32::from(rect.height) - 1)
    } else {
        0
    }
}

impl SignedRect {
    fn rect(self) -> Rect {
        Rect::new(
            u16::try_from(self.x).unwrap_or(0),
            0,
            self.width,
            self.height,
        )
    }
}
