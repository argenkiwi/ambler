//! Nodes: atomic steps, and the state traits they share.

pub mod brief_agent;
pub mod check_project;
pub mod checkpoint;
pub mod choose_task;
pub mod delegate;
pub mod guide;
pub mod plan_pave;
pub mod plan_project;
pub mod report;
pub mod roll_back;
pub mod write;

use crate::scaffold::Plan;
use crate::scaffold::pave::Paving;
use crate::scaffold::target::Target;

/// A state that has the verb's command-line arguments.
pub trait HasArgs {
    fn args(&self) -> &[String];
}

/// A state that has the language of the project being made or worked on.
pub trait HasTarget {
    fn target(&self) -> Target;
}

/// A state that has the verb being handed to the target's native CLI.
pub trait HasVerb {
    fn verb(&self) -> &str;
}

/// A state that has the exit code `ambler` ends with.
pub trait HasExitCode {
    fn exit_code(&self) -> u8;
    fn exit_code_mut(&mut self) -> &mut u8;
}

/// A state that has the changes a verb is about to make.
pub trait HasPlan {
    fn plan(&self) -> &Plan;
    fn plan_mut(&mut self) -> &mut Plan;
}

/// A state that has the reason the verb failed, if it did.
pub trait HasMessage {
    fn message(&self) -> Option<&str>;
    fn message_mut(&mut self) -> &mut Option<String>;
}

/// A state that has the `ambler pave` loop: its settings, last check, task and counters.
pub trait HasPaving {
    fn paving(&self) -> &Paving;
    fn paving_mut(&mut self) -> &mut Paving;
}
