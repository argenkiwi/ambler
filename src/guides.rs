//! The guides `ambler help` prints, compiled into the binary.
//!
//! ambler keeps only what applies before or outside a project: the overview, `new`, `pave`,
//! `help` and `agents`. The guides to a project's components and conventions ship with the
//! project's own CLI (the Ambler Tooling Protocol's `help` verb), next to the code that enforces
//! them, so `ambler help <topic>` hands those topics to it.

use crate::scaffold::target::Target;

/// The verbs of the command line, in the order `main` dispatches them.
pub const VERBS: &[&str] = &["new", "node", "util", "walk", "check", "pave", "help"];

/// The topics whose guides ambler prints itself.
pub const OWN: &[&str] = &["new", "pave", "help", "agents"];

/// The topics whose guides the project's own CLI prints: `<cli> help <topic>`.
pub const NATIVE: &[&str] = &[
    "node",
    "util",
    "walk",
    "check",
    "conventions",
    "node-design",
    "testing",
    "spec",
];

/// Every topic `ambler help <topic>` knows: the verbs, then the conventions behind them.
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

/// Whether `ambler help <topic>` hands the topic to the project's own CLI.
#[must_use]
pub fn is_native(topic: &str) -> bool {
    NATIVE.contains(&topic)
}

/// The guide ambler holds for `topic` in a `target` project, or the general guide for `""`.
/// [`NATIVE`] topics have none here.
#[must_use]
pub fn guide(topic: &str, target: Target) -> Option<&'static str> {
    Some(match (topic, target) {
        ("", Target::TypeScript) => include_str!("guides/ts/general.md"),
        ("", Target::Python { .. }) => include_str!("guides/py/general.md"),
        ("", Target::Rust) => include_str!("guides/general.md"),
        ("new", Target::TypeScript) => include_str!("guides/ts/new.md"),
        ("new", Target::Python { .. }) => include_str!("guides/py/new.md"),
        ("new", Target::Rust) => include_str!("guides/new.md"),
        ("pave", _) => include_str!("guides/pave.md"),
        ("help", _) => include_str!("guides/help.md"),
        ("agents", _) => concat!(
            include_str!("guides/agents.md"),
            include_str!("../templates/AGENTS.md")
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

/// What `text` tells its reader to run: each `ambler <verb>` and `ambler help <topic>` in it.
/// Headings are skipped: a guide's title is `# ambler <topic>`.
fn mentions(text: &str) -> Vec<(&str, Option<&str>)> {
    text.lines()
        .filter(|line| !line.starts_with('#'))
        .flat_map(|line| {
            line.match_indices("ambler ")
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
                Some(format!("ambler {verb}"))
            } else {
                topic
                    .filter(|topic| !TOPICS.contains(topic))
                    .map(|topic| format!("ambler help {topic}"))
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scaffold::context::{AGENTS, GLOBAL_SKILL, SKILL, render};

    const TARGETS: [Target; 3] = [
        Target::Rust,
        Target::TypeScript,
        Target::Python { package: "app" },
    ];

    #[test]
    fn every_own_topic_has_a_guide_titled_after_it_in_every_target() {
        for target in TARGETS {
            assert!(guide("", target).unwrap().starts_with("# ambler\n"));
            for topic in OWN {
                let guide = guide(topic, target).unwrap_or_else(|| panic!("no guide for {topic}"));
                assert!(
                    guide.starts_with(&format!("# ambler {topic}\n")),
                    "{topic} ({target:?})"
                );
            }
        }
    }

    #[test]
    fn every_topic_is_either_own_or_native_and_every_verb_is_a_topic() {
        for topic in TOPICS {
            assert!(OWN.contains(topic) != is_native(topic), "{topic}");
        }
        assert_eq!(OWN.len() + NATIVE.len(), TOPICS.len());
        for verb in VERBS {
            assert!(TOPICS.contains(verb), "{verb}");
        }
    }

    #[test]
    fn native_topics_have_no_guide_here() {
        for topic in NATIVE {
            assert_eq!(guide(topic, Target::Rust), None, "{topic}");
        }
    }

    #[test]
    fn a_project_reads_its_own_new_guide() {
        assert!(
            guide("new", Target::TypeScript)
                .unwrap()
                .contains("--target ts")
        );
        let python = Target::Python { package: "app" };
        assert_eq!(guide("pave", python), guide("pave", Target::Rust));
    }

    #[test]
    fn there_is_no_guide_for_an_unknown_topic() {
        assert_eq!(guide("frobnicate", Target::Rust), None);
        assert!(!is_native("frobnicate"));
    }

    #[test]
    fn finds_the_verbs_and_topics_a_text_mentions() {
        assert_eq!(
            unknown_mentions("Run `ambler node x`, `ambler help walk` and `ambler help <topic>`."),
            Vec::<String>::new()
        );
        assert_eq!(
            unknown_mentions("Run `ambler init`, then `ambler help setup`."),
            ["ambler init", "ambler help setup"]
        );
    }

    #[test]
    fn the_context_files_and_guides_only_mention_what_exists() {
        assert_eq!(
            unknown_mentions(GLOBAL_SKILL),
            Vec::<String>::new(),
            "global skill"
        );
        for target in TARGETS {
            for (name, template) in [("AGENTS.md", AGENTS), ("SKILL.md", SKILL)] {
                assert_eq!(
                    unknown_mentions(&render(template, target)),
                    Vec::<String>::new(),
                    "{name} ({target:?})"
                );
            }
            assert_eq!(
                unknown_mentions(guide("", target).unwrap()),
                Vec::<String>::new(),
                "general ({target:?})"
            );
            for topic in OWN {
                assert_eq!(
                    unknown_mentions(guide(topic, target).unwrap()),
                    Vec::<String>::new(),
                    "{topic} ({target:?})"
                );
            }
        }
    }

    #[test]
    fn the_agents_guide_ends_with_the_protocol_itself() {
        assert!(guide("agents", Target::Rust).unwrap().ends_with(AGENTS));
    }
}
