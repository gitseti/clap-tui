use crate::frame_snapshot::FrameSnapshot;
use crate::input::AppState;

use super::{Action, Effect};

pub(crate) fn apply(action: &Action, state: &mut AppState, snapshot: &FrameSnapshot) -> Effect {
    match action {
        Action::Exit => Effect::Exit,
        Action::Run => {
            state.ui.dismiss_transient_interaction();
            let validation = state.derived_validation();
            if validation.is_valid() {
                return Effect::Run(state.authoritative_argv());
            }
            crate::controller::navigation::focus_first_invalid_field(state, snapshot, &validation);
            Effect::None
        }
        Action::CopyPreview => {
            state.ui.dismiss_transient_interaction();
            match state.rendered_command() {
                Some(command) => Effect::CopyToClipboard(command),
                None => Effect::None,
            }
        }
        _ => Effect::None,
    }
}

#[cfg(test)]
mod tests {
    use clap::{Arg, Command};

    use crate::input::AppState;

    use super::{Action, Effect, apply};

    #[test]
    fn copy_preview_returns_explicit_clipboard_effect_and_clears_transient_ui() {
        let mut state =
            AppState::from_command(&Command::new("tool").arg(Arg::new("name").long("name")));
        state.ui.dropdown_open = Some("name".to_string());

        let effect = apply(
            &Action::CopyPreview,
            &mut state,
            &crate::frame_snapshot::FrameSnapshot::default(),
        );

        assert_eq!(effect, Effect::CopyToClipboard("tool".to_string()));
        assert!(state.ui.dropdown_open.is_none());
    }
}
