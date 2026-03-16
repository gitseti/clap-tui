use std::collections::HashMap;

use tui_textarea::{CursorMove, Input, Key, TextArea};

use crate::runtime::{AppKeyCode, AppKeyEvent};
use crate::spec::CommandPath;

#[derive(Debug, Default)]
pub struct EditorState {
    editors: HashMap<String, HashMap<String, TextEditor>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct TextPosition {
    pub(crate) row: usize,
    pub(crate) col: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TextEditor {
    lines: Vec<String>,
    cursor: TextPosition,
    selection_anchor: Option<TextPosition>,
}

impl Default for TextEditor {
    fn default() -> Self {
        Self::from_displayed("")
    }
}

impl EditorState {
    pub fn editor(&self, command_key: &CommandPath, arg_id: &str) -> Option<&TextEditor> {
        self.editors
            .get(&command_key.storage_key())
            .and_then(|editors| editors.get(arg_id))
    }

    pub fn ensure_editor<'a>(
        &'a mut self,
        command_key: &CommandPath,
        arg_id: &str,
        displayed: &str,
    ) -> &'a mut TextEditor {
        let key = command_key.storage_key();
        let editors = self.editors.entry(key).or_default();
        let editor = editors
            .entry(arg_id.to_string())
            .or_insert_with(|| TextEditor::from_displayed(displayed));
        if editor.text() != displayed {
            *editor = TextEditor::from_displayed(displayed);
        }
        editor
    }
}

impl TextEditor {
    pub(crate) fn from_displayed(displayed: &str) -> Self {
        let lines = if displayed.is_empty() {
            vec![String::new()]
        } else {
            displayed.split('\n').map(ToString::to_string).collect()
        };
        Self {
            lines,
            cursor: TextPosition::default(),
            selection_anchor: None,
        }
    }

    pub(crate) fn text(&self) -> String {
        self.lines.join("\n")
    }

    #[cfg(test)]
    pub(crate) fn cursor(&self) -> TextPosition {
        self.cursor
    }

    pub(crate) fn selection_anchor(&self) -> Option<TextPosition> {
        self.selection_anchor
    }

    pub(crate) fn cancel_selection(&mut self) {
        self.selection_anchor = None;
    }

    pub(crate) fn move_cursor_to(&mut self, row: u16, col: u16) {
        self.cursor = TextPosition {
            row: usize::from(row),
            col: usize::from(col),
        };
    }

    pub(crate) fn start_selection(&mut self, row: u16, col: u16) {
        self.cursor = TextPosition {
            row: usize::from(row),
            col: usize::from(col),
        };
        self.selection_anchor = Some(self.cursor);
    }

    pub(crate) fn apply_key(&mut self, key: AppKeyEvent) -> bool {
        let cursor_before = self.cursor;
        let selection_anchor = selection_anchor_for_key(self.selection_anchor, cursor_before, key);

        let mut textarea = self.to_textarea(selection_anchor);
        let modified = textarea.input(Input::from(key));
        let cursor = textarea.cursor();
        self.lines = textarea.lines().to_vec();
        self.cursor = TextPosition {
            row: cursor.0,
            col: cursor.1,
        };
        self.selection_anchor = if textarea.is_selecting() {
            selection_anchor.filter(|anchor| *anchor != self.cursor)
        } else {
            None
        };
        modified
    }

    pub(crate) fn to_textarea(&self, selection_anchor: Option<TextPosition>) -> TextArea<'static> {
        let mut textarea = TextArea::new(self.lines.clone());
        if let Some(anchor) = selection_anchor {
            textarea.move_cursor(CursorMove::Jump(
                u16::try_from(anchor.row).unwrap_or(u16::MAX),
                u16::try_from(anchor.col).unwrap_or(u16::MAX),
            ));
            textarea.start_selection();
        }
        textarea.move_cursor(CursorMove::Jump(
            u16::try_from(self.cursor.row).unwrap_or(u16::MAX),
            u16::try_from(self.cursor.col).unwrap_or(u16::MAX),
        ));
        textarea
    }
}

impl From<AppKeyEvent> for Input {
    fn from(value: AppKeyEvent) -> Self {
        Self {
            key: Key::from(value.code),
            ctrl: value.modifiers.control,
            alt: value.modifiers.alt,
            shift: value.modifiers.shift,
        }
    }
}

impl From<AppKeyCode> for Key {
    fn from(value: AppKeyCode) -> Self {
        match value {
            AppKeyCode::Char(value) => Self::Char(value),
            AppKeyCode::F(value) => Self::F(value),
            AppKeyCode::Backspace => Self::Backspace,
            AppKeyCode::Enter => Self::Enter,
            AppKeyCode::Left => Self::Left,
            AppKeyCode::Right => Self::Right,
            AppKeyCode::Up => Self::Up,
            AppKeyCode::Down => Self::Down,
            AppKeyCode::Tab | AppKeyCode::BackTab => Self::Tab,
            AppKeyCode::Delete => Self::Delete,
            AppKeyCode::Home => Self::Home,
            AppKeyCode::End => Self::End,
            AppKeyCode::PageUp => Self::PageUp,
            AppKeyCode::PageDown => Self::PageDown,
            AppKeyCode::Esc => Self::Esc,
            AppKeyCode::Null => Self::Null,
        }
    }
}

fn extends_selection(key: AppKeyEvent) -> bool {
    if !key.modifiers.shift {
        return false;
    }
    matches!(
        key.code,
        AppKeyCode::Left
            | AppKeyCode::Right
            | AppKeyCode::Up
            | AppKeyCode::Down
            | AppKeyCode::Home
            | AppKeyCode::End
            | AppKeyCode::PageUp
            | AppKeyCode::PageDown
    )
}

fn selection_anchor_for_key(
    selection_anchor: Option<TextPosition>,
    cursor_before: TextPosition,
    key: AppKeyEvent,
) -> Option<TextPosition> {
    if extends_selection(key) {
        return selection_anchor.or(Some(cursor_before));
    }
    if consumes_selection(key) {
        return selection_anchor;
    }
    None
}

fn consumes_selection(key: AppKeyEvent) -> bool {
    match key.code {
        AppKeyCode::Backspace | AppKeyCode::Delete => true,
        AppKeyCode::Char(_) if !key.modifiers.control && !key.modifiers.alt => true,
        AppKeyCode::Char(c) if key.modifiers.control && !key.modifiers.alt => {
            matches!(c, 'c' | 'd' | 'h' | 'j' | 'k' | 'm' | 'w' | 'x' | 'y')
        }
        AppKeyCode::Char(c) if !key.modifiers.control && key.modifiers.alt => {
            matches!(c, 'd' | 'h')
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::{TextEditor, TextPosition};
    use crate::runtime::{AppKeyCode, AppKeyEvent, AppKeyModifiers};

    fn key(code: AppKeyCode) -> AppKeyEvent {
        AppKeyEvent::new(code, AppKeyModifiers::default())
    }

    #[test]
    fn editor_tracks_text_and_cursor_without_widget_storage() {
        let mut editor = TextEditor::from_displayed("abc");
        editor.apply_key(key(AppKeyCode::End));
        editor.apply_key(key(AppKeyCode::Char('d')));

        assert_eq!(editor.text(), "abcd");
        assert_eq!(editor.cursor(), TextPosition { row: 0, col: 4 });
        assert_eq!(editor.selection_anchor(), None);
    }

    #[test]
    fn editor_tracks_mouse_selection_anchor() {
        let mut editor = TextEditor::from_displayed("alpha");
        editor.start_selection(0, 1);
        editor.move_cursor_to(0, 4);

        assert_eq!(
            editor.selection_anchor(),
            Some(TextPosition { row: 0, col: 1 })
        );
        assert_eq!(editor.cursor(), TextPosition { row: 0, col: 4 });
    }

    #[test]
    fn backspace_deletes_the_entire_selected_range() {
        let mut editor = TextEditor::from_displayed("alpha");
        editor.start_selection(0, 1);
        editor.move_cursor_to(0, 4);

        editor.apply_key(key(AppKeyCode::Backspace));

        assert_eq!(editor.text(), "aa");
        assert_eq!(editor.cursor(), TextPosition { row: 0, col: 1 });
        assert_eq!(editor.selection_anchor(), None);
    }

    #[test]
    fn typing_replaces_the_current_selection() {
        let mut editor = TextEditor::from_displayed("alpha");
        editor.start_selection(0, 1);
        editor.move_cursor_to(0, 4);

        editor.apply_key(key(AppKeyCode::Char('x')));

        assert_eq!(editor.text(), "axa");
        assert_eq!(editor.cursor(), TextPosition { row: 0, col: 2 });
        assert_eq!(editor.selection_anchor(), None);
    }
}
