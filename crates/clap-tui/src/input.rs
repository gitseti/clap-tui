use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

use clap::Command;

use crate::editor_state::EditorState;
use crate::frame_snapshot::FrameSnapshot;
use crate::query::selectors as selector_query;
use crate::spec::{ArgSpec, CommandPath, CommandSpec, SelectionError};

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputSource {
    User,
    Default,
    Env,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputValueOccurrence {
    pub values: Vec<String>,
    pub source: InputSource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArgInput {
    Flag {
        present: bool,
        source: InputSource,
    },
    Count {
        occurrences: usize,
        source: InputSource,
    },
    Values {
        occurrences: Vec<InputValueOccurrence>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArgInputState {
    pub value: ArgInput,
    pub touched: bool,
}

#[derive(Debug, Default, Clone)]
pub struct CommandFormState {
    pub inputs: HashMap<String, ArgInputState>,
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
    pub validation_command: Option<Command>,
    pub selected_path: CommandPath,
    pub expanded: HashSet<String>,
    pub forms: HashMap<String, CommandFormState>,
    derived_revision: u64,
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
    pub dropdown_cursor: usize,
    pub sidebar_scroll: usize,
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
    derived_cache: Option<DerivedCache>,
}

#[derive(Debug, Clone)]
struct DerivedCache {
    revision: u64,
    derived: crate::pipeline::DerivedState,
}

impl DomainState {
    #[allow(dead_code)]
    pub fn new(root: CommandSpec) -> Self {
        Self::new_with_command(root, None)
    }

    pub fn new_with_command(root: CommandSpec, validation_command: Option<Command>) -> Self {
        let mut expanded = HashSet::new();
        expanded.insert(root.name.clone());
        Self {
            root,
            validation_command,
            selected_path: CommandPath::default(),
            expanded,
            forms: HashMap::new(),
            derived_revision: 0,
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

    #[allow(dead_code)]
    pub fn command_path_key(&self) -> String {
        self.command_path_key_for(&self.selected_path)
    }

    pub fn command_path_key_for(&self, path: &CommandPath) -> String {
        path.to_key(&self.root.name)
    }

    pub fn current_form(&self) -> Option<CommandFormState> {
        self.effective_form_for_path(&self.selected_path)
    }

    pub fn initialize_current_form_defaults(&mut self) {
        let mut changed = false;
        let args = self
            .root
            .args_defined_on_path(&self.selected_path)
            .unwrap_or_default()
            .into_iter()
            .map(|(owner_path, arg)| (owner_path.clone(), arg.clone()))
            .collect::<Vec<_>>();

        for (owner_path, arg) in args {
            let key = self.command_path_key_for(&owner_path);
            let default_input = Self::initial_input_state(&arg);
            let should_remove = {
                let form = self.forms.entry(key.clone()).or_default();
                if let Some(input) = default_input {
                    if !form.inputs.contains_key(&arg.id) {
                        form.inputs.insert(arg.id.clone(), input);
                        changed = true;
                    }
                }
                form.is_empty()
            };
            if should_remove && self.forms.remove(&key).is_some() {
                changed = true;
            }
        }
        if changed {
            self.bump_derived_revision();
        }
    }

    pub fn set_text_value(&mut self, arg_id: &str, text: &str) {
        let Some(arg) = self.arg_for_input(arg_id).cloned() else {
            return;
        };
        self.replace_occurrences(arg_id, text_value_occurrences(&arg, text));
    }

    pub fn set_choice_value(&mut self, arg_id: &str, value: String) {
        self.replace_occurrences(
            arg_id,
            vec![InputValueOccurrence {
                values: vec![value],
                source: InputSource::User,
            }],
        );
    }

    pub fn toggle_flag(&mut self, arg_id: &str) {
        if !self
            .arg_for_input(arg_id)
            .is_some_and(crate::spec::ArgSpec::uses_toggle_semantics)
        {
            return;
        }
        let current = self
            .effective_arg_input(arg_id)
            .and_then(|input| match input.value {
                ArgInput::Flag { present, .. } => Some(present),
                _ => None,
            })
            .unwrap_or(false);
        self.with_arg_input_state_mut(arg_id, |_arg, input| {
            input.value = ArgInput::Flag {
                present: !current,
                source: InputSource::User,
            };
        });
    }

    pub fn mark_touched(&mut self, arg_id: &str) {
        self.with_arg_input_state_mut(arg_id, |_arg, input| {
            input.touched = true;
        });
    }

    pub fn toggle_flag_touched(&mut self, arg_id: &str) {
        self.toggle_flag(arg_id);
        self.mark_touched(arg_id);
    }

    pub fn set_choice_value_touched(&mut self, arg_id: &str, value: String) {
        self.set_choice_value(arg_id, value);
        self.mark_touched(arg_id);
    }

    pub fn toggle_choice_value_touched(&mut self, arg_id: &str, value: &str) {
        let Some(arg) = self.arg_for_input(arg_id).cloned() else {
            return;
        };
        let mut values = self
            .current_form()
            .map_or_else(Vec::new, |inputs| inputs.selected_values(&arg));
        if let Some(index) = values.iter().position(|selected| selected == value) {
            values.remove(index);
        } else {
            values.push(value.to_string());
        }
        self.replace_values(arg_id, values);
        self.mark_touched(arg_id);
    }

    pub fn clear_value_and_untouch(&mut self, arg_id: &str) {
        let Some(arg) = self.arg_for_input(arg_id).cloned() else {
            return;
        };
        let Some(owner_key) = self.owner_key_for_arg(arg_id) else {
            return;
        };
        let default_input = Self::initial_input_state(&arg);
        let mut changed = false;
        let should_remove = {
            let form = self.forms.entry(owner_key.clone()).or_default();
            match default_input {
                Some(input) => {
                    if form.inputs.get(arg_id) != Some(&input) {
                        form.inputs.insert(arg_id.to_string(), input);
                        changed = true;
                    }
                }
                None => {
                    if form.inputs.remove(arg_id).is_some() {
                        changed = true;
                    }
                }
            }
            form.is_empty()
        };
        if should_remove && self.forms.remove(&owner_key).is_some() {
            changed = true;
        }
        if changed {
            self.bump_derived_revision();
        }
    }

    pub fn is_touched(&self, arg_id: &str) -> bool {
        self.current_form()
            .is_some_and(|form| form.is_touched(arg_id))
    }

    pub fn select_command_path(&mut self, path: &[String]) -> Result<(), SelectionError> {
        let normalized = self
            .root
            .normalize_path(path)
            .ok_or(SelectionError::UnknownPath)?;
        self.selected_path = normalized.clone();
        let prefix_path_len = normalized.as_slice().len().saturating_sub(1);
        let prefix_path = CommandPath::new(normalized.as_slice()[..prefix_path_len].to_vec());
        for key in self.root.expand_prefix_keys(&prefix_path) {
            self.expanded.insert(key);
        }
        self.bump_derived_revision();
        Ok(())
    }

    pub fn select_command_by_search_path(&mut self, start: &str) -> Result<(), SelectionError> {
        let normalized = self
            .root
            .find_path_by_search_path(start)
            .ok_or(SelectionError::UnknownPath)?;
        self.select_command_path(normalized.as_slice())
    }

    pub fn replace_values(&mut self, arg_id: &str, values: Vec<String>) {
        self.with_arg_input_state_mut(arg_id, move |arg, input| {
            *input = ArgInputState::from_values(arg, values, InputSource::User, true);
        });
    }

    pub fn replace_occurrences(&mut self, arg_id: &str, occurrences: Vec<InputValueOccurrence>) {
        self.with_arg_input_state_mut(arg_id, move |_arg, input| {
            *input = ArgInputState {
                value: ArgInput::Values { occurrences },
                touched: true,
            };
        });
    }

    #[allow(dead_code)]
    pub fn append_value(&mut self, arg_id: &str, value: String) {
        self.with_arg_input_state_mut(arg_id, move |arg, input| match &mut input.value {
            ArgInput::Values { occurrences } => {
                occurrences.push(InputValueOccurrence {
                    values: vec![value],
                    source: InputSource::User,
                });
                input.touched = true;
            }
            _ => {
                *input = ArgInputState::from_values(arg, vec![value], InputSource::User, true);
            }
        });
    }

    #[allow(dead_code)]
    pub fn remove_value(&mut self, arg_id: &str, occurrence_index: usize, value_index: usize) {
        self.with_arg_input_state_mut(arg_id, |_arg, input| {
            if let ArgInput::Values { occurrences } = &mut input.value {
                if let Some(occurrence) = occurrences.get_mut(occurrence_index)
                    && value_index < occurrence.values.len()
                {
                    occurrence.values.remove(value_index);
                }
                occurrences.retain(|occurrence| !occurrence.values.is_empty());
                input.touched = true;
            }
        });
    }

    #[allow(dead_code)]
    pub fn reorder_occurrence(&mut self, arg_id: &str, from: usize, to: usize) {
        self.with_arg_input_state_mut(arg_id, |_arg, input| {
            if let ArgInput::Values { occurrences } = &mut input.value {
                if from >= occurrences.len() || to >= occurrences.len() || from == to {
                    return;
                }
                let occurrence = occurrences.remove(from);
                occurrences.insert(to, occurrence);
                input.touched = true;
            }
        });
    }

    #[allow(dead_code)]
    pub fn increment_counter(&mut self, arg_id: &str) {
        self.with_arg_input_state_mut(arg_id, |_arg, input| {
            let current = match input.value {
                ArgInput::Count { occurrences, .. } => occurrences,
                _ => 0,
            };
            input.value = ArgInput::Count {
                occurrences: current.saturating_add(1),
                source: InputSource::User,
            };
            input.touched = true;
        });
    }

    #[allow(dead_code)]
    pub fn decrement_counter(&mut self, arg_id: &str) {
        self.with_arg_input_state_mut(arg_id, |_arg, input| {
            let current = match input.value {
                ArgInput::Count { occurrences, .. } => occurrences,
                _ => 0,
            };
            input.value = ArgInput::Count {
                occurrences: current.saturating_sub(1),
                source: InputSource::User,
            };
            input.touched = true;
        });
    }

    #[allow(dead_code)]
    pub fn toggle_optional_value_flag(&mut self, arg_id: &str, enabled: bool) {
        self.with_arg_input_state_mut(arg_id, |_arg, input| {
            input.value = ArgInput::Flag {
                present: enabled,
                source: InputSource::User,
            };
            input.touched = true;
        });
    }

    #[allow(dead_code)]
    pub fn resolve_invocation_state(&self) -> Vec<(CommandPath, CommandFormState)> {
        self.root
            .command_lineage(&self.selected_path)
            .unwrap_or_default()
            .into_iter()
            .map(|command| {
                let key = self.command_path_key_for(&command.path);
                (
                    command.path.clone(),
                    self.forms.get(&key).cloned().unwrap_or_default(),
                )
            })
            .collect()
    }

    fn effective_form_for_path(&self, path: &CommandPath) -> Option<CommandFormState> {
        let mut form = CommandFormState::default();

        for (_, arg) in self
            .root
            .effective_args_for_path(path)?
            .into_iter()
            .filter(|(_, arg)| !arg.is_external_subcommand_field() || arg.owner_path() == path)
        {
            let input = self.stored_arg_input(arg);
            if let Some(input) = input {
                form.inputs.insert(arg.id.clone(), input);
            }
        }

        (!form.is_empty()).then_some(form)
    }

    pub(crate) fn arg_for_input(&self, arg_id: &str) -> Option<&ArgSpec> {
        self.root
            .effective_args_for_path(&self.selected_path)?
            .into_iter()
            .map(|(_, arg)| arg)
            .filter(|arg| {
                !arg.is_external_subcommand_field() || arg.owner_path() == &self.selected_path
            })
            .find(|arg| arg.id == arg_id && !arg.is_help_action())
    }

    fn owner_key_for_arg(&self, arg_id: &str) -> Option<String> {
        let owner_path = self
            .arg_for_input(arg_id)
            .map(|arg| arg.owner_path().clone())?;
        Some(self.command_path_key_for(&owner_path))
    }

    fn effective_arg_input(&self, arg_id: &str) -> Option<ArgInputState> {
        let arg = self.arg_for_input(arg_id)?;
        self.stored_arg_input(arg)
    }

    fn stored_arg_input(&self, arg: &ArgSpec) -> Option<ArgInputState> {
        let key = self.command_path_key_for(arg.owner_path());
        self.forms
            .get(&key)
            .and_then(|form| form.inputs.get(&arg.id))
            .cloned()
    }

    fn with_arg_input_state_mut<F>(&mut self, arg_id: &str, mutate: F)
    where
        F: FnOnce(&ArgSpec, &mut ArgInputState),
    {
        let Some(arg) = self.arg_for_input(arg_id).cloned() else {
            return;
        };
        let key = self.command_path_key_for(arg.owner_path());
        let default_input = Self::initial_input_state(&arg);
        let mut changed = false;
        let should_remove = {
            let form = self.forms.entry(key.clone()).or_default();
            let mut input = form
                .inputs
                .get(arg_id)
                .cloned()
                .or_else(|| default_input.clone())
                .unwrap_or_else(|| ArgInputState::empty_for(&arg));
            mutate(&arg, &mut input);
            if input.is_effectively_empty() {
                if form.inputs.remove(arg_id).is_some() {
                    changed = true;
                }
            } else if form.inputs.get(arg_id) != Some(&input) {
                form.inputs.insert(arg_id.to_string(), input);
                changed = true;
            }
            form.is_empty()
        };
        if should_remove && self.forms.remove(&key).is_some() {
            changed = true;
        }
        if changed {
            self.bump_derived_revision();
        }
    }

    pub(crate) fn derived_revision(&self) -> u64 {
        self.derived_revision
    }

    fn bump_derived_revision(&mut self) {
        self.derived_revision = self.derived_revision.saturating_add(1);
    }

    fn initial_input_state(arg: &ArgSpec) -> Option<ArgInputState> {
        if matches!(arg.action_kind(), crate::spec::ArgActionKind::Count) {
            return Some(ArgInputState {
                value: ArgInput::Count {
                    occurrences: 0,
                    source: InputSource::Default,
                },
                touched: false,
            });
        }

        if arg.uses_toggle_semantics() {
            return Some(ArgInputState {
                value: ArgInput::Flag {
                    present: false,
                    source: InputSource::Default,
                },
                touched: false,
            });
        }

        if let Some(env) = arg.metadata.defaults.env.as_deref()
            && let Ok(value) = std::env::var(env)
        {
            return Some(ArgInputState::from_values(
                arg,
                vec![value],
                InputSource::Env,
                false,
            ));
        }

        if !arg.default_values.is_empty() {
            return Some(ArgInputState::from_values(
                arg,
                arg.default_values.clone(),
                InputSource::Default,
                false,
            ));
        }

        None
    }
}

impl CommandFormState {
    pub fn input(&self, arg_id: &str) -> Option<&ArgInputState> {
        self.inputs.get(arg_id)
    }

    pub fn compatibility_value(&self, arg: &ArgSpec) -> Option<ArgValue> {
        self.input(&arg.id)
            .and_then(|input| input.compatibility_value(arg))
    }

    pub fn is_touched(&self, arg_id: &str) -> bool {
        self.input(arg_id).is_some_and(|input| input.touched)
    }

    pub fn is_empty(&self) -> bool {
        self.inputs.is_empty()
    }

    pub fn selected_values(&self, arg: &ArgSpec) -> Vec<String> {
        self.input(&arg.id)
            .map_or_else(Vec::new, ArgInputState::selected_values)
    }

    pub fn input_source(&self, arg_id: &str) -> Option<InputSource> {
        self.input(arg_id).and_then(ArgInputState::input_source)
    }
}

impl ArgInput {
    fn values_from_arg(arg: &ArgSpec, values: Vec<String>, source: InputSource) -> Self {
        let occurrences = if arg.allows_multiple_occurrences() {
            values
                .into_iter()
                .map(|value| InputValueOccurrence {
                    values: vec![value],
                    source,
                })
                .collect()
        } else {
            vec![InputValueOccurrence { values, source }]
        };
        Self::Values { occurrences }
    }
}

impl ArgInputState {
    fn from_values(arg: &ArgSpec, values: Vec<String>, source: InputSource, touched: bool) -> Self {
        Self {
            value: ArgInput::values_from_arg(arg, values, source),
            touched,
        }
    }

    fn empty_for(arg: &ArgSpec) -> Self {
        if arg.uses_toggle_semantics() {
            Self {
                value: ArgInput::Flag {
                    present: false,
                    source: InputSource::Default,
                },
                touched: false,
            }
        } else {
            Self {
                value: ArgInput::Values {
                    occurrences: Vec::new(),
                },
                touched: false,
            }
        }
    }

    pub fn compatibility_value(&self, arg: &ArgSpec) -> Option<ArgValue> {
        match &self.value {
            ArgInput::Flag { present, .. } => Some(ArgValue::Bool(*present)),
            ArgInput::Count { occurrences, .. } => Some(ArgValue::Text(occurrences.to_string())),
            ArgInput::Values { occurrences } => {
                let values = self.selected_values();
                if values.is_empty() {
                    return None;
                }
                if arg.uses_optional_value_semantics() {
                    let rendered = occurrences
                        .iter()
                        .map(|occurrence| render_occurrence_text(arg, &occurrence.values))
                        .filter(|value| !value.is_empty())
                        .collect::<Vec<_>>()
                        .join("\n");
                    return Some(ArgValue::Text(rendered));
                }
                if arg.has_value_choices() && !arg.is_multi_value_input() {
                    values.first().cloned().map(ArgValue::Choice)
                } else {
                    let rendered = occurrences
                        .iter()
                        .map(|occurrence| render_occurrence_text(arg, &occurrence.values))
                        .filter(|value| !value.is_empty())
                        .collect::<Vec<_>>()
                        .join("\n");
                    Some(ArgValue::Text(rendered))
                }
            }
        }
    }

    pub fn selected_values(&self) -> Vec<String> {
        match &self.value {
            ArgInput::Flag { present, .. } => {
                present.then(|| "true".to_string()).into_iter().collect()
            }
            ArgInput::Count { occurrences, .. } => (*occurrences > 0)
                .then(|| occurrences.to_string())
                .into_iter()
                .collect(),
            ArgInput::Values { occurrences } => occurrences
                .iter()
                .flat_map(|occurrence| occurrence.values.iter())
                .filter(|value| !value.is_empty())
                .cloned()
                .collect(),
        }
    }

    pub fn input_source(&self) -> Option<InputSource> {
        match &self.value {
            ArgInput::Flag { source, .. } | ArgInput::Count { source, .. } => Some(*source),
            ArgInput::Values { occurrences } => occurrences
                .iter()
                .find(|occurrence| occurrence.values.iter().any(|value| !value.is_empty()))
                .map(|occurrence| occurrence.source),
        }
    }

    pub fn is_effectively_empty(&self) -> bool {
        match &self.value {
            ArgInput::Flag { present, source } => {
                !present && !self.touched && *source == InputSource::Default
            }
            ArgInput::Count {
                occurrences,
                source,
            } => *occurrences == 0 && !self.touched && *source == InputSource::Default,
            ArgInput::Values { occurrences } => occurrences.is_empty(),
        }
    }
}

fn text_value_occurrences(arg: &ArgSpec, text: &str) -> Vec<InputValueOccurrence> {
    if arg.allows_multiple_occurrences() {
        return text
            .lines()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|value| InputValueOccurrence {
                values: split_occurrence_values(arg, value),
                source: InputSource::User,
            })
            .collect();
    }

    vec![InputValueOccurrence {
        values: split_occurrence_values(arg, text),
        source: InputSource::User,
    }]
}

pub(crate) fn split_occurrence_values(arg: &ArgSpec, text: &str) -> Vec<String> {
    if arg.accepts_multiple_values_per_occurrence() {
        if let Some(delimiter) = arg.metadata.syntax.value_delimiter {
            return text
                .split(delimiter)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string)
                .collect();
        }

        return text
            .lines()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
            .collect();
    }

    vec![text.to_string()]
}

pub(crate) fn render_occurrence_text(arg: &ArgSpec, values: &[String]) -> String {
    if let Some(delimiter) = arg.metadata.syntax.value_delimiter {
        values.join(&delimiter.to_string())
    } else if arg.accepts_multiple_values_per_occurrence() {
        values.join("\n")
    } else {
        values.first().cloned().unwrap_or_default()
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

    pub fn focus_sidebar(&mut self) {
        self.dismiss_transient_interaction();
        self.focus = Focus::Sidebar;
    }

    pub fn focus_form(&mut self) {
        self.focus = Focus::Form;
    }

    pub fn focus_search(&mut self) {
        self.dismiss_transient_interaction();
        self.focus = Focus::Search;
    }

    pub fn focus_next(&mut self) {
        self.dismiss_transient_interaction();
        self.focus = match self.focus {
            Focus::Sidebar => Focus::Search,
            Focus::Search => Focus::Form,
            Focus::Form => Focus::Sidebar,
        };
    }

    pub fn focus_previous(&mut self) {
        self.dismiss_transient_interaction();
        self.focus = match self.focus {
            Focus::Sidebar => Focus::Form,
            Focus::Search => Focus::Sidebar,
            Focus::Form => Focus::Search,
        };
    }

    pub fn toggle_help(&mut self) {
        self.help_open = !self.help_open;
        self.help_scroll = 0;
        self.close_dropdown();
        self.clear_mouse_selection();
    }

    pub fn close_dropdown(&mut self) {
        self.dropdown_open = None;
        self.dropdown_cursor = 0;
    }

    pub fn open_dropdown(&mut self, arg_id: impl Into<String>, scroll: usize, cursor: usize) {
        self.dropdown_open = Some(arg_id.into());
        self.dropdown_scroll = scroll;
        self.dropdown_cursor = cursor;
    }

    pub fn set_dropdown_scroll(&mut self, scroll: usize) {
        self.dropdown_scroll = scroll;
    }

    pub fn set_dropdown_cursor(&mut self, cursor: usize) {
        self.dropdown_cursor = cursor;
    }

    pub fn dropdown_scroll(&self, total_rows: usize, visible_rows: usize) -> usize {
        clamp_dropdown_scroll(self.dropdown_scroll, total_rows, visible_rows)
    }

    pub fn dropdown_cursor(&self, total_rows: usize) -> usize {
        if total_rows == 0 {
            0
        } else {
            self.dropdown_cursor.min(total_rows - 1)
        }
    }

    pub fn clamp_dropdown_scroll(&mut self, total_rows: usize, visible_rows: usize) {
        self.dropdown_scroll = self.dropdown_scroll(total_rows, visible_rows);
    }

    pub fn adjust_dropdown_scroll(&mut self, delta: i16, total_rows: usize, visible_rows: usize) {
        if delta.is_negative() {
            self.dropdown_scroll = self
                .dropdown_scroll
                .saturating_sub(usize::from(delta.unsigned_abs()));
        } else {
            self.dropdown_scroll = self
                .dropdown_scroll
                .saturating_add(usize::from(delta.unsigned_abs()));
        }
        self.clamp_dropdown_scroll(total_rows, visible_rows);
    }

    pub fn set_form_scroll(&mut self, scroll: u16) {
        self.form_scroll = scroll;
    }

    pub fn sidebar_scroll(&self, total_rows: usize, visible_rows: usize) -> usize {
        clamp_dropdown_scroll(self.sidebar_scroll, total_rows, visible_rows)
    }

    pub fn set_sidebar_scroll(&mut self, scroll: usize) {
        self.sidebar_scroll = scroll;
    }

    pub fn clamp_sidebar_scroll(&mut self, total_rows: usize, visible_rows: usize) {
        self.sidebar_scroll = self.sidebar_scroll(total_rows, visible_rows);
    }

    pub fn adjust_sidebar_scroll(&mut self, delta: i16, total_rows: usize, visible_rows: usize) {
        if delta.is_negative() {
            self.sidebar_scroll = self
                .sidebar_scroll
                .saturating_sub(usize::from(delta.unsigned_abs()));
        } else {
            self.sidebar_scroll = self
                .sidebar_scroll
                .saturating_add(usize::from(delta.unsigned_abs()));
        }
        self.clamp_sidebar_scroll(total_rows, visible_rows);
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

    pub fn dismiss_transient_interaction(&mut self) {
        self.close_dropdown();
        self.clear_mouse_selection();
    }

    pub fn reset_transient_form_ui(&mut self) {
        self.form_scroll = 0;
        self.dismiss_transient_interaction();
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
    #[allow(dead_code)]
    pub fn new(root: CommandSpec) -> Self {
        Self::new_with_command(root, None)
    }

    pub fn from_command(command: &Command) -> Self {
        Self::new_with_command(CommandSpec::from_command(command), Some(command.clone()))
    }

    pub fn new_with_command(root: CommandSpec, validation_command: Option<Command>) -> Self {
        let mut state = Self {
            domain: DomainState::new_with_command(root, validation_command),
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
                dropdown_cursor: 0,
                sidebar_scroll: 0,
                form_scroll: 0,
                hover: None,
                hover_tab: None,
                mouse_select: None,
            },
            notifications: NotificationState::default(),
            derived_cache: None,
        };
        state.initialize_current_command();
        state
    }

    pub fn initialize_current_command(&mut self) {
        self.domain.initialize_current_form_defaults();
        self.sync_visible_form_selection();
    }

    pub fn sync_visible_form_selection(&mut self) {
        let active_args = selector_query::visible_form_args(
            &self.domain.root,
            self.domain.selected_path(),
            self.ui.active_tab,
        );
        let visible = selector_query::visible_form_arg_pairs(&active_args);
        self.ui.ensure_active_tab_visible(&visible);
        self.ui.ensure_selected_arg_visible(&visible);
    }

    pub fn select_command_path(&mut self, path: &[String]) -> Result<(), SelectionError> {
        self.domain.select_command_path(path)?;
        self.initialize_current_command();
        Ok(())
    }

    pub fn select_command_by_search_path(&mut self, start: &str) -> Result<(), SelectionError> {
        self.domain.select_command_by_search_path(start)?;
        self.initialize_current_command();
        Ok(())
    }

    pub(crate) fn derived(&mut self) -> &crate::pipeline::DerivedState {
        let revision = self.domain.derived_revision();
        let needs_refresh = self
            .derived_cache
            .as_ref()
            .is_none_or(|cache| cache.revision != revision);
        if needs_refresh {
            let derived = crate::pipeline::derive(self);
            self.derived_cache = Some(DerivedCache { revision, derived });
        }
        &self
            .derived_cache
            .as_ref()
            .expect("derived cache should exist after refresh")
            .derived
    }

    pub(crate) fn preview_argv(&mut self) -> Vec<String> {
        self.derived().argv.clone()
    }

    pub(crate) fn derived_validation(&mut self) -> crate::pipeline::ValidationState {
        self.derived().validation.clone()
    }
}

fn clamp_dropdown_scroll(current: usize, total_rows: usize, visible_rows: usize) -> usize {
    if total_rows == 0 || visible_rows == 0 {
        return 0;
    }
    current.min(total_rows.saturating_sub(visible_rows.min(total_rows)))
}

#[cfg(test)]
mod tests {
    use clap::{Arg, ArgAction, Command};

    use super::{AppState, ArgInput, ArgValue, InputSource};
    use crate::spec::{ArgKind, ArgSpec, CommandSpec, ValueCardinality};

    fn arg(id: &str, name: &str, kind: ArgKind) -> ArgSpec {
        ArgSpec {
            id: id.to_string(),
            display_name: name.to_string(),
            help: None,
            required: false,
            kind,
            default_values: Vec::new(),
            choices: Vec::new(),
            position: None,
            value_cardinality: ValueCardinality::One,
            value_hint: None,
            ..ArgSpec::default()
        }
    }

    #[test]
    fn new_state_initializes_materialized_defaults_without_normalize() {
        let root = CommandSpec {
            name: "tool".to_string(),
            version: None,
            about: None,
            help: String::new(),
            args: vec![arg("verbose", "--verbose", ArgKind::Flag)],
            subcommands: Vec::new(),
            ..CommandSpec::default()
        };

        let state = AppState::new(root);

        assert_eq!(
            state.domain.current_form().and_then(|form| {
                state
                    .domain
                    .current_command()
                    .args
                    .iter()
                    .find(|arg| arg.id == "verbose")
                    .and_then(|arg| form.compatibility_value(arg))
            }),
            Some(ArgValue::Bool(false))
        );
    }

    #[test]
    fn env_backed_defaults_remain_stable_after_env_source_changes() {
        let path = std::env::var("PATH").expect("PATH should exist for env-backed default tests");
        let root = CommandSpec::from_command(
            &Command::new("tool").arg(Arg::new("config").long("config").env("PATH")),
        );
        let mut state = AppState::new(root);
        let initial_form = state.domain.current_form().expect("materialized form");
        let config = state
            .domain
            .current_command()
            .args
            .iter()
            .find(|arg| arg.id == "config")
            .expect("config arg");
        assert_eq!(
            initial_form.compatibility_value(config),
            Some(ArgValue::Text(path.clone()))
        );
        assert_eq!(initial_form.input_source("config"), Some(InputSource::Env));

        state.domain.root.args[0].metadata.defaults.env =
            Some("CLAP_TUI_TEST_ENV_STABILITY_UNUSED".to_string());

        let current_form = state
            .domain
            .current_form()
            .expect("effective form remains available");
        let config = state
            .domain
            .current_command()
            .args
            .iter()
            .find(|arg| arg.id == "config")
            .expect("config arg");
        assert_eq!(
            current_form.compatibility_value(config),
            Some(ArgValue::Text(path))
        );
        assert_eq!(current_form.input_source("config"), Some(InputSource::Env));
    }

    #[test]
    fn clearing_last_optional_value_removes_empty_form_state() {
        let root = CommandSpec {
            name: "tool".to_string(),
            version: None,
            about: None,
            help: String::new(),
            args: vec![arg("target", "--target", ArgKind::Option)],
            subcommands: Vec::new(),
            ..CommandSpec::default()
        };
        let mut state = AppState::new(root);

        state.domain.set_text_value("target", "value");
        state.domain.mark_touched("target");
        state.domain.clear_value_and_untouch("target");

        assert!(state.domain.current_form().is_none());
    }

    #[test]
    fn global_values_are_stored_on_owner_and_projected_into_descendants() {
        let root = CommandSpec::from_command(
            &Command::new("tool")
                .arg(
                    Arg::new("verbose")
                        .long("verbose")
                        .action(ArgAction::SetTrue)
                        .global(true),
                )
                .subcommand(
                    Command::new("build")
                        .arg(Arg::new("target").long("target"))
                        .subcommand(Command::new("release")),
                ),
        );
        let mut state = AppState::new(root);
        state
            .select_command_path(&["build".to_string(), "release".to_string()])
            .expect("valid path");

        state.domain.toggle_flag_touched("verbose");

        let root_form = state
            .domain
            .forms
            .get("tool")
            .expect("root form should own global state");
        let verbose = root_form
            .inputs
            .get("verbose")
            .expect("global flag should be stored");
        assert_eq!(
            verbose,
            &super::ArgInputState {
                value: ArgInput::Flag {
                    present: true,
                    source: InputSource::User,
                },
                touched: true,
            }
        );

        let effective = state.domain.current_form().expect("effective form");
        let verbose_arg = state
            .domain
            .current_command()
            .args
            .iter()
            .find(|arg| arg.id == "verbose")
            .expect("verbose arg");
        assert_eq!(
            effective.compatibility_value(verbose_arg),
            Some(ArgValue::Bool(true))
        );
    }

    #[test]
    fn optional_value_with_choices_stays_text_compatible_for_editing() {
        let mut color = arg("color", "--color", ArgKind::Flag);
        color.metadata.action.value_arity = crate::spec::ArgValueArity::Optional;
        color.choices = vec![
            "auto".to_string(),
            "always".to_string(),
            "never".to_string(),
        ];
        let root = CommandSpec {
            name: "tool".to_string(),
            version: None,
            about: None,
            help: String::new(),
            args: vec![color.clone()],
            subcommands: Vec::new(),
            ..CommandSpec::default()
        };
        let mut state = AppState::new(root);

        state.domain.set_text_value("color", "n");
        state.domain.mark_touched("color");

        assert_eq!(
            state
                .domain
                .current_form()
                .and_then(|form| form.compatibility_value(&color)),
            Some(ArgValue::Text("n".to_string()))
        );
    }

    #[test]
    fn invocation_helpers_cover_append_reorder_and_counter_mutations() {
        let root = CommandSpec::from_command(
            &Command::new("tool")
                .arg(
                    Arg::new("include")
                        .long("include")
                        .action(ArgAction::Append)
                        .num_args(1..),
                )
                .arg(Arg::new("verbose").short('v').action(ArgAction::Count)),
        );
        let mut state = AppState::new(root);

        state
            .domain
            .replace_values("include", vec!["src".to_string()]);
        state.domain.append_value("include", "tests".to_string());
        state.domain.reorder_occurrence("include", 1, 0);
        state.domain.remove_value("include", 1, 0);
        state.domain.increment_counter("verbose");
        state.domain.increment_counter("verbose");
        state.domain.decrement_counter("verbose");
        state.domain.toggle_optional_value_flag("verbose", true);

        let invocation = state.domain.resolve_invocation_state();
        assert_eq!(invocation.len(), 1);
        assert_eq!(state.domain.command_path_key(), "tool");

        let include = invocation[0]
            .1
            .inputs
            .get("include")
            .expect("include input should exist");
        match &include.value {
            ArgInput::Values { occurrences } => {
                assert_eq!(occurrences.len(), 1);
                assert_eq!(occurrences[0].values, vec!["tests".to_string()]);
            }
            other => panic!("expected values input, got {other:?}"),
        }
    }
}
