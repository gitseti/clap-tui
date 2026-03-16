use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

use crate::editor_state::EditorState;
use crate::frame_snapshot::FrameSnapshot;
use crate::spec::{ArgSpec, CommandPath, CommandSpec, SelectionError};

#[derive(Debug, Clone)]
pub enum Focus {
    Sidebar,
    Form,
    Search,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveTab {
    Inputs,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArgValue {
    Bool(bool),
    Text(String),
    Choice(String),
}

#[derive(Debug, Default, Clone)]
pub struct CommandFormState {
    pub values: HashMap<String, ArgValue>,
    pub touched: HashSet<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HoverTarget {
    Run,
    Exit,
    Search,
    Focus,
    Help,
    Preview,
}

#[derive(Debug, Clone)]
pub struct MouseSelection {
    pub arg_id: String,
    pub anchor_row: u16,
    pub anchor_col: u16,
    pub active: bool,
}

#[derive(Debug, Clone)]
pub struct Toast {
    pub message: String,
    pub expires_at: Instant,
    pub is_error: bool,
}

#[derive(Debug)]
pub struct DomainState {
    pub root: CommandSpec,
    pub selected_path: CommandPath,
    pub expanded: HashSet<String>,
    pub forms: HashMap<String, CommandFormState>,
}

#[derive(Debug)]
pub struct UiState {
    pub focus: Focus,
    pub active_tab: ActiveTab,
    pub last_non_help_tab: ActiveTab,
    pub help_open: bool,
    pub help_scroll: u16,
    pub selected_arg_index: usize,
    pub search_query: String,
    pub editors: EditorState,
    pub dropdown_open: Option<String>,
    pub dropdown_scroll: usize,
    pub form_scroll: u16,
    pub hover: Option<HoverTarget>,
    pub hover_tab: Option<ActiveTab>,
    pub mouse_select: Option<MouseSelection>,
}

#[derive(Debug, Default)]
pub struct NotificationState {
    pub toast: Option<Toast>,
}

#[derive(Debug)]
pub struct AppState {
    pub domain: DomainState,
    pub ui: UiState,
    pub notifications: NotificationState,
}

impl DomainState {
    pub fn new(root: CommandSpec) -> Self {
        let mut expanded = HashSet::new();
        expanded.insert(root.name.clone());
        Self {
            root,
            selected_path: CommandPath::default(),
            expanded,
            forms: HashMap::new(),
        }
    }

    pub fn selected_path(&self) -> &CommandPath {
        &self.selected_path
    }

    pub fn current_command(&self) -> &CommandSpec {
        self.domain_resolved_command().command
    }

    fn domain_resolved_command(&self) -> crate::spec::ResolvedCommand<'_> {
        self.root.resolved(&self.selected_path)
    }

    pub fn command_path_key(&self) -> String {
        self.selected_path.to_key(&self.root.name)
    }

    pub fn current_form_mut(&mut self) -> &mut CommandFormState {
        let key = self.command_path_key();
        self.forms.entry(key).or_default()
    }

    pub fn current_form(&self) -> Option<&CommandFormState> {
        let key = self.command_path_key();
        self.forms.get(&key)
    }

    pub fn ensure_defaults(&mut self) {
        let args = self.current_command().args.clone();
        let inputs = self.current_form_mut();
        for arg in &args {
            if inputs.values.contains_key(&arg.id) {
                continue;
            }
            let value = match arg.input_presentation() {
                crate::spec::InputPresentation::Toggle => Some(ArgValue::Bool(false)),
                crate::spec::InputPresentation::ChoiceList { .. } => arg
                    .default_value()
                    .or_else(|| arg.choices.first().map(String::as_str))
                    .map(|value| ArgValue::Choice(value.to_string())),
                crate::spec::InputPresentation::FreeText { .. } => arg
                    .default_value()
                    .map(|value| ArgValue::Text(value.to_string())),
            };
            if let Some(value) = value {
                inputs.values.insert(arg.id.clone(), value);
            }
        }
    }

    pub fn set_text_value(&mut self, arg_id: &str, text: String) {
        self.current_form_mut()
            .values
            .insert(arg_id.to_string(), ArgValue::Text(text));
    }

    pub fn set_choice_value(&mut self, arg_id: &str, value: String) {
        self.current_form_mut()
            .values
            .insert(arg_id.to_string(), ArgValue::Choice(value));
    }

    pub fn clear_value(&mut self, arg_id: &str) {
        self.current_form_mut().values.remove(arg_id);
    }

    pub fn toggle_flag(&mut self, arg_id: &str) {
        let entry = self
            .current_form_mut()
            .values
            .entry(arg_id.to_string())
            .or_insert(ArgValue::Bool(false));
        if let ArgValue::Bool(value) = entry {
            *value = !*value;
        }
    }

    pub fn cycle_choice(&mut self, arg_id: &str, choices: &[String]) {
        if choices.is_empty() {
            return;
        }
        let next_index = self
            .current_form()
            .and_then(|inputs| inputs.values.get(arg_id))
            .and_then(|value| match value {
                ArgValue::Choice(selected) => choices.iter().position(|choice| choice == selected),
                _ => None,
            })
            .map_or(0, |index| (index + 1) % choices.len());
        self.set_choice_value(arg_id, choices[next_index].clone());
    }

    pub fn mark_touched(&mut self, arg_id: &str) {
        self.current_form_mut().touched.insert(arg_id.to_string());
    }

    pub fn toggle_flag_touched(&mut self, arg_id: &str) {
        self.toggle_flag(arg_id);
        self.mark_touched(arg_id);
    }

    pub fn set_choice_value_touched(&mut self, arg_id: &str, value: String) {
        self.set_choice_value(arg_id, value);
        self.mark_touched(arg_id);
    }

    pub fn cycle_choice_touched(&mut self, arg_id: &str, choices: &[String]) {
        self.cycle_choice(arg_id, choices);
        self.mark_touched(arg_id);
    }

    pub fn clear_value_and_untouch(&mut self, arg_id: &str) {
        self.clear_value(arg_id);
        self.clear_touched(arg_id);
    }

    pub fn clear_touched(&mut self, arg_id: &str) {
        self.current_form_mut().touched.remove(arg_id);
    }

    pub fn is_touched(&self, arg_id: &str) -> bool {
        self.current_form()
            .is_some_and(|form| form.touched.contains(arg_id))
    }

    pub fn select_command_path(&mut self, path: &[String]) -> Result<(), SelectionError> {
        let normalized = self
            .root
            .normalize_path(path)
            .ok_or(SelectionError::UnknownPath)?;
        self.selected_path = normalized.clone();
        for key in self.root.expand_prefix_keys(&normalized) {
            self.expanded.insert(key);
        }
        Ok(())
    }

    pub fn select_command_by_search_path(&mut self, start: &str) -> Result<(), SelectionError> {
        let normalized = self
            .root
            .find_path_by_search_path(start)
            .ok_or(SelectionError::UnknownPath)?;
        self.select_command_path(normalized.as_slice())
    }
}

impl UiState {
    pub fn visible_tabs() -> [ActiveTab; 1] {
        [ActiveTab::Inputs]
    }

    pub fn focus_first_tab(&mut self, visible_args: &[(usize, &ArgSpec)]) {
        self.active_tab = Self::visible_tabs()[0];
        self.last_non_help_tab = self.active_tab;
        self.ensure_selected_arg_visible(visible_args);
        self.reset_transient_form_ui();
    }

    pub fn ensure_active_tab_visible(&mut self, visible_args: &[(usize, &ArgSpec)]) {
        if Self::visible_tabs().contains(&self.active_tab) {
            return;
        }

        self.active_tab = Self::visible_tabs()[0];
        self.last_non_help_tab = self.active_tab;
        self.ensure_selected_arg_visible(visible_args);
        self.reset_transient_form_ui();
    }

    pub fn ensure_selected_arg_visible(&mut self, visible_args: &[(usize, &ArgSpec)]) {
        if visible_args.is_empty() {
            self.selected_arg_index = 0;
            return;
        }
        if !visible_args
            .iter()
            .any(|(index, _)| *index == self.selected_arg_index)
        {
            self.selected_arg_index = visible_args[0].0;
        }
    }

    pub fn form_scroll(&self, frame_snapshot: &FrameSnapshot) -> u16 {
        frame_snapshot.form_scroll(self.form_scroll)
    }

    pub fn clamp_form_scroll(&mut self, frame_snapshot: &FrameSnapshot) {
        self.form_scroll = self.form_scroll(frame_snapshot);
    }

    pub fn help_scroll(&self, frame_snapshot: &FrameSnapshot) -> u16 {
        frame_snapshot.help_scroll(self.help_scroll)
    }

    pub fn clamp_help_scroll(&mut self, frame_snapshot: &FrameSnapshot) {
        self.help_scroll = self.help_scroll(frame_snapshot);
    }

    pub fn toggle_focus(&mut self) {
        self.focus = match self.focus {
            Focus::Sidebar => Focus::Form,
            _ => Focus::Sidebar,
        };
    }

    pub fn focus_sidebar(&mut self) {
        self.focus = Focus::Sidebar;
    }

    pub fn focus_form(&mut self) {
        self.focus = Focus::Form;
    }

    pub fn focus_search(&mut self) {
        self.focus = Focus::Search;
    }

    pub fn toggle_help(&mut self) {
        self.help_open = !self.help_open;
        self.help_scroll = 0;
        self.close_dropdown();
        self.clear_mouse_selection();
    }

    pub fn close_dropdown(&mut self) {
        self.dropdown_open = None;
    }

    pub fn open_dropdown(&mut self, arg_id: impl Into<String>, scroll: usize) {
        self.dropdown_open = Some(arg_id.into());
        self.dropdown_scroll = scroll;
    }

    pub fn set_dropdown_scroll(&mut self, scroll: usize) {
        self.dropdown_scroll = scroll;
    }

    pub fn adjust_dropdown_scroll(&mut self, delta: i16) {
        if delta.is_negative() {
            self.dropdown_scroll = self
                .dropdown_scroll
                .saturating_sub(usize::from(delta.unsigned_abs()));
        } else {
            self.dropdown_scroll = self
                .dropdown_scroll
                .saturating_add(usize::from(delta.unsigned_abs()));
        }
    }

    pub fn set_form_scroll(&mut self, scroll: u16) {
        self.form_scroll = scroll;
    }

    pub fn adjust_form_scroll(&mut self, delta: i16) {
        if delta.is_negative() {
            self.form_scroll = self.form_scroll.saturating_sub(delta.unsigned_abs());
        } else {
            self.form_scroll = self.form_scroll.saturating_add(delta.unsigned_abs());
        }
    }

    pub fn adjust_help_scroll(&mut self, delta: i16) {
        if delta.is_negative() {
            self.help_scroll = self.help_scroll.saturating_sub(delta.unsigned_abs());
        } else {
            self.help_scroll = self.help_scroll.saturating_add(delta.unsigned_abs());
        }
    }

    pub fn set_selected_arg_index(&mut self, selected_arg_index: usize) {
        self.selected_arg_index = selected_arg_index;
    }

    pub fn set_mouse_selection(&mut self, mouse_select: Option<MouseSelection>) {
        self.mouse_select = mouse_select;
    }

    pub fn clear_mouse_selection(&mut self) {
        self.mouse_select = None;
    }

    pub fn reset_transient_form_ui(&mut self) {
        self.form_scroll = 0;
        self.close_dropdown();
        self.clear_mouse_selection();
    }

    pub fn set_hover(&mut self, hover: Option<HoverTarget>) {
        self.hover = hover;
    }

    pub fn set_hover_tab(&mut self, hover_tab: Option<ActiveTab>) {
        self.hover_tab = hover_tab;
    }
}

impl NotificationState {
    pub fn show_toast(&mut self, message: impl Into<String>, duration: Duration, is_error: bool) {
        self.toast = Some(Toast {
            message: message.into(),
            expires_at: Instant::now() + duration,
            is_error,
        });
    }

    pub fn clear_expired_toast(&mut self) {
        if self
            .toast
            .as_ref()
            .is_some_and(|toast| Instant::now() >= toast.expires_at)
        {
            self.toast = None;
        }
    }
}

impl AppState {
    pub fn new(root: CommandSpec) -> Self {
        Self {
            domain: DomainState::new(root),
            ui: UiState {
                focus: Focus::Sidebar,
                active_tab: ActiveTab::Inputs,
                last_non_help_tab: ActiveTab::Inputs,
                help_open: false,
                help_scroll: 0,
                selected_arg_index: 0,
                search_query: String::new(),
                editors: EditorState::default(),
                dropdown_open: None,
                dropdown_scroll: 0,
                form_scroll: 0,
                hover: None,
                hover_tab: None,
                mouse_select: None,
            },
            notifications: NotificationState::default(),
        }
    }
}
