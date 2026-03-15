use std::collections::HashSet;

use crate::spec::{CommandPath, CommandSpec};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TreeItem {
    pub(crate) label: String,
    pub(crate) path: CommandPath,
    pub(crate) has_children: bool,
    pub(crate) indent: usize,
    pub(crate) expanded: bool,
}

pub(crate) fn tree_items(
    root: &CommandSpec,
    expanded: &HashSet<String>,
    search: &str,
) -> Vec<TreeItem> {
    let mut items = Vec::new();
    let filter = search.trim().to_lowercase();
    build_tree_items_inner(root, expanded, &filter, &mut Vec::new(), 0, &mut items);
    items
}

fn build_tree_items_inner(
    cmd: &CommandSpec,
    expanded: &HashSet<String>,
    filter: &str,
    path: &mut Vec<String>,
    depth: usize,
    items: &mut Vec<TreeItem>,
) -> bool {
    let matches = filter.is_empty()
        || cmd.name.to_lowercase().contains(filter)
        || cmd
            .about
            .as_deref()
            .unwrap_or("")
            .to_lowercase()
            .contains(filter);

    let mut any_child_matches = false;
    path.push(cmd.name.clone());
    let key = path.join("::");

    let is_expanded = expanded.contains(&key);
    let mut child_items = Vec::new();
    for sub in &cmd.subcommands {
        let mut child_path = path.clone();
        let child_matches = build_tree_items_inner(
            sub,
            expanded,
            filter,
            &mut child_path,
            depth + 1,
            &mut child_items,
        );
        if child_matches {
            any_child_matches = true;
        }
    }

    let include = matches || any_child_matches;
    if include {
        let indent = "  ".repeat(depth);
        let caret = if cmd.subcommands.is_empty() {
            " "
        } else if is_expanded {
            "-"
        } else {
            "+"
        };
        let label = format!("{indent}{caret} {}", cmd.name);
        let display_path = if depth == 0 {
            CommandPath::default()
        } else {
            CommandPath::from(path[1..].to_vec())
        };
        items.push(TreeItem {
            label,
            path: display_path,
            has_children: !cmd.subcommands.is_empty(),
            indent: indent.len(),
            expanded: is_expanded,
        });
        let show_children = if filter.is_empty() {
            is_expanded
        } else {
            any_child_matches
        };
        if show_children {
            items.extend(child_items);
        }
    }

    path.pop();
    include
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::tree_items;
    use crate::spec::{ArgSpec, CommandPath, CommandSpec};

    fn command(name: &str, about: Option<&str>, subcommands: Vec<CommandSpec>) -> CommandSpec {
        CommandSpec {
            name: name.to_string(),
            about: about.map(str::to_string),
            help: String::new(),
            args: Vec::<ArgSpec>::new(),
            subcommands,
        }
    }

    #[test]
    fn root_only_tree_contains_root_item() {
        let root = command("tool", Some("root"), Vec::new());

        let items = tree_items(&root, &HashSet::new(), "");

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].label, "  tool");
        assert!(items[0].path.is_empty());
    }

    #[test]
    fn expanded_nested_subcommands_are_included() {
        let leaf = command("serve", Some("serve"), Vec::new());
        let child = command("api", Some("api"), vec![leaf]);
        let root = command("tool", Some("root"), vec![child]);
        let expanded = HashSet::from(["tool".to_string(), "tool::api".to_string()]);

        let items = tree_items(&root, &expanded, "");

        let labels = items.into_iter().map(|item| item.label).collect::<Vec<_>>();
        assert_eq!(labels, vec!["- tool", "  - api", "      serve"]);
    }

    #[test]
    fn filtered_search_keeps_matching_parent_chain() {
        let leaf = command("deploy", Some("ship release"), Vec::new());
        let child = command("release", Some("release ops"), vec![leaf]);
        let root = command("tool", Some("root"), vec![child]);

        let items = tree_items(&root, &HashSet::new(), "ship");

        let paths = items.into_iter().map(|item| item.path).collect::<Vec<_>>();
        assert_eq!(
            paths,
            vec![
                CommandPath::default(),
                vec!["release".to_string()].into(),
                vec!["release".to_string(), "deploy".to_string()].into(),
            ]
        );
    }

    #[test]
    fn collapsed_children_are_hidden_without_search() {
        let child = command("build", Some("build"), Vec::new());
        let root = command("tool", Some("root"), vec![child]);
        let expanded = HashSet::from(["tool".to_string()]);

        let items = tree_items(&root, &expanded, "");

        assert_eq!(items.len(), 2);
        assert_eq!(items[1].path, vec!["build".to_string()].into());
    }
}
