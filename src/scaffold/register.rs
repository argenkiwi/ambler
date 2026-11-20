//! Edits to a `mod.rs`: registering a module in alphabetical order, appending an item.

/// The module a `pub mod <name>;` line declares, if `line` is one.
#[must_use]
pub fn declared(line: &str) -> Option<&str> {
    line.trim().strip_prefix("pub mod ")?.strip_suffix(';')
}

/// Adds `pub mod <name>;` to `source`, keeping the declarations alphabetical.
///
/// Returns `None` if the module is already registered.
#[must_use]
pub fn register(source: &str, name: &str) -> Option<String> {
    let mut lines: Vec<&str> = source.lines().collect();
    if lines.iter().any(|line| declared(line) == Some(name)) {
        return None;
    }
    let declaration = format!("pub mod {name};");
    let after = lines
        .iter()
        .rposition(|line| declared(line).is_some_and(|other| other < name));
    let before = lines.iter().position(|line| declared(line).is_some());
    match (after, before) {
        (Some(index), _) => lines.insert(index + 1, &declaration),
        (None, Some(index)) => lines.insert(index, &declaration),
        (None, None) => {
            // No declarations yet: go below the module's leading `//!` doc.
            let header = lines
                .iter()
                .take_while(|line| line.starts_with("//!"))
                .count();
            let rest: Vec<&str> = lines[header..]
                .iter()
                .copied()
                .skip_while(|line| line.trim().is_empty())
                .collect();
            lines.truncate(header);
            if header > 0 {
                lines.push("");
            }
            lines.push(&declaration);
            if !rest.is_empty() {
                lines.push("");
                lines.extend(rest);
            }
        }
    }
    Some(lines.join("\n") + "\n")
}

/// Adds `use <path>;` to `source`: among its `use` lines in order, or else in a block of its own
/// below the module declarations (or the module's `//!` doc). Unchanged if it is already there.
#[must_use]
pub fn add_use(source: &str, path: &str) -> String {
    let import = format!("use {path};");
    let mut lines: Vec<&str> = source.lines().collect();
    if lines.iter().any(|line| line.trim() == import) {
        return source.to_string();
    }
    let is_use = |line: &&str| line.starts_with("use ");
    let after_use = lines
        .iter()
        .rposition(|line| is_use(line) && *line < import.as_str());
    let first_use = lines.iter().position(is_use);
    match (after_use, first_use) {
        (Some(index), _) => lines.insert(index + 1, &import),
        (None, Some(index)) => lines.insert(index, &import),
        (None, None) => {
            let after = lines
                .iter()
                .rposition(|line| declared(line).is_some())
                .map(|index| index + 1)
                .unwrap_or_else(|| {
                    lines
                        .iter()
                        .take_while(|line| line.starts_with("//!"))
                        .count()
                });
            let mut block = vec![import.as_str()];
            if after > 0 {
                block.insert(0, "");
            }
            if lines.get(after).is_some_and(|line| !line.trim().is_empty()) {
                block.push("");
            }
            lines.splice(after..after, block);
        }
    }
    lines.join("\n") + "\n"
}

/// Appends `item` to `source` after a blank line.
#[must_use]
pub fn append(source: &str, item: &str) -> String {
    format!("{}\n\n{}\n", source.trim_end(), item.trim_end())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registers_in_alphabetical_order() {
        let source = "pub mod count;\npub mod stop;\n\npub trait HasCount {}\n";

        assert_eq!(
            register(source, "start").unwrap(),
            "pub mod count;\npub mod start;\npub mod stop;\n\npub trait HasCount {}\n"
        );
        assert_eq!(
            register(source, "ask").unwrap(),
            "pub mod ask;\npub mod count;\npub mod stop;\n\npub trait HasCount {}\n"
        );
        assert_eq!(
            register(source, "write").unwrap(),
            "pub mod count;\npub mod stop;\npub mod write;\n\npub trait HasCount {}\n"
        );
    }

    #[test]
    fn first_registration_goes_below_the_module_doc() {
        assert_eq!(
            register("//! Nodes.\n", "start").unwrap(),
            "//! Nodes.\n\npub mod start;\n"
        );
        assert_eq!(register("", "start").unwrap(), "pub mod start;\n");
        assert_eq!(
            register("//! Nodes.\n\npub trait HasCount {}\n", "start").unwrap(),
            "//! Nodes.\n\npub mod start;\n\npub trait HasCount {}\n"
        );
    }

    #[test]
    fn does_not_register_twice() {
        assert_eq!(register("pub mod start;\n", "start"), None);
    }

    #[test]
    fn adds_an_import_among_the_others_in_order() {
        let source = "pub mod start;\n\nuse crate::a::A;\nuse std::io;\n\npub trait HasA {}\n";

        assert_eq!(
            add_use(source, "crate::b::B"),
            "pub mod start;\n\nuse crate::a::A;\nuse crate::b::B;\nuse std::io;\n\n\
             pub trait HasA {}\n"
        );
        assert_eq!(
            add_use(source, "anyhow::Result"),
            "pub mod start;\n\nuse anyhow::Result;\nuse crate::a::A;\nuse std::io;\n\n\
             pub trait HasA {}\n"
        );
        assert_eq!(add_use(source, "std::io"), source);
    }

    #[test]
    fn the_first_import_goes_below_the_module_declarations() {
        assert_eq!(
            add_use(
                "//! Nodes.\n\npub mod start;\n\npub trait HasA {}\n",
                "crate::a::A"
            ),
            "//! Nodes.\n\npub mod start;\n\nuse crate::a::A;\n\npub trait HasA {}\n"
        );
        assert_eq!(
            add_use("//! Nodes.\n", "crate::a::A"),
            "//! Nodes.\n\nuse crate::a::A;\n"
        );
        assert_eq!(add_use("", "crate::a::A"), "use crate::a::A;\n");
    }

    #[test]
    fn appends_after_a_blank_line() {
        assert_eq!(
            append("pub mod start;\n", "pub trait HasCount {}\n"),
            "pub mod start;\n\npub trait HasCount {}\n"
        );
    }
}
