use crate::controller::navigation;
use crate::frame_snapshot::FrameSnapshot;
use crate::input::{AppState, Focus};
use crate::query::{form, tree};

use super::{Action, Effect};

pub(crate) fn apply(
    action: &Action,
    state: &mut AppState,
    frame_snapshot: &FrameSnapshot,
) -> Effect {
    match action {
        Action::MoveSidebarSelection(delta) => navigation::move_sidebar_selection(state, *delta),
        Action::SidebarRight => navigation::sidebar_right(state),
        Action::CollapseSelected => navigation::collapse_selected(state),
        Action::SelectSidebar => navigation::select_sidebar(state),
        Action::ClickSidebar { x, y } => apply_sidebar_click(*x, *y, state, frame_snapshot),
        _ => {}
    }
    Effect::None
}

fn apply_sidebar_click(x: u16, y: u16, state: &mut AppState, frame_snapshot: &FrameSnapshot) {
    if let Some(sidebar_area) = frame_snapshot.layout.sidebar
        && y == sidebar_area.y
    {
        navigation::select_root(state);
        state.ui.focus_sidebar();
        return;
    }

    let Some((path, caret_hit, has_children)) = frame_snapshot.sidebar_item_at(x, y).map(|item| {
        (
            item.path.clone(),
            FrameSnapshot::sidebar_caret_contains(item, x, y),
            item.has_children,
        )
    }) else {
        return;
    };

    if *state.domain.selected_path() != path && state.select_command_path(path.as_slice()).is_ok() {
        let command = state.domain.current_command().clone();
        let args = form::visible_args(&command, state.ui.active_tab);
        state.ui.focus_first_tab(&form::visible_arg_pairs(&args));
    }
    if caret_hit && has_children {
        let items = tree::tree_items(
            &state.domain.root,
            &state.domain.expanded,
            &state.ui.search_query,
        );
        if let Some(item) = items.iter().find(|item| item.path == path) {
            if item.expanded {
                navigation::collapse_selected(state);
            } else {
                navigation::expand_selected(state);
            }
        }
    }
    state.ui.focus = Focus::Sidebar;
}

#[cfg(test)]
mod tests {
    use crate::frame_snapshot::FrameSnapshot;
    use crate::spec::CommandSpec;

    use super::super::{Action, Effect, apply_action};

    #[test]
    fn sidebar_title_click_reselects_root() {
        let mut state = crate::input::AppState::new(CommandSpec {
            name: "tool".to_string(),
            version: Some("1.0.0".to_string()),
            about: None,
            help: String::new(),
            args: Vec::new(),
            subcommands: vec![CommandSpec {
                name: "build".to_string(),
                version: None,
                about: None,
                help: String::new(),
                args: Vec::new(),
                subcommands: Vec::new(),
            }],
        });
        state
            .select_command_path(&["build".to_string()])
            .expect("valid path");
        let mut snapshot = FrameSnapshot::default();
        snapshot.layout.sidebar = Some(ratatui::layout::Rect::new(0, 0, 20, 8));

        let action = Action::ClickSidebar { x: 2, y: 0 };
        let effect = apply_action(&action, &mut state, &snapshot);

        assert_eq!(effect, Effect::None);
        assert!(state.domain.selected_path().is_empty());
        assert_eq!(state.domain.current_command().name, "tool");
        assert!(matches!(state.ui.focus, crate::input::Focus::Sidebar));
    }
}
