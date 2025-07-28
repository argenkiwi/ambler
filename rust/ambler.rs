
use std::future::Future;

pub fn resolve<S, A, D>(result: (S, A), direct: impl Fn(A) -> D) -> (S, D) {
    let (state, action) = result;
    (state, direct(action))
}

pub async fn amble<S, E, F, Fut>(state: S, edge: E, follow: F) -> (S, Option<E>)
where
    F: Fn(S, E) -> Fut,
    Fut: Future<Output = (S, Option<E>)>,{
    let mut current_state = state;
    let mut current_edge = Some(edge);

    while let Some(e) = current_edge {
        let (s, next_edge) = follow(current_state, e).await;
        current_state = s;
        current_edge = next_edge;
    }

    (current_state, None)
}
