use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

use ratatui::layout::Rect;
use tui_textarea::TextArea;

use crate::spec::{ArgKind, ArgSpec, CommandSpec};

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

#[derive(Debug, Clone)]
pub enum ArgValue {
    Bool(bool),
    Text(String),
    Enum(usize),
}

#[derive(Debug, Default, Clone)]
pub struct CommandInputs {
    pub values: HashMap<String, ArgValue>,
}

#[derive(Debug, Default, Clone)]
pub struct LayoutCache {
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
    pub path: Vec<String>,
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
pub struct CommandState {
    pub root: CommandSpec,
    pub selected_path: Vec<String>,
    pub expanded: HashSet<String>,
    pub search: String,
    pub active_tab: ActiveTab,
    pub last_non_help_tab: ActiveTab,
    pub selected_arg_index: usize,
    pub inputs: HashMap<String, CommandInputs>,
    pub touched: HashMap<String, HashSet<String>>,
}

#[derive(Debug)]
pub struct InteractionState {
    pub focus: Focus,
    pub textareas: HashMap<String, HashMap<String, TextArea<'static>>>,
    pub enum_open: Option<String>,
    pub enum_scroll: usize,
    pub form_scroll: u16,
    pub form_scroll_max: u16,
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
    pub command: CommandState,
    pub interaction: InteractionState,
    pub layout: LayoutCache,
    pub notifications: NotificationState,
}

impl AppState {
    pub fn new(root: CommandSpec) -> Self {
        let mut expanded = HashSet::new();
        expanded.insert(root.name.clone());
        Self {
            command: CommandState {
                root,
                selected_path: Vec::new(),
                expanded,
                search: String::new(),
                active_tab: ActiveTab::Options,
                last_non_help_tab: ActiveTab::Options,
                selected_arg_index: 0,
                inputs: HashMap::new(),
                touched: HashMap::new(),
            },
            interaction: InteractionState {
                focus: Focus::Sidebar,
                textareas: HashMap::new(),
                enum_open: None,
                enum_scroll: 0,
                form_scroll: 0,
                form_scroll_max: 0,
                hover: None,
                hover_tab: None,
                mouse_select: None,
            },
            layout: LayoutCache::default(),
            notifications: NotificationState::default(),
        }
    }

    pub fn command_path_key(&self) -> String {
        if self.command.selected_path.is_empty() {
            self.command.root.name.clone()
        } else {
            let mut parts = vec![self.command.root.name.clone()];
            parts.extend(self.command.selected_path.iter().cloned());
            parts.join("::")
        }
    }

    pub fn current_command(&self) -> &CommandSpec {
        let mut cmd = &self.command.root;
        for name in &self.command.selected_path {
            if let Some(next) = cmd
                .subcommands
                .iter()
                .find(|candidate| &candidate.name == name)
            {
                cmd = next;
            }
        }
        cmd
    }

    pub fn current_inputs_mut(&mut self) -> &mut CommandInputs {
        let key = self.command_path_key();
        self.command.inputs.entry(key).or_default()
    }

    pub fn current_inputs(&self) -> Option<&CommandInputs> {
        let key = self.command_path_key();
        self.command.inputs.get(&key)
    }

    pub fn visible_tabs() -> [ActiveTab; 3] {
        [ActiveTab::Options, ActiveTab::Arguments, ActiveTab::Help]
    }

    pub fn focus_first_tab(&mut self, visible_args: &[(usize, &ArgSpec)]) {
        self.command.active_tab = Self::visible_tabs()[0];
        self.command.last_non_help_tab = self.command.active_tab;
        self.ensure_selected_arg_visible(visible_args);
        self.interaction.form_scroll = 0;
        self.interaction.enum_open = None;
        self.interaction.mouse_select = None;
    }

    pub fn ensure_active_tab_visible(&mut self, visible_args: &[(usize, &ArgSpec)]) {
        if Self::visible_tabs().contains(&self.command.active_tab) {
            return;
        }

        self.command.active_tab = Self::visible_tabs()[0];
        if self.command.active_tab != ActiveTab::Help {
            self.command.last_non_help_tab = self.command.active_tab;
            self.ensure_selected_arg_visible(visible_args);
        }
        self.interaction.form_scroll = 0;
        self.interaction.enum_open = None;
        self.interaction.mouse_select = None;
    }

    pub fn ensure_selected_arg_visible(&mut self, visible_args: &[(usize, &ArgSpec)]) {
        if visible_args.is_empty() {
            self.command.selected_arg_index = 0;
            return;
        }
        if !visible_args
            .iter()
            .any(|(index, _)| *index == self.command.selected_arg_index)
        {
            self.command.selected_arg_index = visible_args[0].0;
        }
    }

    pub fn ensure_defaults(&mut self) {
        let args = self.current_command().args.clone();
        let inputs = self.current_inputs_mut();
        for arg in &args {
            if inputs.values.contains_key(&arg.id) {
                continue;
            }
            let value = match arg.kind {
                ArgKind::Flag => Some(ArgValue::Bool(false)),
                _ if arg.uses_choice_input() => arg
                    .possible_values
                    .iter()
                    .position(|value| arg.default.as_deref() == Some(value))
                    .map(ArgValue::Enum)
                    .or_else(|| (!arg.possible_values.is_empty()).then_some(ArgValue::Enum(0))),
                _ => arg.default.clone().map(ArgValue::Text),
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

    pub fn cycle_enum(&mut self, arg_id: &str, max: usize) {
        let entry = self
            .current_inputs_mut()
            .values
            .entry(arg_id.to_string())
            .or_insert(ArgValue::Enum(0));
        if let ArgValue::Enum(index) = entry {
            if max > 0 {
                *index = (*index + 1) % max;
            }
        }
    }

    pub fn mark_touched(&mut self, arg_id: &str) {
        let key = self.command_path_key();
        self.command
            .touched
            .entry(key)
            .or_default()
            .insert(arg_id.to_string());
    }

    pub fn clear_touched(&mut self, arg_id: &str) {
        let key = self.command_path_key();
        if let Some(set) = self.command.touched.get_mut(&key) {
            set.remove(arg_id);
        }
    }

    pub fn is_touched(&self, arg_id: &str) -> bool {
        let key = self.command_path_key();
        self.command
            .touched
            .get(&key)
            .is_some_and(|set| set.contains(arg_id))
    }

    pub fn current_textareas_mut(&mut self) -> &mut HashMap<String, TextArea<'static>> {
        let key = self.command_path_key();
        self.interaction.textareas.entry(key).or_default()
    }

    pub fn textarea_for(&mut self, arg_id: &str, initial: &str) -> &mut TextArea<'static> {
        self.current_textareas_mut()
            .entry(arg_id.to_string())
            .or_insert_with(|| TextArea::new(vec![initial.to_string()]))
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
