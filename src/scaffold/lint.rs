//! The conventions `amble check` verifies, whatever the target: how a violation reads, and the
//! rules on specs. Each target's module checks its own sources; rule ids are explained in
//! `amble help check`.

use super::target::Target;
use super::{py, rust, ts};
use std::collections::BTreeMap;
use std::fmt;

/// One broken convention, and where.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Violation {
    pub path: String,
    pub line: usize,
    pub rule: &'static str,
    pub message: String,
}

impl Violation {
    #[must_use]
    pub fn new(path: &str, line: usize, rule: &'static str, message: impl Into<String>) -> Self {
        Self {
            path: path.to_string(),
            line,
            rule,
            message: message.into(),
        }
    }
}

impl fmt::Display for Violation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            path,
            line,
            rule,
            message,
        } = self;
        write!(f, "{path}:{line}: {rule}: {message}")
    }
}

/// Every convention `files` (the project, by path) breaks, as `target` defines them, sorted.
#[must_use]
pub fn lint(target: Target, files: &BTreeMap<String, String>) -> Vec<Violation> {
    match target {
        Target::Rust => rust::lint::lint(files),
        Target::TypeScript => ts::lint::lint(files),
        Target::Python { .. } => py::lint::lint(files),
    }
}

const SECTIONS: [&str; 3] = ["# Program Specifications", "## Nodes", "## Shared State"];

/// The Markdown headings in `text` outside code fences: `(line, heading)`, lines from 1.
#[must_use]
pub fn headings(text: &str) -> Vec<(usize, &str)> {
    let mut fenced = false;
    text.lines()
        .enumerate()
        .filter(|(_, line)| {
            if line.trim_start().starts_with("```") {
                fenced = !fenced;
            }
            !fenced && line.starts_with('#')
        })
        .map(|(index, line)| (index + 1, line.trim_end()))
        .collect()
}

fn spec_headings<'a>(files: &'a BTreeMap<String, String>, path: &str) -> Vec<(usize, &'a str)> {
    headings(files.get(path).map(String::as_str).unwrap_or_default())
}

/// The spec at `spec` has a `###` section for exactly the nodes its walk (at `walk`) wires.
/// `ids` holds each node's heading without spaces, `CheckConfig`; `show` says how the walk names
/// the node with that heading, `Id::CheckConfig`.
#[must_use]
pub fn spec_nodes(
    files: &BTreeMap<String, String>,
    spec: &str,
    walk: &str,
    ids: &[String],
    show: fn(&str) -> String,
) -> Vec<Violation> {
    let mut found = Vec::new();
    let sections: Vec<(usize, String)> = spec_headings(files, spec)
        .into_iter()
        .filter_map(|(line, heading)| Some((line, heading.strip_prefix("### ")?.replace(' ', ""))))
        .collect();
    for (line, section) in &sections {
        if !ids.contains(section) {
            found.push(Violation::new(
                spec,
                *line,
                "spec-nodes",
                format!("this node is not wired in {walk} (no `{}`)", show(section)),
            ));
        }
    }
    for id in ids {
        if sections.iter().all(|(_, section)| section != id) {
            found.push(Violation::new(
                spec,
                1,
                "spec-nodes",
                format!("`{}` in {walk} has no `###` section here", show(id)),
            ));
        }
    }
    found
}

/// The spec at `path` has the three sections, and no others.
#[must_use]
pub fn spec_sections(files: &BTreeMap<String, String>, path: &str) -> Vec<Violation> {
    let mut found = Vec::new();
    let headings: Vec<(usize, &str)> = spec_headings(files, path)
        .into_iter()
        .filter(|(_, heading)| !heading.starts_with("###"))
        .collect();
    for (line, heading) in &headings {
        if !SECTIONS.contains(heading) {
            found.push(Violation::new(
                path,
                *line,
                "spec-sections",
                format!(
                    "a spec has only {}",
                    SECTIONS.map(|s| format!("`{s}`")).join(", ")
                ),
            ));
        }
    }
    for section in SECTIONS {
        if headings.iter().all(|(_, heading)| *heading != section) {
            found.push(Violation::new(
                path,
                1,
                "spec-sections",
                format!("missing `{section}`"),
            ));
        }
    }
    found
}
