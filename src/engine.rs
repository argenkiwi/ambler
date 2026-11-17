//! The Ambler engine: nodes report outcomes, walks wire outcomes to successors.
//!
//! This file is the same in every Ambler project; `amble new` writes it. Leave it as it is.

use std::future::Future;

/// A processing step in a walk.
///
/// A node reads and mutates the shared state, then reports what happened as an
/// [`Outcome`](Node::Outcome). It never names the node that comes next: wiring outcomes to
/// successors is the walk's job (see [`Walk`]).
pub trait Node<S> {
    /// What happened, one variant per outgoing edge.
    type Outcome;
    /// Why the node could not complete.
    type Error;

    /// Runs the node against `state`.
    ///
    /// # Errors
    ///
    /// Returns [`Self::Error`](Node::Error) if the node fails; the walk decides what that means.
    fn run(&self, state: &mut S)
    -> impl Future<Output = Result<Self::Outcome, Self::Error>> + Send;
}

/// Where a walk goes after a step.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Transition<Id> {
    /// Continue with the node identified by this id.
    Next(Id),
    /// The walk is over.
    Halt,
}

/// A graph of nodes, wired together.
///
/// An implementation owns its nodes and matches on an `Id` enum in [`step`](Walk::step), turning
/// each node's outcome into a [`Transition`]. Because the match is exhaustive, an unwired outcome
/// or an unknown node is a compile error rather than a runtime failure.
pub trait Walk: Sync {
    /// Identifies a node in this walk.
    type Id: Copy + Send;
    /// State shared by every node.
    type State: Send;
    /// Failure of any node in the walk.
    type Error;

    /// Runs the node identified by `id` and decides where to go next.
    ///
    /// # Errors
    ///
    /// Returns [`Self::Error`](Walk::Error) if the node fails.
    fn step(
        &self,
        id: Self::Id,
        state: &mut Self::State,
    ) -> impl Future<Output = Result<Transition<Self::Id>, Self::Error>> + Send;

    /// Steps from `start` until a node halts the walk.
    ///
    /// # Errors
    ///
    /// Stops at, and returns, the first error from [`step`](Walk::step).
    fn run(
        &self,
        start: Self::Id,
        state: &mut Self::State,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send {
        async move {
            let mut id = start;
            while let Transition::Next(next) = self.step(id, state).await? {
                id = next;
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy)]
    enum Id {
        A,
        B,
        C,
    }

    /// Records every id it visits, then follows A -> B -> C and halts; fails at C if asked.
    struct Scripted {
        fail_at_c: bool,
    }

    impl Walk for Scripted {
        type Id = Id;
        type State = Vec<char>;
        type Error = &'static str;

        async fn step(
            &self,
            id: Id,
            state: &mut Vec<char>,
        ) -> Result<Transition<Id>, &'static str> {
            match id {
                Id::A => {
                    state.push('A');
                    Ok(Transition::Next(Id::B))
                }
                Id::B => {
                    state.push('B');
                    Ok(Transition::Next(Id::C))
                }
                Id::C if self.fail_at_c => Err("boom"),
                Id::C => {
                    state.push('C');
                    Ok(Transition::Halt)
                }
            }
        }
    }

    #[tokio::test]
    async fn run_steps_until_a_node_halts() {
        let mut visited = Vec::new();

        let result = Scripted { fail_at_c: false }.run(Id::A, &mut visited).await;

        assert_eq!(result, Ok(()));
        assert_eq!(visited, ['A', 'B', 'C']);
    }

    #[tokio::test]
    async fn run_starts_wherever_it_is_told() {
        let mut visited = Vec::new();

        Scripted { fail_at_c: false }
            .run(Id::B, &mut visited)
            .await
            .unwrap();

        assert_eq!(visited, ['B', 'C']);
    }

    #[tokio::test]
    async fn run_stops_at_the_first_error() {
        let mut visited = Vec::new();

        let result = Scripted { fail_at_c: true }.run(Id::A, &mut visited).await;

        assert_eq!(result, Err("boom"));
        assert_eq!(visited, ['A', 'B']);
    }
}
