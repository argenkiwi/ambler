//! What a walk is made of whatever the target: the edges `amble walk` is given, the checks that
//! they fit the nodes, and the spec it starts.

use super::names;

/// `--edge <from>.<Outcome>=<to>`: where one outcome of a node leads.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Edge {
    pub from: String,
    /// `PascalCase`, as given: the target turns it into a variant or an edge name.
    pub outcome: String,
    /// The next node, or `None` to end the walk (`=halt`).
    pub to: Option<String>,
}

/// Parses `start.Success=count` or `count.Finished=halt`. Node names may be `kebab-case`.
///
/// # Errors
///
/// Returns a message showing the expected shape.
pub fn parse_edge(text: &str) -> Result<Edge, String> {
    let bad =
        || format!("`--edge {text}` should look like `--edge start.Success=count` (or `=halt`)");
    let (source, to) = text.split_once('=').ok_or_else(&bad)?;
    let (from, outcome) = source.split_once('.').ok_or_else(&bad)?;
    if from.is_empty() || outcome.is_empty() || to.is_empty() {
        return Err(bad());
    }
    Ok(Edge {
        from: from.replace('-', "_"),
        outcome: names::pascal(outcome),
        to: (to != "halt").then(|| to.replace('-', "_")),
    })
}

/// A node as a walk sees it: its name and the outcomes it can report, `None` if it ends the walk.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Wired {
    pub name: String,
    pub outcomes: Option<Vec<String>>,
}

/// Checks that the nodes and edges of the walk `walk` make sense together. Each edge's outcome
/// must already be in the nodes' spelling.
///
/// # Errors
///
/// Returns a message naming the first problem.
pub fn validate(walk: &str, nodes: &[Wired], edges: &[Edge]) -> Result<(), String> {
    let node = |name: &str| nodes.iter().find(|node| node.name == name);
    for (index, info) in nodes.iter().enumerate() {
        if nodes[..index].iter().any(|other| other.name == info.name) {
            return Err(format!("node `{}` is listed twice", info.name));
        }
        if info.name == walk {
            return Err(format!(
                "the walk and the node `{}` cannot share a name",
                info.name
            ));
        }
    }
    for (index, edge) in edges.iter().enumerate() {
        let from = node(&edge.from)
            .ok_or_else(|| format!("edge from `{}`: it is not in `--nodes`", edge.from))?;
        let Some(outcomes) = &from.outcomes else {
            return Err(format!(
                "edge from `{}`: it is a terminal node, it has no outcomes",
                edge.from
            ));
        };
        if !outcomes.contains(&edge.outcome) {
            return Err(format!(
                "edge `{}.{}`: `{}` has no such outcome (it has {})",
                edge.from,
                edge.outcome,
                edge.from,
                outcomes.join(", ")
            ));
        }
        if let Some(to) = &edge.to {
            if node(to).is_none() {
                return Err(format!(
                    "edge `{}.{}={to}`: `{to}` is not in `--nodes`",
                    edge.from, edge.outcome
                ));
            }
        }
        let repeated = edges[..index]
            .iter()
            .any(|other| other.from == edge.from && other.outcome == edge.outcome);
        if repeated {
            return Err(format!(
                "edge `{}.{}` is given twice",
                edge.from, edge.outcome
            ));
        }
    }
    Ok(())
}

/// The contents of `specs/<walk>.md`: one section per node, with the edges already known, and
/// the shared state's `fields`.
#[must_use]
pub fn spec_md(nodes: &[Wired], edges: &[Edge], fields: &[String]) -> String {
    let mut out = String::from(
        "# Program Specifications\n\nTODO: describe the program and its purpose.\n\n## Nodes\n",
    );
    for (index, node) in nodes.iter().enumerate() {
        out += &format!("\n### {}\n", names::title(&node.name));
        if index == 0 {
            out += "- Initial node.\n";
        }
        out += "- TODO: describe what this node does.\n";
        match &node.outcomes {
            None => out += "- Terminates the walk.\n",
            Some(outcomes) => {
                for outcome in outcomes {
                    let edge = edges
                        .iter()
                        .find(|edge| edge.from == node.name && edge.outcome == *outcome);
                    out += &match edge {
                        Some(Edge { to: Some(to), .. }) => format!(
                            "- TODO: when `{outcome}`, it transitions to `{}`.\n",
                            names::shout(to)
                        ),
                        Some(Edge { to: None, .. }) => {
                            format!("- TODO: when `{outcome}`, it terminates.\n")
                        }
                        None => format!(
                            "- TODO: when `{outcome}`, it transitions to (not wired yet).\n"
                        ),
                    };
                }
            }
        }
    }
    out += "\n## Shared State\n\n";
    if fields.is_empty() {
        out += "TODO: describe the state the nodes share, if any.\n";
    } else {
        out += "The shared state consists of:\n";
        for field in fields {
            out += &format!("- `{field}`: TODO: what it holds.\n");
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_edges() {
        assert_eq!(
            parse_edge("read-move.invalid_input=start").unwrap(),
            Edge {
                from: "read_move".to_string(),
                outcome: "InvalidInput".to_string(),
                to: Some("start".to_string()),
            }
        );
        assert_eq!(parse_edge("count.Finished=halt").unwrap().to, None);
        assert!(parse_edge("start=count").is_err());
        assert!(parse_edge("start.Success").is_err());
    }
}
