//! Walks: one per verb, each a small state machine of nodes.

pub mod check;
pub mod help;
pub mod new;
pub mod node;
pub mod pave;
pub mod util;
pub mod walk;

use crate::nodes::{
    HasArgs, HasMessage, HasPaving, HasPlan, HasTarget, brief_agent, check_project, checkpoint,
    choose_task, inspect, roll_back, verify, write,
};
use crate::scaffold::Plan;
use crate::scaffold::pave::Paving;
use crate::scaffold::target::Target;

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
    #[error(transparent)]
    CheckProject(#[from] check_project::Error),
    #[error(transparent)]
    Checkpoint(#[from] checkpoint::Error),
    #[error(transparent)]
    ChooseTask(#[from] choose_task::Error),
    #[error(transparent)]
    RollBack(#[from] roll_back::Error),
    #[error(transparent)]
    BriefAgent(#[from] brief_agent::Error),
}

/// The state every verb's walk shares.
#[derive(Clone, Debug, Default)]
pub(crate) struct State {
    /// The language of the project being made or worked on.
    pub target: Target,
    pub args: Vec<String>,
    pub plan: Plan,
    /// The `amble pave` loop; the other verbs leave it at its default.
    pub paving: Paving,
    pub message: Option<String>,
}

impl State {
    fn new(target: Target, args: Vec<String>) -> Self {
        Self {
            target,
            args,
            ..Self::default()
        }
    }

    #[cfg(test)]
    pub fn with_args(args: &[&str]) -> Self {
        Self::new(
            Target::default(),
            args.iter().map(ToString::to_string).collect(),
        )
    }

    #[cfg(test)]
    pub fn with_target(target: Target, args: &[&str]) -> Self {
        Self {
            target,
            ..Self::with_args(args)
        }
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

impl HasTarget for State {
    fn target(&self) -> Target {
        self.target
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

impl HasPaving for State {
    fn paving(&self) -> &Paving {
        &self.paving
    }
    fn paving_mut(&mut self) -> &mut Paving {
        &mut self.paving
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
