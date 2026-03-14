use std::collections::{HashMap, HashSet};

use ratatui::layout::Rect;
use tui_textarea::TextArea;

use crate::spec::{ArgKind, CommandSpec};
use crate::view::form;

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

#[derive(Debug)]
pub struct AppState {
    pub root: CommandSpec,
    pub selected_path: Vec<String>,
    pub expanded: HashSet<String>,
    pub search: String,
    pub focus: Focus,
    pub active_tab: ActiveTab,
    pub last_non_help_tab: ActiveTab,
    pub selected_arg_index: usize,
    pub inputs: HashMap<String, CommandInputs>,
    pub textareas: HashMap<String, HashMap<String, TextArea<'static>>>,
    pub layout: LayoutCache,
    pub enum_open: Option<String>,
    pub enum_scroll: usize,
    pub form_scroll: u16,
    pub form_scroll_max: u16,
    pub hover: Option<HoverTarget>,
    pub hover_tab: Option<ActiveTab>,
    pub touched: HashMap<String, HashSet<String>>,
    pub mouse_select: Option<MouseSelection>,
}

impl AppState {
    pub fn new(root: CommandSpec) -> Self {
        let mut expanded = HashSet::new();
        expanded.insert(root.name.clone());
        Self {
            root,
            selected_path: Vec::new(),
            expanded,
            search: String::new(),
            focus: Focus::Sidebar,
            active_tab: ActiveTab::Options,
            last_non_help_tab: ActiveTab::Options,
            selected_arg_index: 0,
            inputs: HashMap::new(),
            textareas: HashMap::new(),
            layout: LayoutCache::default(),
            enum_open: None,
            enum_scroll: 0,
            form_scroll: 0,
            form_scroll_max: 0,
            hover: None,
            hover_tab: None,
            touched: HashMap::new(),
            mouse_select: None,
        }
    }

    pub fn command_path_key(&self) -> String {
        if self.selected_path.is_empty() {
            self.root.name.clone()
        } else {
            let mut parts = vec![self.root.name.clone()];
            parts.extend(self.selected_path.iter().cloned());
            parts.join("::")
        }
    }

    pub fn current_command(&self) -> &CommandSpec {
        let mut cmd = &self.root;
        for name in &self.selected_path {
            if let Some(next) = cmd.subcommands.iter().find(|c| &c.name == name) {
                cmd = next;
            }
        }
        cmd
    }

    pub fn current_inputs_mut(&mut self) -> &mut CommandInputs {
        let key = self.command_path_key();
        self.inputs.entry(key).or_default()
    }

    pub fn current_inputs(&self) -> Option<&CommandInputs> {
        let key = self.command_path_key();
        self.inputs.get(&key)
    }

    pub fn visible_tabs(&self) -> Vec<ActiveTab> {
        vec![ActiveTab::Options, ActiveTab::Arguments, ActiveTab::Help]
    }

    pub fn focus_first_tab(&mut self) {
        self.active_tab = self
            .visible_tabs()
            .first()
            .copied()
            .unwrap_or(ActiveTab::Options);
        self.last_non_help_tab = self.active_tab;
        self.ensure_selected_arg_visible();
        self.form_scroll = 0;
        self.enum_open = None;
        self.mouse_select = None;
    }

    pub fn visible_args(&self) -> Vec<(usize, &crate::spec::ArgSpec)> {
        form::visible_args(self.current_command(), self.active_tab)
            .into_iter()
            .map(|item| (item.order_index, item.arg))
            .collect()
    }

    pub fn ensure_active_tab_visible(&mut self) {
        let tabs = self.visible_tabs();
        if !tabs.contains(&self.active_tab) {
            self.active_tab = tabs.first().copied().unwrap_or(ActiveTab::Options);
            if self.active_tab != ActiveTab::Help {
                self.last_non_help_tab = self.active_tab;
                self.ensure_selected_arg_visible();
            }
            self.form_scroll = 0;
            self.enum_open = None;
            self.mouse_select = None;
        }
    }

    pub fn ensure_selected_arg_visible(&mut self) {
        let args = self.visible_args();
        if args.is_empty() {
            self.selected_arg_index = 0;
            return;
        }
        if !args.iter().any(|(idx, _)| *idx == self.selected_arg_index) {
            self.selected_arg_index = args[0].0;
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
                ArgKind::Enum => arg
                    .possible_values
                    .iter()
                    .position(|v| arg.default.as_deref() == Some(v))
                    .map(ArgValue::Enum)
                    .or_else(|| {
                        if arg.possible_values.is_empty() {
                            None
                        } else {
                            Some(ArgValue::Enum(0))
                        }
                    }),
                _ => arg.default.clone().map(ArgValue::Text),
            };
            if let Some(value) = value {
                inputs.values.insert(arg.id.clone(), value);
            }
        }
    }

    pub fn set_text_value(&mut self, arg_id: &str, text: String) {
        let inputs = self.current_inputs_mut();
        inputs
            .values
            .insert(arg_id.to_string(), ArgValue::Text(text));
    }

    pub fn toggle_flag(&mut self, arg_id: &str) {
        let inputs = self.current_inputs_mut();
        let entry = inputs
            .values
            .entry(arg_id.to_string())
            .or_insert(ArgValue::Bool(false));
        if let ArgValue::Bool(value) = entry {
            *value = !*value;
        }
    }

    pub fn cycle_enum(&mut self, arg_id: &str, max: usize) {
        let inputs = self.current_inputs_mut();
        let entry = inputs
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
        self.touched
            .entry(key)
            .or_default()
            .insert(arg_id.to_string());
    }

    pub fn clear_touched(&mut self, arg_id: &str) {
        let key = self.command_path_key();
        if let Some(set) = self.touched.get_mut(&key) {
            set.remove(arg_id);
        }
    }

    pub fn is_touched(&self, arg_id: &str) -> bool {
        let key = self.command_path_key();
        self.touched
            .get(&key)
            .map(|set| set.contains(arg_id))
            .unwrap_or(false)
    }

    pub fn current_textareas_mut(&mut self) -> &mut HashMap<String, TextArea<'static>> {
        let key = self.command_path_key();
        self.textareas.entry(key).or_default()
    }

    pub fn textarea_for(&mut self, arg_id: &str, initial: &str) -> &mut TextArea<'static> {
        let textareas = self.current_textareas_mut();
        textareas
            .entry(arg_id.to_string())
            .or_insert_with(|| TextArea::new(vec![initial.to_string()]))
    }
}
