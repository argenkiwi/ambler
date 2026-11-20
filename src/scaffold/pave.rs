//! What `amble pave` decides and says: the settings, the next task, the guards that bound the
//! loop and the prompt for each task. No I/O: the nodes read the files and pass them in.

use super::register::declared;
use super::source::{node_info, state_traits};
use super::{args, lint, names};

pub const USAGE: &str = "usage: amble pave <spec> [--agent '<command>'] [--max-iterations <n>] \
                         [--max-repairs <n>] [--dry-run]";
/// The environment variable that holds the agent command when `--agent` is not given.
pub const AGENT_VAR: &str = "AMBLE_AGENT";
/// The environment variable that overrides where pave's own checks build.
pub const TARGET_VAR: &str = "AMBLE_PAVE_TARGET";
pub const DEFAULT_TARGET: &str = "target/amble-pave";
/// The name of the check log, kept in the git folder so it is never committed.
pub const LOG_NAME: &str = "amble-pave-check.log";
/// Unfinished code, as `amble node` and `amble walk` leave it.
pub const UNFINISHED: &str = concat!("todo", "!(");
/// How many lines of the check output a prompt carries, and how many a failure prints.
pub const PROMPT_LINES: usize = 80;
pub const FAILURE_LINES: usize = 15;
/// How many times in a row the same task may be chosen while the check passes.
const REPEATS: usize = 3;

/// What `amble pave` was asked to do.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Settings {
    /// The spec's name: `specs/<spec>.md`, and the walk `src/walks/<spec>.rs`.
    pub spec: String,
    /// The command that runs one agent turn; the prompt is added as its last argument.
    pub agent: Option<String>,
    pub max_iterations: usize,
    pub max_repairs: usize,
    pub dry_run: bool,
    /// The cargo target folder for pave's own checks.
    pub target: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            spec: String::new(),
            agent: None,
            max_iterations: 30,
            max_repairs: 3,
            dry_run: false,
            target: DEFAULT_TARGET.to_string(),
        }
    }
}

impl Settings {
    #[must_use]
    pub fn spec_path(&self) -> String {
        format!("specs/{}.md", self.spec)
    }

    #[must_use]
    pub fn walk_path(&self) -> String {
        format!("src/walks/{}.rs", self.spec)
    }
}

/// Reads the settings from the arguments after the verb, and from the agent and target
/// variables of the environment.
///
/// # Errors
///
/// Returns a message if the arguments are invalid, or if no agent is given outside a dry run.
pub fn settings(
    args: &[String],
    agent: Option<String>,
    target: Option<String>,
) -> Result<Settings, String> {
    let parsed = args::parse(
        args,
        &["agent", "max-iterations", "max-repairs"],
        &["dry-run"],
    )
    .map_err(|e| format!("{e}\n{USAGE}"))?;
    let [spec] = parsed.positionals.as_slice() else {
        return Err(USAGE.to_string());
    };
    if !names::is_snake(spec) {
        return Err(format!("`{spec}` is not a walk name: use snake_case"));
    }
    let defaults = Settings::default();
    let number = |flag: &str, default: usize| match parsed.values(flag).last() {
        None => Ok(default),
        Some(value) => value
            .parse()
            .map_err(|_| format!("`--{flag}` takes a whole number, not `{value}`")),
    };
    let max_iterations = number("max-iterations", defaults.max_iterations)?;
    let max_repairs = number("max-repairs", defaults.max_repairs)?;
    let dry_run = parsed.has("dry-run");
    let agent = parsed
        .values("agent")
        .last()
        .map(|agent| (*agent).to_string())
        .or(agent)
        .filter(|agent| !agent.trim().is_empty());
    if agent.is_none() && !dry_run {
        return Err(format!(
            "say which agent to run: `--agent '<command>'` or {AGENT_VAR}, for example \
             `--agent 'claude -p'`. The prompt is added as its last argument.\n{USAGE}"
        ));
    }
    Ok(Settings {
        spec: spec.clone(),
        agent,
        max_iterations,
        max_repairs,
        dry_run,
        target: target
            .filter(|target| !target.is_empty())
            .unwrap_or(defaults.target),
    })
}

/// One narrow thing for the agent to do.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Task {
    /// Create the node, or finish it if its file exists.
    Node(String),
    /// Wire the spec's nodes into its walk, or finish the walk if its file exists.
    Walk,
    /// Make a failing check pass.
    Repair,
    /// The walk exists and the check passes.
    Done,
}

impl Task {
    /// How the task is named in logs and checkpoints: `node ask`, `walk greet`, `repair`.
    #[must_use]
    pub fn label(&self, spec: &str) -> String {
        match self {
            Self::Node(name) => format!("node {name}"),
            Self::Walk => format!("walk {spec}"),
            Self::Repair => "repair".to_string(),
            Self::Done => "done".to_string(),
        }
    }
}

/// The loop's settings, what the last check said and the counters that bound it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Paving {
    pub settings: Settings,
    /// Where the full output of the last check is kept.
    pub log: String,
    /// Whether the last check passed.
    pub passed: bool,
    /// The end of the last check's output.
    pub output: String,
    /// The task being worked on.
    pub task: Option<Task>,
    /// The label of the last task chosen, which names the next checkpoint.
    pub label: String,
    /// Failed checks in a row after an agent run.
    pub repairs: usize,
    /// Whether the one roll back is used.
    pub rolled_back: bool,
    /// How many times in a row the same task was chosen while the check passed.
    pub repeats: usize,
    pub iterations: usize,
}

impl Default for Paving {
    fn default() -> Self {
        Self {
            settings: Settings::default(),
            log: String::new(),
            passed: false,
            output: String::new(),
            task: None,
            label: "startup".to_string(),
            repairs: 0,
            rolled_back: false,
            repeats: 0,
            iterations: 0,
        }
    }
}

/// What the guards make of the next task.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Decision {
    /// Give the task to the agent.
    Assigned,
    /// The spec is implemented.
    Done,
    /// The repair budget is spent: roll back to the last checkpoint and retry.
    Exhausted,
    /// Stop: the reason.
    Stuck(String),
}

impl Paving {
    /// Applies the guards to `task`, as chosen from the project, and records it if it stands.
    pub fn decide(&mut self, task: Task) -> Decision {
        if task == Task::Done {
            return Decision::Done;
        }
        let label = task.label(&self.settings.spec);
        if !self.passed && self.repairs >= self.settings.max_repairs {
            if self.rolled_back {
                return Decision::Stuck(format!(
                    "still failing after rolling back once; see `git stash list`. Full check \
                     output: {}",
                    self.log
                ));
            }
            return Decision::Exhausted;
        }
        if self.passed && label == self.label {
            self.repeats += 1;
            if self.repeats >= REPEATS {
                return Decision::Stuck(format!("no progress on `{label}` after {REPEATS} runs"));
            }
        } else {
            self.repeats = 0;
        }
        self.label = label;
        self.iterations += 1;
        if self.iterations > self.settings.max_iterations {
            return Decision::Stuck(format!(
                "reached --max-iterations {}",
                self.settings.max_iterations
            ));
        }
        self.task = Some(task);
        Decision::Assigned
    }
}

/// The last `count` lines of `text`.
#[must_use]
pub fn tail(text: &str, count: usize) -> String {
    let lines: Vec<&str> = text.lines().collect();
    lines[lines.len().saturating_sub(count)..].join("\n")
}

/// A heading's node name: `### Read Move` → `read_move`.
fn node_name(heading: &str) -> String {
    let mut name = String::new();
    for c in heading.chars() {
        if c.is_ascii_alphanumeric() {
            name.push(c.to_ascii_lowercase());
        } else if !name.is_empty() && !name.ends_with('_') {
            name.push('_');
        }
    }
    name.trim_end_matches('_').to_string()
}

/// The spec's nodes, in order, from its `###` headings.
#[must_use]
pub fn spec_nodes(spec: &str) -> Vec<String> {
    lint::headings(spec)
        .into_iter()
        .filter_map(|(_, heading)| heading.strip_prefix("### "))
        .map(node_name)
        .collect()
}

/// The lines of `spec` from the heading at `line` (from 1) up to the next heading of `level` or
/// higher, outside code fences.
fn section(spec: &str, start: usize, level: usize) -> String {
    let end = lint::headings(spec)
        .into_iter()
        .find(|(line, heading)| {
            *line > start && heading.chars().take_while(|c| *c == '#').count() <= level
        })
        .map_or(usize::MAX, |(line, _)| line);
    spec.lines()
        .enumerate()
        .filter(|(index, _)| (start..end).contains(&(index + 1)))
        .map(|(_, line)| line)
        .collect::<Vec<_>>()
        .join("\n")
        .trim_end()
        .to_string()
}

/// The `###` section of the spec that describes the node `name`.
#[must_use]
pub fn node_section(spec: &str, name: &str) -> String {
    lint::headings(spec)
        .into_iter()
        .find(|(_, heading)| {
            heading
                .strip_prefix("### ")
                .is_some_and(|h| node_name(h) == name)
        })
        .map(|(line, _)| section(spec, line, 3))
        .unwrap_or_default()
}

/// The spec's `## Shared State` section.
#[must_use]
pub fn shared_state(spec: &str) -> String {
    lint::headings(spec)
        .into_iter()
        .find(|(_, heading)| *heading == "## Shared State")
        .map(|(line, _)| section(spec, line, 2))
        .unwrap_or_default()
}

/// The next task: an unfinished node first, then (if the check passes) the next missing node,
/// then the walk; a failing check with nothing unfinished is a repair.
///
/// `read` gives the contents of a project file, or `None` if it does not exist.
#[must_use]
pub fn select_task(
    nodes: &[String],
    walk: &str,
    passed: bool,
    read: impl Fn(&str) -> Option<String>,
) -> Task {
    let unfinished = |source: &str| source.contains(UNFINISHED);
    for name in nodes {
        match read(&format!("src/nodes/{name}.rs")) {
            None if passed => return Task::Node(name.clone()),
            None => break,
            Some(source) if unfinished(&source) => return Task::Node(name.clone()),
            Some(_) => {}
        }
    }
    match read(walk) {
        Some(source) if !passed && unfinished(&source) => Task::Walk,
        _ if !passed => Task::Repair,
        Some(_) => Task::Done,
        None => Task::Walk,
    }
}

/// `- ask: Answered, Invalid`, or `- stop: terminal (no outcomes)`, for the node in `source`.
#[must_use]
pub fn outcomes_line(name: &str, source: &str) -> String {
    match node_info(name, source) {
        Ok(info) => match info.outcome {
            None => format!("- {name}: terminal (no outcomes)"),
            Some(outcome) => {
                let variants: Vec<String> = outcome.variants.into_iter().map(|v| v.name).collect();
                format!("- {name}: {}", variants.join(", "))
            }
        },
        Err(error) => format!("- {name}: could not be read ({error})"),
    }
}

/// What every prompt may draw on.
#[derive(Clone, Copy, Debug)]
pub struct Brief<'a> {
    pub settings: &'a Settings,
    /// The contents of the spec.
    pub spec: &'a str,
    /// The contents of `src/nodes/mod.rs`.
    pub modules: &'a str,
    /// Whether the last check passed, and the end of its output.
    pub passed: bool,
    pub output: &'a str,
}

impl Brief<'_> {
    fn check_output(&self) -> String {
        if self.passed {
            String::new()
        } else {
            format!(
                "\n## Last `amble check` output\n```\n{}\n```\n",
                tail(self.output, PROMPT_LINES)
            )
        }
    }

    fn list(items: Vec<String>) -> String {
        if items.is_empty() {
            "none".to_string()
        } else {
            items.join(", ")
        }
    }
}

/// The prompt to create or finish the node `name`. `exists` says whether its file does.
#[must_use]
pub fn node_prompt(brief: &Brief, name: &str, exists: bool) -> String {
    let spec_path = brief.settings.spec_path();
    let nodes = Brief::list(
        brief
            .modules
            .lines()
            .filter_map(declared)
            .map(str::to_string)
            .collect(),
    );
    let traits = Brief::list(
        state_traits(brief.modules)
            .unwrap_or_default()
            .into_iter()
            .map(|item| item.name)
            .collect(),
    );
    let file = if exists {
        format!(
            "`src/nodes/{name}.rs` already exists (an earlier run was interrupted). Do NOT run \
             `amble node` again: read the file and finish it."
        )
    } else {
        format!("`src/nodes/{name}.rs` does not exist yet.")
    };
    format!(
        "You are implementing ONE node of an Ambler project: `{name}` from `{spec_path}`.\n\
         Do only this node. Do not create other nodes, do not create or edit the walk, do not \
         touch `src/engine.rs`.\n\
         \n\
         ## The node, as specified\n\
         {section}\n\
         \n\
         {state}\n\
         \n\
         ## Current project\n\
         - Existing nodes: {nodes}\n\
         - Existing state traits in src/nodes/mod.rs: {traits} (reuse these with `--state \
         <field>` instead of redeclaring them)\n\
         - {file}\n\
         \n\
         ## Steps\n\
         1. Run `amble help node-design` and `amble help testing`.\n\
         2. If the file does not exist: `amble node {name} (--outcomes <a,b> | --terminal) \
         [--fallible] [--state <field>[:<Type>]]...`. Outcomes name what happened (not the \
         next node); the spec's transitions tell you how many there are.\n\
         3. Give `Utils` one method per side effect and implement each in `SystemUtils`.\n\
         4. Write the tests, run `cargo test nodes::{name}::` and see them fail with `not yet \
         implemented`.\n\
         5. Implement `run` without changing the tests, until `cargo test nodes::{name}::` \
         passes.\n\
         6. Replace the TODO doc comments, then run `amble check` and fix anything it reports \
         about this node.\n\
         {output}",
        section = node_section(brief.spec, name),
        state = shared_state(brief.spec),
        output = brief.check_output(),
    )
}

/// The prompt to wire the walk. `outcomes` has one [`outcomes_line`] per node; `exists` says
/// whether the walk's file does.
#[must_use]
pub fn walk_prompt(brief: &Brief, outcomes: &[String], exists: bool) -> String {
    let Settings { spec, .. } = brief.settings;
    let spec_path = brief.settings.spec_path();
    let walk_path = brief.settings.walk_path();
    let wire = if exists {
        format!(
            "`{walk_path}` already exists (an earlier run was interrupted). Do NOT run `amble \
             walk` again: replace its remaining `todo!(\"wire …\")` arms following the spec's \
             transitions."
        )
    } else {
        format!(
            "Run `amble walk {spec} --nodes {nodes} --edge <node>.<Outcome>=<node|halt> ...`, \
             with one --edge per transition in the spec (a terminal node needs none).",
            nodes = spec_nodes(brief.spec).join(","),
        )
    };
    format!(
        "All nodes of `{spec_path}` are implemented. Your task is to wire them into the walk \
         `{spec}`, and nothing else.\n\
         \n\
         ## The spec\n\
         {text}\n\
         \n\
         ## Node outcomes (from src/nodes/)\n\
         {outcomes}\n\
         \n\
         ## Steps\n\
         1. Run `amble help walk`.\n\
         2. {wire}\n\
         3. Finish the wiring test in `{walk_path}` so it walks the main paths of the spec.\n\
         4. Keep `{spec_path}` true: its `###` headings must match the walk's nodes.\n\
         5. Run `amble check` and fix everything it reports.\n\
         {output}",
        text = brief.spec.trim_end(),
        outcomes = outcomes.join("\n"),
        output = brief.check_output(),
    )
}

/// The prompt to make a failing check pass. `status` lists the uncommitted changes.
#[must_use]
pub fn repair_prompt(brief: &Brief, status: &str) -> String {
    format!(
        "`amble check` is failing in this Ambler project. Your only task is to make it pass \
         again.\n\
         Fix the code, not the conventions: do not delete tests or weaken what passing tests \
         assert, do not touch `src/engine.rs`, and do not start new nodes or walks.\n\
         Run `amble help check` if a rule is unclear, and re-run `amble check` until it passes.\n\
         \n\
         ## Last `amble check` output\n\
         ```\n{output}\n```\n\
         \n\
         ## Uncommitted changes (since the last passing checkpoint)\n\
         ```\n{status}\n```\n",
        output = tail(brief.output, PROMPT_LINES),
        status = status.trim_end(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::guides::unknown_mentions;
    use std::collections::BTreeMap;

    const SPEC: &str = "# Program Specifications\n\
                        \n\
                        Plays a game.\n\
                        \n\
                        ## Nodes\n\
                        \n\
                        ### Read Move\n\
                        - Initial node.\n\
                        - If the move is valid, it transitions to `REPORT`.\n\
                        \n\
                        ```\n\
                        ### not a heading\n\
                        ```\n\
                        \n\
                        ### Report\n\
                        - Prints the board and terminates.\n\
                        \n\
                        ## Shared State\n\
                        \n\
                        The shared state consists of:\n\
                        - `board`: the cells.\n";

    const MODULES: &str = "//! Nodes.\n\npub mod read_move;\n\n\
                           pub trait HasBoard {\n    fn board(&self) -> &Vec<u8>;\n    \
                           fn board_mut(&mut self) -> &mut Vec<u8>;\n}\n";

    fn args(args: &[&str]) -> Vec<String> {
        args.iter().map(ToString::to_string).collect()
    }

    fn settings_for(spec: &str) -> Settings {
        Settings {
            spec: spec.to_string(),
            agent: Some("claude -p".to_string()),
            ..Settings::default()
        }
    }

    #[test]
    fn reads_the_settings_from_the_arguments_and_environment() {
        let settings = settings(
            &args(&["game", "--max-iterations", "5", "--max-repairs=1"]),
            Some("pi -p".to_string()),
            Some("/workspace/target".to_string()),
        )
        .unwrap();

        assert_eq!(
            settings,
            Settings {
                spec: "game".to_string(),
                agent: Some("pi -p".to_string()),
                max_iterations: 5,
                max_repairs: 1,
                dry_run: false,
                target: "/workspace/target".to_string(),
            }
        );
        assert_eq!(settings.spec_path(), "specs/game.md");
        assert_eq!(settings.walk_path(), "src/walks/game.rs");
    }

    #[test]
    fn the_agent_flag_wins_over_the_environment() {
        let settings = settings(
            &args(&["game", "--agent", "claude -p"]),
            Some("pi -p".to_string()),
            None,
        )
        .unwrap();

        assert_eq!(settings.agent.as_deref(), Some("claude -p"));
        assert_eq!(settings.target, DEFAULT_TARGET);
    }

    #[test]
    fn a_dry_run_needs_no_agent() {
        let settings = settings(&args(&["game", "--dry-run"]), None, None).unwrap();

        assert!(settings.dry_run);
        assert_eq!(settings.agent, None);
    }

    #[test]
    fn rejects_invalid_arguments() {
        for (args_given, reason) in [
            (&[][..], USAGE),
            (&["a", "b"][..], USAGE),
            (&["Game"][..], "snake_case"),
            (&["game", "--max-repairs", "x"][..], "whole number"),
            (&["game", "--nope"][..], "unknown flag"),
            (&["game"][..], AGENT_VAR),
            (&["game", "--agent", " "][..], AGENT_VAR),
        ] {
            let error = settings(&args(args_given), None, None).unwrap_err();
            assert!(error.contains(reason), "{args_given:?}: {error}");
        }
    }

    #[test]
    fn finds_the_nodes_and_sections_of_a_spec() {
        assert_eq!(spec_nodes(SPEC), ["read_move", "report"]);
        assert_eq!(
            node_section(SPEC, "read_move"),
            "### Read Move\n- Initial node.\n- If the move is valid, it transitions to \
             `REPORT`.\n\n```\n### not a heading\n```"
        );
        assert_eq!(
            node_section(SPEC, "report"),
            "### Report\n- Prints the board and terminates."
        );
        assert_eq!(node_section(SPEC, "missing"), "");
        assert_eq!(
            shared_state(SPEC),
            "## Shared State\n\nThe shared state consists of:\n- `board`: the cells."
        );
    }

    #[test]
    fn names_nodes_after_their_headings() {
        assert_eq!(node_name("Read Move"), "read_move");
        assert_eq!(node_name("Check  Config (v2)"), "check_config_v2");
    }

    fn select(files: &[(&str, &str)], passed: bool) -> Task {
        let files: BTreeMap<String, String> = files
            .iter()
            .map(|(path, source)| ((*path).to_string(), (*source).to_string()))
            .collect();
        select_task(
            &spec_nodes(SPEC),
            "src/walks/game.rs",
            passed,
            |path: &str| files.get(path).cloned(),
        )
    }

    #[test]
    fn chooses_the_first_missing_node_while_the_check_passes() {
        assert_eq!(select(&[], true), Task::Node("read_move".to_string()));
        assert_eq!(
            select(&[("src/nodes/read_move.rs", "done")], true),
            Task::Node("report".to_string())
        );
    }

    #[test]
    fn resumes_an_unfinished_node_whether_or_not_the_check_passes() {
        let unfinished = format!("fn run() {{ {UNFINISHED}) }}");
        let files = [
            ("src/nodes/read_move.rs", "done"),
            ("src/nodes/report.rs", unfinished.as_str()),
        ];

        assert_eq!(select(&files, true), Task::Node("report".to_string()));
        assert_eq!(select(&files, false), Task::Node("report".to_string()));
    }

    #[test]
    fn repairs_a_failing_check_before_starting_a_new_node() {
        assert_eq!(select(&[], false), Task::Repair);
        assert_eq!(
            select(&[("src/nodes/read_move.rs", "done")], false),
            Task::Repair
        );
    }

    #[test]
    fn wires_the_walk_once_every_node_is_done() {
        let done = [
            ("src/nodes/read_move.rs", "done"),
            ("src/nodes/report.rs", "done"),
        ];
        let unfinished = format!("{UNFINISHED}\"wire\")");
        let mut wiring = done.to_vec();
        wiring.push(("src/walks/game.rs", unfinished.as_str()));
        let mut wired = done.to_vec();
        wired.push(("src/walks/game.rs", "done"));

        assert_eq!(select(&done, true), Task::Walk);
        assert_eq!(select(&wiring, false), Task::Walk);
        assert_eq!(select(&wired, false), Task::Repair);
        assert_eq!(select(&wired, true), Task::Done);
    }

    fn paving() -> Paving {
        Paving {
            settings: Settings {
                max_iterations: 4,
                max_repairs: 2,
                ..settings_for("game")
            },
            log: ".git/amble-pave-check.log".to_string(),
            passed: true,
            ..Paving::default()
        }
    }

    #[test]
    fn assigns_a_task_and_counts_the_iteration() {
        let mut paving = paving();

        assert_eq!(paving.decide(Task::Walk), Decision::Assigned);
        assert_eq!(paving.task, Some(Task::Walk));
        assert_eq!(paving.label, "walk game");
        assert_eq!(paving.iterations, 1);
        assert_eq!(paving.decide(Task::Done), Decision::Done);
    }

    #[test]
    fn rolls_back_once_the_repair_budget_is_spent_then_gives_up() {
        let mut paving = Paving {
            passed: false,
            repairs: 2,
            ..paving()
        };

        assert_eq!(paving.decide(Task::Repair), Decision::Exhausted);
        paving.rolled_back = true;
        let Decision::Stuck(reason) = paving.decide(Task::Repair) else {
            panic!("expected to stop");
        };
        assert!(reason.contains("git stash list"));
        assert!(reason.contains(".git/amble-pave-check.log"));
    }

    #[test]
    fn stops_when_the_same_task_makes_no_progress() {
        let mut paving = paving();
        let task = Task::Node("report".to_string());

        for _ in 0..REPEATS {
            assert_eq!(paving.decide(task.clone()), Decision::Assigned);
        }
        assert_eq!(
            paving.decide(task),
            Decision::Stuck("no progress on `node report` after 3 runs".to_string())
        );
    }

    #[test]
    fn a_failing_check_does_not_count_as_no_progress() {
        let mut paving = Paving {
            passed: false,
            ..paving()
        };

        for _ in 0..4 {
            assert_eq!(paving.decide(Task::Repair), Decision::Assigned);
        }
        assert_eq!(paving.repeats, 0);
    }

    #[test]
    fn stops_at_the_iteration_limit() {
        let mut paving = paving();
        for name in ["a", "b", "c", "d"] {
            assert_eq!(paving.decide(Task::Node(name.into())), Decision::Assigned);
        }

        assert_eq!(
            paving.decide(Task::Walk),
            Decision::Stuck("reached --max-iterations 4".to_string())
        );
    }

    #[test]
    fn keeps_the_last_lines_of_the_output() {
        assert_eq!(tail("a\nb\nc\n", 2), "b\nc");
        assert_eq!(tail("a", 5), "a");
        assert_eq!(tail("", 5), "");
    }

    const NODE: &str = "use crate::Node;\n\
                        pub enum Outcome { Valid, Rejected(String) }\n\
                        pub struct ReadMove;\n\
                        impl<S: Send> Node<S> for ReadMove {\n    \
                            type Outcome = Outcome;\n    \
                            type Error = std::convert::Infallible;\n    \
                            async fn run(&self, _: &mut S) -> Result<Outcome, Self::Error> \
                            { Ok(Outcome::Valid) }\n\
                        }\n";

    #[test]
    fn lists_a_nodes_outcomes() {
        assert_eq!(
            outcomes_line("read_move", NODE),
            "- read_move: Valid, Rejected"
        );
        let terminal = NODE.replace("type Outcome = Outcome;", "type Outcome = ();");
        assert_eq!(
            outcomes_line("read_move", &terminal),
            "- read_move: terminal (no outcomes)"
        );
        assert!(outcomes_line("x", "fn x() {}").contains("could not be read"));
    }

    fn brief<'a>(settings: &'a Settings, passed: bool) -> Brief<'a> {
        Brief {
            settings,
            spec: SPEC,
            modules: MODULES,
            passed,
            output: "src/nodes/report.rs:3: todo: finish it\n`cargo test` failed",
        }
    }

    #[test]
    fn a_node_prompt_holds_only_its_section_the_state_and_the_project() {
        let settings = settings_for("game");

        let prompt = node_prompt(&brief(&settings, true), "report", false);

        assert!(prompt.contains("`report` from `specs/game.md`"));
        assert!(prompt.contains("### Report\n- Prints the board"));
        assert!(!prompt.contains("### Read Move"));
        assert!(prompt.contains("- `board`: the cells."));
        assert!(prompt.contains("Existing nodes: read_move\n"));
        assert!(prompt.contains("Existing state traits in src/nodes/mod.rs: HasBoard "));
        assert!(prompt.contains("does not exist yet"));
        assert!(prompt.contains("`amble node report (--outcomes"));
        assert!(!prompt.contains("amble check` output"));
    }

    #[test]
    fn a_node_prompt_resumes_and_carries_a_failing_check() {
        let settings = settings_for("game");

        let prompt = node_prompt(&brief(&settings, false), "report", true);

        assert!(prompt.contains("Do NOT run `amble node` again"));
        assert!(prompt.contains("## Last `amble check` output\n```\nsrc/nodes/report.rs:3"));
    }

    #[test]
    fn a_walk_prompt_fills_in_the_nodes_and_lists_their_outcomes() {
        let settings = settings_for("game");
        let outcomes = ["- read_move: Valid".to_string()];

        let prompt = walk_prompt(&brief(&settings, true), &outcomes, false);

        assert!(prompt.contains("## The spec\n# Program Specifications"));
        assert!(prompt.contains("- read_move: Valid\n"));
        assert!(prompt.contains("`amble walk game --nodes read_move,report --edge"));

        let resumed = walk_prompt(&brief(&settings, false), &outcomes, true);
        assert!(resumed.contains("Do NOT run `amble walk` again"));
        assert!(resumed.contains("`cargo test` failed"));
    }

    #[test]
    fn a_repair_prompt_holds_the_check_output_and_the_changes() {
        let settings = settings_for("game");

        let prompt = repair_prompt(&brief(&settings, false), " M src/nodes/report.rs\n");

        assert!(
            prompt
                .contains("```\nsrc/nodes/report.rs:3: todo: finish it\n`cargo test` failed\n```")
        );
        assert!(prompt.contains("```\n M src/nodes/report.rs\n```"));
    }

    #[test]
    fn the_prompts_only_mention_verbs_and_topics_that_exist() {
        let settings = settings_for("game");
        let brief = brief(&settings, false);
        for prompt in [
            node_prompt(&brief, "report", false),
            node_prompt(&brief, "report", true),
            walk_prompt(&brief, &[], false),
            walk_prompt(&brief, &[], true),
            repair_prompt(&brief, ""),
        ] {
            assert_eq!(unknown_mentions(&prompt), Vec::<String>::new(), "{prompt}");
        }
    }
}
