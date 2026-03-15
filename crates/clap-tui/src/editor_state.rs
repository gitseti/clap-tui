use std::collections::HashMap;

use tui_textarea::TextArea;

use crate::spec::CommandPath;

#[derive(Debug, Default)]
pub struct EditorState {
    editors: HashMap<String, HashMap<String, TextArea<'static>>>,
}

impl EditorState {
    pub fn ensure_editor<'a>(
        &'a mut self,
        command_key: &CommandPath,
        arg_id: &str,
        displayed: &str,
    ) -> &'a mut TextArea<'static> {
        let key = command_key.storage_key();
        let editors = self.editors.entry(key).or_default();
        let textarea = editors
            .entry(arg_id.to_string())
            .or_insert_with(|| TextArea::new(vec![displayed.to_string()]));
        if textarea.lines().join("\n") != displayed {
            *textarea = TextArea::new(vec![displayed.to_string()]);
        }
        textarea
    }
}
