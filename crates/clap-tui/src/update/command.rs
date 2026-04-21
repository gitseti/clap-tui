use crate::input::AppState;

use super::{Action, Effect};

pub(crate) fn apply(action: &Action, state: &mut AppState) -> Effect {
    match action {
        Action::Exit => Effect::Exit,
        Action::Run => Effect::Run(state.authoritative_argv()),
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

        let effect = apply(&Action::CopyPreview, &mut state);

        assert_eq!(effect, Effect::CopyToClipboard("tool".to_string()));
        assert!(state.ui.dropdown_open.is_none());
    }
}
