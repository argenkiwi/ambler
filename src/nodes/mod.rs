//! Nodes: atomic steps, and the state traits they share.

pub mod guide;
pub mod inspect;
pub mod plan_node;
pub mod plan_project;
pub mod plan_util;
pub mod plan_walk;
pub mod report;
pub mod verify;
pub mod write;

use crate::scaffold::Plan;

/// A state that has the verb's command-line arguments.
pub trait HasArgs {
    fn args(&self) -> &[String];
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
