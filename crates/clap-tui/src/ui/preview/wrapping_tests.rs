use super::*;
use crate::frame_snapshot::FrameSnapshot;
use crate::input::AppState;
use crate::runtime::{AppKeyCode, AppKeyEvent, AppKeyModifiers};
use crate::update::{Action, Effect, apply_action};
use clap::{Arg, Command};
use ratatui::text::Text;
use ratatui::{Terminal, backend::TestBackend};

fn command(value: &str) -> RenderedCommand {
    let text = format!("tool --value '{value}'");
    RenderedCommand {
        text: text.clone(),
        tokens: vec![RenderedShellToken {
            text,
            kind: RenderedShellTokenKind::Value,
        }],
    }
}

fn plain(text: &Text<'_>) -> String {
    text.lines
        .iter()
        .flat_map(|line| &line.spans)
        .map(|span| span.content.as_ref())
        .collect()
}

#[test]
fn wrapping_preserves_quoted_whitespace_unicode_and_long_tokens() {
    let config = TuiConfig::default();
    for value in ["a   quoted   path", "界界界e\u{301}🙂", &"x".repeat(300)] {
        let command = command(value);
        for width in [2, 7, 60, 80] {
            let text = wrapped_preview(&config, Some(&command), false, width);
            assert_eq!(plain(&text), format!("$ {}", command.text));
            assert!(
                text.lines
                    .iter()
                    .all(|line| line.width() <= usize::from(width))
            );
        }
    }
}

#[test]
fn dock_uses_spare_rows_and_marks_overflow() {
    let config = TuiConfig::default();
    let area = Rect::new(0, 0, 80, 4);
    for (length, hidden) in [(100, false), (400, true)] {
        let command = command(&"x".repeat(length));
        let mut buffer = Buffer::empty(area);
        PreviewWidget {
            config: &config,
            layout: PreviewLayout::dock(&config, Some(&command), false, area),
        }
        .render(area, &mut buffer);
        let row = |y| (0..80).map(|x| buffer[(x, y)].symbol()).collect::<String>();
        assert!(row(1).contains("$ tool"));
        assert!(row(2).contains('x'));
        assert_eq!(row(0).contains("more ↓ Ctrl+P expand"), hidden);
        assert_eq!(row(0).contains("Ctrl+P expand"), hidden);
    }
}

#[test]
fn expanded_preview_scrolls_resizes_and_copies_the_entire_command() {
    let mut state =
        AppState::from_command(&Command::new("tool").arg(Arg::new("value").long("value")));
    state
        .domain
        .set_text_value("value", &format!("{} THE_END", "quoted value ".repeat(100)));
    let expected = state.rendered_command().unwrap();
    let config = TuiConfig::default();
    let mut terminal = Terminal::new(TestBackend::new(60, 20)).unwrap();
    apply_action(
        &Action::TogglePreview,
        &mut state,
        &FrameSnapshot::default(),
    );
    let mut snapshot = FrameSnapshot::default();
    terminal
        .draw(|frame| {
            snapshot = super::super::screen::render(frame, &mut state, &config);
        })
        .unwrap();
    assert!(snapshot.preview_scroll_max > 0);
    apply_action(
        &Action::PreviewBoundary { end: true },
        &mut state,
        &snapshot,
    );
    terminal
        .draw(|frame| {
            snapshot = super::super::screen::render(frame, &mut state, &config);
        })
        .unwrap();
    let contents = terminal
        .backend()
        .buffer()
        .content
        .iter()
        .map(ratatui::buffer::Cell::symbol)
        .collect::<String>();
    assert!(contents.contains("THE_END"));
    assert_eq!(
        apply_action(&Action::CopyPreview, &mut state, &snapshot),
        Effect::CopyToClipboard(expected)
    );
    assert!(state.ui.preview_open);
    // A resize clamps the displayed scroll, and the next Up starts there.
    terminal.backend_mut().resize(80, 24);
    terminal.autoresize().unwrap();
    terminal
        .draw(|frame| {
            snapshot = super::super::screen::render(frame, &mut state, &config);
        })
        .unwrap();
    apply_action(&Action::ScrollPreview(-1), &mut state, &snapshot);
    assert_eq!(
        state.ui.preview_scroll,
        snapshot.preview_scroll_max.saturating_sub(1)
    );
    let key = AppKeyEvent::new(AppKeyCode::Esc, AppKeyModifiers::default());
    let action = event_action(key, &state, &snapshot, &config).unwrap();
    apply_action(&action, &mut state, &snapshot);
    assert!(!state.ui.preview_open);
}

#[test]
fn expansion_is_available_only_when_the_rendered_dock_has_hidden_content() {
    let config = TuiConfig::default();
    let ctrl_p = AppKeyEvent::new(
        AppKeyCode::Char('p'),
        AppKeyModifiers {
            control: true,
            ..AppKeyModifiers::default()
        },
    );
    let mut state =
        AppState::from_command(&Command::new("tool").arg(Arg::new("value").long("value")));
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
    for (length, has_more) in [(5, false), (100, false), (400, true)] {
        state.domain.set_text_value("value", &"x".repeat(length));
        let mut snapshot = FrameSnapshot::default();
        terminal
            .draw(|frame| {
                snapshot = super::super::screen::render(frame, &mut state, &config);
            })
            .unwrap();
        assert_eq!(snapshot.preview_has_more, has_more);
        let action = event_action(ctrl_p, &state, &snapshot, &config);
        assert_eq!(action, has_more.then_some(Action::TogglePreview));
        let header: String = terminal
            .backend()
            .buffer()
            .content
            .iter()
            .map(ratatui::buffer::Cell::symbol)
            .collect();
        assert_eq!(header.contains("Ctrl+P expand"), has_more);
    }
    apply_action(
        &Action::TogglePreview,
        &mut state,
        &FrameSnapshot::default(),
    );
    // The expanded view remains closable even though its snapshot has no dock.
    let mut snapshot = FrameSnapshot::default();
    terminal
        .draw(|frame| {
            snapshot = super::super::screen::render(frame, &mut state, &config);
        })
        .unwrap();
    assert!(!snapshot.preview_has_more);
    assert_eq!(
        event_action(ctrl_p, &state, &snapshot, &config),
        Some(Action::TogglePreview)
    );
}

fn event_action(
    key: AppKeyEvent,
    state: &AppState,
    snapshot: &FrameSnapshot,
    config: &TuiConfig,
) -> Option<Action> {
    match crate::controller::handle_event(
        &crate::runtime::AppEvent::Key(key),
        state,
        snapshot,
        config,
    ) {
        crate::controller::EventDecision::Action(action) => Some(action),
        _ => None,
    }
}

#[test]
fn controls_remain_visible_without_changing_clipboard_contents() {
    let config = TuiConfig::default();
    let value = "first\nsecond\tthird\r\nlast\u{1b}";
    let command = command(value);
    for width in [2, 7, 80] {
        let text = wrapped_preview(&config, Some(&command), false, width);
        assert_eq!(plain(&text), "$ tool --value 'first␊second␉third␍␊last␛'");
    }
    let mut state =
        AppState::from_command(&Command::new("tool").arg(Arg::new("value").long("value")));
    state.domain.set_text_value("value", value);
    let serialized = state.rendered_command().unwrap();
    assert!(serialized.contains(value));
    assert_eq!(
        apply_action(&Action::CopyPreview, &mut state, &FrameSnapshot::default()),
        Effect::CopyToClipboard(serialized)
    );
}

#[test]
fn wrapping_retains_styled_runs_instead_of_allocating_per_character() {
    let command = command(&"x".repeat(1000));
    let text = wrapped_preview(&TuiConfig::default(), Some(&command), false, 80);
    assert!(text.lines.iter().all(|line| line.spans.len() <= 2));
}

#[test]
fn layout_exposes_overflow_and_scroll_limits_before_any_painting() {
    let config = TuiConfig::default();
    let command = command(&"x".repeat(400));
    let area = Rect::new(0, 0, 80, 4);
    let mut snapshot = FrameSnapshot::default();
    let dock = PreviewLayout::dock(&config, Some(&command), false, area);
    dock.populate_snapshot(&mut snapshot);
    assert!(snapshot.preview_has_more);
    assert_eq!(snapshot.layout.preview, Some(area));
    assert_eq!(snapshot.preview_scroll_max, 0);
    let expanded = PreviewLayout::expanded(&config, Some(&command), u16::MAX, area);
    expanded.populate_snapshot(&mut snapshot);
    assert!(!snapshot.preview_has_more);
    assert!(snapshot.preview_scroll_max > 0);
    assert_eq!(expanded.scroll, snapshot.preview_scroll_max);
}
