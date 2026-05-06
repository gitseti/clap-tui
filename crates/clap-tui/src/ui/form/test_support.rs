#![cfg(test)]

// Shared form UI fixtures must stay test-only; production code should move
// reusable behavior into a named runtime module instead of importing this one.

use ratatui::backend::TestBackend;

use crate::input::{ActiveTab, Focus, UiState};
use crate::spec::{ArgKind, ArgSpec, CommandSpec, ValueCardinality};

pub(super) fn command() -> CommandSpec {
    CommandSpec {
        name: "tool".to_string(),
        version: None,
        about: None,
        help: String::new(),
        args: Vec::new(),
        subcommands: Vec::new(),
        ..CommandSpec::default()
    }
}

pub(super) fn ui_state() -> UiState {
    UiState {
        focus: Focus::Form,
        active_tab: ActiveTab::Inputs,
        last_non_help_tab: ActiveTab::Inputs,
        help_open: false,
        help_scroll: 0,
        selected_arg_index: 0,
        search_query: String::new(),
        editors: crate::editor_state::EditorState::default(),
        dropdown_open: None,
        dropdown_scroll: 0,
        dropdown_cursor: 0,
        sidebar_scroll: 0,
        form_scroll: 0,
        hover: None,
        hover_tab: None,
        mouse_select: None,
    }
}

pub(super) fn option_arg(id: &str, name: &str) -> ArgSpec {
    ArgSpec {
        id: id.to_string(),
        display_name: name.to_string(),
        help: None,
        required: false,
        kind: ArgKind::Option,
        default_values: Vec::new(),
        choices: Vec::new(),
        position: None,
        value_cardinality: ValueCardinality::One,
        value_hint: None,
        ..ArgSpec::default()
    }
}

pub(super) fn choice_arg(id: &str, name: &str, choices: &[&str]) -> ArgSpec {
    let mut arg = option_arg(id, name);
    arg.choices = choices.iter().map(|choice| (*choice).to_string()).collect();
    arg
}

pub(super) fn buffer_text(backend: &TestBackend) -> String {
    backend
        .buffer()
        .content
        .iter()
        .map(ratatui::buffer::Cell::symbol)
        .collect::<String>()
}

pub(super) fn cell_bg(backend: &TestBackend, x: u16, y: u16) -> ratatui::style::Color {
    backend.buffer()[(x, y)].bg
}

pub(super) fn cell_fg(backend: &TestBackend, x: u16, y: u16) -> ratatui::style::Color {
    backend.buffer()[(x, y)].fg
}
