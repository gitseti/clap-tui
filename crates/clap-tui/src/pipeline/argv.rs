use crate::argv_serializer;
use crate::input::AppState;

pub(crate) fn build_command_line(state: &AppState) -> Vec<String> {
    let mut command_line = vec![state.domain.root.name.clone()];
    command_line.extend(state.domain.selected_path().iter().cloned());
    let form = state.domain.current_form().cloned().unwrap_or_default();
    command_line.extend(argv_serializer::build_argv(
        state.domain.current_command(),
        &form,
    ));
    command_line
}
