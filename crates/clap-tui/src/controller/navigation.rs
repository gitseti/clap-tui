use crate::input::{ActiveTab, AppState, ArgValue, Focus};
use crate::ui::dropdown::{MAX_DROPDOWN_ROWS, dropdown_layout};
use crate::view::command_tree;
use crate::view::form;

pub(crate) fn switch_tab(state: &mut AppState, tab: ActiveTab) {
    let tabs = AppState::visible_tabs();
    let target = tabs
        .into_iter()
        .find(|candidate| *candidate == tab)
        .unwrap_or(ActiveTab::Options);
    if target == state.command.active_tab {
        return;
    }
    state.command.active_tab = target;
    if state.command.active_tab != ActiveTab::Help {
        state.command.last_non_help_tab = state.command.active_tab;
        let command = state.current_command().clone();
        let args = form::visible_args(&command, state.command.active_tab);
        state.ensure_selected_arg_visible(&visible_args(&args));
    }
    state.interaction.form_scroll = 0;
    state.interaction.enum_open = None;
    state.interaction.mouse_select = None;
}

pub(crate) fn cycle_tabs(state: &mut AppState) {
    let tabs = AppState::visible_tabs();
    let current = tabs
        .iter()
        .position(|tab| *tab == state.command.active_tab)
        .unwrap_or(0);
    let next = (current + 1) % tabs.len();
    switch_tab(state, tabs[next]);
}

pub(crate) fn toggle_help_tab(state: &mut AppState) {
    if state.command.active_tab == ActiveTab::Help {
        let tabs = AppState::visible_tabs();
        let mut target = state.command.last_non_help_tab;
        if !tabs.contains(&target) {
            target = tabs[0];
        }
        switch_tab(state, target);
    } else {
        state.command.last_non_help_tab = state.command.active_tab;
        switch_tab(state, ActiveTab::Help);
    }
}

pub(crate) fn move_sidebar_selection(state: &mut AppState, delta: isize) {
    let items = command_tree::tree_items(
        &state.command.root,
        &state.command.expanded,
        &state.command.search,
    );
    if items.is_empty() {
        return;
    }
    let current_index = items
        .iter()
        .position(|item| item.path == state.command.selected_path)
        .unwrap_or(0);
    let next_index = current_index
        .saturating_add_signed(delta)
        .min(items.len() - 1);
    if state.command.selected_path != items[next_index].path {
        state
            .command
            .selected_path
            .clone_from(&items[next_index].path);
        let command = state.current_command().clone();
        let args = form::visible_args(&command, state.command.active_tab);
        state.focus_first_tab(&visible_args(&args));
    }
}

pub(crate) fn move_form_selection(state: &mut AppState, delta: isize) {
    if matches!(state.command.active_tab, ActiveTab::Help) {
        return;
    }
    let command = state.current_command().clone();
    let args = form::visible_args(&command, state.command.active_tab);
    if args.is_empty() {
        return;
    }
    let current_pos = args
        .iter()
        .position(|item| item.order_index == state.command.selected_arg_index)
        .unwrap_or(0);
    let next_pos = current_pos.saturating_add_signed(delta).min(args.len() - 1);
    state.command.selected_arg_index = args[next_pos].order_index;
    ensure_form_visible(state);
}

pub(crate) fn select_sidebar(state: &mut AppState) {
    let items = command_tree::tree_items(
        &state.command.root,
        &state.command.expanded,
        &state.command.search,
    );
    if let Some(item) = items
        .iter()
        .find(|item| item.path == state.command.selected_path)
    {
        if item.has_children {
            toggle_expand(state, &item.path, item.expanded);
        }
    }
}

pub(crate) fn collapse_selected(state: &mut AppState) {
    let items = command_tree::tree_items(
        &state.command.root,
        &state.command.expanded,
        &state.command.search,
    );
    if let Some(item) = items
        .iter()
        .find(|item| item.path == state.command.selected_path)
    {
        if item.has_children && item.expanded {
            toggle_expand(state, &item.path, true);
        }
    }
}

pub(crate) fn expand_selected(state: &mut AppState) {
    let items = command_tree::tree_items(
        &state.command.root,
        &state.command.expanded,
        &state.command.search,
    );
    if let Some(item) = items
        .iter()
        .find(|item| item.path == state.command.selected_path)
    {
        if item.has_children && !item.expanded {
            toggle_expand(state, &item.path, false);
        }
    }
}

pub(crate) fn activate_form_field(state: &mut AppState) {
    if matches!(state.command.active_tab, ActiveTab::Help) {
        return;
    }
    let command = state.current_command().clone();
    let args = form::visible_args(&command, state.command.active_tab);
    let Some(item) = args
        .iter()
        .find(|item| item.order_index == state.command.selected_arg_index)
    else {
        return;
    };
    let arg = item.arg;
    if arg.is_flag() {
        state.toggle_flag(&arg.id);
        state.mark_touched(&arg.id);
    } else if arg.uses_choice_input() {
        if arg.possible_values.is_empty()
            || state.interaction.enum_open.as_deref() == Some(arg.id.as_str())
        {
            state.interaction.enum_open = None;
        } else {
            open_enum_dropdown(state, &arg.id, arg.possible_values.len());
        }
    } else {
        state.interaction.focus = Focus::Form;
    }
}

pub(crate) fn open_enum_dropdown(state: &mut AppState, arg_id: &str, total: usize) {
    if total == 0 {
        state.interaction.enum_open = None;
        state.interaction.enum_scroll = 0;
        return;
    }

    state.interaction.enum_open = Some(arg_id.to_string());
    let current = state
        .current_inputs()
        .and_then(|inputs| inputs.values.get(arg_id))
        .and_then(|value| match value {
            ArgValue::Enum(index) => Some(*index),
            _ => None,
        })
        .unwrap_or(0);
    let visible_rows = state
        .layout
        .form_view
        .zip(state.layout.form_inputs.get(arg_id).copied())
        .and_then(|(form_view, input_rect)| dropdown_layout(form_view, input_rect, total))
        .map_or(total.min(usize::from(MAX_DROPDOWN_ROWS)), |layout| {
            layout.visible_rows
        });
    let max_scroll = total.saturating_sub(visible_rows);
    let centered_scroll = current.saturating_sub(visible_rows / 2);
    state.interaction.enum_scroll = centered_scroll.min(max_scroll);
}

pub(crate) fn ensure_form_visible(state: &mut AppState) {
    if matches!(state.command.active_tab, ActiveTab::Help) {
        return;
    }
    let Some(form_area) = state.layout.form_view else {
        return;
    };
    let command = state.current_command().clone();
    let args = form::visible_args(&command, state.command.active_tab);
    let Some((input_top, input_bottom)) =
        form::field_content_bounds(&args, state.command.selected_arg_index)
    else {
        return;
    };
    let visible_top = state.interaction.form_scroll;
    let visible_bottom = state
        .interaction
        .form_scroll
        .saturating_add(form_area.height);

    if input_top < visible_top {
        state.interaction.form_scroll = input_top;
    } else if input_bottom > visible_bottom {
        let delta = input_bottom.saturating_sub(visible_bottom);
        state.interaction.form_scroll = state.interaction.form_scroll.saturating_add(delta);
    }
    state.interaction.form_scroll = state
        .interaction
        .form_scroll
        .min(state.interaction.form_scroll_max);
}

pub(crate) fn scroll_form(state: &mut AppState, delta: i16) {
    if delta.is_negative() {
        state.interaction.form_scroll = state
            .interaction
            .form_scroll
            .saturating_sub(delta.unsigned_abs());
    } else {
        state.interaction.form_scroll = state
            .interaction
            .form_scroll
            .saturating_add(delta.unsigned_abs());
    }
    state.interaction.form_scroll = state
        .interaction
        .form_scroll
        .min(state.interaction.form_scroll_max);
}

pub(crate) fn scroll_enum(state: &mut AppState, delta: i16) {
    if delta.is_negative() {
        state.interaction.enum_scroll = state
            .interaction
            .enum_scroll
            .saturating_sub(usize::from(delta.unsigned_abs()));
    } else {
        state.interaction.enum_scroll = state
            .interaction
            .enum_scroll
            .saturating_add(usize::from(delta.unsigned_abs()));
    }
}

pub(crate) fn ensure_enum_visible(state: &mut AppState, index: usize, total: usize) {
    let Some(dropdown) = state.layout.dropdown else {
        return;
    };
    let visible = usize::from(dropdown.height.saturating_sub(2));
    if visible == 0 {
        return;
    }
    let max_scroll = total.saturating_sub(visible);
    if index < state.interaction.enum_scroll {
        state.interaction.enum_scroll = index;
    } else if index >= state.interaction.enum_scroll + visible {
        state.interaction.enum_scroll = index.saturating_sub(visible - 1);
    }
    state.interaction.enum_scroll = state.interaction.enum_scroll.min(max_scroll);
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
    if path.is_empty() {
        return;
    }

    state.command.selected_path = path;
    let command = state.current_command().clone();
    let args = form::visible_args(&command, state.command.active_tab);
    state.focus_first_tab(&visible_args(&args));

    let mut prefix = vec![state.command.root.name.clone()];
    state.command.expanded.insert(prefix.join("::"));
    for part in &state.command.selected_path {
        prefix.push(part.clone());
        state.command.expanded.insert(prefix.join("::"));
    }
}

fn toggle_expand(state: &mut AppState, path: &[String], expanded: bool) {
    let mut full_path = vec![state.command.root.name.clone()];
    full_path.extend(path.iter().cloned());
    let key = full_path.join("::");
    if expanded {
        state.command.expanded.remove(&key);
    } else {
        state.command.expanded.insert(key);
    }
}

fn visible_args<'a>(args: &[form::OrderedArg<'a>]) -> Vec<(usize, &'a crate::spec::ArgSpec)> {
    args.iter()
        .map(|item| (item.order_index, item.arg))
        .collect()
}

#[cfg(test)]
mod tests {
    use ratatui::layout::Rect;

    use super::{
        ensure_form_visible, move_sidebar_selection, open_enum_dropdown, switch_tab,
        toggle_help_tab,
    };
    use crate::input::{ActiveTab, AppState, ArgValue, Focus};
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

        assert_eq!(state.command.selected_path, vec!["build".to_string()]);
    }

    #[test]
    fn tab_switching_preserves_last_non_help_tab() {
        let root = command("tool", Vec::new(), Vec::new());
        let mut state = AppState::new(root);
        state.command.active_tab = ActiveTab::Arguments;

        toggle_help_tab(&mut state);
        assert_eq!(state.command.active_tab, ActiveTab::Help);

        toggle_help_tab(&mut state);
        assert_eq!(state.command.active_tab, ActiveTab::Arguments);
    }

    #[test]
    fn ensure_form_visible_scrolls_selected_field_into_view() {
        let option_one = arg("a", "--a", ArgKind::Option);
        let mut option_two = arg("b", "--b", ArgKind::Option);
        option_two.help = Some("second".to_string());
        let root = command("tool", vec![option_one, option_two], Vec::new());
        let mut state = AppState::new(root);
        state.layout.form_view = Some(Rect::new(0, 0, 30, 4));
        state.interaction.form_scroll_max = 10;
        state.command.active_tab = ActiveTab::Options;
        state.command.selected_arg_index = 1;

        ensure_form_visible(&mut state);

        assert!(state.interaction.form_scroll > 0);
    }

    #[test]
    fn opening_dropdown_centers_current_value_when_possible() {
        let mut color = arg("color", "--color", ArgKind::Enum);
        color.possible_values = vec![
            "red".to_string(),
            "green".to_string(),
            "blue".to_string(),
            "yellow".to_string(),
            "purple".to_string(),
            "orange".to_string(),
        ];
        let root = command("tool", vec![color], Vec::new());
        let mut state = AppState::new(root);
        state.layout.form_view = Some(Rect::new(0, 0, 40, 10));
        state
            .layout
            .form_inputs
            .insert("color".to_string(), Rect::new(0, 2, 20, 3));
        state
            .current_inputs_mut()
            .values
            .insert("color".to_string(), ArgValue::Enum(3));

        open_enum_dropdown(&mut state, "color", 6);

        assert_eq!(state.interaction.enum_open.as_deref(), Some("color"));
        assert_eq!(state.interaction.enum_scroll, 2);
    }

    #[test]
    fn opening_dropdown_clamps_scroll_near_edges() {
        let mut color = arg("color", "--color", ArgKind::Enum);
        color.possible_values = vec![
            "red".to_string(),
            "green".to_string(),
            "blue".to_string(),
            "yellow".to_string(),
        ];
        let root = command("tool", vec![color], Vec::new());
        let mut state = AppState::new(root);
        state.layout.form_view = Some(Rect::new(0, 0, 40, 7));
        state
            .layout
            .form_inputs
            .insert("color".to_string(), Rect::new(0, 2, 20, 3));
        state
            .current_inputs_mut()
            .values
            .insert("color".to_string(), ArgValue::Enum(0));

        open_enum_dropdown(&mut state, "color", 4);
        assert_eq!(state.interaction.enum_scroll, 0);

        state
            .current_inputs_mut()
            .values
            .insert("color".to_string(), ArgValue::Enum(3));
        open_enum_dropdown(&mut state, "color", 4);
        assert_eq!(state.interaction.enum_scroll, 0);
    }

    #[test]
    fn activating_open_enum_toggles_it_closed() {
        let mut color = arg("color", "--color", ArgKind::Enum);
        color.possible_values = vec!["red".to_string(), "blue".to_string()];
        let root = command("tool", vec![color], Vec::new());
        let mut state = AppState::new(root);
        state.command.active_tab = ActiveTab::Options;
        state.interaction.focus = Focus::Form;
        state.command.selected_arg_index = 0;
        state.layout.form_view = Some(Rect::new(0, 0, 40, 10));
        state
            .layout
            .form_inputs
            .insert("color".to_string(), Rect::new(0, 2, 20, 3));

        super::activate_form_field(&mut state);
        assert_eq!(state.interaction.enum_open.as_deref(), Some("color"));

        super::activate_form_field(&mut state);
        assert!(state.interaction.enum_open.is_none());
    }

    #[test]
    fn switch_tab_keeps_selection_valid() {
        let mut positional = arg("path", "path", ArgKind::Positional);
        positional.positional_index = Some(1);
        let option = arg("target", "--target", ArgKind::Option);
        let root = command("tool", vec![positional, option], Vec::new());
        let mut state = AppState::new(root);
        state.command.active_tab = ActiveTab::Arguments;
        state.command.selected_arg_index = 0;

        switch_tab(&mut state, ActiveTab::Options);

        assert_eq!(state.command.active_tab, ActiveTab::Options);
        assert_eq!(state.command.selected_arg_index, 1);
    }
}
