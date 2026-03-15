use crate::frame_snapshot::FrameSnapshot;
use crate::input::{ActiveTab, AppState, ArgValue, Focus, UiState};
use crate::spec::{ArgSpec, CommandPath, SelectionError};
use crate::ui::dropdown::{MAX_DROPDOWN_ROWS, dropdown_layout};
use crate::view::command_tree;
use crate::view::form;

pub(crate) fn switch_tab(state: &mut AppState, tab: ActiveTab) {
    let tabs = UiState::visible_tabs();
    let target = tabs
        .into_iter()
        .find(|candidate| *candidate == tab)
        .unwrap_or(ActiveTab::Options);
    if target == state.ui.active_tab {
        return;
    }
    state.ui.active_tab = target;
    if state.ui.active_tab != ActiveTab::Help {
        state.ui.last_non_help_tab = state.ui.active_tab;
        let command = state.domain.current_command().clone();
        let args = form::visible_args(&command, state.ui.active_tab);
        state.ui.ensure_selected_arg_visible(&visible_args(&args));
    }
    reset_transient_form_ui(state);
}

pub(crate) fn cycle_tabs(state: &mut AppState) {
    let tabs = UiState::visible_tabs();
    let current = tabs
        .iter()
        .position(|tab| *tab == state.ui.active_tab)
        .unwrap_or(0);
    switch_tab(state, tabs[(current + 1) % tabs.len()]);
}

pub(crate) fn toggle_help_tab(state: &mut AppState) {
    if state.ui.active_tab == ActiveTab::Help {
        let tabs = UiState::visible_tabs();
        let mut target = state.ui.last_non_help_tab;
        if !tabs.contains(&target) {
            target = tabs[0];
        }
        switch_tab(state, target);
    } else {
        state.ui.last_non_help_tab = state.ui.active_tab;
        switch_tab(state, ActiveTab::Help);
    }
}

pub(crate) fn move_sidebar_selection(state: &mut AppState, delta: isize) {
    let items = command_tree::tree_items(
        &state.domain.root,
        &state.domain.expanded,
        &state.ui.search_query,
    );
    if items.is_empty() {
        return;
    }
    let current_index = items
        .iter()
        .position(|item| item.path == *state.domain.selected_path())
        .unwrap_or(0);
    let next_index = current_index
        .saturating_add_signed(delta)
        .min(items.len() - 1);
    if *state.domain.selected_path() != items[next_index].path {
        select_command(state, items[next_index].path.as_slice());
    }
}

pub(crate) fn move_form_selection(state: &mut AppState, frame_snapshot: &FrameSnapshot, delta: isize) {
    if matches!(state.ui.active_tab, ActiveTab::Help) {
        return;
    }
    let command = state.domain.current_command().clone();
    let args = form::visible_args(&command, state.ui.active_tab);
    if args.is_empty() {
        return;
    }
    let current_pos = args
        .iter()
        .position(|item| item.order_index == state.ui.selected_arg_index)
        .unwrap_or(0);
    let next_pos = current_pos.saturating_add_signed(delta).min(args.len() - 1);
    state.ui.selected_arg_index = args[next_pos].order_index;
    ensure_form_visible(state, frame_snapshot);
}

pub(crate) fn select_sidebar(state: &mut AppState) {
    let items = command_tree::tree_items(
        &state.domain.root,
        &state.domain.expanded,
        &state.ui.search_query,
    );
    if let Some(item) = items
        .iter()
        .find(|item| item.path == *state.domain.selected_path())
        .filter(|item| item.has_children)
    {
        toggle_expand(state, &item.path, item.expanded);
    }
}

pub(crate) fn collapse_selected(state: &mut AppState) {
    let items = command_tree::tree_items(
        &state.domain.root,
        &state.domain.expanded,
        &state.ui.search_query,
    );
    if let Some(item) = items
        .iter()
        .find(|item| item.path == *state.domain.selected_path())
        .filter(|item| item.has_children && item.expanded)
    {
        toggle_expand(state, &item.path, true);
    }
}

pub(crate) fn expand_selected(state: &mut AppState) {
    let items = command_tree::tree_items(
        &state.domain.root,
        &state.domain.expanded,
        &state.ui.search_query,
    );
    if let Some(item) = items
        .iter()
        .find(|item| item.path == *state.domain.selected_path())
        .filter(|item| item.has_children && !item.expanded)
    {
        toggle_expand(state, &item.path, false);
    }
}

pub(crate) fn activate_form_field(state: &mut AppState, frame_snapshot: &FrameSnapshot) {
    if matches!(state.ui.active_tab, ActiveTab::Help) {
        return;
    }
    let command = state.domain.current_command().clone();
    let args = form::visible_args(&command, state.ui.active_tab);
    let Some(item) = args
        .iter()
        .find(|item| item.order_index == state.ui.selected_arg_index)
    else {
        return;
    };
    let arg = item.arg;
    if arg.is_flag() {
        state.domain.toggle_flag(&arg.id);
        state.domain.mark_touched(&arg.id);
    } else if arg.uses_choice_input() {
        if arg.choices.is_empty() || state.ui.dropdown_open.as_deref() == Some(arg.id.as_str()) {
            state.ui.dropdown_open = None;
        } else {
            open_enum_dropdown(state, frame_snapshot, &arg.id, arg.choices.len());
        }
    } else {
        state.ui.focus = Focus::Form;
    }
}

pub(crate) fn open_enum_dropdown(
    state: &mut AppState,
    frame_snapshot: &FrameSnapshot,
    arg_id: &str,
    total: usize,
) {
    if total == 0 {
        state.ui.dropdown_open = None;
        state.ui.dropdown_scroll = 0;
        return;
    }

    state.ui.dropdown_open = Some(arg_id.to_string());
    let current = state
        .domain
        .current_form()
        .and_then(|inputs| inputs.values.get(arg_id))
        .and_then(|value| match value {
            ArgValue::Choice(selected) => state
                .domain
                .current_command()
                .args
                .iter()
                .find(|arg| arg.id == arg_id)
                .and_then(|arg| arg.choices.iter().position(|choice| choice == selected)),
            _ => None,
        })
        .unwrap_or(0);
    let visible_rows = frame_snapshot
        .layout
        .form_view
        .zip(frame_snapshot.form_input_rect(arg_id))
        .and_then(|(form_view, input_rect)| dropdown_layout(form_view, input_rect, total))
        .map_or(total.min(usize::from(MAX_DROPDOWN_ROWS)), |layout| {
            layout.visible_rows
        });
    let max_scroll = total.saturating_sub(visible_rows);
    state.ui.dropdown_scroll = current.saturating_sub(visible_rows / 2).min(max_scroll);
}

pub(crate) fn ensure_form_visible(state: &mut AppState, frame_snapshot: &FrameSnapshot) {
    if matches!(state.ui.active_tab, ActiveTab::Help) {
        return;
    }
    let Some(form_area) = frame_snapshot.form_view_rect() else {
        return;
    };
    let command = state.domain.current_command().clone();
    let args = form::visible_args(&command, state.ui.active_tab);
    let Some((input_top, input_bottom)) =
        form::field_content_bounds(&args, state.ui.selected_arg_index)
    else {
        return;
    };
    let visible_top = state.ui.form_scroll(frame_snapshot);
    let visible_bottom = visible_top.saturating_add(form_area.height);

    if input_top < visible_top {
        state.ui.form_scroll = input_top;
    } else if input_bottom > visible_bottom {
        state.ui.form_scroll = state
            .ui
            .form_scroll
            .saturating_add(input_bottom.saturating_sub(visible_bottom));
    }
    state.ui.clamp_form_scroll(frame_snapshot);
}

pub(crate) fn scroll_form(state: &mut AppState, frame_snapshot: &FrameSnapshot, delta: i16) {
    if delta.is_negative() {
        state.ui.form_scroll = state.ui.form_scroll.saturating_sub(delta.unsigned_abs());
    } else {
        state.ui.form_scroll = state.ui.form_scroll.saturating_add(delta.unsigned_abs());
    }
    state.ui.clamp_form_scroll(frame_snapshot);
}

pub(crate) fn scroll_enum(state: &mut AppState, delta: i16) {
    if delta.is_negative() {
        state.ui.dropdown_scroll = state
            .ui
            .dropdown_scroll
            .saturating_sub(usize::from(delta.unsigned_abs()));
    } else {
        state.ui.dropdown_scroll = state
            .ui
            .dropdown_scroll
            .saturating_add(usize::from(delta.unsigned_abs()));
    }
}

pub(crate) fn ensure_enum_visible(
    state: &mut AppState,
    frame_snapshot: &FrameSnapshot,
    index: usize,
    total: usize,
) {
    let Some(visible) = frame_snapshot.dropdown_visible_rows() else {
        return;
    };
    if visible == 0 {
        return;
    }
    let max_scroll = total.saturating_sub(visible);
    if index < state.ui.dropdown_scroll {
        state.ui.dropdown_scroll = index;
    } else if index >= state.ui.dropdown_scroll + visible {
        state.ui.dropdown_scroll = index.saturating_sub(visible - 1);
    }
    state.ui.dropdown_scroll = state.ui.dropdown_scroll.min(max_scroll);
}

pub(crate) fn apply_start_command(state: &mut AppState, start: &str) {
    match state.domain.select_command_by_search_path(start) {
        Ok(()) => {
            let command = state.domain.current_command().clone();
            let args = form::visible_args(&command, state.ui.active_tab);
            state.ui.focus_first_tab(&visible_args(&args));
        }
        Err(SelectionError::UnknownPath) => {
            state.notifications.show_toast(
                format!("Unknown start command: {start}"),
                std::time::Duration::from_secs(2),
                false,
            );
        }
    }
}

fn select_command(state: &mut AppState, path: &[String]) {
    if state.domain.select_command_path(path).is_ok() {
        let command = state.domain.current_command().clone();
        let args = form::visible_args(&command, state.ui.active_tab);
        state.ui.focus_first_tab(&visible_args(&args));
    }
}

fn toggle_expand(state: &mut AppState, path: &CommandPath, expanded: bool) {
    let key = path.to_key(&state.domain.root.name);
    if expanded {
        state.domain.expanded.remove(&key);
    } else {
        state.domain.expanded.insert(key);
    }
}

fn reset_transient_form_ui(state: &mut AppState) {
    state.ui.form_scroll = 0;
    state.ui.dropdown_open = None;
    state.ui.mouse_select = None;
}

fn visible_args<'a>(args: &[form::OrderedArg<'a>]) -> Vec<(usize, &'a ArgSpec)> {
    args.iter()
        .map(|item| (item.order_index, item.arg))
        .collect()
}

#[cfg(test)]
mod tests {
    use ratatui::layout::Rect;

    use crate::frame_snapshot::FrameSnapshot;
    use crate::input::AppState;
    use crate::spec::{ArgKind, ArgSpec, CommandSpec, ValueCardinality};

    use super::{
        apply_start_command, collapse_selected, ensure_enum_visible, expand_selected,
        move_form_selection, move_sidebar_selection, open_enum_dropdown,
    };

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
    fn valid_start_command_selects_resolved_path() {
        let root = command(
            "tool",
            vec![arg("verbose", "--verbose", ArgKind::Flag)],
            vec![command("build", Vec::new(), vec![command("release", Vec::new(), Vec::new())])],
        );
        let mut state = AppState::new(root);

        apply_start_command(&mut state, "build::release");

        assert_eq!(
            state.domain.selected_path().as_slice(),
            &["build".to_string(), "release".to_string()]
        );
        assert_eq!(state.domain.current_command().name, "release");
        assert_eq!(state.domain.command_path_key(), "tool::build::release");
    }

    #[test]
    fn invalid_start_command_keeps_root_selected_and_does_not_create_orphan_form_state() {
        let root = command("tool", vec![arg("verbose", "--verbose", ArgKind::Flag)], Vec::new());
        let mut state = AppState::new(root);

        apply_start_command(&mut state, "missing");

        assert!(state.domain.selected_path().is_empty());
        assert_eq!(state.domain.current_command().name, "tool");
        assert!(state.domain.current_form().is_none());
        let toast = state.notifications.toast.as_ref().expect("toast");
        assert_eq!(toast.message, "Unknown start command: missing");
        assert!(!toast.is_error);
    }

    #[test]
    fn sidebar_selection_always_resolves_to_valid_command() {
        let root = command(
            "tool",
            Vec::new(),
            vec![command("build", Vec::new(), Vec::new())],
        );
        let mut state = AppState::new(root);

        move_sidebar_selection(&mut state, 1);

        assert_eq!(state.domain.current_command().name, "build");
        assert!(state
            .domain
            .root
            .resolve_path(state.domain.selected_path().as_slice())
            .is_some());
    }

    #[test]
    fn selecting_invalid_command_path_is_rejected() {
        let root = command("tool", Vec::new(), vec![command("build", Vec::new(), Vec::new())]);
        let mut state = AppState::new(root);

        let result = state.domain.select_command_path(&["missing".to_string()]);

        assert!(result.is_err());
        assert!(state.domain.selected_path().is_empty());
        assert_eq!(state.domain.current_command().name, "tool");
    }

    #[test]
    fn expand_and_collapse_selected_updates_expanded_keys() {
        let root = command(
            "tool",
            Vec::new(),
            vec![command(
                "build",
                Vec::new(),
                vec![command("release", Vec::new(), Vec::new())],
            )],
        );
        let mut state = AppState::new(root);
        state
            .domain
            .select_command_path(&["build".to_string()])
            .expect("valid path");
        state.domain.expanded.remove("tool::build");

        expand_selected(&mut state);
        assert!(state.domain.expanded.contains("tool::build"));

        collapse_selected(&mut state);
        assert!(!state.domain.expanded.contains("tool::build"));
    }

    #[test]
    fn move_form_selection_clamps_and_scrolls_selected_field_into_view() {
        let root = command(
            "tool",
            vec![
                arg("alpha", "--alpha", ArgKind::Option),
                arg("beta", "--beta", ArgKind::Option),
                arg("gamma", "--gamma", ArgKind::Option),
            ],
            Vec::new(),
        );
        let mut state = AppState::new(root);
        let mut frame_snapshot = FrameSnapshot::default();
        frame_snapshot.layout.form_view = Some(Rect::new(0, 0, 30, 3));
        frame_snapshot.form_scroll_max = 20;

        move_form_selection(&mut state, &frame_snapshot, 2);

        assert_eq!(state.ui.selected_arg_index, 2);
        assert_eq!(state.ui.form_scroll, 11);
    }

    #[test]
    fn opening_dropdown_centers_current_choice_and_clamps_scroll() {
        let mut color = arg("color", "--color", ArgKind::Enum);
        color.choices = (0..10).map(|index| format!("choice-{index}")).collect();
        let mut state = AppState::new(command("tool", vec![color], Vec::new()));
        let mut frame_snapshot = FrameSnapshot::default();
        state.domain.set_choice_value("color", "choice-8".to_string());
        frame_snapshot.layout.form_view = Some(Rect::new(0, 0, 30, 10));
        frame_snapshot.layout.form_inputs
            .insert("color".to_string(), Rect::new(1, 1, 20, 1));

        open_enum_dropdown(&mut state, &frame_snapshot, "color", 10);

        assert_eq!(state.ui.dropdown_open.as_deref(), Some("color"));
        assert_eq!(state.ui.dropdown_scroll, 4);
    }

    #[test]
    fn ensure_enum_visible_uses_dropdown_height_to_adjust_scroll() {
        let root = command("tool", Vec::new(), Vec::new());
        let mut state = AppState::new(root);
        let mut frame_snapshot = FrameSnapshot::default();
        frame_snapshot.layout.dropdown = Some(Rect::new(0, 0, 20, 5));

        ensure_enum_visible(&mut state, &frame_snapshot, 4, 6);

        assert_eq!(state.ui.dropdown_scroll, 2);
    }
}
