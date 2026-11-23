//! `amble node`: adds a node to the project. See `specs/node.md`.

use super::State;
use crate::nodes::plan_node::{self, PlanNode};
use crate::nodes::report::{self, Report};
use crate::nodes::write::{self, Write};
use crate::scaffold::target::Target;
use crate::{Node as _, Transition, Walk};

/// A rejected invocation, or a file that could not be written.
pub type Error = super::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Id {
    PlanNode,
    Write,
    Report,
}

#[derive(Default)]
struct Node<A = plan_node::SystemUtils, B = write::SystemUtils, C = report::SystemUtils> {
    plan_node: PlanNode<A>,
    write: Write<B>,
    report: Report<C>,
}

impl Node {
    /// Production wiring: every node uses its `SystemUtils`.
    fn new() -> Self {
        Self::default()
    }
}

impl<A, B, C> Walk for Node<A, B, C>
where
    A: plan_node::Utils + Sync,
    B: write::Utils + Sync,
    C: report::Utils + Sync,
{
    type Id = Id;
    type State = State;
    type Error = Error;

    async fn step(&self, id: Id, state: &mut State) -> Result<Transition<Id>, Error> {
        Ok(match id {
            Id::PlanNode => {
                let Ok(outcome) = self.plan_node.run(state).await;
                match outcome {
                    plan_node::Outcome::Planned => Transition::Next(Id::Write),
                    plan_node::Outcome::Rejected => Transition::Next(Id::Report),
                }
            }
            Id::Write => {
                self.write.run(state).await?;
                Transition::Halt
            }
            Id::Report => {
                let Ok(()) = self.report.run(state).await;
                Transition::Halt
            }
        })
    }
}

/// Runs `amble node <name> …`.
///
/// # Errors
///
/// Returns [`Error`] if the invocation is rejected (the reason is already on standard error) or
/// a file cannot be written.
pub async fn run(target: Target, args: Vec<String>) -> Result<(), Error> {
    if let Some(ok) = crate::utils::runner::delegate(target, "node", &args)? {
        return if ok { Ok(()) } else { Err(Error::Failed) };
    }
    let mut state = State::new(target, args);
    Node::new().run(Id::PlanNode, &mut state).await?;
    state.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::Fake;

    async fn node(fake: &Fake, args: &[&str]) -> State {
        let walk = Node {
            plan_node: PlanNode::with_utils(fake),
            write: Write::with_utils(fake),
            report: Report::with_utils(fake),
        };
        let mut state = State::with_args(args);
        walk.run(Id::PlanNode, &mut state).await.unwrap();
        state
    }

    #[tokio::test]
    async fn writes_the_node_registers_it_and_formats_it() {
        let fake = Fake::with_files(&[("src/nodes/mod.rs", "//! Nodes.\n")]);

        let state = node(
            &fake,
            &["ask", "--outcomes", "answered", "--state", "answer:String"],
        )
        .await;

        assert_eq!(state.message, None);
        assert!(
            fake.file("src/nodes/ask.rs")
                .unwrap()
                .contains("pub struct Ask<U = SystemUtils>")
        );
        let modules = fake.file("src/nodes/mod.rs").unwrap();
        assert!(modules.starts_with("//! Nodes.\n\npub mod ask;\n\n"));
        assert!(modules.contains("pub trait HasAnswer {"));
        assert_eq!(*fake.formatted.lock().unwrap(), ["src/nodes/ask.rs"]);
        assert!(fake.eprinted().is_empty());
    }

    #[tokio::test]
    async fn reports_a_rejected_invocation_and_writes_nothing() {
        let fake = Fake::with_files(&[("src/nodes/mod.rs", "//! Nodes.\n")]);

        let state = node(&fake, &["ask"]).await;

        assert!(state.finish().is_err());
        assert!(fake.eprinted()[0].contains("say what the node reports"));
        assert_eq!(fake.paths(), ["src/nodes/mod.rs"]);
        assert_eq!(
            fake.file("src/nodes/mod.rs").as_deref(),
            Some("//! Nodes.\n")
        );
    }
}
