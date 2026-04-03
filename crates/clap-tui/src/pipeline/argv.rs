use crate::argv_serializer;
use crate::input::AppState;

pub(crate) fn build_command_line(state: &AppState) -> Vec<String> {
    build_command_line_with_mode(state, false)
}

pub(crate) fn build_parse_command_line(state: &AppState) -> Vec<String> {
    build_command_line_with_mode(state, true)
}

fn build_command_line_with_mode(
    state: &AppState,
    include_materialized_non_user: bool,
) -> Vec<String> {
    let mut command_line = vec![state.domain.root.name.clone()];
    let lineage = state
        .domain
        .root
        .command_lineage(state.domain.selected_path())
        .unwrap_or_default();

    for (index, command) in lineage.iter().enumerate() {
        let key = state.domain.command_path_key_for(&command.path);
        let form = state.domain.forms.get(&key).cloned().unwrap_or_default();
        command_line.extend(argv_serializer::build_argv(
            command,
            &form,
            lineage.get(index + 1).is_some(),
            include_materialized_non_user,
        ));
        if let Some(next) = lineage.get(index + 1) {
            command_line.push(next.name.clone());
        }
    }

    command_line
}
