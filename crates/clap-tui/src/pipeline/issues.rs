#[cfg(test)]
use std::collections::BTreeMap;

use crate::query::form::OrderedArg;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ValidationTarget {
    Arguments(Vec<String>),
    CommandSelection,
    None,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ValidationIssue {
    pub(crate) message: String,
    pub(crate) target: ValidationTarget,
}

impl ValidationIssue {
    pub(crate) fn arguments(message: impl Into<String>, ids: Vec<String>) -> Self {
        Self {
            message: message.into(),
            target: if ids.is_empty() {
                ValidationTarget::None
            } else {
                ValidationTarget::Arguments(ids)
            },
        }
    }

    pub(crate) fn argument_ids(&self) -> &[String] {
        match &self.target {
            ValidationTarget::Arguments(ids) => ids,
            _ => &[],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct ValidationState {
    pub(crate) issues: Vec<ValidationIssue>,
}

impl ValidationState {
    pub(crate) fn is_valid(&self) -> bool {
        self.issues.is_empty()
    }

    pub(crate) fn summary(&self) -> Option<String> {
        self.issues.first().map(|issue| {
            let message = issue.message.lines().collect::<Vec<_>>().join(" ");
            if self.issues.len() == 1 {
                message
            } else {
                format!("{message} (+{} more)", self.issues.len() - 1)
            }
        })
    }

    pub(crate) fn field_error(&self, id: &str) -> Option<&str> {
        self.issues
            .iter()
            .find(|issue| issue.argument_ids().iter().any(|member| member == id))
            .map(|issue| issue.message.as_str())
    }

    pub(crate) fn inline_error(&self, id: &str) -> Option<&str> {
        self.issues
            .iter()
            .find(|issue| {
                issue
                    .argument_ids()
                    .first()
                    .is_some_and(|first| first == id)
            })
            .map(|issue| issue.message.as_str())
    }

    #[cfg(test)]
    pub(crate) fn field_errors(&self) -> BTreeMap<String, String> {
        self.issues
            .iter()
            .flat_map(|issue| {
                issue
                    .argument_ids()
                    .iter()
                    .map(|id| (id.clone(), issue.message.clone()))
            })
            .collect()
    }

    pub(crate) fn has_target(&self) -> bool {
        self.issues
            .iter()
            .any(|issue| !matches!(issue.target, ValidationTarget::None))
    }

    pub(crate) fn order_by_form(&mut self, args: &[OrderedArg<'_>]) {
        let rank = |id: &str| {
            args.iter()
                .position(|item| item.arg.id == id)
                .unwrap_or(usize::MAX)
        };
        for issue in &mut self.issues {
            if let ValidationTarget::Arguments(ids) = &mut issue.target {
                ids.sort_by_key(|id| rank(id));
            }
        }
        self.issues.sort_by_key(|issue| match &issue.target {
            ValidationTarget::Arguments(ids) => ids.first().map_or(usize::MAX, |id| rank(id)),
            ValidationTarget::CommandSelection => usize::MAX - 1,
            ValidationTarget::None => usize::MAX,
        });
    }
}
