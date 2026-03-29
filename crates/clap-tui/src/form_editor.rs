use crate::editor_state::TextEditor;
use crate::input::{AppState, ArgValue, UiState};
use crate::runtime::AppKeyCode;
use crate::runtime::AppKeyEvent;
use crate::spec::{ArgModel, CommandPath};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EditResult {
    Ignored,
    Handled,
}

pub(crate) fn displayed_text(state: &AppState, arg: &ArgModel) -> String {
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

    arg.allows_multiple_occurrences() && normalize_repeated_editor_text(&editor.text()) == displayed
}

fn normalize_repeated_editor_text(text: &str) -> String {
    text.lines()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
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
    } else {
        state.domain.set_text_value(&arg.id, &text);
        state.domain.mark_touched(&arg.id);
    }
    EditResult::Handled
}

fn sync_editor_to_domain(state: &mut AppState, arg_id: &str, text: &str) {
    state.domain.set_text_value(arg_id, text);
    state.domain.mark_touched(arg_id);
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
    let text = editor.text();
    sync_editor_to_domain(state, &arg.id, &text);
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
    sync_editor_to_domain(state, &arg.id, &text);
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
    let text = editor.text();
    sync_editor_to_domain(state, &arg.id, &text);
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
    let text = editor.text();
    sync_editor_to_domain(state, &arg.id, &text);
    EditResult::Handled
}
