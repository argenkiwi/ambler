//! Command-line arguments: positionals, `--flag value` (or `--flag=value`) and `--switch`.

/// The arguments of one invocation, sorted by kind.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Parsed {
    pub positionals: Vec<String>,
    flags: Vec<(String, String)>,
    switches: Vec<String>,
}

impl Parsed {
    /// Whether `--switch` was given.
    #[must_use]
    pub fn has(&self, switch: &str) -> bool {
        self.switches.iter().any(|s| s == switch)
    }

    /// Every value given for `--flag`, in order. A flag may be repeated.
    #[must_use]
    pub fn values(&self, flag: &str) -> Vec<&str> {
        self.flags
            .iter()
            .filter(|(name, _)| name == flag)
            .map(|(_, value)| value.as_str())
            .collect()
    }

    /// Like [`values`](Self::values), with each value also split on commas.
    #[must_use]
    pub fn list(&self, flag: &str) -> Vec<&str> {
        self.values(flag)
            .into_iter()
            .flat_map(|value| value.split(','))
            .map(str::trim)
            .filter(|item| !item.is_empty())
            .collect()
    }
}

/// Sorts `args` into positionals, the flags named in `valued` and the switches in `switches`.
///
/// # Errors
///
/// Returns a message naming an unknown flag, or a flag that is missing its value.
pub fn parse(args: &[String], valued: &[&str], switches: &[&str]) -> Result<Parsed, String> {
    let mut parsed = Parsed::default();
    let mut args = args.iter();
    while let Some(arg) = args.next() {
        let Some(flag) = arg.strip_prefix("--") else {
            parsed.positionals.push(arg.clone());
            continue;
        };
        let (name, inline) = match flag.split_once('=') {
            Some((name, value)) => (name, Some(value.to_string())),
            None => (flag, None),
        };
        if switches.contains(&name) && inline.is_none() {
            parsed.switches.push(name.to_string());
        } else if valued.contains(&name) {
            let value = match inline {
                Some(value) => value,
                None => args
                    .next()
                    .cloned()
                    .ok_or_else(|| format!("`--{name}` needs a value"))?,
            };
            parsed.flags.push((name.to_string(), value));
        } else {
            return Err(format!("unknown flag `--{name}`"));
        }
    }
    Ok(parsed)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(args: &[&str]) -> Vec<String> {
        args.iter().map(ToString::to_string).collect()
    }

    #[test]
    fn sorts_positionals_flags_and_switches() {
        let parsed = parse(
            &args(&["start", "--outcomes", "a, b", "--state=x:i32", "--fallible"]),
            &["outcomes", "state"],
            &["fallible"],
        )
        .unwrap();

        assert_eq!(parsed.positionals, ["start"]);
        assert_eq!(parsed.list("outcomes"), ["a", "b"]);
        assert_eq!(parsed.values("state"), ["x:i32"]);
        assert!(parsed.has("fallible"));
        assert!(!parsed.has("terminal"));
    }

    #[test]
    fn rejects_unknown_flags_and_missing_values() {
        assert_eq!(
            parse(&args(&["--nope"]), &[], &[]),
            Err("unknown flag `--nope`".to_string())
        );
        assert_eq!(
            parse(&args(&["--nodes"]), &["nodes"], &[]),
            Err("`--nodes` needs a value".to_string())
        );
    }
}
