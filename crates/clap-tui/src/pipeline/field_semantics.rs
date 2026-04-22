use std::collections::BTreeMap;

use crate::input::{AppState, ArgInput, ArgInputState, InputSource};
use crate::pipeline::ValidationState;
use crate::spec::{ArgSpec, CommandModel, CommandPath};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct FieldInstanceId {
    pub(crate) owner_path: CommandPath,
    pub(crate) arg_id: String,
}

impl FieldInstanceId {
    pub(crate) fn from_arg(arg: &ArgSpec) -> Self {
        Self {
            owner_path: arg.owner_path().clone(),
            arg_id: arg.id.clone(),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum FieldVisibility {
    #[default]
    Visible,
    Hidden,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum FieldActivity {
    #[default]
    Active,
    NeutralInherited,
    Disabled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum FieldConflictState {
    #[default]
    None,
    PotentialPathConflict,
    ActualValidationConflict,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FieldSemantics {
    pub(crate) id: FieldInstanceId,
    pub(crate) arg_id: String,
    pub(crate) owner_path: CommandPath,
    pub(crate) visibility: FieldVisibility,
    pub(crate) activity: FieldActivity,
    pub(crate) conflict: FieldConflictState,
    pub(crate) required_badge: bool,
    pub(crate) can_edit: bool,
    pub(crate) reason: Option<String>,
}

pub(crate) fn derive_field_semantics(
    state: &AppState,
    validation: &ValidationState,
) -> BTreeMap<FieldInstanceId, FieldSemantics> {
    let selected_path = state.domain.selected_path();
    let current_form = state.domain.current_form().unwrap_or_default();
    state
        .domain
        .root
        .effective_args_for_path(selected_path)
        .into_iter()
        .flatten()
        .filter(|(_, arg)| !arg.is_external_subcommand_field() || arg.owner_path() == selected_path)
        .map(|(_, arg)| {
            let id = FieldInstanceId::from_arg(arg);
            let owner_path = arg.owner_path().clone();
            let inherited = arg.is_inherited_for(selected_path);
            let requirement_negated =
                inherited && owner_negates_requirements(&state.domain.root, selected_path, arg);
            let path_conflict = inherited
                && owner_args_conflict_with_selected_subcommand(
                    &state.domain.root,
                    selected_path,
                    arg,
                );
            let authored = current_form
                .input(&arg.id)
                .is_some_and(input_state_is_user_provided);
            let actual_validation_conflict =
                path_conflict && authored && validation.field_errors.contains_key(&arg.id);

            let conflict = if actual_validation_conflict {
                FieldConflictState::ActualValidationConflict
            } else if path_conflict {
                FieldConflictState::PotentialPathConflict
            } else {
                FieldConflictState::None
            };

            let activity = if path_conflict && !authored {
                FieldActivity::Disabled
            } else if inherited {
                FieldActivity::NeutralInherited
            } else {
                FieldActivity::Active
            };

            let reason = match (conflict, activity) {
                (FieldConflictState::ActualValidationConflict, _) => {
                    Some("Conflicts with the selected subcommand.".to_string())
                }
                (FieldConflictState::PotentialPathConflict, FieldActivity::Disabled) => {
                    Some("Disabled because it conflicts with the selected subcommand.".to_string())
                }
                (FieldConflictState::PotentialPathConflict, _) => {
                    Some("May conflict with the selected subcommand.".to_string())
                }
                _ => None,
            };

            (
                id.clone(),
                FieldSemantics {
                    id,
                    arg_id: arg.id.clone(),
                    owner_path,
                    visibility: FieldVisibility::Visible,
                    activity,
                    conflict,
                    required_badge: arg.required && !requirement_negated,
                    can_edit: activity != FieldActivity::Disabled,
                    reason,
                },
            )
        })
        .collect()
}

fn owner_negates_requirements(
    root: &CommandModel,
    selected_path: &CommandPath,
    arg: &ArgSpec,
) -> bool {
    selected_path_descends_from(selected_path, arg.owner_path())
        && root
            .resolve_path(arg.owner_path().as_slice())
            .is_some_and(|owner| owner.parser_rules.subcommand_negates_reqs)
}

fn owner_args_conflict_with_selected_subcommand(
    root: &CommandModel,
    selected_path: &CommandPath,
    arg: &ArgSpec,
) -> bool {
    selected_path_descends_from(selected_path, arg.owner_path())
        && root
            .resolve_path(arg.owner_path().as_slice())
            .is_some_and(|owner| owner.parser_rules.args_conflicts_with_subcommands)
}

fn selected_path_descends_from(selected_path: &CommandPath, owner_path: &CommandPath) -> bool {
    selected_path.as_slice().len() > owner_path.as_slice().len()
        && selected_path.as_slice().starts_with(owner_path.as_slice())
}

fn input_state_is_user_provided(input: &ArgInputState) -> bool {
    if input.touched {
        return true;
    }

    match &input.value {
        ArgInput::Flag { present, source } => *present && *source == InputSource::User,
        ArgInput::Count {
            occurrences,
            source,
        } => *occurrences > 0 && *source == InputSource::User,
        ArgInput::Values { occurrences } => occurrences.iter().any(|occurrence| {
            occurrence.source == InputSource::User
                && occurrence.values.iter().any(|value| !value.is_empty())
        }),
    }
}
