use crate::nodes::HasCount;
use crate::Node;
use std::sync::Arc;

pub struct Edges<K> {
    pub on_done: Option<K>,
}

pub trait Utils: Send + Sync {
    fn print(&self, message: &str);
}

pub struct DefaultUtils;

impl Utils for DefaultUtils {
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

    Box::new(move |state: S| {
        utils.print(&format!("Final count: {}", state.count()));
        Box::pin(std::future::ready((edges.on_done.clone(), state)))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

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

    #[derive(Default)]
    struct MockUtils {
        message: Mutex<Option<String>>,
    }

    impl Utils for MockUtils {
        fn print(&self, message: &str) {
            *self.message.lock().unwrap() = Some(message.to_string());
        }
    }

    #[tokio::test]
    async fn prints_final_count_and_transitions_to_on_done() {
        let utils = Arc::new(MockUtils::default());
        let node: Node<MockState, &'static str> =
            factory(Edges { on_done: None }, Some(utils.clone()));

        let (next, state) = node(MockState { count: 15 }).await;

        assert_eq!(next, None);
        assert_eq!(state.count, 15);
        assert_eq!(
            utils.message.lock().unwrap().as_deref(),
            Some("Final count: 15")
        );
    }
}
