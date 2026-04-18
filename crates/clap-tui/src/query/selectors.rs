use std::collections::{BTreeMap, HashSet};

use crate::input::ActiveTab;
use crate::spec::{ArgSpec, CommandPath, CommandSpec};

use super::{
    form::{self, OrderedArg},
    tree::{self, TreeItem, TreeRow},
};

pub(crate) fn visible_sidebar_rows(
    root: &CommandSpec,
    expanded: &HashSet<String>,
    search: &str,
) -> Vec<TreeRow> {
    tree::tree_rows(root, expanded, search)
}

pub(crate) fn visible_sidebar_items(
    root: &CommandSpec,
    expanded: &HashSet<String>,
    search: &str,
) -> Vec<TreeItem> {
    tree::tree_items(root, expanded, search)
}

pub(crate) fn visible_form_args<'a>(
    root: &'a CommandSpec,
    selected_path: &CommandPath,
    active_tab: ActiveTab,
) -> Vec<OrderedArg<'a>> {
    form::visible_args_for_path(root, selected_path, active_tab)
}

pub(crate) fn visible_form_arg_pairs<'a>(args: &[OrderedArg<'a>]) -> Vec<(usize, &'a ArgSpec)> {
    form::visible_arg_pairs(args)
}

pub(crate) fn active_form_field<'a>(
    args: &'a [OrderedArg<'a>],
    selected_arg_index: usize,
) -> Option<&'a OrderedArg<'a>> {
    args.iter()
        .find(|item| item.order_index == selected_arg_index)
}

pub(crate) fn first_invalid_visible_field<'a>(
    args: &'a [OrderedArg<'a>],
    field_errors: &BTreeMap<String, String>,
) -> Option<&'a OrderedArg<'a>> {
    args.iter()
        .find(|item| field_errors.contains_key(&item.arg.id))
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, HashSet};

    use clap::Command;

    use super::{
        active_form_field, first_invalid_visible_field, visible_form_args, visible_sidebar_items,
    };
    use crate::input::ActiveTab;
    use crate::spec::{ArgKind, ArgSpec, CommandPath, CommandSpec, ValueCardinality};

    fn arg(id: &str, name: &str) -> ArgSpec {
        ArgSpec {
            id: id.to_string(),
            display_name: name.to_string(),
            help: None,
            required: false,
            kind: ArgKind::Option,
            default_values: Vec::new(),
            choices: Vec::new(),
            position: None,
            value_cardinality: ValueCardinality::One,
            value_hint: None,
            ..ArgSpec::default()
        }
    }

    #[test]
    fn active_form_field_resolves_selected_visible_arg() {
        let command = CommandSpec {
            name: "tool".to_string(),
            version: None,
            about: None,
            help: String::new(),
            args: vec![arg("alpha", "--alpha"), arg("beta", "--beta")],
            subcommands: Vec::new(),
            ..CommandSpec::default()
        };
        let args = visible_form_args(&command, &CommandPath::default(), ActiveTab::Inputs);

        let active = active_form_field(&args, 1).expect("selected field");

        assert_eq!(active.arg.id, "beta");
    }

    #[test]
    fn visible_sidebar_items_follow_search_projection() {
        let root = CommandSpec::from_command(&Command::new("tool").subcommand(
            Command::new("release").subcommand(Command::new("deploy").visible_alias("ship")),
        ));

        let items = visible_sidebar_items(&root, &HashSet::new(), "ship");

        assert_eq!(items.len(), 2);
        assert_eq!(items[0].path.as_slice(), &["release".to_string()]);
        assert_eq!(
            items[1].path.as_slice(),
            &["release".to_string(), "deploy".to_string()]
        );
    }

    #[test]
    fn first_invalid_visible_field_follows_form_order() {
        let command = CommandSpec {
            name: "tool".to_string(),
            version: None,
            about: None,
            help: String::new(),
            args: vec![arg("alpha", "--alpha"), arg("beta", "--beta")],
            subcommands: Vec::new(),
            ..CommandSpec::default()
        };
        let args = visible_form_args(&command, &CommandPath::default(), ActiveTab::Inputs);
        let field_errors = BTreeMap::from([
            ("beta".to_string(), "Required argument".to_string()),
            ("alpha".to_string(), "Required argument".to_string()),
        ]);

        let invalid = first_invalid_visible_field(&args, &field_errors).expect("invalid field");

        assert_eq!(invalid.arg.id, "alpha");
    }
}
