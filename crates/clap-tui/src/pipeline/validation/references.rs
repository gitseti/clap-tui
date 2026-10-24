use std::fmt;

use crate::spec::ArgModel;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum ArgumentReference {
    Option(String),
    Positional(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Reference {
    Argument(ArgumentReference),
    Group(Vec<ArgumentReference>),
}

impl Reference {
    pub(super) fn parse(value: &str) -> Option<Self> {
        let value = value.trim();
        if let Some(members) = group_members(value) {
            return members
                .into_iter()
                .map(ArgumentReference::parse)
                .collect::<Option<Vec<_>>>()
                .map(Self::Group);
        }
        // Bare positional names occur inside group wrappers. A bare value in
        // another error context does not reliably identify an argument.
        if !value.starts_with(['-', '<', '[']) {
            return None;
        }
        ArgumentReference::parse(value).map(Self::Argument)
    }

    pub(super) fn resolve<'a>(&self, candidates: &[&'a ArgModel]) -> Option<Vec<&'a ArgModel>> {
        match self {
            Self::Argument(reference) => reference.resolve(candidates).map(|arg| vec![arg]),
            Self::Group(members) => members
                .iter()
                .map(|member| member.resolve(candidates))
                .collect(),
        }
    }
}

impl ArgumentReference {
    fn resolve<'a>(&self, candidates: &[&'a ArgModel]) -> Option<&'a ArgModel> {
        let mut matches = candidates.iter().copied().filter(|arg| self.matches(arg));
        let arg = matches.next()?;
        matches.next().is_none().then_some(arg)
    }

    fn parse(value: &str) -> Option<Self> {
        let value = value.trim();
        let value = value.strip_suffix("...").unwrap_or(value);
        let reference = if value.starts_with('-') {
            // Option spelling precedes its value placeholders, including
            // optional equals syntax such as --config[=<PATH>].
            let name = value
                .split(|ch: char| ch.is_whitespace() || ch == '=' || ch == '[')
                .next()?;
            Self::Option(name.to_owned())
        } else {
            let name = if let Some(inner) = value.strip_prefix('<') {
                inner.split_once('>')?.0
            } else if let Some(inner) = value.strip_prefix('[') {
                inner.split_once(']')?.0
            } else {
                value
            };
            if name.is_empty() {
                return None;
            }
            Self::Positional(name.to_owned())
        };
        Some(reference)
    }

    pub(super) fn matches(&self, arg: &ArgModel) -> bool {
        match self {
            Self::Positional(name) => arg.is_positional() && arg.display_label() == name,
            Self::Option(name) => {
                if let Some(long) = name.strip_prefix("--") {
                    !arg.is_positional()
                        && (arg.long() == Some(long)
                            || arg.visible_aliases().iter().any(|alias| alias == long))
                } else {
                    let mut chars = name.strip_prefix('-').unwrap_or_default().chars();
                    let short = chars.next();
                    !arg.is_positional()
                        && chars.next().is_none()
                        && short.is_some_and(|short| {
                            arg.short() == Some(short)
                                || arg.visible_short_aliases().contains(&short)
                        })
                }
            }
        }
    }
}

impl fmt::Display for ArgumentReference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Option(name) => f.write_str(name),
            Self::Positional(name) => write!(f, "<{name}>"),
        }
    }
}

impl fmt::Display for Reference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Argument(reference) => reference.fmt(f),
            Self::Group(members) => {
                f.write_str("<")?;
                for (index, member) in members.iter().enumerate() {
                    if index > 0 {
                        f.write_str("|")?;
                    }
                    let (ArgumentReference::Option(name) | ArgumentReference::Positional(name)) =
                        member;
                    f.write_str(name)?;
                }
                f.write_str(">")
            }
        }
    }
}

fn group_members(value: &str) -> Option<Vec<&str>> {
    let inner = value.strip_prefix('<')?.strip_suffix('>')?;
    let mut members = Vec::new();
    let mut depth = 0_usize;
    let mut start = 0;
    for (index, ch) in inner.char_indices() {
        match ch {
            '<' | '[' => depth += 1,
            '>' | ']' => depth = depth.checked_sub(1)?,
            '|' if depth == 0 => {
                members.push(&inner[start..index]);
                start = index + 1;
            }
            _ => {}
        }
    }
    if depth != 0 || (members.is_empty() && !inner.starts_with(['-', '<', '['])) {
        return None;
    }
    members.push(&inner[start..]);
    Some(members)
}
