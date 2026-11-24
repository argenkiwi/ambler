//! Naming conventions: `snake_case` files and fields, `PascalCase` types, spec headings.

const KEYWORDS: &[&str] = &[
    "abstract", "as", "async", "await", "become", "box", "break", "const", "continue", "crate",
    "do", "dyn", "else", "enum", "extern", "false", "final", "fn", "for", "gen", "if", "impl",
    "in", "let", "loop", "macro", "match", "mod", "move", "mut", "override", "priv", "pub", "ref",
    "return", "self", "static", "struct", "super", "trait", "true", "try", "type", "typeof",
    "unsafe", "unsized", "use", "virtual", "where", "while", "yield",
    // Python's, so a name is a valid module, class and field name in every target.
    "and", "assert", "class", "def", "del", "elif", "except", "finally", "from", "global", "import",
    "is", "lambda", "nonlocal", "not", "or", "pass", "raise", "with",
];

/// Whether `name` can be a module, file and field name: `snake_case` and not a keyword.
#[must_use]
pub fn is_snake(name: &str) -> bool {
    name.starts_with(|c: char| c.is_ascii_lowercase())
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
        && !KEYWORDS.contains(&name)
}

/// Whether `name` can be a type or variant name.
#[must_use]
pub fn is_pascal(name: &str) -> bool {
    name.starts_with(|c: char| c.is_ascii_uppercase())
        && name.chars().all(|c| c.is_ascii_alphanumeric())
}

fn words(name: &str) -> impl Iterator<Item = &str> {
    name.split('_').filter(|word| !word.is_empty())
}

fn capitalised(word: &str) -> String {
    let mut chars = word.chars();
    chars.next().map_or_else(String::new, |first| {
        first.to_ascii_uppercase().to_string() + chars.as_str()
    })
}

/// `check_config` → `CheckConfig`. A name that is already `PascalCase` is returned unchanged.
#[must_use]
pub fn pascal(name: &str) -> String {
    words(name).map(capitalised).collect()
}

/// `InvalidInput` → `invalid_input`.
#[must_use]
pub fn snake(name: &str) -> String {
    let mut out = String::new();
    for c in name.chars() {
        if c.is_ascii_uppercase() && !out.is_empty() {
            out.push('_');
        }
        out.push(c.to_ascii_lowercase());
    }
    out
}

/// `check_config` → `Check Config`, the heading a node has in a spec.
#[must_use]
pub fn title(name: &str) -> String {
    words(name).map(capitalised).collect::<Vec<_>>().join(" ")
}

/// `check_config` → `CHECK_CONFIG`, how a spec refers to a node in an edge.
#[must_use]
pub fn shout(name: &str) -> String {
    name.to_ascii_uppercase()
}

/// `check_config` → `check-config`, a TypeScript file name.
#[must_use]
pub fn kebab(name: &str) -> String {
    words(name).collect::<Vec<_>>().join("-")
}

/// `check_config` → `checkConfig`, a TypeScript variable or property name.
#[must_use]
pub fn camel(name: &str) -> String {
    let pascal = pascal(name);
    let mut chars = pascal.chars();
    chars.next().map_or_else(String::new, |first| {
        first.to_ascii_lowercase().to_string() + chars.as_str()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snake_names_are_lowercase_and_not_keywords() {
        assert!(is_snake("check_config"));
        assert!(is_snake("step2"));
        assert!(!is_snake("CheckConfig"));
        assert!(!is_snake("2step"));
        assert!(!is_snake("match"));
        assert!(!is_snake(""));
    }

    #[test]
    fn converts_between_the_cases() {
        assert_eq!(pascal("check_config"), "CheckConfig");
        assert_eq!(pascal("InvalidInput"), "InvalidInput");
        assert_eq!(snake("InvalidInput"), "invalid_input");
        assert_eq!(title("check_config"), "Check Config");
        assert_eq!(shout("check_config"), "CHECK_CONFIG");
        assert_eq!(kebab("check_config"), "check-config");
        assert_eq!(camel("check_config"), "checkConfig");
        assert_eq!(camel("InvalidInput"), "invalidInput");
    }
}
