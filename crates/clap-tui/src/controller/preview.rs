use super::EventDecision;
use crate::frame_snapshot::FrameSnapshot;
use crate::input::AppState;
use crate::runtime::{AppEvent, AppKeyCode, AppMouseButton, AppMouseEventKind};
use crate::update::Action;

// None delegates to the normal controller. An open preview owns every event,
// including events it deliberately ignores, so form input cannot leak through.
pub(super) fn route(
    event: &AppEvent,
    state: &AppState,
    snapshot: &FrameSnapshot,
) -> Option<EventDecision> {
    if let AppEvent::Key(key) = event
        && key.modifiers.control
        && key.code == AppKeyCode::Char('p')
    {
        return Some(if state.ui.preview_open || snapshot.preview_has_more {
            EventDecision::Action(Action::TogglePreview)
        } else {
            EventDecision::Ignore
        });
    }
    if !state.ui.preview_open {
        return None;
    }
    let action = match event {
        AppEvent::Key(key) => match key.code {
            AppKeyCode::Char('c') if key.modifiers.control => Some(Action::Exit),
            AppKeyCode::Char('y') if key.modifiers.control => Some(Action::CopyPreview),
            AppKeyCode::Esc => Some(Action::TogglePreview),
            AppKeyCode::Up => Some(Action::ScrollPreview(-1)),
            AppKeyCode::Down => Some(Action::ScrollPreview(1)),
            AppKeyCode::PageUp => Some(Action::ScrollPreview(-10)),
            AppKeyCode::PageDown => Some(Action::ScrollPreview(10)),
            AppKeyCode::Home => Some(Action::PreviewBoundary { end: false }),
            AppKeyCode::End => Some(Action::PreviewBoundary { end: true }),
            _ => None,
        },
        AppEvent::Mouse(mouse) => match mouse.kind {
            AppMouseEventKind::ScrollUp => Some(Action::ScrollPreview(-2)),
            AppMouseEventKind::ScrollDown => Some(Action::ScrollPreview(2)),
            AppMouseEventKind::Down(AppMouseButton::Left) => Some(Action::CopyPreview),
            _ => None,
        },
        AppEvent::Resize { .. } => return Some(EventDecision::Redraw),
        _ => None,
    };
    Some(action.map_or(EventDecision::Ignore, EventDecision::Action))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::{AppKeyEvent, AppKeyModifiers, AppMouseEvent};

    #[test]
    fn modal_policy_owns_all_event_types_and_delegates_when_closed() {
        let mut state = AppState::from_command(&clap::Command::new("tool"));
        let snapshot = FrameSnapshot::default();
        let paste = AppEvent::Paste("must not reach the form".into());
        assert_eq!(route(&paste, &state, &snapshot), None);
        state.ui.preview_open = true;
        assert_eq!(
            route(&paste, &state, &snapshot),
            Some(EventDecision::Ignore)
        );
        assert_eq!(
            route(
                &AppEvent::Resize {
                    width: 80,
                    height: 24
                },
                &state,
                &snapshot
            ),
            Some(EventDecision::Redraw)
        );
        for kind in [AppMouseEventKind::ScrollUp, AppMouseEventKind::ScrollDown] {
            let event = AppEvent::Mouse(AppMouseEvent {
                kind,
                column: 0,
                row: 0,
                modifiers: AppKeyModifiers::default(),
            });
            let expected = if kind == AppMouseEventKind::ScrollUp {
                -2
            } else {
                2
            };
            assert_eq!(
                route(&event, &state, &snapshot),
                Some(EventDecision::Action(Action::ScrollPreview(expected)))
            );
        }
        for (code, expected) in [
            ('r', EventDecision::Ignore),
            ('y', EventDecision::Action(Action::CopyPreview)),
            ('c', EventDecision::Action(Action::Exit)),
        ] {
            let event = AppEvent::Key(AppKeyEvent::new(
                AppKeyCode::Char(code),
                AppKeyModifiers {
                    control: true,
                    ..AppKeyModifiers::default()
                },
            ));
            assert_eq!(route(&event, &state, &snapshot), Some(expected));
        }
    }
}
