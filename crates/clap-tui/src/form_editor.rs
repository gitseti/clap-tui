use crate::editor_state::TextEditor;
use crate::input::{
    AppState, ArgInput, ArgValue, InputSource, InputValueOccurrence, UiState,
    render_occurrence_text, split_occurrence_values,
};
use crate::runtime::AppKeyCode;
use crate::runtime::AppKeyEvent;
use crate::spec::{ArgModel, CommandPath};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EditResult {
    Ignored,
    Handled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RowEditorMode {
    Occurrence,
    FixedArityOccurrence { values_per_occurrence: usize },
    GroupedValues,
}

pub(crate) fn displayed_text(state: &AppState, arg: &ArgModel) -> String {
    if uses_row_editor(arg)
        && let Some(text) = row_editor_displayed_text(state, arg)
    {
        return text;
    }
    if let Some(inputs) = state.domain.current_form()
        && let Some(ArgValue::Text(text)) = inputs.compatibility_value(arg)
    {
        return text;
    }
    if arg.default_value().is_some() && !state.domain.is_touched(&arg.id) {
        return arg.default_value().unwrap_or_default().to_string();
    }
    String::new()
}

pub(crate) fn editor_for_render(
    ui: &UiState,
    command_key: &CommandPath,
    arg: &ArgModel,
    displayed: &str,
) -> TextEditor {
    ui.editors
        .editor(command_key, &arg.id)
        .filter(|editor| editor_matches_displayed(arg, editor, displayed))
        .cloned()
        .unwrap_or_else(|| TextEditor::from_displayed(displayed))
}

pub(crate) fn ensure_editor<'a>(
    ui: &'a mut UiState,
    command_key: &CommandPath,
    arg: &ArgModel,
    displayed: &str,
) -> &'a mut TextEditor {
    ui.editors
        .ensure_editor_with(command_key, &arg.id, displayed, |editor, current| {
            editor_matches_displayed(arg, editor, current)
        })
}

fn editor_matches_displayed(arg: &ArgModel, editor: &TextEditor, displayed: &str) -> bool {
    if editor.text() == displayed {
        return true;
    }

    uses_row_editor(arg) && normalize_row_editor_text(arg, &editor.text()) == displayed
}

fn normalize_row_editor_text(arg: &ArgModel, text: &str) -> String {
    let rows = text
        .lines()
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();

    match row_editor_mode(arg) {
        Some(RowEditorMode::Occurrence) => rows
            .into_iter()
            .map(|row| render_occurrence_row_text(arg, &split_row_occurrence_values(arg, row)))
            .filter(|value| !value.is_empty())
            .collect::<Vec<_>>()
            .join("\n"),
        Some(RowEditorMode::FixedArityOccurrence {
            values_per_occurrence,
        }) => group_occurrence_values(
            rows.into_iter()
                .flat_map(|row| split_row_occurrence_values(arg, row))
                .filter(|value| !value.is_empty())
                .collect(),
            values_per_occurrence,
        )
        .into_iter()
        .map(|values| render_occurrence_row_text(arg, &values))
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>()
        .join("\n"),
        Some(RowEditorMode::GroupedValues) => rows
            .into_iter()
            .flat_map(|row| split_occurrence_values(arg, row))
            .filter(|value| !value.is_empty())
            .collect::<Vec<_>>()
            .join("\n"),
        None => rows.join("\n"),
    }
}

pub(crate) fn apply_key_to_text_field(
    state: &mut AppState,
    arg: &ArgModel,
    key: AppKeyEvent,
) -> EditResult {
    let displayed = displayed_text(state, arg);
    let command_key = arg.owner_path().clone();
    let is_touched = state.domain.is_touched(&arg.id);
    let has_default = arg.default_value().is_some();
    let textarea = ensure_editor(&mut state.ui, &command_key, arg, &displayed);
    if row_editor_boundary_merge(textarea, key, arg) {
        return EditResult::Ignored;
    }
    if has_default && !is_touched {
        match key.code {
            AppKeyCode::Char(_) | AppKeyCode::Backspace => {
                *textarea = TextEditor::from_displayed("");
            }
            _ => {}
        }
    }
    if !textarea.apply_key(key) {
        return EditResult::Ignored;
    }
    let text = textarea.text();
    if text.is_empty() && has_default {
        state.domain.clear_value_and_untouch(&arg.id);
    } else if text.is_empty() && arg.uses_optional_value_semantics() {
        state.domain.toggle_optional_value_flag(&arg.id, true);
    } else if uses_row_editor(arg) {
        let rows = textarea.lines().to_vec();
        sync_row_editor_values(state, arg, &rows);
    } else {
        state.domain.set_text_value(&arg.id, &text);
        state.domain.mark_touched(&arg.id);
    }
    EditResult::Handled
}

pub(crate) fn apply_paste_to_text_field(
    state: &mut AppState,
    arg: &ArgModel,
    text: &str,
) -> EditResult {
    let displayed = displayed_text(state, arg);
    let command_key = arg.owner_path().clone();
    let is_touched = state.domain.is_touched(&arg.id);
    let has_default = arg.default_value().is_some();
    let textarea = ensure_editor(&mut state.ui, &command_key, arg, &displayed);
    if has_default && !is_touched {
        *textarea = TextEditor::from_displayed("");
    }
    if !textarea.insert_str(text) {
        return EditResult::Ignored;
    }

    let text = textarea.text();
    if text.is_empty() && has_default {
        state.domain.clear_value_and_untouch(&arg.id);
    } else if text.is_empty() && arg.uses_optional_value_semantics() {
        state.domain.toggle_optional_value_flag(&arg.id, true);
    } else if uses_row_editor(arg) {
        let rows = textarea.lines().to_vec();
        sync_row_editor_values(state, arg, &rows);
    } else {
        state.domain.set_text_value(&arg.id, &text);
        state.domain.mark_touched(&arg.id);
    }
    EditResult::Handled
}

fn sync_row_editor_values(state: &mut AppState, arg: &ArgModel, rows: &[String]) {
    let occurrences = match row_editor_mode(arg) {
        Some(RowEditorMode::Occurrence) => rows
            .iter()
            .filter(|value| !value.is_empty())
            .map(|value| InputValueOccurrence {
                values: split_row_occurrence_values(arg, value),
                source: InputSource::User,
            })
            .filter(|occurrence| !occurrence.values.is_empty())
            .collect(),
        Some(RowEditorMode::FixedArityOccurrence {
            values_per_occurrence,
        }) => group_occurrence_values(
            rows.iter()
                .filter(|value| !value.is_empty())
                .flat_map(|value| split_row_occurrence_values(arg, value))
                .filter(|value| !value.is_empty())
                .collect(),
            values_per_occurrence,
        )
        .into_iter()
        .map(|values| InputValueOccurrence {
            values,
            source: InputSource::User,
        })
        .collect(),
        Some(RowEditorMode::GroupedValues) => {
            let values = rows
                .iter()
                .filter(|value| !value.is_empty())
                .flat_map(|value| split_occurrence_values(arg, value))
                .filter(|value| !value.is_empty())
                .collect::<Vec<_>>();
            if values.is_empty() {
                Vec::new()
            } else {
                vec![InputValueOccurrence {
                    values,
                    source: InputSource::User,
                }]
            }
        }
        None => Vec::new(),
    };
    state.domain.replace_occurrences(&arg.id, occurrences);
}

pub(crate) fn clear_selection(state: &mut AppState, arg: &ArgModel) {
    let displayed = displayed_text(state, arg);
    let command_key = arg.owner_path().clone();
    let textarea = ensure_editor(&mut state.ui, &command_key, arg, &displayed);
    textarea.cancel_selection();
}

pub(crate) fn start_selection(state: &mut AppState, arg: &ArgModel, row: u16, col: u16) {
    let displayed = displayed_text(state, arg);
    let command_key = arg.owner_path().clone();
    let textarea = ensure_editor(&mut state.ui, &command_key, arg, &displayed);
    textarea.start_selection(row, col);
}

pub(crate) fn set_cursor_from_click(state: &mut AppState, arg: &ArgModel, row: u16, col: u16) {
    let displayed = displayed_text(state, arg);
    let command_key = arg.owner_path().clone();
    let textarea = ensure_editor(&mut state.ui, &command_key, arg, &displayed);
    textarea.move_cursor_to(row, col);
}

pub(crate) fn insert_repeated_row(state: &mut AppState, arg: &ArgModel) -> EditResult {
    let displayed = displayed_text(state, arg);
    let command_key = arg.owner_path().clone();
    let editor = ensure_editor(&mut state.ui, &command_key, arg, &displayed);
    editor.insert_row_below();
    let rows = editor.lines().to_vec();
    sync_row_editor_values(state, arg, &rows);
    EditResult::Handled
}

pub(crate) fn remove_repeated_row(state: &mut AppState, arg: &ArgModel) -> EditResult {
    let displayed = displayed_text(state, arg);
    let command_key = arg.owner_path().clone();
    let editor = ensure_editor(&mut state.ui, &command_key, arg, &displayed);
    let before = editor.text();
    editor.remove_current_row();
    let text = editor.text();
    if text == before {
        return EditResult::Ignored;
    }
    let rows = editor.lines().to_vec();
    sync_row_editor_values(state, arg, &rows);
    EditResult::Handled
}

pub(crate) fn move_repeated_row_up(state: &mut AppState, arg: &ArgModel) -> EditResult {
    let displayed = displayed_text(state, arg);
    let command_key = arg.owner_path().clone();
    let editor = ensure_editor(&mut state.ui, &command_key, arg, &displayed);
    if editor.row_count() <= 1 || editor.current_row() == 0 {
        return EditResult::Ignored;
    }
    editor.move_current_row_up();
    let rows = editor.lines().to_vec();
    sync_row_editor_values(state, arg, &rows);
    EditResult::Handled
}

pub(crate) fn move_repeated_row_down(state: &mut AppState, arg: &ArgModel) -> EditResult {
    let displayed = displayed_text(state, arg);
    let command_key = arg.owner_path().clone();
    let editor = ensure_editor(&mut state.ui, &command_key, arg, &displayed);
    if editor.row_count() <= 1 || editor.current_row() + 1 >= editor.row_count() {
        return EditResult::Ignored;
    }
    editor.move_current_row_down();
    let rows = editor.lines().to_vec();
    sync_row_editor_values(state, arg, &rows);
    EditResult::Handled
}

fn uses_row_editor(arg: &ArgModel) -> bool {
    row_editor_mode(arg).is_some()
}

fn uses_occurrence_rows(arg: &ArgModel) -> bool {
    matches!(
        row_editor_mode(arg),
        Some(RowEditorMode::Occurrence | RowEditorMode::FixedArityOccurrence { .. })
    )
}

fn uses_grouped_value_rows(arg: &ArgModel) -> bool {
    matches!(row_editor_mode(arg), Some(RowEditorMode::GroupedValues))
}

fn row_editor_mode(arg: &ArgModel) -> Option<RowEditorMode> {
    if arg.allows_multiple_occurrences() {
        if arg.accepts_multiple_values_per_occurrence()
            && arg.metadata.syntax.value_delimiter.is_none()
            && let Some(values_per_occurrence) = fixed_occurrence_group_size(arg)
        {
            return Some(RowEditorMode::FixedArityOccurrence {
                values_per_occurrence,
            });
        }
        return Some(RowEditorMode::Occurrence);
    }

    if arg.accepts_multiple_values_per_occurrence() {
        return Some(RowEditorMode::GroupedValues);
    }

    None
}

fn fixed_occurrence_group_size(arg: &ArgModel) -> Option<usize> {
    let cardinality = arg.metadata.cardinality;
    match cardinality.max_values {
        Some(max_values) if cardinality.min_values == max_values && max_values > 1 => {
            Some(max_values)
        }
        _ => None,
    }
}

fn split_row_occurrence_values(arg: &ArgModel, text: &str) -> Vec<String> {
    if arg.accepts_multiple_values_per_occurrence()
        && arg.allows_multiple_occurrences()
        && arg.metadata.syntax.value_delimiter.is_none()
    {
        return text.split_whitespace().map(str::to_string).collect();
    }

    split_occurrence_values(arg, text)
}

fn render_occurrence_row_text(arg: &ArgModel, values: &[String]) -> String {
    if arg.accepts_multiple_values_per_occurrence()
        && arg.allows_multiple_occurrences()
        && arg.metadata.syntax.value_delimiter.is_none()
    {
        return values.join(" ");
    }

    render_occurrence_text(arg, values)
}

fn group_occurrence_values(values: Vec<String>, values_per_occurrence: usize) -> Vec<Vec<String>> {
    if values_per_occurrence == 0 {
        return Vec::new();
    }

    let mut grouped = Vec::new();
    let mut current = Vec::new();
    for value in values {
        current.push(value);
        if current.len() == values_per_occurrence {
            grouped.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        grouped.push(current);
    }
    grouped
}

fn row_editor_displayed_text(state: &AppState, arg: &ArgModel) -> Option<String> {
    let form = state.domain.current_form()?;
    let input = form.input(&arg.id)?;
    let ArgValue::Text(fallback) = input.compatibility_value(arg)? else {
        return None;
    };
    let ArgInput::Values { occurrences } = &input.value else {
        return Some(fallback);
    };

    if uses_occurrence_rows(arg) {
        return Some(
            occurrences
                .iter()
                .map(|occurrence| render_occurrence_row_text(arg, &occurrence.values))
                .filter(|value| !value.is_empty())
                .collect::<Vec<_>>()
                .join("\n"),
        );
    }

    if uses_grouped_value_rows(arg) {
        return Some(
            occurrences
                .first()
                .map(|occurrence| {
                    occurrence
                        .values
                        .iter()
                        .filter(|value| !value.is_empty())
                        .cloned()
                        .collect::<Vec<_>>()
                        .join("\n")
                })
                .unwrap_or_default(),
        );
    }

    Some(fallback)
}

fn row_editor_boundary_merge(editor: &TextEditor, key: AppKeyEvent, arg: &ArgModel) -> bool {
    if !uses_row_editor(arg) || editor.selection_anchor().is_some() {
        return false;
    }

    let cursor = editor.cursor();
    match key.code {
        AppKeyCode::Backspace => cursor.col == 0 && cursor.row > 0,
        AppKeyCode::Delete => {
            cursor.col >= editor.current_line_len() && cursor.row + 1 < editor.row_count()
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use clap::{Arg, ArgAction, Command};

    use super::{EditResult, apply_key_to_text_field};
    use crate::input::{AppState, ArgInput, InputSource, InputValueOccurrence};
    use crate::runtime::{AppKeyCode, AppKeyEvent, AppKeyModifiers};

    fn key(code: AppKeyCode) -> AppKeyEvent {
        AppKeyEvent::new(code, AppKeyModifiers::default())
    }

    fn include_arg(state: &AppState) -> crate::spec::ArgSpec {
        state
            .domain
            .current_command()
            .args
            .iter()
            .find(|arg| arg.id == "include")
            .cloned()
            .expect("include arg")
    }

    #[test]
    fn repeated_text_backspace_at_row_start_does_not_merge_occurrences() {
        let mut state = AppState::from_command(
            &Command::new("tool").arg(
                Arg::new("include")
                    .long("include")
                    .action(ArgAction::Append)
                    .num_args(1),
            ),
        );
        state.domain.set_text_value("include", "alpha\nbeta");
        let arg = include_arg(&state);
        super::set_cursor_from_click(&mut state, &arg, 1, 0);

        let result = apply_key_to_text_field(&mut state, &arg, key(AppKeyCode::Backspace));

        assert_eq!(result, EditResult::Ignored);
        assert_eq!(
            crate::pipeline::build_command_line(&state),
            vec![
                "tool".to_string(),
                "--include".to_string(),
                "alpha".to_string(),
                "--include".to_string(),
                "beta".to_string(),
            ]
        );
    }

    #[test]
    fn repeated_text_delete_at_row_end_does_not_merge_occurrences() {
        let mut state = AppState::from_command(
            &Command::new("tool").arg(
                Arg::new("include")
                    .long("include")
                    .action(ArgAction::Append)
                    .num_args(1),
            ),
        );
        state.domain.set_text_value("include", "alpha\nbeta");
        let arg = include_arg(&state);
        super::set_cursor_from_click(&mut state, &arg, 0, 5);

        let result = apply_key_to_text_field(&mut state, &arg, key(AppKeyCode::Delete));

        assert_eq!(result, EditResult::Ignored);
        assert_eq!(
            crate::pipeline::build_command_line(&state),
            vec![
                "tool".to_string(),
                "--include".to_string(),
                "alpha".to_string(),
                "--include".to_string(),
                "beta".to_string(),
            ]
        );
    }

    #[test]
    fn repeated_text_editor_sync_preserves_other_occurrence_whitespace() {
        let mut state = AppState::from_command(
            &Command::new("tool").arg(
                Arg::new("include")
                    .long("include")
                    .action(ArgAction::Append)
                    .num_args(1),
            ),
        );
        state.domain.replace_occurrences(
            "include",
            vec![
                InputValueOccurrence {
                    values: vec!["  alpha  ".to_string()],
                    source: InputSource::User,
                },
                InputValueOccurrence {
                    values: vec!["beta".to_string()],
                    source: InputSource::User,
                },
            ],
        );
        let arg = include_arg(&state);
        super::set_cursor_from_click(&mut state, &arg, 1, 4);

        let result = apply_key_to_text_field(&mut state, &arg, key(AppKeyCode::Char('!')));

        assert_eq!(result, EditResult::Handled);
        let form = state.domain.current_form().expect("form");
        let input = form.input("include").expect("include input");
        match &input.value {
            ArgInput::Values { occurrences } => {
                assert_eq!(occurrences[0].values, vec!["  alpha  ".to_string()]);
                assert_eq!(occurrences[1].values, vec!["beta!".to_string()]);
            }
            other => panic!("expected value occurrences, got {other:?}"),
        }
    }

    #[test]
    fn grouped_row_editor_keeps_values_in_one_occurrence() {
        let mut state = AppState::from_command(
            &Command::new("tool").arg(Arg::new("pair").long("pair").num_args(1..)),
        );
        state.domain.set_text_value("pair", "alpha\nbeta");
        let arg = state
            .domain
            .current_command()
            .args
            .iter()
            .find(|arg| arg.id == "pair")
            .cloned()
            .expect("pair arg");
        super::set_cursor_from_click(&mut state, &arg, 1, 4);

        let result = apply_key_to_text_field(&mut state, &arg, key(AppKeyCode::Char('!')));

        assert_eq!(result, EditResult::Handled);
        let form = state.domain.current_form().expect("form");
        let input = form.input("pair").expect("pair input");
        match &input.value {
            ArgInput::Values { occurrences } => {
                assert_eq!(occurrences.len(), 1);
                assert_eq!(
                    occurrences[0].values,
                    vec!["alpha".to_string(), "beta!".to_string()]
                );
            }
            other => panic!("expected grouped values, got {other:?}"),
        }
        assert_eq!(
            crate::pipeline::build_command_line(&state),
            vec![
                "tool".to_string(),
                "--pair".to_string(),
                "alpha".to_string(),
                "beta!".to_string(),
            ]
        );
    }
}
