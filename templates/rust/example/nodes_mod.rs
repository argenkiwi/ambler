//! Nodes: atomic steps, and the state traits they share.

pub mod count;
pub mod start;
pub mod stop;

/// A state that has a count field.
pub trait HasCount {
    fn count(&self) -> i32;
    fn count_mut(&mut self) -> &mut i32;
}
