mod keyboard;
mod mouse;
pub(crate) mod navigation;

pub(crate) use keyboard::handle_key_event;
pub(crate) use mouse::handle_mouse_event;

mod preview;

use crate::config::TuiConfig;
use crate::frame_snapshot::FrameSnapshot;
use crate::input::AppState;
use crate::runtime::AppEvent;
use crate::update::Action;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum EventDecision {
    Action(Action),
    Redraw,
    Ignore,
}

pub(crate) fn handle_event(
    event: &AppEvent,
    state: &AppState,
    snapshot: &FrameSnapshot,
    config: &TuiConfig,
) -> EventDecision {
    if let Some(decision) = preview::route(event, state, snapshot) {
        return decision;
    }
    let action = match event {
        AppEvent::Key(key) => handle_key_event(*key, state, snapshot, config),
        AppEvent::Mouse(mouse) => handle_mouse_event(*mouse, state, snapshot, config),
        AppEvent::Paste(text) => Some(Action::Paste(text.clone())),
        AppEvent::Resize { .. } => return EventDecision::Redraw,
        AppEvent::FocusGained | AppEvent::FocusLost | AppEvent::Unsupported => None,
    };
    action.map_or(EventDecision::Ignore, EventDecision::Action)
}
