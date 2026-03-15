use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

use ratatui::layout::Rect;
use tui_textarea::TextArea;

use crate::spec::{ArgSpec, CommandPath, CommandSpec, SelectionError};

#[derive(Debug, Clone)]
pub enum Focus {
    Sidebar,
    Form,
    Search,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveTab {
    Options,
    Arguments,
    Help,
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

#[derive(Debug, Default, Clone)]
pub struct FrameLayout {
    pub sidebar: Option<Rect>,
    pub form: Option<Rect>,
    pub search: Option<Rect>,
    pub preview: Option<Rect>,
    pub footer: Option<Rect>,
    pub dropdown: Option<Rect>,
    pub sidebar_items: Vec<SidebarItemLayout>,
    pub form_inputs: HashMap<String, Rect>,
    pub form_view: Option<Rect>,
    pub form_tabs: Vec<TabButtonLayout>,
    pub footer_buttons: Vec<FooterButtonLayout>,
}

#[derive(Debug, Default, Clone)]
pub struct FrameState {
    pub layout: FrameLayout,
    pub form_scroll_max: u16,
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
pub struct FooterButtonLayout {
    pub target: HoverTarget,
    pub rect: Rect,
}

#[derive(Debug, Clone)]
pub struct SidebarItemLayout {
    pub path: CommandPath,
    pub row: Rect,
    pub caret: Option<Rect>,
    pub has_children: bool,
}

#[derive(Debug, Clone)]
pub struct TabButtonLayout {
    pub tab: ActiveTab,
    pub rect: Rect,
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
    pub selected_arg_index: usize,
    pub search_query: String,
    pub textareas: HashMap<String, HashMap<String, TextArea<'static>>>,
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
    pub frame: FrameState,
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

impl AppState {
    pub fn new(root: CommandSpec) -> Self {
        Self {
            domain: DomainState::new(root),
            ui: UiState {
                focus: Focus::Sidebar,
                active_tab: ActiveTab::Options,
                last_non_help_tab: ActiveTab::Options,
                selected_arg_index: 0,
                search_query: String::new(),
                textareas: HashMap::new(),
                dropdown_open: None,
                dropdown_scroll: 0,
                form_scroll: 0,
                hover: None,
                hover_tab: None,
                mouse_select: None,
            },
            frame: FrameState::default(),
            notifications: NotificationState::default(),
        }
    }

    pub fn selected_path(&self) -> &CommandPath {
        self.domain.selected_path()
    }

    pub fn current_command(&self) -> &CommandSpec {
        self.domain.current_command()
    }

    pub fn select_command_path(&mut self, path: &[String]) -> Result<(), SelectionError> {
        self.domain.select_command_path(path)
    }

    pub fn select_command_by_search_path(&mut self, start: &str) -> Result<(), SelectionError> {
        self.domain.select_command_by_search_path(start)
    }

    pub fn current_inputs_mut(&mut self) -> &mut CommandFormState {
        self.domain.current_form_mut()
    }

    pub fn current_inputs(&self) -> Option<&CommandFormState> {
        self.domain.current_form()
    }

    pub fn visible_tabs() -> [ActiveTab; 3] {
        [ActiveTab::Options, ActiveTab::Arguments, ActiveTab::Help]
    }

    pub fn focus_first_tab(&mut self, visible_args: &[(usize, &ArgSpec)]) {
        self.ui.active_tab = Self::visible_tabs()[0];
        self.ui.last_non_help_tab = self.ui.active_tab;
        self.ensure_selected_arg_visible(visible_args);
        self.ui.form_scroll = 0;
        self.ui.dropdown_open = None;
        self.ui.mouse_select = None;
    }

    pub fn ensure_active_tab_visible(&mut self, visible_args: &[(usize, &ArgSpec)]) {
        if Self::visible_tabs().contains(&self.ui.active_tab) {
            return;
        }

        self.ui.active_tab = Self::visible_tabs()[0];
        if self.ui.active_tab != ActiveTab::Help {
            self.ui.last_non_help_tab = self.ui.active_tab;
            self.ensure_selected_arg_visible(visible_args);
        }
        self.ui.form_scroll = 0;
        self.ui.dropdown_open = None;
        self.ui.mouse_select = None;
    }

    pub fn ensure_selected_arg_visible(&mut self, visible_args: &[(usize, &ArgSpec)]) {
        if visible_args.is_empty() {
            self.ui.selected_arg_index = 0;
            return;
        }
        if !visible_args
            .iter()
            .any(|(index, _)| *index == self.ui.selected_arg_index)
        {
            self.ui.selected_arg_index = visible_args[0].0;
        }
    }

    pub fn ensure_defaults(&mut self) {
        let args = self.current_command().args.clone();
        let inputs = self.current_inputs_mut();
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
                crate::spec::InputPresentation::FreeText { .. } => {
                    arg.default_value().map(|value| ArgValue::Text(value.to_string()))
                }
            };
            if let Some(value) = value {
                inputs.values.insert(arg.id.clone(), value);
            }
        }
    }

    pub fn set_text_value(&mut self, arg_id: &str, text: String) {
        self.current_inputs_mut()
            .values
            .insert(arg_id.to_string(), ArgValue::Text(text));
    }

    pub fn set_choice_value(&mut self, arg_id: &str, value: String) {
        self.current_inputs_mut()
            .values
            .insert(arg_id.to_string(), ArgValue::Choice(value));
    }

    pub fn toggle_flag(&mut self, arg_id: &str) {
        let entry = self
            .current_inputs_mut()
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
            .current_inputs()
            .and_then(|inputs| inputs.values.get(arg_id))
            .and_then(|value| match value {
                ArgValue::Choice(selected) => choices.iter().position(|choice| choice == selected),
                _ => None,
            })
            .map_or(0, |index| (index + 1) % choices.len());
        self.set_choice_value(arg_id, choices[next_index].clone());
    }

    pub fn mark_touched(&mut self, arg_id: &str) {
        self.current_inputs_mut().touched.insert(arg_id.to_string());
    }

    pub fn clear_touched(&mut self, arg_id: &str) {
        self.current_inputs_mut().touched.remove(arg_id);
    }

    pub fn is_touched(&self, arg_id: &str) -> bool {
        self.current_inputs()
            .is_some_and(|form| form.touched.contains(arg_id))
    }

    pub fn form_scroll(&self) -> u16 {
        self.ui.form_scroll.min(self.frame.form_scroll_max)
    }

    pub fn clamp_form_scroll(&mut self) {
        self.ui.form_scroll = self.form_scroll();
    }

    pub fn show_toast(&mut self, message: impl Into<String>, duration: Duration, is_error: bool) {
        self.notifications.toast = Some(Toast {
            message: message.into(),
            expires_at: Instant::now() + duration,
            is_error,
        });
    }

    pub fn clear_expired_toast(&mut self) {
        if self
            .notifications
            .toast
            .as_ref()
            .is_some_and(|toast| Instant::now() >= toast.expires_at)
        {
            self.notifications.toast = None;
        }
    }
}
