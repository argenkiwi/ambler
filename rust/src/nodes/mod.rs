pub mod count;
pub mod start;
pub mod stop;

/// A state that has a count field.
pub trait HasCount {
    fn count(&self) -> i32;
    fn set_count(&mut self, count: i32);
}
