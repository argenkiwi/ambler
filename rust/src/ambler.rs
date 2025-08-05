use std::future::Future;

pub async fn amble<S, L, F, Fut>(mut state: S, mut lead: L, mut follow: F) -> S
where
    F: FnMut(L, S) -> Fut,
    Fut: Future<Output = (S, Option<L>)>,{
    loop {
        let (new_state, next_lead) = follow(lead, state).await;
        state = new_state;
        if let Some(l) = next_lead {
            lead = l;
        } else {
            break;
        }
    }
    state
}
