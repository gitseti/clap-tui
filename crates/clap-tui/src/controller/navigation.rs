use crate::frame_snapshot::FrameSnapshot;
use crate::input::{ActiveTab, AppState, ArgValue, Focus, UiState};
use crate::query::{
    form::{self, FieldWidget},
    tree::{self, TreeRow},
};
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

pub(crate) fn toggle_help_tab(state: &mut AppState) {
    state.ui.toggle_help();
}

pub(crate) fn move_sidebar_selection(
    state: &mut AppState,
    frame_snapshot: &FrameSnapshot,
    delta: isize,
) {
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
    ensure_selected_sidebar_visible(state, frame_snapshot);
}

pub(crate) fn move_form_selection(
    state: &mut AppState,
    frame_snapshot: &FrameSnapshot,
    delta: isize,
) {
    if state.ui.help_open {
        return;
    }
    let root = state.domain.root.clone();
    let selected_path = state.domain.selected_path().clone();
    let args = form::visible_args_for_path(&root, &selected_path, state.ui.active_tab);
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

pub(crate) fn select_sidebar(state: &mut AppState, frame_snapshot: &FrameSnapshot) {
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
    ensure_selected_sidebar_visible(state, frame_snapshot);
}

pub(crate) fn collapse_selected(state: &mut AppState, frame_snapshot: &FrameSnapshot) {
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
    ensure_selected_sidebar_visible(state, frame_snapshot);
}

pub(crate) fn expand_selected(state: &mut AppState, frame_snapshot: &FrameSnapshot) {
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
    ensure_selected_sidebar_visible(state, frame_snapshot);
}

pub(crate) fn sidebar_right(state: &mut AppState, frame_snapshot: &FrameSnapshot) {
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
        ensure_selected_sidebar_visible(state, frame_snapshot);
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
    let root = state.domain.root.clone();
    let selected_path = state.domain.selected_path().clone();
    let args = form::visible_args_for_path(&root, &selected_path, state.ui.active_tab);
    let Some(item) = args
        .iter()
        .find(|item| item.order_index == state.ui.selected_arg_index)
    else {
        return;
    };
    let arg = item.arg;
    if matches!(item.widget, FieldWidget::Toggle) {
        state.domain.toggle_flag_touched(&arg.id);
    } else if matches!(
        item.widget,
        FieldWidget::SingleChoice | FieldWidget::MultiChoice
    ) {
        if arg.choices.is_empty() || state.ui.dropdown_open.as_deref() == Some(arg.id.as_str()) {
            state.ui.close_dropdown();
        } else {
            open_enum_dropdown(state, frame_snapshot, &arg.id, arg.choices.len());
        }
    } else if matches!(item.widget, FieldWidget::Counter) {
        state.domain.increment_counter(&arg.id);
    } else if matches!(item.widget, FieldWidget::OptionalValue) {
        let owner_key = state.domain.command_path_key_for(arg.owner_path());
        let state_kind = state
            .domain
            .forms
            .get(&owner_key)
            .and_then(|form| form.input(&arg.id))
            .map(|input| match &input.value {
                crate::input::ArgInput::Flag { present: true, .. } => "flag",
                crate::input::ArgInput::Values { occurrences }
                    if input.touched
                        && occurrences.iter().any(|occurrence| {
                            occurrence.values.iter().any(|value| !value.is_empty())
                        }) =>
                {
                    "value"
                }
                _ => "empty",
            });
        match state_kind {
            Some("flag") => state.domain.clear_value_and_untouch(&arg.id),
            Some("value") => state.ui.focus_form(),
            _ => state.domain.toggle_optional_value_flag(&arg.id, true),
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

    state.ui.open_dropdown(arg_id.to_string(), 0, 0);
    let current = state
        .domain
        .arg_for_input(arg_id)
        .and_then(|arg| {
            state
                .domain
                .current_form()
                .and_then(|inputs| inputs.selected_values(arg).first().cloned())
                .and_then(|selected| arg.choices.iter().position(|choice| *choice == selected))
                .or_else(|| {
                    state
                        .domain
                        .current_form()
                        .and_then(|inputs| inputs.compatibility_value(arg))
                        .and_then(|value| match value {
                            ArgValue::Choice(selected) => {
                                arg.choices.iter().position(|choice| *choice == selected)
                            }
                            _ => None,
                        })
                })
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
    state.ui.set_dropdown_cursor(current);
}

pub(crate) fn ensure_form_visible(state: &mut AppState, frame_snapshot: &FrameSnapshot) {
    if state.ui.help_open {
        return;
    }
    let Some(form_area) = frame_snapshot.form_view_rect() else {
        return;
    };
    let root = state.domain.root.clone();
    let selected_path = state.domain.selected_path().clone();
    let args = form::visible_args_for_path(&root, &selected_path, state.ui.active_tab);
    let validation = state.derived_validation();
    let Some((input_top, input_bottom)) = form::field_content_bounds_with_errors(
        &args,
        state.ui.selected_arg_index,
        &validation.field_errors,
    ) else {
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

pub(crate) fn focus_first_invalid_field(state: &mut AppState, frame_snapshot: &FrameSnapshot) {
    let Some(arg_id) = frame_snapshot.first_invalid_field_id() else {
        return;
    };
    let root = state.domain.root.clone();
    let selected_path = state.domain.selected_path().clone();
    let args = form::visible_args_for_path(&root, &selected_path, state.ui.active_tab);
    let Some(item) = args.iter().find(|item| item.arg.id == arg_id) else {
        return;
    };
    state.ui.set_selected_arg_index(item.order_index);
    state.ui.focus_form();
    ensure_form_visible(state, frame_snapshot);
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
        .arg_for_input(arg_id)
        .map_or(0, |arg| arg.choices.len());
    let Some(visible) = frame_snapshot.dropdown_visible_rows() else {
        return;
    };
    state.ui.adjust_dropdown_scroll(delta, total, visible);
}

pub(crate) fn scroll_sidebar(state: &mut AppState, frame_snapshot: &FrameSnapshot, delta: i16) {
    let rows = tree::tree_rows(
        &state.domain.root,
        &state.domain.expanded,
        &state.ui.search_query,
    );
    let Some(visible_rows) = frame_snapshot.sidebar_visible_rows() else {
        return;
    };
    if visible_rows == 0 {
        return;
    }
    let selected_row = selected_sidebar_row(&rows, state.domain.selected_path());
    state
        .ui
        .adjust_sidebar_scroll(delta, rows.len(), visible_rows, selected_row);
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

pub(crate) fn clamp_sidebar_selection_to_search(
    state: &mut AppState,
    frame_snapshot: &FrameSnapshot,
) {
    let items = tree::tree_items(
        &state.domain.root,
        &state.domain.expanded,
        &state.ui.search_query,
    );
    if items.is_empty() {
        if !state.domain.selected_path().is_empty() {
            select_root(state);
        }
        state.ui.set_sidebar_scroll(0);
        return;
    }

    if !items
        .iter()
        .any(|item| item.path == *state.domain.selected_path())
    {
        select_command(state, items[0].path.as_slice());
    }

    ensure_selected_sidebar_visible(state, frame_snapshot);
}

pub(crate) fn ensure_selected_sidebar_visible(
    state: &mut AppState,
    frame_snapshot: &FrameSnapshot,
) {
    let rows = tree::tree_rows(
        &state.domain.root,
        &state.domain.expanded,
        &state.ui.search_query,
    );
    let Some(visible_rows) = frame_snapshot.sidebar_visible_rows() else {
        return;
    };
    let Some(selected_row) = selected_sidebar_row(&rows, state.domain.selected_path()) else {
        state.ui.set_sidebar_scroll(0);
        return;
    };
    if visible_rows == 0 {
        state.ui.set_sidebar_scroll(0);
        return;
    }

    let max_scroll = rows.len().saturating_sub(visible_rows.min(rows.len()));
    let mut scroll = state.ui.sidebar_scroll(rows.len(), visible_rows);
    if selected_row < scroll {
        scroll = selected_row;
    } else if selected_row >= scroll.saturating_add(visible_rows) {
        scroll = selected_row.saturating_add(1).saturating_sub(visible_rows);
    }
    state.ui.set_sidebar_scroll(scroll.min(max_scroll));
}

pub(crate) fn apply_start_command(state: &mut AppState, start: &str) {
    match state.select_command_by_search_path(start) {
        Ok(()) => {
            let root = state.domain.root.clone();
            let selected_path = state.domain.selected_path().clone();
            let args = form::visible_args_for_path(&root, &selected_path, state.ui.active_tab);
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

fn selected_sidebar_row(rows: &[TreeRow], selected_path: &CommandPath) -> Option<usize> {
    rows.iter().position(|row| match row {
        TreeRow::Item(item) => item.path == *selected_path,
        TreeRow::Heading { .. } => false,
    })
}

fn select_command(state: &mut AppState, path: &[String]) {
    if state.select_command_path(path).is_ok() {
        let root = state.domain.root.clone();
        let selected_path = state.domain.selected_path().clone();
        let args = form::visible_args_for_path(&root, &selected_path, state.ui.active_tab);
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
    use clap::{Arg, Command};
    use ratatui::layout::Rect;

    use crate::frame_snapshot::FrameSnapshot;
    use crate::input::{AppState, Focus};
    use crate::spec::{ArgKind, ArgSpec, CommandSpec, ValueCardinality};

    use super::{
        activate_form_field, apply_start_command, clamp_sidebar_selection_to_search,
        collapse_selected, ensure_enum_visible, expand_selected, handle_escape,
        move_form_selection, move_sidebar_selection, open_enum_dropdown, scroll_sidebar,
        sidebar_right,
    };

    fn sidebar_snapshot() -> FrameSnapshot {
        let mut snapshot = FrameSnapshot::default();
        snapshot.layout.sidebar_list = Some(Rect::new(0, 0, 20, 3));
        snapshot
    }

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

    fn command(name: &str, args: Vec<ArgSpec>, subcommands: Vec<CommandSpec>) -> CommandSpec {
        CommandSpec {
            name: name.to_string(),
            version: None,
            about: None,
            help: String::new(),
            args,
            subcommands,
            ..CommandSpec::default()
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

        move_sidebar_selection(&mut state, &sidebar_snapshot(), 1);

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

        let snapshot = sidebar_snapshot();
        move_sidebar_selection(&mut state, &snapshot, 1);
        move_sidebar_selection(&mut state, &snapshot, -1);

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

        move_sidebar_selection(&mut state, &sidebar_snapshot(), 1);

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

        move_sidebar_selection(&mut state, &sidebar_snapshot(), -1);

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

        let snapshot = sidebar_snapshot();
        expand_selected(&mut state, &snapshot);
        assert!(state.domain.expanded.contains("tool::build"));

        collapse_selected(&mut state, &snapshot);
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

        sidebar_right(&mut state, &sidebar_snapshot());

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

        sidebar_right(&mut state, &sidebar_snapshot());

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

        sidebar_right(&mut state, &sidebar_snapshot());
        assert!(matches!(state.ui.focus, Focus::Form));

        state.ui.focus = Focus::Sidebar;
        state
            .select_command_path(&["build".to_string()])
            .expect("valid path");

        sidebar_right(&mut state, &sidebar_snapshot());
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

        move_sidebar_selection(&mut state, &sidebar_snapshot(), 1);

        assert_eq!(
            state.domain.selected_path().as_slice(),
            &["build".to_string()]
        );
        assert!(!state.domain.expanded.contains("tool::build"));
    }

    #[test]
    fn moving_sidebar_selection_scrolls_window_to_keep_selected_row_visible() {
        let root = command(
            "tool",
            Vec::new(),
            vec![
                command("one", Vec::new(), Vec::new()),
                command("two", Vec::new(), Vec::new()),
                command("three", Vec::new(), Vec::new()),
                command("four", Vec::new(), Vec::new()),
                command("five", Vec::new(), Vec::new()),
            ],
        );
        let mut state = AppState::new(root);
        let snapshot = sidebar_snapshot();

        for _ in 0..4 {
            move_sidebar_selection(&mut state, &snapshot, 1);
        }

        assert_eq!(state.domain.current_command().name, "four");
        assert!(state.ui.sidebar_scroll > 0);
    }

    #[test]
    fn search_clamp_reselects_first_visible_match_and_resets_sidebar_window() {
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
        state.ui.sidebar_scroll = 3;

        clamp_sidebar_selection_to_search(&mut state, &sidebar_snapshot());

        assert_eq!(state.domain.current_command().name, "deploy");
        assert_eq!(state.ui.sidebar_scroll, 0);
    }

    #[test]
    fn sidebar_wheel_scroll_keeps_selected_row_visible() {
        let root = command(
            "tool",
            Vec::new(),
            vec![
                command("one", Vec::new(), Vec::new()),
                command("two", Vec::new(), Vec::new()),
                command("three", Vec::new(), Vec::new()),
                command("four", Vec::new(), Vec::new()),
                command("five", Vec::new(), Vec::new()),
            ],
        );
        let mut state = AppState::new(root);
        state
            .select_command_path(&["three".to_string()])
            .expect("valid path");
        let snapshot = sidebar_snapshot();

        scroll_sidebar(&mut state, &snapshot, 3);

        assert!(state.ui.sidebar_scroll <= 2);
        assert!(state.ui.sidebar_scroll >= 1);
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

    #[test]
    fn activating_optional_value_with_existing_text_preserves_the_value() {
        let mut color = arg("color", "--color", ArgKind::Flag);
        color.metadata.action.value_arity = crate::spec::ArgValueArity::Optional;
        let mut state = AppState::new(command("tool", vec![color], Vec::new()));
        state.domain.set_text_value("color", "blue");
        state.domain.mark_touched("color");
        state.ui.focus_form();

        activate_form_field(&mut state, &FrameSnapshot::default());

        let arg = state
            .domain
            .current_command()
            .args
            .iter()
            .find(|arg| arg.id == "color")
            .expect("color arg");
        assert_eq!(
            state
                .domain
                .current_form()
                .and_then(|form| form.compatibility_value(arg)),
            Some(crate::input::ArgValue::Text("blue".to_string()))
        );
    }

    #[test]
    fn activating_optional_value_without_explicit_text_toggles_it_back_off() {
        let mut color = arg("color", "--color", ArgKind::Flag);
        color.metadata.action.value_arity = crate::spec::ArgValueArity::Optional;
        let mut state = AppState::new(command("tool", vec![color], Vec::new()));
        state.domain.toggle_optional_value_flag("color", true);
        state.ui.focus_form();

        activate_form_field(&mut state, &FrameSnapshot::default());

        assert!(state.domain.current_form().is_none());
    }

    #[test]
    fn activating_optional_value_with_default_value_enables_presence() {
        let root = CommandSpec::from_command(
            &Command::new("tool").arg(
                Arg::new("color")
                    .long("color")
                    .num_args(0..=1)
                    .require_equals(true)
                    .default_value("auto")
                    .default_missing_value("always"),
            ),
        );
        let mut state = AppState::new(root);
        state.ui.focus_form();

        activate_form_field(&mut state, &FrameSnapshot::default());

        let color = state
            .domain
            .current_command()
            .args
            .iter()
            .find(|arg| arg.id == "color")
            .expect("color arg");
        assert_eq!(
            state
                .domain
                .current_form()
                .and_then(|form| form.compatibility_value(color)),
            Some(crate::input::ArgValue::Bool(true))
        );
    }
}
