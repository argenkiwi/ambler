use crate::Node;
use crate::guides;
use crate::nodes::{HasArgs, HasMessage, HasTarget};
use std::convert::Infallible;

/// Where [`Guide`] leaves the walk.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// The guide was printed.
    Printed,
    /// There is no such topic; the message lists the ones there are.
    Unknown,
}

pub trait Utils {
    fn print(&self, message: &str);
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SystemUtils;

impl Utils for SystemUtils {
    fn print(&self, message: &str) {
        println!("{message}");
    }
}

/// Prints the guide for the topic in the first argument, or the general guide without one.
#[derive(Clone, Debug, Default)]
pub struct Guide<U = SystemUtils> {
    utils: U,
}

impl Guide {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl<U> Guide<U> {
    #[must_use]
    pub fn with_utils(utils: U) -> Self {
        Self { utils }
    }
}

impl<S, U> Node<S> for Guide<U>
where
    S: HasTarget + HasArgs + HasMessage + Send,
    U: Utils + Sync,
{
    type Outcome = Outcome;
    type Error = Infallible;

    async fn run(&self, state: &mut S) -> Result<Outcome, Infallible> {
        let topic = state.args().first().map_or("", String::as_str);
        Ok(match guides::guide(topic, state.target()) {
            Some(guide) => {
                self.utils.print(guide.trim_end());
                Outcome::Printed
            }
            None => {
                *state.message_mut() = Some(format!(
                    "amble: no verb or help topic called `{topic}`\ntopics: {}",
                    guides::TOPICS.join(", ")
                ));
                Outcome::Unknown
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{Fake, State};

    async fn run(fake: &Fake, args: &[&str]) -> (Outcome, State) {
        let mut state = State::with_args(args);
        let Ok(outcome) = Guide::with_utils(fake).run(&mut state).await;
        (outcome, state)
    }

    #[tokio::test]
    async fn prints_the_general_guide_without_a_topic() {
        let fake = Fake::default();

        let (outcome, _) = run(&fake, &[]).await;

        assert_eq!(outcome, Outcome::Printed);
        assert!(fake.printed()[0].starts_with("# amble\n"));
    }

    #[tokio::test]
    async fn prints_the_guide_of_a_topic() {
        let fake = Fake::default();

        let (outcome, state) = run(&fake, &["walk"]).await;

        assert_eq!(outcome, Outcome::Printed);
        assert!(fake.printed()[0].starts_with("# amble walk\n"));
        assert_eq!(state.message, None);
    }

    #[tokio::test]
    async fn lists_the_topics_when_there_is_no_such_one() {
        let fake = Fake::default();

        let (outcome, state) = run(&fake, &["frobnicate"]).await;

        assert_eq!(outcome, Outcome::Unknown);
        assert!(fake.printed().is_empty());
        let message = state.message.unwrap();
        assert!(message.contains("`frobnicate`"));
        assert!(message.contains("node-design"));
    }
}
