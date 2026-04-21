use crate::argv_serializer;
use crate::argv_serializer::SerializationResult;
use crate::input::AppState;
use crate::spec::CommandPath;

pub(crate) fn serialize_authoritative_invocation(state: &AppState) -> SerializationResult {
    let mut result = SerializationResult::default();
    argv_serializer::push_command_token(
        &mut result,
        state.domain.root.name.clone(),
        CommandPath::default(),
    );
    let lineage = state
        .domain
        .root
        .command_lineage(state.domain.selected_path())
        .unwrap_or_default();

    for (index, command) in lineage.iter().enumerate() {
        let key = state.domain.command_path_key_for(&command.path);
        let form = state.domain.forms.get(&key).cloned().unwrap_or_default();
        let serialized =
            argv_serializer::build_argv(command, &form, lineage.get(index + 1).is_some());
        let offset = result.argv.len();
        result.argv.extend(serialized.argv);
        result
            .provenance
            .extend(serialized.provenance.into_iter().map(|mut provenance| {
                provenance.token_index += offset;
                provenance
            }));
        result.diagnostics.extend(serialized.diagnostics);
        if let Some(next) = lineage.get(index + 1) {
            argv_serializer::push_subcommand_token(
                &mut result,
                next.name.clone(),
                next.path.clone(),
            );
        }
    }

    result
}
