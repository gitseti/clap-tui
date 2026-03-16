use std::collections::BTreeMap;

use crate::input::{AppState, ArgValue, CommandFormState};
use crate::pipeline::ValidationState;
use crate::spec::{ArgModel, CommandModel};

pub(crate) fn build_validation_state(state: &AppState) -> ValidationState {
    build_validation_state_for(
        state.domain.current_command(),
        &state.domain.current_form().cloned().unwrap_or_default(),
    )
}

fn build_validation_state_for(command: &CommandModel, form: &CommandFormState) -> ValidationState {
    let mut field_errors = BTreeMap::new();
    let mut missing = Vec::new();

    for arg in command.args.iter().filter(|arg| arg.required) {
        if arg_is_missing(arg, form) {
            field_errors.insert(arg.id.clone(), "Required argument".to_string());
            missing.push(arg.display_name.clone());
        }
    }

    if field_errors.is_empty() {
        ValidationState {
            is_valid: true,
            summary: None,
            field_errors,
        }
    } else {
        ValidationState {
            is_valid: false,
            summary: Some(format!("Missing required: {}", missing.join(", "))),
            field_errors,
        }
    }
}

fn arg_is_missing(arg: &ArgModel, form: &CommandFormState) -> bool {
    match form.values.get(&arg.id) {
        Some(ArgValue::Text(value)) => value.is_empty(),
        Some(ArgValue::Bool(value)) => !value,
        Some(ArgValue::Choice(_)) => false,
        None => true,
    }
}
