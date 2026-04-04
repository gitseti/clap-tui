use std::collections::HashSet;

use crate::spec::{CommandPath, CommandSpec};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TreeItem {
    pub(crate) name: String,
    pub(crate) display_label: String,
    pub(crate) version: Option<String>,
    pub(crate) path: CommandPath,
    pub(crate) has_children: bool,
    pub(crate) indent: usize,
    pub(crate) expanded: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TreeRow {
    Heading { title: String, indent: usize },
    Item(TreeItem),
}

impl TreeItem {
    pub(crate) fn prefix(&self) -> String {
        let indent = "| ".repeat(self.indent / 2);
        let caret = if !self.has_children {
            " "
        } else if self.expanded {
            "-"
        } else {
            "+"
        };
        format!("{indent}{caret} ")
    }

    #[cfg(test)]
    pub(crate) fn label(&self) -> String {
        let mut label = format!("{}{}", self.prefix(), self.display_label);
        if let Some(version) = &self.version {
            label.push(' ');
            label.push_str(version);
        }
        label
    }
}

pub(crate) fn tree_items(
    root: &CommandSpec,
    expanded: &HashSet<String>,
    search: &str,
) -> Vec<TreeItem> {
    tree_rows(root, expanded, search)
        .into_iter()
        .filter_map(|row| match row {
            TreeRow::Item(item) => Some(item),
            TreeRow::Heading { .. } => None,
        })
        .collect()
}

pub(crate) fn tree_rows(
    root: &CommandSpec,
    expanded: &HashSet<String>,
    search: &str,
) -> Vec<TreeRow> {
    let mut rows = Vec::new();
    let filter = search.trim().to_lowercase();
    let root_path = vec![root.name.clone()];
    let mut child_rows = Vec::new();
    for subcommand in &root.subcommands {
        let mut child_path = root_path.clone();
        build_tree_rows_inner(
            subcommand,
            expanded,
            &filter,
            &mut child_path,
            0,
            &mut child_rows,
        );
    }

    if !child_rows.is_empty() {
        if let Some(heading) = root
            .subcommand_help_heading()
            .filter(|heading| !heading.is_empty())
        {
            rows.push(TreeRow::Heading {
                title: heading.to_string(),
                indent: 0,
            });
        }
        rows.extend(child_rows);
    }

    rows
}

fn build_tree_rows_inner(
    cmd: &CommandSpec,
    expanded: &HashSet<String>,
    filter: &str,
    path: &mut Vec<String>,
    depth: usize,
    rows: &mut Vec<TreeRow>,
) -> bool {
    let matches = command_matches_filter(cmd, filter);

    let mut any_child_matches = false;
    path.push(cmd.name.clone());
    let key = path.join("::");

    let is_expanded = expanded.contains(&key);
    let mut child_rows = Vec::new();
    for sub in &cmd.subcommands {
        let mut child_path = path.clone();
        let child_matches = build_tree_rows_inner(
            sub,
            expanded,
            filter,
            &mut child_path,
            depth + 1,
            &mut child_rows,
        );
        if child_matches {
            any_child_matches = true;
        }
    }

    let include = matches || any_child_matches;
    if include {
        let display_path = CommandPath::from(path[1..].to_vec());
        rows.push(TreeRow::Item(TreeItem {
            name: cmd.name.clone(),
            display_label: cmd.display_label().to_string(),
            version: None,
            path: display_path,
            has_children: !cmd.subcommands.is_empty(),
            indent: depth * 2,
            expanded: is_expanded,
        }));
        let show_children = if filter.is_empty() {
            is_expanded
        } else {
            any_child_matches
        };
        if show_children && !child_rows.is_empty() {
            if let Some(heading) = cmd
                .subcommand_help_heading()
                .filter(|heading| !heading.is_empty())
            {
                rows.push(TreeRow::Heading {
                    title: heading.to_string(),
                    indent: depth * 2 + 2,
                });
            }
            rows.extend(child_rows);
        }
    }

    path.pop();
    include
}

fn command_matches_filter(cmd: &CommandSpec, filter: &str) -> bool {
    filter.is_empty()
        || cmd.name.to_lowercase().contains(filter)
        || cmd.display_label().to_lowercase().contains(filter)
        || cmd
            .visible_aliases()
            .iter()
            .any(|alias| alias.to_lowercase().contains(filter))
        || cmd
            .about
            .as_deref()
            .unwrap_or("")
            .to_lowercase()
            .contains(filter)
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::{TreeItem, TreeRow, tree_items, tree_rows};
    use crate::spec::{ArgSpec, CommandSpec};

    fn command(name: &str, about: Option<&str>, subcommands: Vec<CommandSpec>) -> CommandSpec {
        CommandSpec {
            name: name.to_string(),
            version: None,
            about: about.map(str::to_string),
            help: String::new(),
            args: Vec::<ArgSpec>::new(),
            subcommands,
            ..CommandSpec::default()
        }
    }

    #[test]
    fn root_only_tree_contains_root_item() {
        let root = command("tool", Some("root"), Vec::new());

        let items = tree_items(&root, &HashSet::new(), "");

        assert!(items.is_empty());
    }

    #[test]
    fn expanded_nested_subcommands_are_included() {
        let leaf = command("serve", Some("serve"), Vec::new());
        let child = command("api", Some("api"), vec![leaf]);
        let root = command("tool", Some("root"), vec![child]);
        let expanded = HashSet::from(["tool".to_string(), "tool::api".to_string()]);

        let items = tree_items(&root, &expanded, "");

        let labels = items
            .into_iter()
            .map(|item| item.label())
            .collect::<Vec<_>>();
        assert_eq!(labels, vec!["- api", "|   serve"]);
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

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].path, vec!["build".to_string()].into());
    }

    #[test]
    fn filtered_search_matches_visible_aliases() {
        let mut child = command("deploy", Some("ship release"), Vec::new());
        child.display.visible_aliases = vec!["ship".to_string()];
        child.display.display_label = "deploy (ship)".to_string();
        let root = command("tool", Some("root"), vec![child]);

        let items = tree_items(&root, &HashSet::new(), "ship");

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].name, "deploy");
    }

    #[test]
    fn tree_rows_include_subcommand_heading_before_children() {
        let mut root = command(
            "tool",
            Some("root"),
            vec![command("build", Some("build"), Vec::new())],
        );
        root.display.subcommand_help_heading = Some("Applets".to_string());

        let rows = tree_rows(&root, &HashSet::new(), "");

        assert_eq!(
            rows,
            vec![
                TreeRow::Heading {
                    title: "Applets".to_string(),
                    indent: 0,
                },
                TreeRow::Item(TreeItem {
                    name: "build".to_string(),
                    display_label: "build".to_string(),
                    version: None,
                    path: vec!["build".to_string()].into(),
                    has_children: false,
                    indent: 0,
                    expanded: false,
                }),
            ]
        );
    }
}
