use crate::nodes::HasCount;
use crate::Node;
use futures::future::BoxFuture;
use std::sync::Arc;
use std::time::Duration;

pub struct Edges<K> {
    pub on_count: Option<K>,
    pub on_stop: Option<K>,
}

pub trait Utils: Send + Sync {
    fn print(&self, message: &str);
    fn sleep(&self, ms: u64) -> BoxFuture<'static, ()>;
    fn random(&self) -> f64;
}

pub struct DefaultUtils;

impl Utils for DefaultUtils {
    fn print(&self, message: &str) {
        println!("{message}");
    }

    fn sleep(&self, ms: u64) -> BoxFuture<'static, ()> {
        Box::pin(tokio::time::sleep(Duration::from_millis(ms)))
    }

    fn random(&self) -> f64 {
        rand::random::<f64>()
    }
}

/// Creates the node. `None` utils uses [`DefaultUtils`].
pub fn factory<S, K>(edges: Edges<K>, utils: Option<Arc<dyn Utils>>) -> Node<S, K>
where
    S: HasCount + Send + 'static,
    K: Clone + Send + Sync + 'static,
{
    let utils = utils.unwrap_or_else(|| Arc::new(DefaultUtils));
    let edges = Arc::new(edges);

    Box::new(move |mut state: S| {
        let utils = utils.clone();
        let edges = edges.clone();
        Box::pin(async move {
            utils.print(&format!("Current count: {}", state.count()));
            utils.sleep(1000).await;
            state.set_count(state.count() + 1);

            if utils.random() > 0.5 {
                (edges.on_count.clone(), state)
            } else {
                (edges.on_stop.clone(), state)
            }
        })
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
        random: f64,
    }

    impl Utils for MockUtils {
        fn print(&self, _message: &str) {}
        fn sleep(&self, _ms: u64) -> BoxFuture<'static, ()> {
            Box::pin(async {})
        }
        fn random(&self) -> f64 {
            self.random
        }
    }

    fn node(random: f64) -> Node<MockState, &'static str> {
        factory(
            Edges {
                on_count: Some("onCount"),
                on_stop: Some("onStop"),
            },
            Some(Arc::new(MockUtils { random })),
        )
    }

    #[tokio::test]
    async fn increments_count_and_transitions_to_on_count_if_random_above_half() {
        let (next, state) = node(0.6)(MockState { count: 5 }).await;

        assert_eq!(next, Some("onCount"));
        assert_eq!(state.count, 6);
    }

    #[tokio::test]
    async fn increments_count_and_transitions_to_on_stop_if_random_at_most_half() {
        let (next, state) = node(0.4)(MockState { count: 10 }).await;

        assert_eq!(next, Some("onStop"));
        assert_eq!(state.count, 11);
    }
}
