use super::*;
use crate::input::AppState;
use crate::runtime::{AppKeyCode, AppKeyEvent, AppKeyModifiers};
use crate::update::{Action, Effect, apply_action};
use clap::{Arg, Command};
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
            command: Some(&command),
            hovered: false,
        }
        .render(area, &mut buffer);
        let row = |y| (0..80).map(|x| buffer[(x, y)].symbol()).collect::<String>();
        assert!(row(1).contains("$ tool"));
        assert!(row(2).contains('x'));
        assert_eq!(row(0).contains("more ↓"), hidden);
        assert!(row(0).contains("Ctrl+P expand"));
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
    let action = crate::controller::handle_key_event(key, &state, &snapshot, &config).unwrap();
    apply_action(&action, &mut state, &snapshot);
    assert!(!state.ui.preview_open);
}
