use crate::input::AppState;
use crate::pipeline;

use super::{Action, Effect};

pub(crate) fn apply(action: &Action, state: &mut AppState) -> Effect {
    match action {
        Action::Exit => Effect::Exit,
        Action::Run => Effect::Run(pipeline::build_command_line(state)),
        Action::CopyPreview => Effect::CopyToClipboard(pipeline::build_command_line(state).join(" ")),
        _ => Effect::None,
    }
}
