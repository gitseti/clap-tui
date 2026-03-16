use crate::frame_snapshot::FrameSnapshot;
use crate::input::{ActiveTab, AppState, ArgValue, Focus, UiState};
use crate::query::{form, tree};
use crate::spec::{CommandPath, SelectionError};

pub(crate) fn switch_tab(state: &mut AppState, tab: ActiveTab) {
    let tabs = UiState::visible_tabs();
    let target = tabs
        .into_iter()
        .find(|candidate| *candidate == tab)
        .unwrap_or(ActiveTab::Inputs);
    if target == state.ui.active_tab {
        return;
    }
    state.ui.active_tab = target;
    state.ui.last_non_help_tab = state.ui.active_tab;
    state.sync_visible_form_selection();
    state.ui.reset_transient_form_ui();
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
    state.ui.toggle_help();
}

pub(crate) fn move_sidebar_selection(state: &mut AppState, delta: isize) {
    let items = tree::tree_items(
        &state.domain.root,
        &state.domain.expanded,
        &state.ui.search_query,
    );
    if items.is_empty() {
        return;
    }
    if state.domain.selected_path().is_empty() {
        if delta > 0 {
            select_command(state, items[0].path.as_slice());
        }
        return;
    }
    let current_index = match items
        .iter()
        .position(|item| item.path == *state.domain.selected_path())
    {
        Some(current_index) => current_index,
        None if delta > 0 => {
            select_command(state, items[0].path.as_slice());
            return;
        }
        None if delta < 0 => {
            select_command(state, items[items.len() - 1].path.as_slice());
            return;
        }
        None => return,
    };
    if delta < 0 && current_index == 0 {
        select_root(state);
        return;
    }
    let next_index = current_index
        .saturating_add_signed(delta)
        .min(items.len() - 1);
    if *state.domain.selected_path() != items[next_index].path {
        select_command(state, items[next_index].path.as_slice());
    }
}

pub(crate) fn move_form_selection(
    state: &mut AppState,
    frame_snapshot: &FrameSnapshot,
    delta: isize,
) {
    if state.ui.help_open {
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
    state.ui.set_selected_arg_index(args[next_pos].order_index);
    ensure_form_visible(state, frame_snapshot);
}

pub(crate) fn select_sidebar(state: &mut AppState) {
    let items = tree::tree_items(
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
    let items = tree::tree_items(
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
    let items = tree::tree_items(
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

pub(crate) fn sidebar_right(state: &mut AppState) {
    if state.domain.selected_path().is_empty() {
        state.ui.focus_form();
        return;
    }

    let items = tree::tree_items(
        &state.domain.root,
        &state.domain.expanded,
        &state.ui.search_query,
    );
    let Some(item) = items
        .iter()
        .find(|item| item.path == *state.domain.selected_path())
    else {
        state.ui.focus_form();
        return;
    };

    if item.has_children && !item.expanded {
        toggle_expand(state, &item.path, false);
    } else {
        state.ui.focus_form();
    }
}

pub(crate) fn handle_escape(state: &mut AppState) {
    if state.ui.help_open {
        state.ui.toggle_help();
        return;
    }
    if state.ui.dropdown_open.is_some() {
        state.ui.close_dropdown();
        return;
    }

    match state.ui.focus {
        Focus::Search | Focus::Form => state.ui.focus_sidebar(),
        Focus::Sidebar if !state.domain.selected_path().is_empty() => select_root(state),
        Focus::Sidebar => {}
    }
}

pub(crate) fn activate_form_field(state: &mut AppState, frame_snapshot: &FrameSnapshot) {
    if state.ui.help_open {
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
        state.domain.toggle_flag_touched(&arg.id);
    } else if arg.uses_choice_input() {
        if arg.choices.is_empty() || state.ui.dropdown_open.as_deref() == Some(arg.id.as_str()) {
            state.ui.close_dropdown();
        } else {
            open_enum_dropdown(state, frame_snapshot, &arg.id, arg.choices.len());
        }
    } else {
        state.ui.focus_form();
    }
}

pub(crate) fn open_enum_dropdown(
    state: &mut AppState,
    frame_snapshot: &FrameSnapshot,
    arg_id: &str,
    total: usize,
) {
    if total == 0 {
        state.ui.close_dropdown();
        state.ui.set_dropdown_scroll(0);
        return;
    }

    state.ui.open_dropdown(arg_id.to_string(), 0);
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
        .dropdown_geometry_for_input(arg_id, total)
        .map_or(
            total.min(usize::from(crate::frame_snapshot::MAX_DROPDOWN_ROWS)),
            |layout| layout.visible_rows,
        );
    let max_scroll = total.saturating_sub(visible_rows);
    state
        .ui
        .set_dropdown_scroll(current.saturating_sub(visible_rows / 2).min(max_scroll));
}

pub(crate) fn ensure_form_visible(state: &mut AppState, frame_snapshot: &FrameSnapshot) {
    if state.ui.help_open {
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
        state.ui.set_form_scroll(input_top);
    } else if input_bottom > visible_bottom {
        state.ui.set_form_scroll(
            state
                .ui
                .form_scroll
                .saturating_add(input_bottom.saturating_sub(visible_bottom)),
        );
    }
    state.ui.clamp_form_scroll(frame_snapshot);
}

pub(crate) fn scroll_form(state: &mut AppState, frame_snapshot: &FrameSnapshot, delta: i16) {
    if state.ui.help_open {
        state.ui.adjust_help_scroll(delta);
        state.ui.clamp_help_scroll(frame_snapshot);
        return;
    }
    state.ui.adjust_form_scroll(delta);
    state.ui.clamp_form_scroll(frame_snapshot);
}

pub(crate) fn scroll_enum(state: &mut AppState, frame_snapshot: &FrameSnapshot, delta: i16) {
    let Some(arg_id) = state.ui.dropdown_open.as_deref() else {
        return;
    };
    let total = state
        .domain
        .current_command()
        .args
        .iter()
        .find(|arg| arg.id == arg_id)
        .map_or(0, |arg| arg.choices.len());
    let Some(visible) = frame_snapshot.dropdown_visible_rows() else {
        return;
    };
    state.ui.adjust_dropdown_scroll(delta, total, visible);
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
        state.ui.set_dropdown_scroll(index);
    } else if index >= state.ui.dropdown_scroll + visible {
        state
            .ui
            .set_dropdown_scroll(index.saturating_sub(visible - 1));
    }
    state
        .ui
        .set_dropdown_scroll(state.ui.dropdown_scroll.min(max_scroll));
}

pub(crate) fn apply_start_command(state: &mut AppState, start: &str) {
    match state.select_command_by_search_path(start) {
        Ok(()) => {
            let command = state.domain.current_command().clone();
            let args = form::visible_args(&command, state.ui.active_tab);
            state.ui.focus_first_tab(&form::visible_arg_pairs(&args));
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

pub(crate) fn select_root(state: &mut AppState) {
    select_command(state, &[]);
}

fn select_command(state: &mut AppState, path: &[String]) {
    if state.select_command_path(path).is_ok() {
        let command = state.domain.current_command().clone();
        let args = form::visible_args(&command, state.ui.active_tab);
        state.ui.focus_first_tab(&form::visible_arg_pairs(&args));
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

#[cfg(test)]
mod tests {
    use ratatui::layout::Rect;

    use crate::frame_snapshot::FrameSnapshot;
    use crate::input::{AppState, Focus};
    use crate::spec::{ArgKind, ArgSpec, CommandSpec, ValueCardinality};

    use super::{
        apply_start_command, collapse_selected, ensure_enum_visible, expand_selected,
        handle_escape, move_form_selection, move_sidebar_selection, open_enum_dropdown,
        sidebar_right,
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
            version: None,
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
            vec![command(
                "build",
                Vec::new(),
                vec![command("release", Vec::new(), Vec::new())],
            )],
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
        let root = command("tool", Vec::new(), Vec::new());
        let mut state = AppState::new(root);

        apply_start_command(&mut state, "missing");

        assert!(state.domain.selected_path().is_empty());
        assert_eq!(state.domain.current_command().name, "tool");
        assert!(state.domain.current_form().is_none());
        assert_eq!(state.domain.forms.len(), 0);
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
        assert!(
            state
                .domain
                .root
                .resolve_path(state.domain.selected_path().as_slice())
                .is_some()
        );
    }

    #[test]
    fn moving_sidebar_up_from_first_child_reselects_root() {
        let root = command(
            "tool",
            Vec::new(),
            vec![command("build", Vec::new(), Vec::new())],
        );
        let mut state = AppState::new(root);

        move_sidebar_selection(&mut state, 1);
        move_sidebar_selection(&mut state, -1);

        assert!(state.domain.selected_path().is_empty());
        assert_eq!(state.domain.current_command().name, "tool");
    }

    #[test]
    fn filtered_sidebar_navigation_recovers_with_first_visible_match() {
        let root = command(
            "tool",
            Vec::new(),
            vec![
                command("build", Vec::new(), Vec::new()),
                command("deploy", Vec::new(), Vec::new()),
                command("debug", Vec::new(), Vec::new()),
            ],
        );
        let mut state = AppState::new(root);
        state
            .select_command_path(&["build".to_string()])
            .expect("valid path");
        state.ui.search_query = "de".to_string();

        move_sidebar_selection(&mut state, 1);

        assert_eq!(state.domain.current_command().name, "deploy");
        assert_eq!(
            state.domain.selected_path().as_slice(),
            &["deploy".to_string()]
        );
    }

    #[test]
    fn filtered_sidebar_navigation_recovers_with_last_visible_match() {
        let root = command(
            "tool",
            Vec::new(),
            vec![
                command("build", Vec::new(), Vec::new()),
                command("deploy", Vec::new(), Vec::new()),
                command("debug", Vec::new(), Vec::new()),
            ],
        );
        let mut state = AppState::new(root);
        state
            .select_command_path(&["build".to_string()])
            .expect("valid path");
        state.ui.search_query = "de".to_string();

        move_sidebar_selection(&mut state, -1);

        assert_eq!(state.domain.current_command().name, "debug");
        assert_eq!(
            state.domain.selected_path().as_slice(),
            &["debug".to_string()]
        );
    }

    #[test]
    fn selecting_invalid_command_path_is_rejected() {
        let root = command(
            "tool",
            Vec::new(),
            vec![command("build", Vec::new(), Vec::new())],
        );
        let mut state = AppState::new(root);

        let result = state.select_command_path(&["missing".to_string()]);

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
            .select_command_path(&["build".to_string()])
            .expect("valid path");
        state.domain.expanded.remove("tool::build");

        expand_selected(&mut state);
        assert!(state.domain.expanded.contains("tool::build"));

        collapse_selected(&mut state);
        assert!(!state.domain.expanded.contains("tool::build"));
    }

    #[test]
    fn sidebar_right_expands_collapsed_branch_and_keeps_sidebar_focus() {
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
            .select_command_path(&["build".to_string()])
            .expect("valid path");
        state.ui.focus = Focus::Sidebar;

        sidebar_right(&mut state);

        assert!(state.domain.expanded.contains("tool::build"));
        assert!(matches!(state.ui.focus, Focus::Sidebar));
        assert_eq!(
            state.domain.selected_path().as_slice(),
            &["build".to_string()]
        );
    }

    #[test]
    fn sidebar_right_moves_focus_to_form_for_expanded_branch() {
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
            .select_command_path(&["build".to_string()])
            .expect("valid path");
        state.domain.expanded.insert("tool::build".to_string());
        state.ui.focus = Focus::Sidebar;

        sidebar_right(&mut state);

        assert!(matches!(state.ui.focus, Focus::Form));
        assert!(state.domain.expanded.contains("tool::build"));
    }

    #[test]
    fn sidebar_right_moves_focus_to_form_for_leaf_and_root() {
        let root = command(
            "tool",
            Vec::new(),
            vec![command("build", Vec::new(), Vec::new())],
        );
        let mut state = AppState::new(root);
        state.ui.focus = Focus::Sidebar;

        sidebar_right(&mut state);
        assert!(matches!(state.ui.focus, Focus::Form));

        state.ui.focus = Focus::Sidebar;
        state
            .select_command_path(&["build".to_string()])
            .expect("valid path");

        sidebar_right(&mut state);
        assert!(matches!(state.ui.focus, Focus::Form));
    }

    #[test]
    fn move_sidebar_selection_does_not_auto_expand_selected_branch() {
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

        move_sidebar_selection(&mut state, 1);

        assert_eq!(
            state.domain.selected_path().as_slice(),
            &["build".to_string()]
        );
        assert!(!state.domain.expanded.contains("tool::build"));
    }

    #[test]
    fn handle_escape_reselects_root_when_sidebar_has_non_root_selection() {
        let root = command(
            "tool",
            Vec::new(),
            vec![command("build", Vec::new(), Vec::new())],
        );
        let mut state = AppState::new(root);
        state
            .select_command_path(&["build".to_string()])
            .expect("valid path");
        state.ui.focus = Focus::Sidebar;

        handle_escape(&mut state);

        assert!(state.domain.selected_path().is_empty());
        assert_eq!(state.domain.current_command().name, "tool");
        assert!(matches!(state.ui.focus, Focus::Sidebar));
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
        state
            .domain
            .set_choice_value("color", "choice-8".to_string());
        frame_snapshot.layout.form_view = Some(Rect::new(0, 0, 30, 10));
        frame_snapshot
            .layout
            .form_inputs
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
