use std::io::{self, Stdout};
use std::time::Duration;

use clap::{Command, CommandFactory, Parser};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEventKind};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use tui_textarea::{CursorMove, TextArea};

use crate::config::TuiConfig;
use crate::error::TuiError;
use crate::input::{ActiveTab, AppState, ArgValue, Focus, MouseSelection};
use crate::spec::{ArgKind, CommandSpec};
use crate::ui;

/// Primary entry point for building and running the TUI.
pub struct TuiApp {
    command: Command,
    config: TuiConfig,
}

impl TuiApp {
    /// Create from a `clap::Command`.
    pub fn from_command(command: Command) -> Self {
        Self {
            command,
            config: TuiConfig::default(),
        }
    }

    /// Create from a `clap::CommandFactory` (derive-based CLI).
    pub fn from_factory<T: CommandFactory>() -> Self {
        Self::from_command(T::command())
    }

    /// Apply configuration.
    pub fn with_config(mut self, config: TuiConfig) -> Self {
        self.config = config;
        self
    }

    /// Run the TUI and return the argv for the selected command.
    pub fn run(self) -> Result<Vec<String>, TuiError> {
        match self.run_inner() {
            Ok(argv) => Ok(argv),
            Err(TuiError::Cancelled) => Ok(Vec::new()),
            Err(err) => Err(err),
        }
    }

    /// Run the TUI and execute a custom handler with `ArgMatches`.
    pub fn run_with_matches<F>(self, runner: F) -> Result<(), TuiError>
    where
        F: FnOnce(clap::ArgMatches) -> anyhow::Result<()>,
    {
        let command = self.command.clone();
        let argv = match self.run_inner() {
            Ok(argv) => argv,
            Err(TuiError::Cancelled) => return Ok(()),
            Err(err) => return Err(err),
        };
        let matches = command.try_get_matches_from(argv)?;
        runner(matches).map_err(|err| TuiError::Terminal(io::Error::new(io::ErrorKind::Other, err)))
    }

    /// Run the TUI and parse into a `clap::Parser` struct.
    pub fn run_with_parser<T, F>(self, runner: F) -> Result<(), TuiError>
    where
        T: Parser,
        F: FnOnce(T) -> anyhow::Result<()>,
    {
        let argv = match self.run_inner() {
            Ok(argv) => argv,
            Err(TuiError::Cancelled) => return Ok(()),
            Err(err) => return Err(err),
        };
        let parsed = T::try_parse_from(argv)?;
        runner(parsed).map_err(|err| TuiError::Terminal(io::Error::new(io::ErrorKind::Other, err)))
    }

    fn run_inner(mut self) -> Result<Vec<String>, TuiError> {
        let mut stdout = io::stdout();
        enable_raw_mode()?;
        execute!(stdout, EnterAlternateScreen)?;
        execute!(
            stdout,
            crossterm::terminal::Clear(crossterm::terminal::ClearType::All)
        )?;
        #[cfg(feature = "mouse")]
        {
            execute!(stdout, crossterm::event::EnableMouseCapture)?;
        }

        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        let result = self.event_loop(&mut terminal);

        disable_raw_mode()?;
        #[cfg(feature = "mouse")]
        {
            execute!(
                terminal.backend_mut(),
                crossterm::event::DisableMouseCapture
            )?;
        }
        execute!(
            terminal.backend_mut(),
            crossterm::terminal::Clear(crossterm::terminal::ClearType::All)
        )?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;

        result
    }

    fn event_loop(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    ) -> Result<Vec<String>, TuiError> {
        let spec = CommandSpec::from_command(&self.command);
        let mut state = AppState::new(spec);
        if let Some(start) = self.config.start_command.clone() {
            apply_start_command(&mut state, &start);
        }

        loop {
            terminal.draw(|frame| ui::render(frame, &mut state, &self.config))?;

            if event::poll(Duration::from_millis(200))? {
                match event::read()? {
                    Event::Key(key) => {
                        if let Some(action) = handle_key_event(key, &mut state, &self.config) {
                            match action {
                                Action::Run(argv) => return Ok(argv),
                                Action::Exit => return Err(TuiError::Cancelled),
                            }
                        }
                    }
                    Event::Mouse(mouse) => {
                        if let Some(action) = handle_mouse_event(mouse, &mut state, &self.config) {
                            match action {
                                Action::Run(argv) => return Ok(argv),
                                Action::Exit => return Err(TuiError::Cancelled),
                            }
                        }
                    }
                    Event::Resize(_, _) => {
                        // Layout recalculates on next draw.
                    }
                    _ => {}
                }
            }
        }
    }
}

enum Action {
    Run(Vec<String>),
    Exit,
}

fn handle_key_event(key: KeyEvent, state: &mut AppState, config: &TuiConfig) -> Option<Action> {
    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
        return Some(Action::Exit);
    }

    if key.code == KeyCode::Enter && key.modifiers.contains(KeyModifiers::CONTROL) {
        return Some(Action::Run(build_argv(state)));
    }

    if matches!(state.focus, Focus::Search) {
        handle_search_input(key, state);
        return None;
    }

    if let Some(active_enum) = state.enum_open.clone() {
        if handle_enum_input(key, state, &active_enum) {
            return None;
        }
    }

    if matches!(state.focus, Focus::Form) {
        if handle_form_text_input(key, state, config) {
            return None;
        }
    }

    match key.code {
        KeyCode::Tab => {
            state.focus = match state.focus {
                Focus::Sidebar => Focus::Form,
                _ => Focus::Sidebar,
            };
        }
        KeyCode::Char(c) if c == config.keymap.help => {
            toggle_help_tab(state);
        }
        KeyCode::F(1) => {
            toggle_help_tab(state);
        }
        KeyCode::BackTab => {
            if matches!(state.focus, Focus::Form) {
                cycle_tabs(state);
            }
        }
        KeyCode::Char(c) if c == config.keymap.search => {
            state.focus = Focus::Search;
        }
        KeyCode::Up => match state.focus {
            Focus::Sidebar => move_sidebar_selection(state, -1),
            Focus::Form => move_form_selection(state, -1),
            _ => {}
        },
        KeyCode::Down => match state.focus {
            Focus::Sidebar => move_sidebar_selection(state, 1),
            Focus::Form => move_form_selection(state, 1),
            _ => {}
        },
        KeyCode::Left => {
            if matches!(state.focus, Focus::Sidebar) {
                collapse_selected(state);
            }
        }
        KeyCode::Right => {
            if matches!(state.focus, Focus::Sidebar) {
                expand_selected(state);
            }
        }
        KeyCode::Enter => {
            if matches!(state.focus, Focus::Sidebar) {
                select_sidebar(state);
            } else if matches!(state.focus, Focus::Form) {
                activate_form_field(state);
            }
        }
        KeyCode::Char(' ') => {
            if matches!(state.focus, Focus::Form) {
                activate_form_field(state);
            }
        }
        _ => {}
    }

    None
}

fn handle_search_input(key: KeyEvent, state: &mut AppState) -> bool {
    match key.code {
        KeyCode::Esc => {
            state.focus = Focus::Sidebar;
        }
        KeyCode::Enter => {
            state.focus = Focus::Sidebar;
        }
        KeyCode::Backspace => {
            state.search.pop();
        }
        KeyCode::Char(c) => {
            state.search.push(c);
        }
        _ => {}
    }
    true
}

fn handle_form_text_input(key: KeyEvent, state: &mut AppState, config: &TuiConfig) -> bool {
    if matches!(state.active_tab, ActiveTab::Help) {
        return false;
    }
    let args = state.visible_args();
    if args.is_empty() {
        return false;
    }
    let Some((_, arg)) = args
        .iter()
        .find(|(idx, _)| *idx == state.selected_arg_index)
    else {
        return false;
    };
    let arg_id = arg.id.clone();
    let arg_kind = arg.kind;
    if !matches!(arg_kind, ArgKind::Option | ArgKind::Positional) {
        return false;
    }

    match key.code {
        KeyCode::Tab | KeyCode::Up | KeyCode::Down | KeyCode::Enter => return false,
        KeyCode::Char(c) if c == config.keymap.search => return false,
        _ => {}
    }

    let current = state
        .current_inputs()
        .and_then(|inputs| inputs.values.get(&arg_id))
        .and_then(|value| match value {
            ArgValue::Text(text) => Some(text.clone()),
            _ => None,
        })
        .unwrap_or_default();

    let default_value = state
        .current_command()
        .args
        .iter()
        .find(|a| a.id == arg_id)
        .and_then(|a| a.default.clone());

    let has_default = state
        .current_command()
        .args
        .iter()
        .find(|a| a.id == arg_id)
        .and_then(|a| a.default.clone())
        .is_some();
    let is_touched = state.is_touched(&arg_id);
    let textarea = state.textarea_for(&arg_id, &current);
    if has_default && !is_touched {
        match key.code {
            KeyCode::Char(_) | KeyCode::Backspace => {
                *textarea = TextArea::new(vec![String::new()]);
            }
            _ => {}
        }
    }
    let modified = textarea.input(key);
    if modified {
        let text = textarea.lines().join("\n");
        if text.is_empty() && default_value.is_some() {
            state.current_inputs_mut().values.remove(&arg_id);
            state.clear_touched(&arg_id);
        } else {
            state.set_text_value(&arg_id, text);
            state.mark_touched(&arg_id);
        }
    }
    true
}

fn switch_tab(state: &mut AppState, tab: ActiveTab) {
    let tabs = state.visible_tabs();
    let target = if tabs.contains(&tab) {
        tab
    } else {
        tabs.first().copied().unwrap_or(ActiveTab::Options)
    };
    if target == state.active_tab {
        return;
    }
    state.active_tab = target;
    if state.active_tab != ActiveTab::Help {
        state.last_non_help_tab = state.active_tab;
        state.ensure_selected_arg_visible();
    }
    state.form_scroll = 0;
    state.enum_open = None;
    state.mouse_select = None;
}

fn cycle_tabs(state: &mut AppState) {
    let tabs = state.visible_tabs();
    if tabs.len() <= 1 {
        return;
    }
    let current = tabs
        .iter()
        .position(|t| *t == state.active_tab)
        .unwrap_or(0);
    let next = (current + 1) % tabs.len();
    switch_tab(state, tabs[next]);
}

fn toggle_help_tab(state: &mut AppState) {
    if state.active_tab == ActiveTab::Help {
        let tabs = state.visible_tabs();
        let mut target = state.last_non_help_tab;
        if !tabs.contains(&target) {
            target = tabs.first().copied().unwrap_or(ActiveTab::Options);
        }
        switch_tab(state, target);
    } else {
        state.last_non_help_tab = state.active_tab;
        switch_tab(state, ActiveTab::Help);
    }
}

fn handle_enum_input(key: KeyEvent, state: &mut AppState, arg_id: &str) -> bool {
    let arg = match state.current_command().args.iter().find(|a| a.id == arg_id) {
        Some(arg) => arg,
        None => {
            state.enum_open = None;
            return true;
        }
    };
    let arg_id = arg.id.clone();
    let len = arg.possible_values.len();

    match key.code {
        KeyCode::Esc => {
            state.enum_open = None;
            return true;
        }
        KeyCode::Up => {
            if len == 0 {
                return true;
            }
            let current = state
                .current_inputs()
                .and_then(|inputs| inputs.values.get(&arg_id))
                .and_then(|value| match value {
                    ArgValue::Enum(idx) => Some(*idx),
                    _ => None,
                })
                .unwrap_or(0);
            let next = if current == 0 { len - 1 } else { current - 1 };
            let inputs = state.current_inputs_mut();
            inputs.values.insert(arg_id.clone(), ArgValue::Enum(next));
            state.mark_touched(&arg_id);
            ensure_enum_visible(state, next, len);
            return true;
        }
        KeyCode::Down => {
            state.cycle_enum(&arg_id, len);
            let current = state
                .current_inputs()
                .and_then(|inputs| inputs.values.get(&arg_id))
                .and_then(|value| match value {
                    ArgValue::Enum(idx) => Some(*idx),
                    _ => None,
                })
                .unwrap_or(0);
            state.mark_touched(&arg_id);
            ensure_enum_visible(state, current, len);
            return true;
        }
        KeyCode::Enter | KeyCode::Char(' ') => {
            state.enum_open = None;
            return true;
        }
        _ => {}
    }
    false
}

fn move_sidebar_selection(state: &mut AppState, delta: isize) {
    let items = ui::tree_items(state);
    if items.is_empty() {
        return;
    }
    let current_index = items
        .iter()
        .position(|item| item.path == state.selected_path)
        .unwrap_or(0) as isize;
    let next_index = (current_index + delta).clamp(0, items.len() as isize - 1) as usize;
    if state.selected_path != items[next_index].path {
        state.selected_path = items[next_index].path.clone();
        state.focus_first_tab();
    }
}

fn move_form_selection(state: &mut AppState, delta: isize) {
    if matches!(state.active_tab, ActiveTab::Help) {
        return;
    }
    let args = state.visible_args();
    if args.is_empty() {
        return;
    }
    let current_pos = args
        .iter()
        .position(|(idx, _)| *idx == state.selected_arg_index)
        .unwrap_or(0) as isize;
    let max = args.len() as isize - 1;
    let next_pos = (current_pos + delta).clamp(0, max) as usize;
    state.selected_arg_index = args[next_pos].0;
    ensure_form_visible(state);
}

fn select_sidebar(state: &mut AppState) {
    let items = ui::tree_items(state);
    let selected = items.iter().find(|item| item.path == state.selected_path);
    if let Some(item) = selected {
        if item.has_children {
            toggle_expand(state, item);
        }
    }
}

fn toggle_expand(state: &mut AppState, item: &ui::TreeItem) {
    let mut path = vec![state.root.name.clone()];
    path.extend(item.path.clone());
    let key = path.join("::");
    if item.expanded {
        state.expanded.remove(&key);
    } else {
        state.expanded.insert(key);
    }
}

fn collapse_selected(state: &mut AppState) {
    let items = ui::tree_items(state);
    if let Some(item) = items.iter().find(|item| item.path == state.selected_path) {
        if item.has_children && item.expanded {
            toggle_expand(state, item);
        }
    }
}

fn expand_selected(state: &mut AppState) {
    let items = ui::tree_items(state);
    if let Some(item) = items.iter().find(|item| item.path == state.selected_path) {
        if item.has_children && !item.expanded {
            toggle_expand(state, item);
        }
    }
}

fn activate_form_field(state: &mut AppState) {
    if matches!(state.active_tab, ActiveTab::Help) {
        return;
    }
    let args = state.visible_args();
    if args.is_empty() {
        return;
    }
    let Some((_, arg)) = args
        .iter()
        .find(|(idx, _)| *idx == state.selected_arg_index)
    else {
        return;
    };
    let arg_id = arg.id.clone();
    let arg_kind = arg.kind;
    let enum_len = arg.possible_values.len();
    match arg_kind {
        ArgKind::Flag => {
            state.toggle_flag(&arg_id);
            state.mark_touched(&arg_id);
        }
        ArgKind::Enum => {
            if enum_len > 0 {
                if state.enum_open.as_deref() == Some(&arg_id) {
                    state.enum_open = None;
                } else {
                    state.enum_open = Some(arg_id);
                }
            }
        }
        _ => {
            state.focus = Focus::Form;
        }
    }
}

fn handle_mouse_event(
    event: event::MouseEvent,
    state: &mut AppState,
    _config: &TuiConfig,
) -> Option<Action> {
    let layout = state.layout.clone();
    if let MouseEventKind::Moved = event.kind {
        if update_mouse_selection(state, event) {
            return None;
        }
        update_hover(state, event.column, event.row);
    }
    if let MouseEventKind::Drag(MouseButton::Left) = event.kind {
        update_mouse_selection(state, event);
        return None;
    }
    if let MouseEventKind::Up(MouseButton::Left) = event.kind {
        state.mouse_select = None;
    }
    if let MouseEventKind::Down(MouseButton::Left) = event.kind {
        if let Some(footer_area) = layout.footer {
            if contains(footer_area, event.column, event.row) {
                return handle_footer_click(event, state, footer_area);
            }
        }
        if let Some(search_area) = layout.search {
            if contains(search_area, event.column, event.row) {
                state.focus = Focus::Search;
                return None;
            }
        }
        if let Some(sidebar_area) = layout.sidebar {
            if contains(sidebar_area, event.column, event.row) {
                handle_sidebar_click(event, state, sidebar_area);
                return None;
            }
        }
        if !state.layout.form_tabs.is_empty() {
            if handle_tabs_click(event, state) {
                return None;
            }
        }
        if let Some(form_area) = layout.form {
            if contains(form_area, event.column, event.row) {
                handle_form_click(event, state, form_area);
                return None;
            }
        }
    }
    if let Some(dropdown) = state.layout.dropdown {
        if contains(dropdown, event.column, event.row) {
            if let Some(_active) = state.enum_open.clone() {
                match event.kind {
                    MouseEventKind::ScrollDown => {
                        scroll_enum(state, 1);
                        return None;
                    }
                    MouseEventKind::ScrollUp => {
                        scroll_enum(state, -1);
                        return None;
                    }
                    _ => {}
                }
            }
        }
    }
    if let MouseEventKind::ScrollDown = event.kind {
        scroll_form(state, 2);
    }
    if let MouseEventKind::ScrollUp = event.kind {
        scroll_form(state, -2);
    }
    None
}

fn update_mouse_selection(state: &mut AppState, event: event::MouseEvent) -> bool {
    let Some(mut selection) = state.mouse_select.take() else {
        return false;
    };
    if let Some((row, col)) = input_position_from_event(state, &selection.arg_id, event, true) {
        let textarea = ensure_textarea_for_displayed(state, &selection.arg_id);
        if !selection.active {
            textarea.move_cursor(CursorMove::Jump(selection.anchor_row, selection.anchor_col));
            textarea.start_selection();
            selection.active = true;
        }
        textarea.move_cursor(CursorMove::Jump(row, col));
    }
    state.mouse_select = Some(selection);
    true
}

fn handle_sidebar_click(
    event: event::MouseEvent,
    state: &mut AppState,
    _area: ratatui::layout::Rect,
) {
    let Some((path, caret, has_children)) = state
        .layout
        .sidebar_items
        .iter()
        .find(|item| contains(item.row, event.column, event.row))
        .map(|item| (item.path.clone(), item.caret, item.has_children))
    else {
        return;
    };

    if state.selected_path != path {
        state.selected_path = path.clone();
        state.focus_first_tab();
    }
    if let Some(caret) = caret {
        if contains(caret, event.column, event.row) && has_children {
            let tree_items = ui::tree_items(state);
            if let Some(found) = tree_items.iter().find(|i| i.path == path) {
                toggle_expand(state, found);
            }
        }
    }
    state.focus = Focus::Sidebar;
}

fn handle_form_click(event: event::MouseEvent, state: &mut AppState, _area: ratatui::layout::Rect) {
    if matches!(state.active_tab, ActiveTab::Help) {
        return;
    }
    if let Some(dropdown) = state.layout.dropdown {
        if contains(dropdown, event.column, event.row) {
            if let Some(active) = state.enum_open.clone() {
                handle_dropdown_click(event, state, dropdown, &active);
            }
            return;
        }
    }
    let Some(form_view) = state.layout.form_view else {
        return;
    };
    if event.row < form_view.y || event.row >= form_view.y + form_view.height {
        return;
    }
    let content_y = event
        .row
        .saturating_sub(form_view.y)
        .saturating_add(state.form_scroll);
    if let Some(hit) = hit_test_form_content(state, content_y) {
        let (order_index, kind, arg_id, in_input, in_label) = hit;
        state.selected_arg_index = order_index;
        state.focus = Focus::Form;
        if kind == ArgKind::Flag && (in_input || in_label) {
            state.toggle_flag(&arg_id);
            state.mark_touched(&arg_id);
        }
        if kind == ArgKind::Enum && in_input {
            state.enum_open = Some(arg_id.clone());
            state.enum_scroll = 0;
        }
        if matches!(kind, ArgKind::Option | ArgKind::Positional) {
            let textarea = ensure_textarea_for_displayed(state, &arg_id);
            textarea.cancel_selection();
            state.mouse_select = None;
            if let Some((row, col)) = input_position_from_event(state, &arg_id, event, false) {
                state.mouse_select = Some(MouseSelection {
                    arg_id: arg_id.clone(),
                    anchor_row: row,
                    anchor_col: col,
                    active: false,
                });
            }
            set_textarea_cursor_from_click(state, &arg_id, event, in_label);
        }
        ensure_form_visible(state);
    }
}

fn set_textarea_cursor_from_click(
    state: &mut AppState,
    arg_id: &str,
    event: event::MouseEvent,
    in_label: bool,
) {
    if let Some((row, col)) = input_position_from_event(state, arg_id, event, false) {
        set_textarea_cursor(state, arg_id, row, col);
    } else if in_label {
        set_textarea_cursor(state, arg_id, 0, 0);
    }
}

fn set_textarea_cursor(state: &mut AppState, arg_id: &str, row: u16, col: u16) {
    let default_value = state
        .current_command()
        .args
        .iter()
        .find(|a| a.id == arg_id)
        .and_then(|a| a.default.clone());
    if default_value.is_some() && !state.is_touched(arg_id) {
        let textarea = state.textarea_for(arg_id, default_value.as_deref().unwrap_or(""));
        textarea.move_cursor(CursorMove::Jump(0, 0));
        return;
    }
    let textarea = ensure_textarea_for_displayed(state, arg_id);
    textarea.move_cursor(CursorMove::Jump(row, col));
}

fn displayed_text_for_arg(state: &AppState, arg_id: &str) -> String {
    if let Some(inputs) = state.current_inputs() {
        if let Some(ArgValue::Text(text)) = inputs.values.get(arg_id) {
            return text.clone();
        }
    }
    let default_value = state
        .current_command()
        .args
        .iter()
        .find(|a| a.id == arg_id)
        .and_then(|a| a.default.clone());
    if default_value.is_some() && !state.is_touched(arg_id) {
        return default_value.unwrap_or_default();
    }
    String::new()
}

fn ensure_textarea_for_displayed<'a>(
    state: &'a mut AppState,
    arg_id: &str,
) -> &'a mut TextArea<'static> {
    let displayed = displayed_text_for_arg(state, arg_id);
    let textarea = state.textarea_for(arg_id, &displayed);
    if textarea.lines().join("\n") != displayed {
        *textarea = TextArea::new(vec![displayed]);
    }
    textarea
}

fn input_position_from_event(
    state: &AppState,
    arg_id: &str,
    event: event::MouseEvent,
    clamp: bool,
) -> Option<(u16, u16)> {
    let input_rect = state.layout.form_inputs.get(arg_id).copied()?;
    let inner_x = input_rect.x.saturating_add(1);
    let inner_y = input_rect.y.saturating_add(1);
    let inner_w = input_rect.width.saturating_sub(2);
    let inner_h = input_rect.height.saturating_sub(2);
    if inner_w == 0 || inner_h == 0 {
        return None;
    }
    if !clamp
        && (event.column < inner_x
            || event.row < inner_y
            || event.column >= inner_x + inner_w
            || event.row >= inner_y + inner_h)
    {
        return None;
    }
    let x = if clamp {
        event.column.clamp(inner_x, inner_x + inner_w - 1)
    } else {
        event.column
    };
    let y = if clamp {
        event.row.clamp(inner_y, inner_y + inner_h - 1)
    } else {
        event.row
    };
    let col = x.saturating_sub(inner_x).min(inner_w.saturating_sub(1));
    let row = y.saturating_sub(inner_y).min(inner_h.saturating_sub(1));
    Some((row, col))
}

fn handle_dropdown_click(
    event: event::MouseEvent,
    state: &mut AppState,
    area: ratatui::layout::Rect,
    arg_id: &str,
) {
    let Some(arg) = state.current_command().args.iter().find(|a| a.id == arg_id) else {
        return;
    };
    let values = arg.possible_values.clone();
    let id = arg.id.clone();
    if event.row <= area.y || event.row >= area.y + area.height - 1 {
        return;
    }
    let index = event.row.saturating_sub(area.y + 1) as usize + state.enum_scroll;
    if index < values.len() {
        let inputs = state.current_inputs_mut();
        inputs.values.insert(id.clone(), ArgValue::Enum(index));
        state.enum_open = None;
        state.mark_touched(&id);
    }
}

fn handle_tabs_click(event: event::MouseEvent, state: &mut AppState) -> bool {
    for tab in &state.layout.form_tabs {
        if contains(tab.rect, event.column, event.row) {
            switch_tab(state, tab.tab);
            return true;
        }
    }
    false
}

fn handle_footer_click(
    event: event::MouseEvent,
    state: &mut AppState,
    _area: ratatui::layout::Rect,
) -> Option<Action> {
    for btn in &state.layout.footer_buttons {
        if contains(btn.rect, event.column, event.row) {
            match btn.target {
                crate::input::HoverTarget::Run => return Some(Action::Run(build_argv(state))),
                crate::input::HoverTarget::Exit => return Some(Action::Exit),
                crate::input::HoverTarget::Search => {
                    state.focus = Focus::Search;
                    return None;
                }
                crate::input::HoverTarget::Focus => {
                    state.focus = Focus::Sidebar;
                    return None;
                }
            }
        }
    }
    None
}

fn update_hover(state: &mut AppState, x: u16, y: u16) {
    let target = state
        .layout
        .footer_buttons
        .iter()
        .find(|btn| contains(btn.rect, x, y))
        .map(|btn| btn.target);
    state.hover = target;

    let tab = state
        .layout
        .form_tabs
        .iter()
        .find(|tab| contains(tab.rect, x, y))
        .map(|tab| tab.tab);
    state.hover_tab = tab;
}

fn contains(area: ratatui::layout::Rect, x: u16, y: u16) -> bool {
    x >= area.x && x < area.x + area.width && y >= area.y && y < area.y + area.height
}

fn build_argv(state: &AppState) -> Vec<String> {
    let mut argv = Vec::new();
    argv.push(state.root.name.clone());
    argv.extend(state.selected_path.iter().cloned());

    let inputs = state.current_inputs();
    let args = state.current_command().args.iter();
    let mut positionals: Vec<(usize, usize, String)> = Vec::new();
    let mut pos_seq: usize = 0;

    for arg in args {
        let is_touched = state.is_touched(&arg.id);
        if arg.default.is_some() && !is_touched {
            continue;
        }
        match inputs.and_then(|i| i.values.get(&arg.id)) {
            Some(ArgValue::Bool(true)) => {
                argv.push(arg.name.clone());
            }
            Some(ArgValue::Text(value)) if !value.is_empty() => {
                if arg.kind == ArgKind::Positional {
                    if let Some(idx) = arg.positional_index {
                        if arg.is_multi {
                            for part in value.lines().filter(|s| !s.trim().is_empty()) {
                                positionals.push((idx, pos_seq, part.to_string()));
                                pos_seq += 1;
                            }
                        } else {
                            positionals.push((idx, pos_seq, value.clone()));
                            pos_seq += 1;
                        }
                    }
                } else {
                    if arg.is_multi {
                        for part in value.lines().filter(|s| !s.trim().is_empty()) {
                            argv.push(arg.name.clone());
                            argv.push(part.to_string());
                        }
                    } else {
                        argv.push(arg.name.clone());
                        argv.push(value.clone());
                    }
                }
            }
            Some(ArgValue::Enum(idx)) => {
                if let Some(val) = arg.possible_values.get(*idx) {
                    argv.push(arg.name.clone());
                    argv.push(val.clone());
                }
            }
            _ => {}
        }
    }

    positionals.sort_by_key(|(idx, seq, _)| (*idx, *seq));
    for (_, _, value) in positionals {
        argv.push(value);
    }

    argv
}

fn ensure_form_visible(state: &mut AppState) {
    if matches!(state.active_tab, ActiveTab::Help) {
        return;
    }
    let Some(form) = state.layout.form_view else {
        return;
    };
    let view_height = form.height;
    let Some((input_top, input_bottom)) = field_content_bounds(state, state.selected_arg_index)
    else {
        return;
    };

    let visible_top = state.form_scroll;
    let visible_bottom = state.form_scroll.saturating_add(view_height);

    if input_top < visible_top {
        state.form_scroll = input_top;
    } else if input_bottom > visible_bottom {
        let delta = input_bottom.saturating_sub(visible_bottom);
        state.form_scroll = state.form_scroll.saturating_add(delta);
    }
    state.form_scroll = state.form_scroll.min(state.form_scroll_max);
}

fn scroll_form(state: &mut AppState, delta: i16) {
    if delta.is_negative() {
        let delta = (-delta) as u16;
        state.form_scroll = state.form_scroll.saturating_sub(delta);
    } else {
        state.form_scroll = state.form_scroll.saturating_add(delta as u16);
    }
    state.form_scroll = state.form_scroll.min(state.form_scroll_max);
}

fn field_content_bounds(state: &AppState, target_index: usize) -> Option<(u16, u16)> {
    let args = state.visible_args();
    let mut y: u16 = 0;
    for (order_index, arg) in args {
        let input_height = if arg.is_multi { 5 } else { 3 };
        let label_y = y;
        let input_top = label_y.saturating_add(1);
        let input_bottom = input_top.saturating_add(input_height);
        if order_index == target_index {
            return Some((input_top, input_bottom));
        }
        y = y.saturating_add(1 + input_height + 1); // label + input + gap
        let has_help = arg.help.is_some() || arg.value_hint.is_some();
        if has_help {
            y = y.saturating_add(1);
        }
    }
    None
}

fn hit_test_form_content(
    state: &AppState,
    content_y: u16,
) -> Option<(usize, ArgKind, String, bool, bool)> {
    let args = state.visible_args();
    let mut y: u16 = 0;
    for (order_index, arg) in args {
        let input_height = if arg.is_multi { 5 } else { 3 };
        let label_y = y;
        let input_top = label_y.saturating_add(1);
        let input_bottom = input_top.saturating_add(input_height);
        let has_help = arg.help.is_some() || arg.value_hint.is_some();
        let help_y = input_bottom.saturating_add(1);

        let in_label = content_y == label_y;
        let in_input = content_y >= input_top && content_y < input_bottom;
        let in_help = has_help && content_y == help_y;

        if in_label || in_input || in_help {
            return Some((order_index, arg.kind, arg.id.clone(), in_input, in_label));
        }
        y = y.saturating_add(1 + input_height + 1);
        if has_help {
            y = y.saturating_add(1);
        }
    }
    None
}

fn scroll_enum(state: &mut AppState, delta: i16) {
    if delta.is_negative() {
        let delta = (-delta) as usize;
        state.enum_scroll = state.enum_scroll.saturating_sub(delta);
    } else {
        state.enum_scroll = state.enum_scroll.saturating_add(delta as usize);
    }
}

fn ensure_enum_visible(state: &mut AppState, index: usize, total: usize) {
    let Some(dropdown) = state.layout.dropdown else {
        return;
    };
    let visible = dropdown.height.saturating_sub(2) as usize;
    if visible == 0 {
        return;
    }
    let max_scroll = total.saturating_sub(visible);
    if index < state.enum_scroll {
        state.enum_scroll = index;
    } else if index >= state.enum_scroll + visible {
        state.enum_scroll = index.saturating_sub(visible - 1);
    }
    state.enum_scroll = state.enum_scroll.min(max_scroll);
}

fn apply_start_command(state: &mut AppState, start: &str) {
    let path = if start.contains("::") {
        start.split("::").map(|s| s.to_string()).collect::<Vec<_>>()
    } else {
        start
            .split_whitespace()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
    };
    if !path.is_empty() {
        state.selected_path = path;
        state.focus_first_tab();
        let mut prefix = vec![state.root.name.clone()];
        state.expanded.insert(prefix.join("::"));
        for part in &state.selected_path {
            prefix.push(part.clone());
            state.expanded.insert(prefix.join("::"));
        }
    }
}
