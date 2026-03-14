use crate::input::{ActiveTab, AppState, Focus};
use crate::view::command_tree;
use crate::view::form;

pub(crate) fn switch_tab(state: &mut AppState, tab: ActiveTab) {
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

pub(crate) fn cycle_tabs(state: &mut AppState) {
    let tabs = state.visible_tabs();
    if tabs.len() <= 1 {
        return;
    }
    let current = tabs
        .iter()
        .position(|tab| *tab == state.active_tab)
        .unwrap_or(0);
    let next = (current + 1) % tabs.len();
    switch_tab(state, tabs[next]);
}

pub(crate) fn toggle_help_tab(state: &mut AppState) {
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

pub(crate) fn move_sidebar_selection(state: &mut AppState, delta: isize) {
    let items = command_tree::tree_items(&state.root, &state.expanded, &state.search);
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

pub(crate) fn move_form_selection(state: &mut AppState, delta: isize) {
    if matches!(state.active_tab, ActiveTab::Help) {
        return;
    }
    let command = state.current_command().clone();
    let args = form::visible_args(&command, state.active_tab);
    if args.is_empty() {
        return;
    }
    let current_pos = args
        .iter()
        .position(|item| item.order_index == state.selected_arg_index)
        .unwrap_or(0) as isize;
    let max = args.len() as isize - 1;
    let next_pos = (current_pos + delta).clamp(0, max) as usize;
    state.selected_arg_index = args[next_pos].order_index;
    ensure_form_visible(state);
}

pub(crate) fn select_sidebar(state: &mut AppState) {
    let items = command_tree::tree_items(&state.root, &state.expanded, &state.search);
    if let Some(item) = items.iter().find(|item| item.path == state.selected_path) {
        if item.has_children {
            toggle_expand(state, &item.path, item.expanded);
        }
    }
}

pub(crate) fn collapse_selected(state: &mut AppState) {
    let items = command_tree::tree_items(&state.root, &state.expanded, &state.search);
    if let Some(item) = items.iter().find(|item| item.path == state.selected_path) {
        if item.has_children && item.expanded {
            toggle_expand(state, &item.path, true);
        }
    }
}

pub(crate) fn expand_selected(state: &mut AppState) {
    let items = command_tree::tree_items(&state.root, &state.expanded, &state.search);
    if let Some(item) = items.iter().find(|item| item.path == state.selected_path) {
        if item.has_children && !item.expanded {
            toggle_expand(state, &item.path, false);
        }
    }
}

pub(crate) fn activate_form_field(state: &mut AppState) {
    if matches!(state.active_tab, ActiveTab::Help) {
        return;
    }
    let command = state.current_command().clone();
    let args = form::visible_args(&command, state.active_tab);
    if args.is_empty() {
        return;
    }
    let Some(item) = args
        .iter()
        .find(|item| item.order_index == state.selected_arg_index)
    else {
        return;
    };
    let arg_id = item.arg.id.clone();
    let arg_kind = item.arg.kind;
    let enum_len = item.arg.possible_values.len();
    match arg_kind {
        crate::spec::ArgKind::Flag => {
            state.toggle_flag(&arg_id);
            state.mark_touched(&arg_id);
        }
        crate::spec::ArgKind::Enum => {
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

pub(crate) fn ensure_form_visible(state: &mut AppState) {
    if matches!(state.active_tab, ActiveTab::Help) {
        return;
    }
    let Some(form_area) = state.layout.form_view else {
        return;
    };
    let command = state.current_command().clone();
    let args = form::visible_args(&command, state.active_tab);
    let Some((input_top, input_bottom)) =
        form::field_content_bounds(&args, state.selected_arg_index)
    else {
        return;
    };
    let visible_top = state.form_scroll;
    let visible_bottom = state.form_scroll.saturating_add(form_area.height);

    if input_top < visible_top {
        state.form_scroll = input_top;
    } else if input_bottom > visible_bottom {
        let delta = input_bottom.saturating_sub(visible_bottom);
        state.form_scroll = state.form_scroll.saturating_add(delta);
    }
    state.form_scroll = state.form_scroll.min(state.form_scroll_max);
}

pub(crate) fn scroll_form(state: &mut AppState, delta: i16) {
    if delta.is_negative() {
        state.form_scroll = state.form_scroll.saturating_sub((-delta) as u16);
    } else {
        state.form_scroll = state.form_scroll.saturating_add(delta as u16);
    }
    state.form_scroll = state.form_scroll.min(state.form_scroll_max);
}

pub(crate) fn scroll_enum(state: &mut AppState, delta: i16) {
    if delta.is_negative() {
        state.enum_scroll = state.enum_scroll.saturating_sub((-delta) as usize);
    } else {
        state.enum_scroll = state.enum_scroll.saturating_add(delta as usize);
    }
}

pub(crate) fn ensure_enum_visible(state: &mut AppState, index: usize, total: usize) {
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

pub(crate) fn apply_start_command(state: &mut AppState, start: &str) {
    let path = if start.contains("::") {
        start.split("::").map(str::to_string).collect::<Vec<_>>()
    } else {
        start
            .split_whitespace()
            .map(str::to_string)
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

fn toggle_expand(state: &mut AppState, path: &[String], expanded: bool) {
    let mut full_path = vec![state.root.name.clone()];
    full_path.extend(path.iter().cloned());
    let key = full_path.join("::");
    if expanded {
        state.expanded.remove(&key);
    } else {
        state.expanded.insert(key);
    }
}

#[cfg(test)]
mod tests {
    use ratatui::layout::Rect;

    use super::{ensure_form_visible, move_sidebar_selection, switch_tab, toggle_help_tab};
    use crate::input::{ActiveTab, AppState};
    use crate::spec::{ArgKind, ArgSpec, CommandSpec};

    fn arg(id: &str, name: &str, kind: ArgKind) -> ArgSpec {
        ArgSpec {
            id: id.to_string(),
            name: name.to_string(),
            help: None,
            required: false,
            kind,
            default: None,
            possible_values: Vec::new(),
            positional_index: None,
            is_multi: false,
            value_hint: None,
        }
    }

    fn command(name: &str, args: Vec<ArgSpec>, subcommands: Vec<CommandSpec>) -> CommandSpec {
        CommandSpec {
            name: name.to_string(),
            about: None,
            help: String::new(),
            args,
            subcommands,
        }
    }

    #[test]
    fn moving_sidebar_selection_updates_selected_path() {
        let root = command(
            "tool",
            Vec::new(),
            vec![command("build", Vec::new(), Vec::new())],
        );
        let mut state = AppState::new(root);

        move_sidebar_selection(&mut state, 1);

        assert_eq!(state.selected_path, vec!["build".to_string()]);
    }

    #[test]
    fn tab_switching_preserves_last_non_help_tab() {
        let root = command("tool", Vec::new(), Vec::new());
        let mut state = AppState::new(root);

        switch_tab(&mut state, ActiveTab::Arguments);
        toggle_help_tab(&mut state);
        toggle_help_tab(&mut state);

        assert_eq!(state.active_tab, ActiveTab::Arguments);
    }

    #[test]
    fn ensure_form_visible_scrolls_selected_field_into_view() {
        let option_one = arg("a", "--a", ArgKind::Option);
        let mut option_two = arg("b", "--b", ArgKind::Option);
        option_two.help = Some("help".to_string());
        let root = command("tool", vec![option_one, option_two], Vec::new());
        let mut state = AppState::new(root);
        state.active_tab = ActiveTab::Options;
        state.selected_arg_index = 1;
        state.layout.form_view = Some(Rect::new(0, 0, 10, 4));
        state.form_scroll_max = 20;

        ensure_form_visible(&mut state);

        assert!(state.form_scroll > 0);
    }
}
