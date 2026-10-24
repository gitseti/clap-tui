use super::{preview_command_inset, preview_header_line, wrapped_preview};
use crate::argv_serializer::RenderedCommand;
use crate::config::TuiConfig;
use crate::frame_snapshot::FrameSnapshot;
use ratatui::layout::{Margin, Rect};
use ratatui::text::{Line, Text};

// One projection owns the text, geometry and interaction metadata for a frame.
#[derive(Debug, Clone)]
pub(crate) struct PreviewLayout {
    pub(super) area: Rect,
    pub(super) header_area: Rect,
    pub(super) content_area: Rect,
    pub(super) header: Line<'static>,
    pub(super) footer: Option<(Rect, Line<'static>)>,
    pub(super) text: Text<'static>,
    pub(super) scroll: u16,
    pub(super) hovered: bool,
    scroll_max: u16,
    has_more: bool,
}

impl PreviewLayout {
    pub(crate) fn dock(
        config: &TuiConfig,
        command: Option<&RenderedCommand>,
        hovered: bool,
        area: Rect,
    ) -> Self {
        let inner = area.inner(Margin {
            horizontal: 1,
            vertical: 0,
        });
        let inset = preview_command_inset(inner.width).min(inner.width);
        let width = inner.width.saturating_sub(inset);
        let text = wrapped_preview(config, command, hovered, width);
        let offset = if area.height >= 4 && text.lines.len() == 1 {
            2
        } else {
            1
        }
        .min(area.height);
        let content_area = Rect::new(
            inner.x.saturating_add(inset),
            area.y.saturating_add(offset),
            width,
            area.height.saturating_sub(offset),
        );
        let has_more = text.lines.len() > usize::from(content_area.height);
        Self {
            area,
            content_area,
            text,
            hovered,
            header_area: Rect::new(inner.x, area.y, inner.width, area.height.min(1)),
            header: preview_header_line(config, hovered, inner.width, has_more),
            footer: None,
            scroll: 0,
            scroll_max: 0,
            has_more,
        }
    }

    pub(crate) fn expanded(
        config: &TuiConfig,
        command: Option<&RenderedCommand>,
        requested_scroll: u16,
        area: Rect,
    ) -> Self {
        let content_area = area.inner(Margin {
            horizontal: 1,
            vertical: 1,
        });
        let text = wrapped_preview(config, command, false, content_area.width);
        let lines = text.lines.len();
        let scroll_max = u16::try_from(lines.saturating_sub(usize::from(content_area.height)))
            .unwrap_or(u16::MAX);
        let scroll = requested_scroll.min(scroll_max);
        let start = if lines == 0 {
            0
        } else {
            usize::from(scroll) + 1
        };
        let end = (usize::from(scroll) + usize::from(content_area.height)).min(lines);
        Self {
            area,
            content_area,
            text,
            scroll,
            scroll_max,
            header_area: Rect::new(area.x, area.y, area.width, area.height.min(1)),
            header: Line::styled(
                format!(" Command Preview · lines {start}–{end}/{lines}"),
                super::styles::preview_title(config),
            ),
            footer: (area.height > 1).then(|| {
                (
                    Rect::new(area.x, area.bottom().saturating_sub(1), area.width, 1),
                    Line::from(" Esc close · ↑/↓ scroll · PgUp/Dn · Ctrl+Y copy"),
                )
            }),
            hovered: false,
            has_more: false,
        }
    }

    pub(crate) fn populate_snapshot(&self, snapshot: &mut FrameSnapshot) {
        snapshot.layout.preview = Some(self.area);
        snapshot.preview_has_more = self.has_more;
        snapshot.preview_scroll_max = self.scroll_max;
    }
}
