//! The guides `amble help` prints, compiled into the binary.
//!
//! They are the single place the conventions are written down: `AGENTS.md` and the skill only
//! point here.

/// The verbs of the command line, in the order `main` dispatches them.
pub const VERBS: &[&str] = &["new", "node", "util", "walk", "check", "pave", "help"];

/// Every topic `amble help <topic>` knows: the verbs, then the conventions behind them.
pub const TOPICS: &[&str] = &[
    "new",
    "node",
    "util",
    "walk",
    "check",
    "pave",
    "help",
    "conventions",
    "node-design",
    "testing",
    "spec",
    "agents",
];

/// The guide for `topic`, or the general guide for `""`.
#[must_use]
pub fn guide(topic: &str) -> Option<&'static str> {
    Some(match topic {
        "" => include_str!("guides/general.md"),
        "new" => include_str!("guides/new.md"),
        "node" => include_str!("guides/node.md"),
        "util" => include_str!("guides/util.md"),
        "walk" => include_str!("guides/walk.md"),
        "check" => include_str!("guides/check.md"),
        "pave" => include_str!("guides/pave.md"),
        "help" => include_str!("guides/help.md"),
        "conventions" => include_str!("guides/conventions.md"),
        "node-design" => include_str!("guides/node-design.md"),
        "testing" => include_str!("guides/testing.md"),
        "spec" => include_str!("guides/spec.md"),
        "agents" => concat!(
            include_str!("guides/agents.md"),
            include_str!("../AGENTS.md")
        ),
        _ => return None,
    })
}

/// The lowercase word `text` starts with, if it starts with one.
fn word(text: &str) -> Option<&str> {
    let end = text
        .find(|c: char| !(c.is_ascii_lowercase() || c == '-'))
        .unwrap_or(text.len());
    (end > 0).then(|| &text[..end])
}

/// What `text` tells its reader to run: each `amble <verb>` and `amble help <topic>` in it.
/// Headings are skipped: a guide's title is `# amble <topic>`.
fn mentions(text: &str) -> Vec<(&str, Option<&str>)> {
    text.lines()
        .filter(|line| !line.starts_with('#'))
        .flat_map(|line| {
            line.match_indices("amble ")
                .filter_map(move |(index, matched)| {
                    let rest = &line[index + matched.len()..];
                    let verb = word(rest)?;
                    let topic = rest[verb.len()..].strip_prefix(' ').and_then(word);
                    Some((verb, topic.filter(|_| verb == "help")))
                })
        })
        .collect()
}

/// The verbs and topics `text` mentions that do not exist.
#[must_use]
pub fn unknown_mentions(text: &str) -> Vec<String> {
    mentions(text)
        .into_iter()
        .filter_map(|(verb, topic)| {
            if !VERBS.contains(&verb) {
                Some(format!("amble {verb}"))
            } else {
                topic
                    .filter(|topic| !TOPICS.contains(topic))
                    .map(|topic| format!("amble help {topic}"))
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scaffold::context::{AGENTS, SKILL};

    #[test]
    fn every_topic_has_a_guide_titled_after_it() {
        assert!(guide("").unwrap().starts_with("# amble\n"));
        for topic in TOPICS {
            let guide = guide(topic).unwrap_or_else(|| panic!("no guide for {topic}"));
            assert!(guide.starts_with(&format!("# amble {topic}\n")), "{topic}");
        }
        for verb in VERBS {
            assert!(TOPICS.contains(verb), "{verb}");
        }
    }

    #[test]
    fn there_is_no_guide_for_an_unknown_topic() {
        assert_eq!(guide("frobnicate"), None);
    }

    #[test]
    fn finds_the_verbs_and_topics_a_text_mentions() {
        assert_eq!(
            unknown_mentions("Run `amble node x`, `amble help walk` and `amble help <topic>`."),
            Vec::<String>::new()
        );
        assert_eq!(
            unknown_mentions("Run `amble init`, then `amble help setup`."),
            ["amble init", "amble help setup"]
        );
    }

    #[test]
    fn the_context_files_and_guides_only_mention_what_exists() {
        assert_eq!(unknown_mentions(AGENTS), Vec::<String>::new(), "AGENTS.md");
        assert_eq!(unknown_mentions(SKILL), Vec::<String>::new(), "SKILL.md");
        assert_eq!(
            unknown_mentions(guide("").unwrap()),
            Vec::<String>::new(),
            "general"
        );
        for topic in TOPICS {
            assert_eq!(
                unknown_mentions(guide(topic).unwrap()),
                Vec::<String>::new(),
                "{topic}"
            );
        }
    }

    #[test]
    fn the_agents_guide_ends_with_the_protocol_itself() {
        assert!(guide("agents").unwrap().ends_with(AGENTS));
    }
}
