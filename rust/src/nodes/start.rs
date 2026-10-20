use crate::nodes::HasCount;
use crate::Node;
use std::sync::Arc;

pub struct Edges<K> {
    pub on_success: Option<K>,
    pub on_error: Option<K>,
}

pub trait Utils: Send + Sync {
    /// Prints `message` and reads a line, or `None` if no input could be read.
    fn read_line(&self, message: &str) -> Option<String>;
    fn print(&self, message: &str);
}

pub struct DefaultUtils;

impl Utils for DefaultUtils {
    fn read_line(&self, message: &str) -> Option<String> {
        use std::io::{self, Write};
        print!("{message}");
        io::stdout().flush().ok()?;
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(0) | Err(_) => None,
            Ok(_) => Some(input.trim_end_matches(['\r', '\n']).to_string()),
        }
    }

    fn print(&self, message: &str) {
        println!("{message}");
    }
}

/// Creates the node. `None` utils uses [`DefaultUtils`].
pub fn factory<S, K>(edges: Edges<K>, utils: Option<Arc<dyn Utils>>) -> Node<S, K>
where
    S: HasCount + Send + 'static,
    K: Clone + Send + Sync + 'static,
{
    let utils = utils.unwrap_or_else(|| Arc::new(DefaultUtils));

    Box::new(move |mut state: S| {
        let next = match utils.read_line("Enter a starting number: ").as_deref() {
            None | Some("") => {
                state.set_count(0);
                edges.on_success.clone()
            }
            Some(input) => match input.parse::<i32>() {
                Ok(n) => {
                    state.set_count(n);
                    edges.on_success.clone()
                }
                Err(_) => {
                    utils.print("Error: Invalid input. Please enter a number.");
                    edges.on_error.clone()
                }
            },
        };
        Box::pin(std::future::ready((next, state)))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockState {
        count: i32,
    }

    impl HasCount for MockState {
        fn count(&self) -> i32 {
            self.count
        }
        fn set_count(&mut self, count: i32) {
            self.count = count;
        }
    }

    struct MockUtils {
        input: &'static str,
    }

    impl Utils for MockUtils {
        fn read_line(&self, _message: &str) -> Option<String> {
            Some(self.input.to_string())
        }
        fn print(&self, _message: &str) {}
    }

    fn node(input: &'static str) -> Node<MockState, &'static str> {
        factory(
            Edges {
                on_success: Some("NEXT"),
                on_error: Some("ERROR"),
            },
            Some(Arc::new(MockUtils { input })),
        )
    }

    #[tokio::test]
    async fn transitions_to_on_success_with_0_if_input_is_empty() {
        let (next, state) = node("")(MockState { count: 0 }).await;

        assert_eq!(next, Some("NEXT"));
        assert_eq!(state.count, 0);
    }

    #[tokio::test]
    async fn transitions_to_on_success_with_input_number() {
        let (next, state) = node("42")(MockState { count: 0 }).await;

        assert_eq!(next, Some("NEXT"));
        assert_eq!(state.count, 42);
    }

    #[tokio::test]
    async fn transitions_to_on_error_if_input_is_invalid() {
        let (next, state) = node("abc")(MockState { count: 123 }).await;

        assert_eq!(next, Some("ERROR"));
        assert_eq!(state.count, 123);
    }
}
