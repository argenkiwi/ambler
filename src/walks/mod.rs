//! Walks: one per verb, each a small state machine of nodes.

pub mod check;
pub mod help;
pub mod new;
pub mod node;
pub mod util;
pub mod walk;

use crate::nodes::{HasArgs, HasMessage, HasPlan, inspect, verify, write};
use crate::scaffold::Plan;

/// Why a verb did not complete.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The invocation was rejected or a check failed. The walk has already said why.
    #[error("the command failed")]
    Failed,
    #[error(transparent)]
    Write(#[from] write::Error),
    #[error(transparent)]
    Inspect(#[from] inspect::Error),
    #[error(transparent)]
    Verify(#[from] verify::Error),
}

/// The state every verb's walk shares.
#[derive(Clone, Debug, Default)]
pub(crate) struct State {
    pub args: Vec<String>,
    pub plan: Plan,
    pub message: Option<String>,
}

impl State {
    fn new(args: Vec<String>) -> Self {
        Self {
            args,
            ..Self::default()
        }
    }

    #[cfg(test)]
    pub fn with_args(args: &[&str]) -> Self {
        Self::new(args.iter().map(ToString::to_string).collect())
    }

    /// A walk that left a message failed, and has reported it.
    fn finish(self) -> Result<(), Error> {
        match self.message {
            Some(_) => Err(Error::Failed),
            None => Ok(()),
        }
    }
}

impl HasArgs for State {
    fn args(&self) -> &[String] {
        &self.args
    }
}

impl HasPlan for State {
    fn plan(&self) -> &Plan {
        &self.plan
    }
    fn plan_mut(&mut self) -> &mut Plan {
        &mut self.plan
    }
}

impl HasMessage for State {
    fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }
    fn message_mut(&mut self) -> &mut Option<String> {
        &mut self.message
    }
}
