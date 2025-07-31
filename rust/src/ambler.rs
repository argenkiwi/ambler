
use std::future::Future;
use std::pin::Pin;

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub struct Next<'a> {
    pub run: Box<dyn Fn() -> BoxFuture<'a, Option<Next<'a>>> + Send + Sync + 'a>,
}

impl<'a> Next<'a> {
    pub fn new<F>(run: F) -> Self
    where
        F: Fn() -> BoxFuture<'a, Option<Next<'a>>> + Send + Sync + 'a,
    {
        Next {
            run: Box::new(run),
        }
    }
}

pub async fn amble<'a, S, F>(initial: F, state: S)
where
    F: Fn(S) -> BoxFuture<'a, Option<Next<'a>>> + Send + Sync + 'a,
{
    let mut next = initial(state).await;
    while let Some(n) = next {
        next = (n.run)().await;
    }
}
