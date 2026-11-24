use crate::Node;
use crate::nodes::HasMessage;
use std::convert::Infallible;

pub trait Utils {
    fn eprint(&self, message: &str);
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SystemUtils;

impl Utils for SystemUtils {
    fn eprint(&self, message: &str) {
        eprintln!("{message}");
    }
}

/// Prints why the verb failed, on standard error. Terminal: it has no outgoing edges.
#[derive(Clone, Debug, Default)]
pub struct Report<U = SystemUtils> {
    utils: U,
}

impl Report {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl<U> Report<U> {
    #[must_use]
    pub fn with_utils(utils: U) -> Self {
        Self { utils }
    }
}

impl<S, U> Node<S> for Report<U>
where
    S: HasMessage + Send,
    U: Utils + Sync,
{
    type Outcome = ();
    type Error = Infallible;

    async fn run(&self, state: &mut S) -> Result<(), Infallible> {
        if let Some(message) = state.message() {
            self.utils.eprint(message);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{Fake, State};

    #[tokio::test]
    async fn prints_the_message_on_standard_error_and_keeps_it() {
        let fake = Fake::default();
        let mut state = State {
            message: Some("usage: ambler new <name>".to_string()),
            ..State::default()
        };

        let Ok(()) = Report::with_utils(&fake).run(&mut state).await;

        assert_eq!(fake.eprinted(), ["usage: ambler new <name>"]);
        assert!(fake.printed().is_empty());
        assert_eq!(state.message.as_deref(), Some("usage: ambler new <name>"));
    }

    #[tokio::test]
    async fn prints_nothing_without_a_message() {
        let fake = Fake::default();

        let Ok(()) = Report::with_utils(&fake).run(&mut State::default()).await;

        assert!(fake.eprinted().is_empty());
    }
}
