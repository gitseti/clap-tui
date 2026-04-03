use crate::input::AppState;

use super::{Action, Effect};

pub(crate) fn apply(action: &Action, state: &mut AppState) -> Effect {
    match action {
        Action::Exit => Effect::Exit,
        Action::Run => Effect::Run(state.preview_argv()),
        Action::CopyPreview => {
            state.ui.dismiss_transient_interaction();
            Effect::CopyToClipboard(state.preview_argv().join(" "))
        }
        _ => Effect::None,
    }
}
