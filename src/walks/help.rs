//! `amble help`: prints the general guide, or one topic's. See `specs/help.md`.

use super::State;
use crate::nodes::guide::{self, Guide};
use crate::nodes::report::{self, Report};
use crate::scaffold::target::Target;
use crate::{Node, Transition, Walk};

/// An unknown verb or topic.
pub type Error = super::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Id {
    Guide,
    Report,
}

#[derive(Default)]
struct Help<A = guide::SystemUtils, B = report::SystemUtils> {
    guide: Guide<A>,
    report: Report<B>,
}

impl Help {
    /// Production wiring: every node uses its `SystemUtils`.
    fn new() -> Self {
        Self::default()
    }
}

impl<A, B> Walk for Help<A, B>
where
    A: guide::Utils + Sync,
    B: report::Utils + Sync,
{
    type Id = Id;
    type State = State;
    type Error = Error;

    async fn step(&self, id: Id, state: &mut State) -> Result<Transition<Id>, Error> {
        Ok(match id {
            Id::Guide => {
                let Ok(outcome) = self.guide.run(state).await;
                match outcome {
                    guide::Outcome::Printed => Transition::Halt,
                    guide::Outcome::Unknown => Transition::Next(Id::Report),
                }
            }
            Id::Report => {
                let Ok(()) = self.report.run(state).await;
                Transition::Halt
            }
        })
    }
}

/// Runs `amble help [topic]`.
///
/// # Errors
///
/// Returns [`Error`] if there is no such topic; the topics there are have been listed on
/// standard error.
pub async fn run(target: Target, args: Vec<String>) -> Result<(), Error> {
    let mut state = State::new(target, args);
    Help::new().run(Id::Guide, &mut state).await?;
    state.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::Fake;

    async fn help(fake: &Fake, args: &[&str]) -> State {
        let walk = Help {
            guide: Guide::with_utils(fake),
            report: Report::with_utils(fake),
        };
        let mut state = State::with_args(args);
        walk.run(Id::Guide, &mut state).await.unwrap();
        state
    }

    #[tokio::test]
    async fn prints_a_guide_on_standard_output() {
        let fake = Fake::default();

        let state = help(&fake, &["check"]).await;

        assert_eq!(state.message, None);
        assert!(fake.printed()[0].starts_with("# amble check\n"));
        assert!(fake.eprinted().is_empty());
    }

    #[tokio::test]
    async fn an_unknown_topic_lists_the_topics_on_standard_error_and_fails() {
        let fake = Fake::default();

        let state = help(&fake, &["frobnicate"]).await;

        assert!(fake.printed().is_empty());
        assert!(fake.eprinted()[0].contains("topics: new, node, util, walk, check, pave, help"));
        assert!(state.finish().is_err());
    }
}
